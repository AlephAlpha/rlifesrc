use crate::cells::Coord;
use ca_rules::ParseRuleError;
use std::{
    error,
    fmt::{self, Display, Formatter},
};

/// All kinds of errors in this crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Errors when trying to get the state of a cell.
    GetCellError(Coord),
    /// Errors when trying to set a cell.
    SetCellError(Coord),
    /// Errors when parsing rule strings.
    ParseRuleError(ParseRuleError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::GetCellError(coord) => write!(f, "Unable to get the state of cell {:?}.", coord),
            Error::SetCellError(coord) => write!(f, "Unable to set cell at {:?}.", coord),
            Error::ParseRuleError(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {}
