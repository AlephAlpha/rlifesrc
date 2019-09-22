//! Cellular automata rules.

pub mod life;
pub mod ntlife;

use crate::{
    cells::{LifeCell, State},
    world::World,
};

/// A trait for “neighborhood descriptors”.
///
/// It describes the states of the neighbors of a cell,
/// and is used to determine the state of the cell in the next generation.
pub trait Desc: Copy {
    /// Generates a neighborhood descriptor which says that all neighboring
    /// cells have states `state`.
    fn new(state: Option<State>) -> Self;

    /// Updates the neighborhood descriptors of all neighbors when the state
    /// of one cell is changed.
    fn update_desc(cell: &LifeCell<Self>, old_state: Option<State>, state: Option<State>);
}

/// A cellular automaton rule.
pub trait Rule: Sized {
    /// The type of neighborhood descriptor of the rule.
    type Desc: Desc;

    /// Whether the rule contains `B0`.
    ///
    /// In other words, whether a cell would become `alive` in the next
    /// generation, if all its neighbors in this generation are dead.
    fn b0(&self) -> bool;

    /// Given the states of a cell and its neighborhood descriptor,
    /// deduces the state of the cell in the next generation.
    fn transition(&self, state: Option<State>, desc: Self::Desc) -> Option<State>;

    /// Given the neighborhood descriptor of a cell, and the state in the next
    /// generation (which must be known),
    /// deduces its state in the current generation.
    fn implication(&self, desc: Self::Desc, succ_state: State) -> Option<State>;

    /// Given the states of a cell, its neighborhood descriptor, and the state
    /// in the next generation (which must be known),
    /// deduces the states of some of its unknown neighbors,
    /// and push a reference to each deduced cell into the `stack`.
    fn consistify_nbhd<'a>(
        &self,
        cell: &LifeCell<'a, Self::Desc>,
        world: &World<'a, Self::Desc, Self>,
        desc: Self::Desc,
        state: Option<State>,
        succ_state: State,
        stack: &mut Vec<&'a LifeCell<'a, Self::Desc>>,
    );
}
