//! The search process, without backjumping.
use crate::{
    cells::{CellRef, State},
    rules::Rule,
    search::{private::Sealed, Algorithm, Reason as TraitReason, SetCell},
    world::World,
};
use typebool::Bool;

#[cfg(feature = "serde")]
use crate::{error::Error, save::ReasonSer};

#[cfg(doc)]
use crate::cells::LifeCell;

/// The default algorithm based on David Bell's
/// [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct LifeSrc;

impl Sealed for LifeSrc {}

impl<R: Rule> Algorithm<R> for LifeSrc {
    type Reason = Reason;

    type ConflReason = ();

    fn new() -> Self {
        Self
    }

    #[inline]
    fn confl_from_cell(_cell: CellRef<R>) -> Self::ConflReason {}

    #[inline]
    fn confl_from_sym(_cell: CellRef<R>, _sym: CellRef<R>) -> Self::ConflReason {}

    #[inline]
    fn init_front(world: World<R, Self>) -> World<R, Self> {
        world
    }

    #[inline]
    fn set_cell(
        world: &mut World<R, Self>,
        cell: CellRef<R>,
        state: State,
        reason: Self::Reason,
    ) -> Result<(), Self::ConflReason> {
        world.set_cell_impl(cell, state, reason)
    }

    #[inline]
    fn go(world: &mut World<R, Self>, step: &mut u64) -> bool {
        world.go(step)
    }

    #[inline]
    fn retreat(world: &mut World<R, Self>) -> bool {
        world.retreat_impl()
    }

    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    #[inline]
    fn deser_reason(_world: &World<R, Self>, ser: &ReasonSer) -> Result<Self::Reason, Error> {
        Ok(match *ser {
            ReasonSer::Known => Reason::Known,
            ReasonSer::Decide => Reason::Decide,
            ReasonSer::TryAnother(n) => Reason::TryAnother(n),
            _ => Reason::Deduce,
        })
    }
}

/// Reasons for setting a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reason {
    /// Known before the search starts,
    Known,

    /// Decides the state of a cell by choice.
    Decide,

    /// Determines the state of a cell by other cells.
    Deduce,

    /// Tries another state of a cell when the original state
    /// leads to a conflict.
    ///
    /// Remembers the number of remaining states to try.
    ///
    /// Only used in Generations rules.
    TryAnother(usize),
}

impl<R: Rule> TraitReason<R> for Reason {
    const KNOWN: Self = Self::Known;
    const DECIDED: Self = Self::Decide;

    #[inline]
    fn from_cell(_cell: CellRef<R>) -> Self {
        Self::Deduce
    }

    #[inline]
    fn from_sym(_cell: CellRef<R>) -> Self {
        Self::Deduce
    }

    #[inline]
    fn is_decided(&self) -> bool {
        matches!(self, Self::Decide | Self::TryAnother(_))
    }

    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    #[inline]
    fn ser(&self) -> ReasonSer {
        match self {
            Self::Known => ReasonSer::Known,
            Self::Decide => ReasonSer::Decide,
            Self::Deduce => ReasonSer::Deduce,
            Self::TryAnother(n) => ReasonSer::TryAnother(*n),
        }
    }
}

impl<R: Rule> World<R, LifeSrc> {
    /// Sets the [`state`](LifeCell#structfield.state) of a cell,
    /// push it to the [`set_stack`](#structfield.set_stack),
    /// and update the neighborhood descriptor of its neighbors.
    ///
    /// The original state of the cell must be unknown.
    ///
    /// Return `false` if the number of living cells exceeds the
    /// [`max_cell_count`](#structfield.max_cell_count) or the front becomes empty.
    pub(crate) fn set_cell_impl(
        &mut self,
        cell: CellRef<R>,
        state: State,
        reason: Reason,
    ) -> Result<(), ()> {
        cell.state.set(Some(state));
        let mut result = Ok(());
        cell.update_desc(Some(state), true);
        if state == !cell.background {
            self.cell_count[cell.coord.2 as usize] += 1;
            if let Some(max) = self.config.max_cell_count {
                if self.cell_count() > max {
                    result = Err(());
                }
            }
        }
        if cell.is_front && state == cell.background {
            self.front_cell_count -= 1;
            if self.non_empty_front && self.front_cell_count == 0 {
                result = Err(());
            }
        }
        self.set_stack.push(SetCell::new(cell, reason));
        result
    }

    /// Retreats to the last time when a unknown cell is decided by choice,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    fn retreat_impl(&mut self) -> bool {
        while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
            match reason {
                Reason::Decide => {
                    let (state, reason) = if R::IsGen::VALUE {
                        let State(j) = cell.state.get().unwrap();
                        (
                            State((j + 1) % self.rule.gen()),
                            Reason::TryAnother(self.rule.gen() - 2),
                        )
                    } else {
                        (!cell.state.get().unwrap(), Reason::Deduce)
                    };

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.clear_cell(cell);
                    if self.set_cell_impl(cell, state, reason).is_ok() {
                        return true;
                    }
                }
                Reason::TryAnother(n) => {
                    let State(j) = cell.state.get().unwrap();
                    let state = State((j + 1) % self.rule.gen());
                    let reason = if n == 1 {
                        Reason::Deduce
                    } else {
                        Reason::TryAnother(n - 1)
                    };

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.clear_cell(cell);
                    if self.set_cell_impl(cell, state, reason).is_ok() {
                        return true;
                    }
                }
                Reason::Known => {
                    break;
                }
                Reason::Deduce => {
                    self.clear_cell(cell);
                }
            }
        }
        self.set_stack.clear();
        self.check_index = 0;
        self.next_unknown = None;
        false
    }

    /// Keeps proceeding and backtracking,
    /// until there are no more cells to examine (and returns `true`),
    /// or the backtracking goes back to the time before the first cell is set
    /// (and returns `false`).
    ///
    /// It also records the number of steps it has walked in the parameter
    /// `step`. A step consists of a [`proceed`](Self::proceed) and a [`retreat`](Self::retreat).
    fn go(&mut self, step: &mut u64) -> bool {
        loop {
            *step += 1;
            if self.proceed().is_ok() {
                return true;
            } else {
                self.conflicts += 1;
                if !self.retreat_impl() {
                    return false;
                }
            }
        }
    }
}
