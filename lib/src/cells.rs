//! Cells in the cellular automaton.

use crate::rules::Rule;
use educe::Educe;
use std::{
    cell::Cell,
    fmt::{Debug, Error, Formatter},
    hash::{Hash, Hasher},
    ops::{Deref, Not},
    ptr,
};

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
use serde::{Deserialize, Serialize};

/// Possible states of a known cell.
///
/// During the search, the state of a cell is represented by `Option<State>`,
/// where `None` means that the state of the cell is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct State(pub usize);

/// The Dead state.
pub const DEAD: State = State(0);
/// The Alive state.
pub const ALIVE: State = State(1);

/// Flips the state.
///
/// For Generations rules, the `not` of a dying state is [`ALIVE`].
impl Not for State {
    type Output = Self;

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
pub type Coord = (i32, i32, i32);

/// A cell in the cellular automaton.
///
/// The name `LifeCell` is chosen to avoid ambiguity with
/// [`std::cell::Cell`].
pub struct LifeCell<'a, R: Rule> {
    /// The coordinates of a cell.
    pub coord: Coord,

    /// The background state of the cell.
    ///
    /// For rules without `B0`, it is always [`DEAD`].
    ///
    /// For rules with `B0`, the background changes periodically.
    /// For example, for non-Generations rules, it is [`DEAD`] on even generations,
    /// [`ALIVE`] on odd generations.
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

    /// The next cell to be searched when searching for an unknown cell.
    pub(crate) next: Option<CellRef<'a, R>>,

    /// Whether the cell is on the first row or column.
    ///
    /// Here the choice of row or column depends on the search order.
    pub(crate) is_front: bool,

    /// The decision level for assigning the cell state.
    ///
    /// Only used when backjumping is enabled.
    pub(crate) level: Cell<u32>,

    /// Whether the cell has been seen in the analysis.
    ///
    /// Only used when backjumping is enabled.
    pub(crate) seen: Cell<bool>,
}

impl<'a, R: Rule> LifeCell<'a, R> {
    /// Generates a new cell with background state, such that its neighborhood
    /// descriptor says that all neighboring cells also have the same state.
    ///
    /// `is_front` are set to `false`.
    pub(crate) fn new(coord: Coord, background: State, succ_state: State) -> Self {
        LifeCell {
            coord,
            background,
            state: Cell::new(Some(background)),
            desc: Cell::new(R::new_desc(background, succ_state)),
            pred: Default::default(),
            succ: Default::default(),
            nbhd: Default::default(),
            sym: Default::default(),
            next: Default::default(),
            is_front: false,
            level: Cell::new(0),
            seen: Cell::new(false),
        }
    }

    /// Returns a [`CellRef`] from a [`LifeCell`].
    pub(crate) fn borrow(&self) -> CellRef<'a, R> {
        let cell = unsafe { (self as *const LifeCell<'a, R>).as_ref().unwrap() };
        CellRef { cell }
    }
}

impl<'a, R: Rule<Desc = D>, D: Copy + Debug> Debug for LifeCell<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("LifeCell")
            .field("coord", &self.coord)
            .field("state", &self.state.get())
            .field("desc", &self.desc.get())
            .field("is_front", &self.is_front)
            .finish()
    }
}

/// A reference to a [`LifeCell`] which has the same lifetime as the cell
/// it refers to.
#[derive(Educe)]
#[educe(Clone, Copy, Eq)]
pub struct CellRef<'a, R: Rule> {
    cell: &'a LifeCell<'a, R>,
}

impl<'a, R: Rule> CellRef<'a, R> {
    /// Updates the neighborhood descriptors of all neighbors and the predecessor
    /// when the state of one cell is changed.
    ///
    /// Here `state` is the new state of the cell when `new` is true,
    /// the old state when `new` is false.
    pub(crate) fn update_desc(self, state: Option<State>, new: bool) {
        R::update_desc(self, state, new);
    }
}

impl<'a, R: Rule> PartialEq for CellRef<'a, R> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.cell, other.cell)
    }
}

impl<'a, R: Rule> Deref for CellRef<'a, R> {
    type Target = LifeCell<'a, R>;

    fn deref(&self) -> &Self::Target {
        self.cell
    }
}

impl<'a, R: Rule> Debug for CellRef<'a, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("CellRef")
            .field("coord", &self.coord)
            .finish()
    }
}

impl<'a, R: Rule> Hash for CellRef<'a, R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self.cell, state)
    }
}
