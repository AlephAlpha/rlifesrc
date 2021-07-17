//! Cells in the cellular automaton.

use crate::rules::Rule;
use educe::Educe;
use std::{
    cell::Cell,
    fmt::{Debug, Error, Formatter},
    ops::{Deref, Not},
    ptr::NonNull,
};

#[cfg(feature = "serde")]
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

    #[inline]
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
pub struct LifeCell<R: Rule> {
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
    pub(crate) pred: Option<CellRef<R>>,
    /// The successor of the cell.
    ///
    /// The cell in the next generation at the same position.
    pub(crate) succ: Option<CellRef<R>>,
    /// The eight cells in the neighborhood.
    pub(crate) nbhd: [Option<CellRef<R>>; 8],
    /// The cells in the same generation that must has the same state
    /// with this cell because of the symmetry.
    pub(crate) sym: Vec<CellRef<R>>,

    /// The next cell to be searched when searching for an unknown cell.
    pub(crate) next: Option<CellRef<R>>,

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

impl<R: Rule> LifeCell<R> {
    /// Generates a new cell with background state, such that its neighborhood
    /// descriptor says that all neighboring cells also have the same state.
    ///
    /// `is_front` are set to `false`.
    #[inline]
    pub(crate) fn new(coord: Coord, background: State, succ_state: State) -> Self {
        Self {
            coord,
            background,
            state: Cell::new(Some(background)),
            desc: Cell::new(R::new_desc(background, succ_state)),
            pred: None,
            succ: None,
            nbhd: [None; 8],
            sym: Vec::new(),
            next: None,
            is_front: false,
            level: Cell::new(0),
            seen: Cell::new(false),
        }
    }

    /// Updates the neighborhood descriptors of all neighbors and the predecessor
    /// when the state of one cell is changed.
    ///
    /// Here `state` is the new state of the cell when `new` is true,
    /// the old state when `new` is false.
    pub(crate) fn update_desc(&self, state: Option<State>, new: bool) {
        R::update_desc(self, state, new);
    }
}

impl<R: Rule<Desc = D>, D: Copy + Debug> Debug for LifeCell<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("LifeCell")
            .field("coord", &self.coord)
            .field("state", &self.state.get())
            .field("desc", &self.desc.get())
            .field("is_front", &self.is_front)
            .finish()
    }
}

/// A reference to a [`LifeCell`]. It is just a wrapped [`NonNull`] pointer.
///
/// # Safety
///
/// This type is just a wrapped raw pointer. Dereferring a [`CellRef`] should
/// follow the same guarantees for dereferring a raw mut pointer.
///
/// Furthermore, a [`CellRef`] referring to a cell in one world should never be
/// used in any function or method involving another world.
#[repr(transparent)]
#[derive(Educe)]
#[educe(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellRef<R: Rule> {
    /// The [`LifeCell`] it refers to.
    cell: NonNull<LifeCell<R>>,
}

impl<R: Rule> CellRef<R> {
    /// Creates a new [`CellRef`] from a mut pointer to a [`LifeCell`].
    #[inline]
    pub(crate) unsafe fn new(ptr: *mut LifeCell<R>) -> Self {
        Self {
            cell: NonNull::new_unchecked(ptr),
        }
    }
}

impl<R: Rule> Deref for CellRef<R> {
    type Target = LifeCell<R>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.cell.as_ref() }
    }
}

impl<R: Rule> Debug for CellRef<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("CellRef")
            .field("coord", &self.coord)
            .finish()
    }
}
