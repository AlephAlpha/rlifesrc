//! Cells in the cellular automaton.

use crate::rules::Rule;
use derivative::Derivative;
use std::{
    cell::Cell,
    fmt::{Debug, Error, Formatter},
    ops::{Deref, Not},
};

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
pub struct State(pub usize);

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

/// The coordinates of a cell.
///
/// `(x-coordinate, y-coordinate, time)`.
/// All three coordinates are 0-indexed.
pub type Coord = (isize, isize, isize);

/// A cell in the cellular automaton.
///
/// The name `LifeCell` is chosen to avoid ambiguity with
/// [`std::cell::Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html).
pub struct LifeCell<'a, R: Rule> {
    /// The coordinates of a cell.
    pub coord: Coord,

    /// The background state of the cell.
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
    pub(crate) fn new(coord: Coord, background: State, b0: bool) -> Self {
        let succ_state = if b0 { !background } else { background };
        LifeCell {
            coord,
            background,
            state: Cell::new(Some(background)),
            desc: Cell::new(R::new_desc(background, succ_state)),
            pred: Default::default(),
            succ: Default::default(),
            nbhd: Default::default(),
            sym: Default::default(),
            is_front: false,
        }
    }

    /// Returns a `CellRef` from a `LifeCell`.
    pub(crate) fn borrow(&self) -> CellRef<'a, R> {
        let cell = unsafe { (self as *const LifeCell<'a, R>).as_ref().unwrap() };
        CellRef { cell }
    }
}

impl<'a, R: Rule<Desc = D>, D: Copy + Debug> Debug for LifeCell<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "LifeCell {{ coord: {:?}, state: {:?}, desc: {:?} }}",
            self.coord,
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
    /// Updates the neighborhood descriptors of all neighbors and the predecessor
    /// when the state of one cell is changed.
    ///
    /// The `state` is the new state of the cell when `new` is true,
    /// the old state when `new` is false.
    pub(crate) fn update_desc(self, state: Option<State>, new: bool) {
        R::update_desc(self, state, new);
    }
}

impl<'a, R: Rule> PartialEq for CellRef<'a, R> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.cell, other.cell)
    }
}

impl<'a, R: Rule> Eq for CellRef<'a, R> {}

impl<'a, R: Rule> Deref for CellRef<'a, R> {
    type Target = LifeCell<'a, R>;

    fn deref(&self) -> &Self::Target {
        self.cell
    }
}

impl<'a, R: Rule<Desc = D>, D: Copy + Debug> Debug for CellRef<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "CellRef {{ coord: {:?} }}", self.coord)
    }
}
