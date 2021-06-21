//! Reasons for setting a cell.

use crate::{cells::CellRef, rules::Rule};

#[cfg(feature = "serde")]
use crate::save::ReasonSer;

/// Reasons for setting a cell.
pub trait Reason<R: Rule> {
    /// Known before the search starts,
    const KNOWN: Self;

    /// Decides the state of a cell by choice.
    const DECIDED: Self;

    /// Deduced from the rule when constitifying another cell.
    fn from_cell(cell: CellRef<R>) -> Self;

    /// Deduced from symmetry.
    fn from_sym(cell: CellRef<R>) -> Self;

    /// Decided or trying another state for generations rules.
    fn is_decided(&self) -> bool;

    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    /// Saves the reason as a [`ReasonSer`].
    fn ser(&self) -> ReasonSer;
}
