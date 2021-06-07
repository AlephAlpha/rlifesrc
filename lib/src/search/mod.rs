//! The searching algorithms.

use crate::{
    cells::{CellRef, State},
    config::NewState,
    rules::Rule,
    world::World,
};
use rand::{thread_rng, Rng};

mod backjump;
mod lifesrc;

pub use backjump::Backjump;
pub use lifesrc::LifeSrc;

#[cfg(doc)]
use crate::cells::LifeCell;

#[cfg(feature = "serde")]
#[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
use crate::{
    error::Error,
    save::{ReasonSer, SetCellSer},
};
#[cfg(feature = "serde")]
#[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
use serde::{Deserialize, Serialize};

/// Search status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Status {
    /// Initial status. Waiting to start.
    Initial,
    /// A result is found.
    Found,
    /// Such pattern does not exist.
    None,
    /// Still searching.
    Searching,
}

/// The search algorithms.
///
/// Currently only two algorithms are supported:
///
/// - [`LifeSrc`]: The default algorithm based on David Bell's
///   [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch).
/// - [`Backjump`]: __(Experimental)__ Adding [Backjumping](https://en.wikipedia.org/wiki/Backjumping)
///   to the original lifesrc algorithm. Very slow. Do not use it.
///
/// This trait is sealed and cannot be implemented outside of this crate.
#[cfg_attr(not(github_io), doc = "Some details of it is hidden in the doc.")]
pub trait Algorithm<'a, R: Rule>: private::Sealed {
    /// Reasons for setting a cell.
    type Reason: Reason<'a, R>;

    /// Reasons for a conflict. Ignored in [`LifeSrc`] algorithm.
    #[cfg_attr(not(github_io), doc(hidden))]
    type ConflReason;

    /// Generate new algorithm data.
    fn new() -> Self;

    /// Conflict when constitifying a cell.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn confl_from_cell(cell: CellRef<'a, R>) -> Self::ConflReason;

    /// Conflict from symmetry.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn confl_from_sym(cell: CellRef<'a, R>, sym: CellRef<'a, R>) -> Self::ConflReason;

    /// Conflict when constitifying a cell.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn init_front(world: World<'a, R, Self>) -> World<'a, R, Self>;

    /// Sets the [`state`](LifeCell#structfield.state) of a cell,
    /// push it to the [`set_stack`](World#structfield.set_stack),
    /// and update the neighborhood descriptor of its neighbors.
    ///
    /// The original state of the cell must be unknown.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn set_cell(
        world: &mut World<'a, R, Self>,
        cell: CellRef<'a, R>,
        state: State,
        reason: Self::Reason,
    ) -> Result<(), Self::ConflReason>;

    /// Keeps proceeding and backtracking,
    /// until there are no more cells to examine (and returns `true`),
    /// or the backtracking goes back to the time before the first cell is set
    /// (and returns `false`).
    ///
    /// It also records the number of steps it has walked in the parameter
    /// `step`.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn go(world: &mut World<'a, R, Self>, step: &mut u64) -> bool;

    /// Retreats to the last time when a unknown cell is decided by choice,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn retreat(world: &mut World<'a, R, Self>) -> bool;

    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    /// Restore the reason from a [`ReasonSer`].
    fn deser_reason(world: &World<'a, R, Self>, ser: &ReasonSer) -> Result<Self::Reason, Error>;
}

/// Reasons for setting a cell.
pub trait Reason<'a, R: Rule> {
    /// Known before the search starts,
    const KNOWN: Self;

    /// Decides the state of a cell by choice.
    const DECIDED: Self;

    /// Deduced from the rule when constitifying another cell.
    fn from_cell(cell: CellRef<'a, R>) -> Self;

    /// Deduced from symmetry.
    fn from_sym(cell: CellRef<'a, R>) -> Self;

    /// Decided or trying another state for generations rules.
    fn is_decided(&self) -> bool;

    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    /// Saves the reason as a [`ReasonSer`].
    fn ser(&self) -> ReasonSer;
}

/// A helper mod for [sealing](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
/// the [`Algorithm`] trait.
mod private {
    /// A helper trait for [sealing](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
    /// the [`Algorithm`](super::Algorithm) trait.
    pub trait Sealed: Sized {}
}

/// Records the cells whose values are set and their reasons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetCell<'a, R: Rule, A: Algorithm<'a, R>> {
    /// The set cell.
    pub(crate) cell: CellRef<'a, R>,

    /// The reason for setting a cell.
    pub(crate) reason: A::Reason,
}

impl<'a, R: Rule, A: Algorithm<'a, R>> SetCell<'a, R, A> {
    /// Get a reference to the set cell.
    pub(crate) fn new(cell: CellRef<'a, R>, reason: A::Reason) -> Self {
        SetCell { cell, reason }
    }

    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    /// Saves the [`SetCell`] as a [`SetCellSer`].
    pub(crate) fn ser(&self) -> SetCellSer {
        SetCellSer {
            coord: self.cell.coord,
            state: self.cell.state.get().unwrap(),
            reason: self.reason.ser(),
        }
    }
}

impl<'a, R: Rule, A: Algorithm<'a, R>> World<'a, R, A> {
    /// Consistifies a cell.
    ///
    /// Examines the state and the neighborhood descriptor of the cell,
    /// and makes sure that it can validly produce the cell in the next
    /// generation. If possible, determines the states of some of the
    /// cells involved.
    ///
    /// If there is a conflict, returns its reason.
    fn consistify(&mut self, cell: CellRef<'a, R>) -> Result<(), A::ConflReason> {
        Rule::consistify(self, cell)
    }

    /// Consistifies a cell, its neighbors, and its predecessor.
    ///
    /// If there is a conflict, returns its reason.
    fn consistify10(&mut self, cell: CellRef<'a, R>) -> Result<(), A::ConflReason> {
        self.consistify(cell)?;

        if let Some(pred) = cell.pred {
            self.consistify(pred)?;
        }
        for &neigh in cell.nbhd.iter() {
            if let Some(neigh) = neigh {
                self.consistify(neigh)?;
            }
        }
        Ok(())
    }

    /// Deduces all the consequences by [`consistify`](Self::consistify) and symmetry.
    ///
    /// If there is a conflict, returns its reason.
    pub(crate) fn proceed(&mut self) -> Result<(), A::ConflReason> {
        while self.check_index < self.set_stack.len() as u32 {
            let cell = self.set_stack[self.check_index as usize].cell;
            let state = cell.state.get().unwrap();

            // Determines some cells by symmetry.
            for &sym in cell.sym.iter() {
                if let Some(old_state) = sym.state.get() {
                    if state != old_state {
                        return Err(A::confl_from_sym(cell, sym));
                    }
                } else {
                    self.set_cell(sym, state, A::Reason::from_sym(cell))?;
                }
            }

            // Determines some cells by `consistify`.
            self.consistify10(cell)?;

            self.check_index += 1;
        }
        Ok(())
    }

    /// Retreats to the last time when a unknown cell is decided by choice,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    pub(crate) fn retreat(&mut self) -> bool {
        A::retreat(self)
    }

    /// Makes a decision.
    ///
    /// Chooses an unknown cell, assigns a state for it,
    /// and push a reference to it to the [`set_stack`](#structfield.set_stack).
    ///
    /// Returns `None` is there is no unknown cell,
    /// `Some(false)` if the new state leads to an immediate conflict.
    fn decide(&mut self) -> Option<bool> {
        if let Some(cell) = self.get_unknown() {
            self.next_unknown = cell.next;
            let state = match self.config.new_state {
                NewState::ChooseDead => cell.background,
                NewState::ChooseAlive => !cell.background,
                NewState::Random => State(thread_rng().gen_range(0..self.rule.gen())),
            };
            Some(self.set_cell(cell, state, A::Reason::DECIDED).is_ok())
        } else {
            None
        }
    }

    /// Deduces all cells that could be deduced before the first decision.
    pub(crate) fn presearch(mut self) -> Self {
        loop {
            if self.proceed().is_ok() {
                self.set_stack.clear();
                self.check_index = 0;
                return self;
            } else {
                self.conflicts += 1;
                if !self.retreat() {
                    return self;
                }
            }
        }
    }

    /// The search function.
    ///
    /// Returns [`Status::Found`] if a result is found,
    /// [`Status::None`] if such pattern does not exist,
    /// [`Status::Searching`] if the number of steps exceeds `max_step`
    /// and no results are found.
    pub fn search(&mut self, max_step: Option<u64>) -> Status {
        let mut step_count = 0;
        if self.next_unknown.is_none() && !self.retreat() {
            return Status::None;
        }
        while A::go(self, &mut step_count) {
            if let Some(result) = self.decide() {
                if !result && !self.retreat() {
                    return Status::None;
                }
            } else if !self.is_boring() {
                if self.config.reduce_max {
                    self.config.max_cell_count = Some(self.cell_count() - 1);
                }
                return Status::Found;
            } else if !self.retreat() {
                return Status::None;
            }

            if let Some(max) = max_step {
                if step_count > max {
                    return Status::Searching;
                }
            }
        }
        Status::None
    }
}
