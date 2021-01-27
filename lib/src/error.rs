//! All kinds of errors in this crate.

use crate::cells::Coord;
use ca_rules::ParseRuleError;
use thiserror::Error;

/// All kinds of errors in this crate.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// Unable to get the state of a cell.
    #[error("Unable to get the state of cell {0:?}.")]
    GetCellError(Coord),
    /// Unable to set a cell.
    #[error("Unable to set cell at {0:?}.")]
    SetCellError(Coord),
    /// Invalid rule.
    #[error("Invalid rule: {0:?}.")]
    ParseRuleError(#[from] ParseRuleError),
    /// B0S8 rules are not supported yet. Please use the inverted rule.
    #[error("B0S8 rules are not supported yet. Please use the inverted rule.")]
    B0S8Error,
    /// Symmetry or transformation requires the world to be square.
    #[error("Symmetry or transformation requires the world to be square.")]
    SquareWorldError,
    /// Width / height / period should be positive.
    #[error("Width / height / period should be positive.")]
    NonPositiveError,
}
