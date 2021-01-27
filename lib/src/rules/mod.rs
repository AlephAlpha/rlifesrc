//! Cellular automata rules.
//!
//! For the notations of rule strings, please see
//! [this article on LifeWiki](https://conwaylife.com/wiki/Rulestring).

#[doc(hidden)]
mod macros;

mod life;
mod ntlife;

use crate::{
    cells::{CellRef, State},
    world::World,
};
pub use life::{Life, LifeGen};
pub use ntlife::{NtLife, NtLifeGen};

#[cfg(doc)]
use crate::cells::ALIVE;

/// A cellular automaton rule.
///
/// Some details of this trait is hidden in the doc.
/// Please use the following structs instead of implementing by yourself:
/// - [`Life`]
/// - [`LifeGen`]
/// - [`NtLife`]
/// - [`NtLifeGen`]
pub trait Rule: Sized {
    /// The type of neighborhood descriptor of the rule.
    ///
    /// It describes the states of the successor and neighbors of a cell,
    /// and is used to determine the state of the cell in the next generation.
    #[doc(hidden)]
    type Desc: Copy;

    /// Whether the rule is a Generations rule.
    const IS_GEN: bool;

    /// Whether the rule contains `B0`.
    ///
    /// In other words, whether a dead cell would become [`ALIVE`] in the next
    /// generation, if all its neighbors in this generation are dead.
    fn has_b0(&self) -> bool;

    /// Whether the rule contains both `B0` and `S8`.
    ///
    /// In a rule that contains `B0`, a dead cell would become [`ALIVE`] in the next
    /// generation, if all its neighbors in this generation are dead.
    ///
    /// In a rule that contains `S8`, a living cell would stay [`ALIVE`] in the next
    /// generation, if all its neighbors in this generation are alive.
    fn has_b0_s8(&self) -> bool;

    /// The number of states.
    fn gen(&self) -> usize;

    /// Generates a neighborhood descriptor which says that all neighboring
    /// cells have states `state`, and the successor has state `succ_state`.
    #[doc(hidden)]
    fn new_desc(state: State, succ_state: State) -> Self::Desc;

    /// Updates the neighborhood descriptors of all neighbors and the predecessor
    /// when the state of one cell is changed.
    ///
    /// The `state` is the new state of the cell when `new` is true,
    /// the old state when `new` is false.
    #[doc(hidden)]
    fn update_desc(cell: CellRef<Self>, state: Option<State>, new: bool);

    /// Consistifies a cell.
    ///
    /// Examines the state and the neighborhood descriptor of the cell,
    /// and makes sure that it can validly produce the cell in the next
    /// generation. If possible, determines the states of some of the
    /// cells involved.
    ///
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    #[doc(hidden)]
    fn consistify<'a>(world: &mut World<'a, Self>, cell: CellRef<'a, Self>) -> bool;
}
