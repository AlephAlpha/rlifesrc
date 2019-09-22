//! Cells in the cellular automaton.

use crate::rules::Desc;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::cell::Cell;
pub use State::{Alive, Dead};

#[cfg(feature = "stdweb")]
use serde::{Deserialize, Serialize};

/// Possible states of a known cell.
///
/// During the search, the state of a cell is represented by `Option<State>`,
/// where `None` means that the state of the cell is unknown.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
pub enum State {
    Alive,
    Dead,
}

/// Randomly chooses between `Alive` and `Dead`.
///
/// The probability of either state is 1/2.
impl Distribution<State> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> State {
        match rng.gen_range(0, 2) {
            0 => Dead,
            _ => Alive,
        }
    }
}

/// A cell in the cellular automaton.
///
/// The name `LifeCell` is chosen to avoid ambiguity with
/// [`std::cell::Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html).
pub struct LifeCell<'a, D: Desc> {
    /// The state of the cell.
    ///
    /// `None` means that the state of the cell is unknown.
    pub state: Cell<Option<State>>,

    /// The “neighborhood descriptors” of the cell.
    ///
    /// It describes the states of the neighboring cells.
    pub desc: Cell<D>,

    /// Whether the decision of the state depends on other cells.
    ///
    /// For known cells, `true` means that the decision is free, while
    /// `false` means that its state is implied by some other cells.
    ///
    /// For unknown cells, `false` means that we don't care about the cell's
    /// state, so that it is always unknown, even if its state can be implied
    /// by cells.
    pub free: Cell<bool>,

    /// The cell in the last generation at the same position.
    pub pred: Option<&'a LifeCell<'a, D>>,
    /// The cell in the next generation at the same position.
    pub succ: Option<&'a LifeCell<'a, D>>,
    /// The eight cells in the neighborhood.
    pub nbhd: [Option<&'a LifeCell<'a, D>>; 8],
    /// The cells in the same generation that must has the same state
    /// with this cell because of the symmetry.
    pub sym: Vec<&'a LifeCell<'a, D>>,
    /// Whether the cell is in the first generation.
    pub first_gen: bool,
    /// Whether the cell is on the first row or column.
    ///
    /// Here the choice of row or column depends on the search order.
    pub first_col: bool,
}

/// The state of a default cell is `Dead`.
/// Its neighborhood descriptor says that all neighboring cells are dead.
///
/// All references to other cells are `None`.
/// `free`, `first_gen` and `first_col` are set to `false`.
impl<'a, D: Desc> Default for LifeCell<'a, D> {
    fn default() -> Self {
        LifeCell {
            state: Cell::new(Some(Dead)),
            desc: Cell::new(D::new(Some(Dead))),
            free: Cell::new(false),
            pred: Default::default(),
            succ: Default::default(),
            nbhd: Default::default(),
            sym: Default::default(),
            first_gen: false,
            first_col: false,
        }
    }
}
