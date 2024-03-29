//! World configuration.

use crate::{
    cells::{Coord, State},
    error::Error,
    poly_world::PolyWorld,
    rules::{Life, LifeGen, NtLife, NtLifeGen, Rule},
    world::World,
};
use educe::Educe;

#[cfg(feature = "read-rle")]
#[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "read-rle")))]
use ca_formats::{
    rle::{Error as RleError, Rle},
    CellData, Input,
};

mod d8;
mod search_order;

pub use d8::{Symmetry, Transform};
pub use search_order::SearchOrder;

#[cfg(doc)]
use crate::cells::{ALIVE, DEAD};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// How to choose a state for an unknown cell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NewState {
    /// Chooses the background state.
    ///
    /// For rules without `B0`, it always chooses [`DEAD`].
    ///
    /// For rules with `B0`, the background changes periodically.
    /// For example, for non-Generations rules,
    /// it chooses [`DEAD`] on even generations,
    /// [`ALIVE`] on odd generations.
    ChooseDead,

    /// Chooses the opposite of the background state.
    ///
    /// For rules without `B0`, it always chooses [`ALIVE`].
    ///
    /// For rules with `B0`, the background changes periodically.
    /// For example, for non-Generations rules,
    /// it chooses [`ALIVE`] on even generations,
    /// [`DEAD`] on odd generations.
    #[default]
    ChooseAlive,

    /// Random.
    ///
    /// For non-Generations rules,
    /// the probability of either state is `1/2`.
    ///
    /// For Generations rules with `n` states,
    /// the probability of each state is `1/n`.
    Random,
}

/// A cell whose state is known before the search.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KnownCell {
    /// The coordinates of the set cell.
    pub coord: Coord,

    /// The state.
    pub state: State,
}

#[cfg(feature = "read-rle")]
#[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "read-rle")))]
impl KnownCell {
    /// Convert a [`CellData`] to a [`KnownCell`].
    pub const fn from_cell_data(data: CellData, gen: i32) -> Self {
        let (x, y) = data.position;
        let coord = (x as i32, y as i32, gen);
        let state = State(data.state as usize);
        Self { coord, state }
    }

    /// Get a list [`KnownCell`] from multiple RLE's in one string.
    pub fn from_rles<I, L>(input: I) -> Result<Vec<Self>, RleError>
    where
        I: Input<Lines = L>,
        L: Input<Lines = L>,
    {
        let mut known_cells = Vec::new();
        let rle = Rle::new(input)?;

        Self::from_rles_iter(rle, &mut known_cells, 0)?;

        Ok(known_cells)
    }

    fn from_rles_iter<I, L>(
        rle: Rle<I>,
        known_cells: &mut Vec<Self>,
        gen: i32,
    ) -> Result<(), RleError>
    where
        I: Input<Lines = L>,
        L: Input<Lines = L>,
    {
        let mut rle = rle.with_unknown();

        for data in &mut rle {
            known_cells.push(Self::from_cell_data(data?, gen));
        }

        if let Some(rle) = rle.try_remains()? {
            Self::from_rles_iter(rle, known_cells, gen + 1)?;
        }

        Ok(())
    }
}

/// World configuration.
///
/// The world will be generated from this configuration.
#[derive(Clone, Debug, Educe, PartialEq, Eq, Hash)]
#[educe(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Config {
    /// Width.
    #[educe(Default = 16)]
    pub width: i32,

    /// Height.
    #[educe(Default = 16)]
    pub height: i32,

    /// Period.
    #[educe(Default = 1)]
    pub period: i32,

    /// Horizontal translation.
    pub dx: i32,

    /// Vertical translation.
    pub dy: i32,

    /// Transformations (rotations and reflections) after the last generation
    /// in a period.
    ///
    /// After the last generation in a period, the pattern will return to
    /// the first generation, applying this transformation first,
    /// and then the translation defined by `dx` and `dy`.
    pub transform: Transform,

    /// Symmetries of the pattern.
    pub symmetry: Symmetry,

    /// The order to find a new unknown cell.
    ///
    /// It will always search all generations of one cell
    /// before going to another cell.
    ///
    /// `None` means that it will automatically choose a search order
    /// according to the width and height of the world.
    pub search_order: Option<SearchOrder>,

    /// How to choose a state for an unknown cell.
    pub new_state: NewState,

    /// The number of minimum living cells in all generations must not
    /// exceed this number.
    ///
    /// `None` means that there is no limit for the cell count.
    pub max_cell_count: Option<u32>,

    /// Whether to automatically reduce the [`max_cell_count`](#structfield.max_cell_count)
    /// when a result is found.
    ///
    /// The [`max_cell_count`](#structfield.max_cell_count) will be set to the cell count of
    /// the current result minus one.
    pub reduce_max: bool,

    /// The rule string of the cellular automaton.
    #[educe(Default = "B3/S23")]
    pub rule_string: String,

    /// Diagonal width.
    ///
    /// If the diagonal width is `n`, the cells at position `(x, y)`
    /// where `abs(x - y) >= n` are assumed to be dead.
    pub diagonal_width: Option<i32>,

    /// Whether to skip patterns whose fundamental period are smaller than the given period.
    #[educe(Default = true)]
    pub skip_subperiod: bool,

    /// Whether to skip patterns which are invariant under more transformations than
    /// required by the given symmetry.
    ///
    /// In another word, whether to skip patterns whose symmetry group properly contains
    /// the given symmetry group.
    pub skip_subsymmetry: bool,

    /// Cells whose states are known before the search.
    pub known_cells: Vec<KnownCell>,

    /// __(Experimental)__ Whether to enable [backjumping](https://en.wikipedia.org/wiki/Backjumping).
    ///
    /// Backjumping will reduce the number of steps, but each step will takes
    /// a much longer time. The current implementation is slower for most search,
    /// only useful for large (e.g., 64x64) still lifes.
    ///
    /// Currently it is only supported for non-Generations rules. Generations rules
    /// will ignore this option.
    pub backjump: bool,
}

impl Config {
    /// Sets up a new configuration with given size.
    #[must_use]
    #[inline]
    pub fn new(width: i32, height: i32, period: i32) -> Self {
        Self {
            width,
            height,
            period,
            ..Self::default()
        }
    }

    /// Sets the translations `(dx, dy)`.
    #[must_use]
    #[inline]
    pub const fn set_translate(mut self, dx: i32, dy: i32) -> Self {
        self.dx = dx;
        self.dy = dy;
        self
    }

    /// Sets the transformation.
    #[must_use]
    #[inline]
    pub const fn set_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Sets the symmetry.
    #[must_use]
    #[inline]
    pub const fn set_symmetry(mut self, symmetry: Symmetry) -> Self {
        self.symmetry = symmetry;
        self
    }

    /// Sets the search order.
    #[must_use]
    #[inline]
    pub fn set_search_order<T: Into<Option<SearchOrder>>>(mut self, search_order: T) -> Self {
        self.search_order = search_order.into();
        self
    }

    /// Sets how to choose a state for an unknown cell.
    #[must_use]
    #[inline]
    pub const fn set_new_state(mut self, new_state: NewState) -> Self {
        self.new_state = new_state;
        self
    }

    /// Sets the maximal number of living cells.
    #[must_use]
    #[inline]
    pub fn set_max_cell_count<T: Into<Option<u32>>>(mut self, max_cell_count: T) -> Self {
        self.max_cell_count = max_cell_count.into();
        self
    }

    /// Sets whether to automatically reduce the `max_cell_count`
    /// when a result is found.
    #[must_use]
    #[inline]
    pub const fn set_reduce_max(mut self, reduce_max: bool) -> Self {
        self.reduce_max = reduce_max;
        self
    }

    /// Sets the rule string.
    #[must_use]
    #[inline]
    pub fn set_rule_string<S: ToString>(mut self, rule_string: S) -> Self {
        self.rule_string = rule_string.to_string();
        self
    }

    /// Sets the diagonal width.
    #[must_use]
    #[inline]
    pub fn set_diagonal_width<T: Into<Option<i32>>>(mut self, diagonal_width: T) -> Self {
        self.diagonal_width = diagonal_width.into();
        self
    }

    /// Sets whether to skip patterns whose fundamental period
    /// is smaller than the given period.
    #[must_use]
    #[inline]
    pub const fn set_skip_subperiod(mut self, skip_subperiod: bool) -> Self {
        self.skip_subperiod = skip_subperiod;
        self
    }

    /// Sets whether to skip patterns which are invariant under
    /// more transformations than required by the given symmetry.
    #[must_use]
    #[inline]
    pub const fn set_skip_subsymmetry(mut self, skip_subsymmetry: bool) -> Self {
        self.skip_subsymmetry = skip_subsymmetry;
        self
    }

    /// Sets cells whose states are known before the search.
    #[must_use]
    #[inline]
    pub fn set_known_cells<T: Into<Vec<KnownCell>>>(mut self, known_cells: T) -> Self {
        self.known_cells = known_cells.into();
        self
    }

    /// Sets cells whose states are known before the search.
    ///
    /// The cells are specified by a list of RLE strings.
    #[cfg(feature = "read-rle")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "read-rle")))]
    #[inline]
    pub fn set_known_cells_from_rles<I, L>(mut self, input: I) -> Result<Self, RleError>
    where
        I: Input<Lines = L>,
        L: Input<Lines = L>,
    {
        self.known_cells = KnownCell::from_rles(input)?;
        Ok(self)
    }

    /// Sets whether to enable backjumping.
    #[must_use]
    #[inline]
    pub const fn set_backjump(mut self, backjump: bool) -> Self {
        self.backjump = backjump;
        self
    }

    /// Whether the configuration requires the world to be square.
    #[inline]
    pub const fn require_square_world(&self) -> bool {
        self.symmetry.require_square_world()
            || self.transform.require_square_world()
            || matches!(self.search_order, Some(SearchOrder::Diagonal))
    }

    /// Whether the configuration requires the world to have no diagonal width.
    #[inline]
    pub const fn require_no_diagonal_width(&self) -> bool {
        self.symmetry.require_no_diagonal_width() || self.transform.require_no_diagonal_width()
    }

    /// Whether the cell is contained in the world.
    ///
    /// If `including_border` is true, this includes the cells at the border.
    ///
    /// If `check_diagonal_width` is true, this excludes the cells outside of the diagonal_width.
    pub(crate) fn contains(
        &self,
        (x, y, t): Coord,
        including_border: bool,
        check_diagonal_width: bool,
    ) -> bool {
        if including_border {
            x >= -1
                && x <= self.width
                && y >= -1
                && y <= self.height
                && t >= 0
                && t < self.period
                && (!check_diagonal_width
                    || (self.diagonal_width.is_none()
                        || (x - y).abs() <= self.diagonal_width.unwrap() + 1))
        } else {
            x >= 0
                && x < self.width
                && y >= 0
                && y < self.height
                && t >= 0
                && t < self.period
                && (!check_diagonal_width
                    || (self.diagonal_width.is_none()
                        || (x - y).abs() < self.diagonal_width.unwrap()))
        }
    }

    /// Creates a new world from the configuration.
    /// Returns an error if the rule string is invalid.
    pub fn world(&self) -> Result<PolyWorld, Error> {
        macro_rules! new_world {
            ($rule:expr) => {{
                for known_cell in &self.known_cells {
                    if known_cell.state.0 >= 2 {
                        return Err(Error::InvalidState(known_cell.coord, known_cell.state));
                    }
                }
                if self.backjump && self.max_cell_count.is_none() {
                    Ok(World::new_backjump(&self, $rule).into())
                } else {
                    Ok(World::new_lifesrc(&self, $rule).into())
                }
            }};
        }

        macro_rules! new_world_gen {
            ($rule:expr) => {{
                if $rule.gen() > 2 {
                    for known_cell in &self.known_cells {
                        if known_cell.state.0 >= $rule.gen() {
                            return Err(Error::InvalidState(known_cell.coord, known_cell.state));
                        }
                    }
                    Ok(World::new_lifesrc(&self, $rule).into())
                } else {
                    new_world!($rule.non_gen())
                }
            }};
        }

        if self.width <= 0 || self.height <= 0 || self.period <= 0 {
            return Err(Error::NonPositiveError);
        }
        if let Some(diagonal_width) = self.diagonal_width {
            if diagonal_width <= 0 {
                return Err(Error::NonPositiveError);
            }
        }
        if self.require_square_world() && self.width != self.height {
            return Err(Error::SquareWorldError);
        }
        if self.require_no_diagonal_width() && self.diagonal_width.is_some() {
            return Err(Error::DiagonalWidthError);
        }

        if let Ok(rule) = self.rule_string.parse::<Life>() {
            new_world!(rule)
        } else if let Ok(rule) = self.rule_string.parse::<NtLife>() {
            new_world!(rule)
        } else if let Ok(rule) = self.rule_string.parse::<LifeGen>() {
            new_world_gen!(rule)
        } else {
            let rule = self.rule_string.parse::<NtLifeGen>()?;
            new_world_gen!(rule)
        }
    }
}
