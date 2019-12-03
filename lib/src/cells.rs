//! Cells in the cellular automaton.

use crate::rules::Rule;
use derivative::Derivative;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{
    cell::Cell,
    fmt::{Debug, Error, Formatter},
    ops::{Deref, Not},
};
pub use State::{Alive, Dead};

#[cfg(feature = "stdweb")]
use serde::{Deserialize, Serialize};

/// Possible states of a known cell.
///
/// During the search, the state of a cell is represented by `Option<State>`,
/// where `None` means that the state of the cell is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
pub enum State {
    Alive = 0b01,
    Dead = 0b10,
}

/// Flips the state.
impl Not for State {
    type Output = State;

    fn not(self) -> Self::Output {
        match self {
            Alive => Dead,
            Dead => Alive,
        }
    }
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
pub struct LifeCell<'a, R: Rule> {
    /// The background state of a cell.
    ///
    /// For rules without `B0`, it is always dead.
    /// For rules with `B0`, it is dead on even generations,
    /// alive on odd generations.
    pub(crate) background: State,

    /// The state of the cell.
    ///
    /// `None` means that the state of the cell is unknown.
    pub(crate) state: Cell<Option<State>>,

    /// The “neighborhood descriptors” of the cell.
    ///
    /// It describes the states of the cell itself, its neighbors,
    /// and its successor.
    pub(crate) desc: Cell<R::Desc>,

    /// The predecessor of the cell.
    ///
    /// The cell in the last generation at the same position.
    pub(crate) pred: Option<CellRef<'a, R>>,
    /// The successor of the cell.
    ///
    /// The cell in the next generation at the same position.
    pub(crate) succ: Option<CellRef<'a, R>>,
    /// The eight cells in the neighborhood.
    pub(crate) nbhd: [Option<CellRef<'a, R>>; 8],
    /// The cells in the same generation that must has the same state
    /// with this cell because of the symmetry.
    pub(crate) sym: Vec<CellRef<'a, R>>,

    /// The generation of the cell.
    pub(crate) gen: usize,
    /// Whether the cell is on the first row or column.
    ///
    /// Here the choice of row or column depends on the search order.
    pub(crate) is_front: bool,
}

impl<'a, R: Rule> LifeCell<'a, R> {
    /// Generates a new cell with state `state`, such that its neighborhood
    /// descriptor says that all neighboring cells also have the same state.
    ///
    /// `first_gen` and `first_col` are set to `false`.
    pub(crate) fn new(background: State, b0: bool, gen: usize) -> Self {
        let succ_state = if b0 { !background } else { background };
        LifeCell {
            background,
            state: Cell::new(Some(background)),
            desc: Cell::new(R::new_desc(background, succ_state)),
            pred: Default::default(),
            succ: Default::default(),
            nbhd: Default::default(),
            sym: Default::default(),
            gen,
            is_front: false,
        }
    }

    pub(crate) unsafe fn to_ref(&self) -> CellRef<'a, R> {
        let cell = (self as *const LifeCell<'a, R>).as_ref().unwrap();
        CellRef { cell }
    }
}

impl<'a, R: Rule<Desc = D>, D: Copy + Debug> Debug for LifeCell<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "LifeCell {{ state: {:?}, desc: {:?} }}",
            self.state.get(),
            self.desc.get()
        )
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct CellRef<'a, R: Rule> {
    cell: &'a LifeCell<'a, R>,
}

impl<'a, R: Rule> CellRef<'a, R> {
    pub(crate) fn update_desc(self, old_state: Option<State>, state: Option<State>) {
        R::update_desc(self, old_state, state);
    }
}

impl<'a, R: Rule> Deref for CellRef<'a, R> {
    type Target = LifeCell<'a, R>;

    fn deref(&self) -> &Self::Target {
        self.cell
    }
}

impl<'a, R: Rule> PartialEq for CellRef<'a, R> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.cell, other.cell)
    }
}

impl<'a, R: Rule> Eq for CellRef<'a, R> {}

impl<'a, R: Rule<Desc = D>, D: Copy + Debug> Debug for CellRef<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "CellRef {{ cell: {:?} }}", self.cell)
    }
}
