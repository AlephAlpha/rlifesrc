//! The searching algorithms.

use crate::{
    cells::{CellRef, State},
    config::NewState,
    rules::Rule,
    world::World,
};
use rand::{thread_rng, Rng};

mod backjump;
mod no_backjump;

pub use backjump::ReasonBackjump;
pub use no_backjump::ReasonNoBackjump;

#[cfg(feature = "serde")]
use crate::{
    error::Error,
    save::{ReasonSer, SetCellSer},
};
#[cfg(feature = "serde")]
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
    /// Paused.
    Paused,
}

/// Reasons for setting a cell.
///
/// Different choices of reason set will result in
/// different choices of algorithm.
///
/// Some details of this trait is hidden in the doc.
/// Please use the following structs instead of implementing by yourself:
/// - [`ReasonBackjump`]
/// - [`ReasonNoBackjump`]
pub trait Reason<'a, R: Rule>: Sized {
    /// Known before the search starts,
    const KNOWN: Self;

    /// Decides the state of a cell by choice.
    const DECIDED: Self;

    /// Whether to record the level.
    const LEVEL: bool;

    /// Deduced from the rule when constitifying another cell.
    fn from_cell(cell: CellRef<'a, R>) -> Self;

    /// Decided or trying another state for generations rules.
    fn is_decided(&self) -> bool;

    /// Keeps proceeding and backtracking,
    /// until there are no more cells to examine (and returns `true`),
    /// or the backtracking goes back to the time before the first cell is set
    /// (and returns `false`).
    ///
    /// It also records the number of steps it has walked in the parameter
    /// `step`.
    #[doc(hidden)]
    fn go(world: &mut World<'a, R, Self>, step: &mut u64) -> bool;

    /// Retreats to the last time when a unknown cell is decided by choice,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    #[doc(hidden)]
    fn retreat(world: &mut World<'a, R, Self>) -> bool;

    #[doc(hidden)]
    /// Deduces all cells that could be deduced before the first decision.
    fn presearch(world: World<'a, R, Self>) -> World<'a, R, Self>;

    #[cfg(feature = "serde")]
    /// Saves the reason as a [`ReasonSer`].
    fn ser(&self) -> ReasonSer;

    #[cfg(feature = "serde")]
    /// Restore the reason from a [`ReasonSer`].
    fn deser(ser: &ReasonSer, world: &World<'a, R, Self>) -> Result<Self, Error>;
}

/// Records the cells whose values are set and their reasons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetCell<'a, R: Rule, RE: Reason<'a, R>> {
    /// The set cell.
    pub(crate) cell: CellRef<'a, R>,

    /// The reason for setting a cell.
    pub(crate) reason: RE,
}

impl<'a, R: Rule, RE: Reason<'a, R>> SetCell<'a, R, RE> {
    /// Get a reference to the set cell.
    pub(crate) fn new(cell: CellRef<'a, R>, reason: RE) -> Self {
        SetCell { cell, reason }
    }

    #[cfg(feature = "serde")]
    /// Saves the [`SetCell`] as a [`SetCellSer`].
    pub(crate) fn ser(&self) -> SetCellSer {
        SetCellSer {
            coord: self.cell.coord,
            state: self.cell.state.get().unwrap(),
            reason: self.reason.ser(),
        }
    }
}

impl<'a, R: Rule, RE: Reason<'a, R>> World<'a, R, RE> {
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
            Some(self.set_cell(cell, state, RE::DECIDED))
        } else {
            None
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
        if self.next_unknown.is_none() && !RE::retreat(self) {
            return Status::None;
        }
        while RE::go(self, &mut step_count) {
            if let Some(result) = self.decide() {
                if !result && !RE::retreat(self) {
                    return Status::None;
                }
            } else if !self.is_boring() {
                if self.config.reduce_max {
                    self.config.max_cell_count = Some(self.cell_count() - 1);
                }
                return Status::Found;
            } else if !RE::retreat(self) {
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
