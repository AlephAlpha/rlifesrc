//! The search process, without backjumping.
use crate::{
    cells::{CellRef, State},
    rules::Rule,
    search::{Reason, SetCell},
    world::World,
};

#[cfg(feature = "serde")]
use crate::{error::Error, save::ReasonSer};

/// Reasons for setting a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReasonNoBackjump {
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

impl<'a, R: Rule + 'a> Reason<'a, R> for ReasonNoBackjump {
    type ConflReason = ();
    const KNOWN: Self = ReasonNoBackjump::Known;
    const DECIDED: Self = ReasonNoBackjump::Decide;

    #[inline]
    fn from_cell(_cell: CellRef<'a, R>) -> Self {
        ReasonNoBackjump::Deduce
    }

    #[inline]
    fn from_sym(_cell: CellRef<'a, R>) -> Self {
        ReasonNoBackjump::Deduce
    }

    #[inline]
    fn is_decided(&self) -> bool {
        matches!(
            self,
            ReasonNoBackjump::Decide | ReasonNoBackjump::TryAnother(_)
        )
    }

    #[inline]
    fn confl_from_cell(_cell: CellRef<'a, R>) -> Self::ConflReason {}

    #[inline]
    fn confl_from_sym(_cell: CellRef<'a, R>, _sym: CellRef<'a, R>) -> Self::ConflReason {}

    #[inline]
    fn init_front(world: World<'a, R, Self>) -> World<'a, R, Self> {
        world
    }

    #[inline]
    fn set_cell(
        self,
        world: &mut World<'a, R, Self>,
        cell: CellRef<'a, R>,
        state: State,
    ) -> Result<(), Self::ConflReason> {
        world.set_cell_impl(cell, state, self)
    }

    #[inline]
    fn go(world: &mut World<'a, R, Self>, step: &mut u64) -> bool {
        world.go(step)
    }

    #[inline]
    fn retreat(world: &mut World<'a, R, Self>) -> bool {
        world.retreat_impl()
    }

    #[cfg(feature = "serde")]
    fn ser(&self) -> ReasonSer {
        match self {
            ReasonNoBackjump::Known => ReasonSer::Known,
            ReasonNoBackjump::Decide => ReasonSer::Decide,
            ReasonNoBackjump::Deduce => ReasonSer::Deduce,
            ReasonNoBackjump::TryAnother(n) => ReasonSer::TryAnother(*n),
        }
    }

    #[cfg(feature = "serde")]
    fn deser(ser: &ReasonSer, _world: &World<'a, R, Self>) -> Result<Self, Error> {
        Ok(match *ser {
            ReasonSer::Known => ReasonNoBackjump::Known,
            ReasonSer::Decide => ReasonNoBackjump::Decide,
            ReasonSer::TryAnother(n) => ReasonNoBackjump::TryAnother(n),
            _ => ReasonNoBackjump::Deduce,
        })
    }
}

impl<'a, R: Rule> World<'a, R, ReasonNoBackjump> {
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
        cell: CellRef<'a, R>,
        state: State,
        reason: ReasonNoBackjump,
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
                ReasonNoBackjump::Decide => {
                    let state;
                    let reason;
                    if R::IS_GEN {
                        let State(j) = cell.state.get().unwrap();
                        state = State((j + 1) % self.rule.gen());
                        reason = ReasonNoBackjump::TryAnother(self.rule.gen() - 2);
                    } else {
                        state = !cell.state.get().unwrap();
                        reason = ReasonNoBackjump::Deduce;
                    }

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.clear_cell(cell);
                    if self.set_cell_impl(cell, state, reason).is_ok() {
                        return true;
                    }
                }
                ReasonNoBackjump::TryAnother(n) => {
                    let State(j) = cell.state.get().unwrap();
                    let state = State((j + 1) % self.rule.gen());
                    let reason = if n == 1 {
                        ReasonNoBackjump::Deduce
                    } else {
                        ReasonNoBackjump::TryAnother(n - 1)
                    };

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.clear_cell(cell);
                    if self.set_cell_impl(cell, state, reason).is_ok() {
                        return true;
                    }
                }
                ReasonNoBackjump::Known => {
                    break;
                }
                ReasonNoBackjump::Deduce => {
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
