//! Cellular automata rules.
//!
//! For the notations of rule strings, please see
//! [this article on LifeWiki](https://conwaylife.com/wiki/Rulestring).

mod life;
mod ntlife;

use crate::{
    cells::{CellRef, LifeCell, State},
    config::Symmetry,
    search::Algorithm,
    world::World,
};
pub use life::{Life, LifeGen};
pub use ntlife::{NtLife, NtLifeGen};

#[cfg(doc)]
use crate::cells::ALIVE;

use typebool::Bool;

/// Type level boolean values.
pub(crate) mod typebool {
    /// A type level boolean value.
    pub trait Bool {
        /// The runtime boolean value.
        const VALUE: bool;
    }

    /// Type level `true`.
    #[derive(Debug, Clone, Copy)]
    pub struct True;

    impl Bool for True {
        const VALUE: bool = true;
    }

    /// Type level `false`.
    #[derive(Debug, Clone, Copy)]
    pub struct False;

    impl Bool for False {
        const VALUE: bool = false;
    }
}

/// A cellular automaton rule.
///
/// The following rules are supported:
///
/// - [`Life`]
/// - [`LifeGen`]
/// - [`NtLife`]
/// - [`NtLifeGen`]
///
/// This trait is sealed and cannot be implemented outside of this crate.
#[cfg_attr(not(github_io), doc = "Some of its items are hidden in the doc.")]
pub trait Rule: private::Sealed {
    /// The type of neighborhood descriptor of the rule.
    ///
    /// It describes the states of the successor and neighbors of a cell,
    /// and is used to determine the state of the cell in the next generation.
    type Desc: Copy;

    /// Whether the rule is a Generations rule.
    type IsGen: Bool;

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

    /// The symmetry of the rule.
    fn symmetry(&self) -> Symmetry;

    /// Generates a neighborhood descriptor which says that all neighboring
    /// cells have states `state`, and the successor has state `succ_state`.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn new_desc(state: State, succ_state: State) -> Self::Desc;

    /// Updates the neighborhood descriptors of all neighbors and the predecessor
    /// when the state of one cell is changed.
    ///
    /// The `state` is the new state of the cell when `new` is true,
    /// the old state when `new` is false.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn update_desc(cell: &LifeCell<Self>, state: State, new: bool);

    /// Consistifies a cell.
    ///
    /// Examines the state and the neighborhood descriptor of the cell,
    /// and makes sure that it can validly produce the cell in the next
    /// generation. If possible, determines the states of some of the
    /// cells involved.
    ///
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    #[cfg_attr(not(github_io), doc(hidden))]
    fn consistify<A: Algorithm<Self>>(
        world: &mut World<Self, A>,
        cell: CellRef<Self>,
    ) -> Result<(), A::ConflReason>;
}

/// A helper mod for [sealing](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
/// the [`Rule`] trait.
mod private {
    /// A helper trait for [sealing](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
    /// the [`Rule`](super::Rule) trait.
    pub trait Sealed: Sized {}
}
