mod backjump;
mod no_backjump;

use crate::{cells::CellRef, rules::Rule, world::World};
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
/// Some details of this trait is hidden in the doc.
/// Please use the following structs instead of implementing by yourself:
/// - [`ReasonBackJump`]
pub trait Reason<'a, R: Rule>: Sized {
    /// Known before the search starts,
    const KNOWN: Self;

    /// Decides the state of a cell by choice.
    const DECIDED: Self;

    /// Deduced from the rule when constitifying another cell.
    fn from_cell(cell: CellRef<'a, R>) -> Self;

    /// Decided or trying another state for generations rules.
    fn is_decided(&self) -> bool;

    /// The search function.
    ///
    /// Returns [`Status::Found`] if a result is found,
    /// [`Status::None`] if such pattern does not exist,
    /// [`Status::Searching`] if the number of steps exceeds `max_step`
    /// and no results are found.
    #[doc(hidden)]
    fn search(world: &mut World<'a, R, Self>, max_step: Option<u64>) -> Status;

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
