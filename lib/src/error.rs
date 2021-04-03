//! All kinds of errors in this crate.

use crate::cells::{Coord, State};
use ca_rules::ParseRuleError;
use displaydoc::Display;
use thiserror::Error;

/// All kinds of errors in this crate.
#[derive(Clone, Debug, PartialEq, Eq, Display, Error)]
pub enum Error {
    /// Unable to set cell at {0:?}.
    SetCellError(Coord),
    /// Invalid rule: {0:?}.
    ParseRuleError(#[from] ParseRuleError),
    /// B0S8 rules are not supported yet. Please use the inverted rule.
    B0S8Error,
    /// Symmetry or transformation requires the world to be square.
    SquareWorldError,
    /// Symmetry or transformation requires the world to have no diagonal width.
    DiagonalWidthError,
    /// Width / height / period should be positive.
    NonPositiveError,
    /// Cell at {0:?} has invalid state: {1:?}.
    InvalidState(Coord, State),
}
