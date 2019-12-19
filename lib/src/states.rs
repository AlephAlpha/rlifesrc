use std::{fmt::Debug, ops::Not};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

pub const DEAD: State = State(0);
pub const ALIVE: State = State(1);

/// Possible states of a known cell.
///
/// During the search, the state of a cell is represented by `Option<State>`,
/// where `None` means that the state of the cell is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct State(pub(crate) usize);

/// Flips the state.
impl Not for State {
    type Output = State;

    fn not(self) -> Self::Output {
        match self {
            ALIVE => DEAD,
            _ => ALIVE,
        }
    }
}
