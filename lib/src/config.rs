//! World configuration.

use crate::{
    cells::Coord,
    error::Error,
    rules::{Life, LifeGen, NtLife, NtLifeGen, Rule},
    traits::Search,
    world::World,
};
use derivative::Derivative;
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    str::FromStr,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Transformations (rotations and reflections) after the last generation
/// in a period.
///
/// After the last generation in a period, the pattern will return to
/// the first generation, applying this transformation first,
/// and then the translation defined by `dx` and `dy`.
///
/// 8 different values correspond to 8 elements of the dihedral group
/// _D_<sub>8</sub>.
///
/// `Id` is the identity transformation.
///
/// `R` means rotations around the center of the world.
/// The number after it is the counterclockwise rotation angle in degrees.
///
/// `F` means reflections (flips).
/// The symbol after it is the axis of reflection.
///
/// Some of the transformations are only valid when the world is square.
#[derive(Clone, Copy, Derivative, PartialEq, Eq)]
#[derivative(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Transform {
    /// `Id`.
    ///
    /// Identity transformation.
    #[derivative(Default)]
    Id,
    /// `R90`.
    ///
    /// 90° rotation counterclockwise.
    Rotate90,
    /// `R180`.
    ///
    /// 180° rotation counterclockwise.
    Rotate180,
    /// `R270`.
    ///
    /// 270° rotation counterclockwise.
    Rotate270,
    /// `F-`.
    ///
    /// Reflection across the middle row.
    FlipRow,
    /// `F|`.
    ///
    /// Reflection across the middle column.
    FlipCol,
    /// `F\`.
    ///
    /// Reflection across the diagonal.
    FlipDiag,
    /// `F/`.
    ///
    /// Reflection across the antidiagonal.
    FlipAntidiag,
}

impl FromStr for Transform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Id" => Ok(Transform::Id),
            "R90" => Ok(Transform::Rotate90),
            "R180" => Ok(Transform::Rotate180),
            "R270" => Ok(Transform::Rotate270),
            "F-" => Ok(Transform::FlipRow),
            "F|" => Ok(Transform::FlipCol),
            "F\\" => Ok(Transform::FlipDiag),
            "F/" => Ok(Transform::FlipAntidiag),
            _ => Err(String::from("invalid Transform")),
        }
    }
}

impl Debug for Transform {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            Transform::Id => "Id",
            Transform::Rotate90 => "R90",
            Transform::Rotate180 => "R180",
            Transform::Rotate270 => "R270",
            Transform::FlipRow => "F-",
            Transform::FlipCol => "F|",
            Transform::FlipDiag => "F\\",
            Transform::FlipAntidiag => "F/",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

impl Transform {
    /// Whether this transformation requires the world to be square.
    ///
    /// Returns `true` for `R90`, `R270`, `F\` and `F/`.
    pub fn square_world(self) -> bool {
        matches!(
            self,
            Transform::Rotate90
                | Transform::Rotate270
                | Transform::FlipDiag
                | Transform::FlipAntidiag
        )
    }
}

/// Symmetries of the pattern.
///
/// 10 different values correspond to 10 subgroups of the dihedral group
/// _D_<sub>8</sub>.
///
/// The notations are stolen from Oscar Cunningham's
/// [Logic Life Search](https://github.com/OscarCunningham/logic-life-search).
/// Please see the [Life Wiki](https://conwaylife.com/wiki/Symmetry) for details.
///
/// Some of the symmetries are only valid when the world is square.
#[derive(Clone, Copy, Derivative, PartialEq, Eq)]
#[derivative(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Symmetry {
    /// `C1`.
    ///
    /// No symmetry at all.
    #[derivative(Default)]
    C1,
    /// `C2`.
    ///
    /// Symmetry under 180° rotation.
    C2,
    /// `C4`.
    ///
    /// Symmetry under 90° rotation.
    C4,
    /// `D2-`.
    ///
    /// Symmetry under reflection across the middle row.
    D2Row,
    /// `D2|`.
    ///
    /// Symmetry under reflection across the middle column.
    D2Col,
    /// `D2\`.
    ///
    /// Symmetry under reflection across the diagonal.
    D2Diag,
    /// `D2/`.
    ///
    /// Symmetry under reflection across the antidiagonal.
    D2Antidiag,
    /// `D4+`.
    ///
    /// Symmetry under reflections across the middle row
    /// and the middle column.
    D4Ortho,
    /// `D4X`.
    ///
    /// Symmetry under reflections across the diagonal
    /// and the antidiagonal.
    D4Diag,
    /// `D8`.
    ///
    /// Symmetry under all 8 transformations.
    D8,
}

impl FromStr for Symmetry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C1" => Ok(Symmetry::C1),
            "C2" => Ok(Symmetry::C2),
            "C4" => Ok(Symmetry::C4),
            "D2-" => Ok(Symmetry::D2Row),
            "D2|" => Ok(Symmetry::D2Col),
            "D2\\" => Ok(Symmetry::D2Diag),
            "D2/" => Ok(Symmetry::D2Antidiag),
            "D4+" => Ok(Symmetry::D4Ortho),
            "D4X" => Ok(Symmetry::D4Diag),
            "D8" => Ok(Symmetry::D8),
            _ => Err(String::from("invalid symmetry")),
        }
    }
}

impl Debug for Symmetry {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            Symmetry::C1 => "C1",
            Symmetry::C2 => "C2",
            Symmetry::C4 => "C4",
            Symmetry::D2Row => "D2-",
            Symmetry::D2Col => "D2|",
            Symmetry::D2Diag => "D2\\",
            Symmetry::D2Antidiag => "D2/",
            Symmetry::D4Ortho => "D4+",
            Symmetry::D4Diag => "D4X",
            Symmetry::D8 => "D8",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

impl Symmetry {
    /// Whether this transformation requires the world to be square.
    ///
    /// Returns `true` for `C4`, `D2\`, `D2/`, `D4X` and `D8`.
    pub fn square_world(self) -> bool {
        matches!(
            self,
            Symmetry::C4
                | Symmetry::D2Diag
                | Symmetry::D2Antidiag
                | Symmetry::D4Diag
                | Symmetry::D8
        )
    }
}

/// The order to find a new unknown cell.
///
/// It will always search all generations of one cell
/// before going to another cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SearchOrder {
    /// Searches all cells of one row before going to the next row.
    ///
    /// ```plaintext
    /// 123
    /// 456
    /// 789
    /// ```
    RowFirst,

    /// Searches all cells of one column before going to the next column.
    ///
    /// ```plaintext
    /// 147
    /// 258
    /// 369
    /// ```
    ColumnFirst,

    /// Diagonal.
    ///
    /// ```plaintext
    /// 136
    /// 258
    /// 479
    /// ```
    ///
    /// This search order requires the world to be square.
    Diagonal,
}

/// How to choose a state for an unknown cell.
#[derive(Clone, Copy, Debug, Derivative, PartialEq, Eq)]
#[derivative(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NewState {
    /// Chooses the background state.
    ///
    /// For rules without `B0`, it always chooses `DEAD`.
    ///
    /// For rules with `B0`, the background changes periodically.
    /// For example, for non-Generations rules,
    /// it chooses `DEAD` on even generations,
    /// `ALIVE` on odd generations.
    ChooseDead,

    /// Chooses the opposite of the background state.
    ///
    /// For rules without `B0`, it always chooses `ALIVE`.
    ///
    /// For rules with `B0`, the background changes periodically.
    /// For example, for non-Generations rules,
    /// it chooses `ALIVE` on even generations,
    /// `DEAD` on odd generations.
    #[derivative(Default)]
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

/// World configuration.
///
/// The world will be generated from this configuration.
#[derive(Clone, Debug, Derivative, PartialEq, Eq)]
#[derivative(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    /// Width.
    #[derivative(Default(value = "16"))]
    pub width: isize,

    /// Height.
    #[derivative(Default(value = "16"))]
    pub height: isize,

    /// Period.
    #[derivative(Default(value = "1"))]
    pub period: isize,

    /// Horizontal translation.
    pub dx: isize,

    /// Vertical translation.
    pub dy: isize,

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
    pub max_cell_count: Option<usize>,

    /// Whether to force the first row/column to be nonempty.
    ///
    /// Here 'front' means the first row or column to be searched,
    /// according to the search order.
    #[derivative(Default(value = "true"))]
    pub non_empty_front: bool,

    /// Whether to automatically reduce the `max_cell_count`
    /// when a result is found.
    ///
    /// The `max_cell_count` will be set to the cell count of
    /// the current result minus one.
    pub reduce_max: bool,

    /// The rule string of the cellular automaton.
    #[derivative(Default(value = "String::from(\"B3/S23\")"))]
    pub rule_string: String,

    /// Diagonal width.
    ///
    /// If the diagonal width is `n`, the cells at position `(x, y)`
    /// where `abs(x - y) >= n` are assumed to be dead.
    pub diagonal_width: Option<isize>,
}

impl Config {
    /// Sets up a new configuration with given size.
    pub fn new(width: isize, height: isize, period: isize) -> Self {
        Config {
            width,
            height,
            period,
            ..Config::default()
        }
    }

    /// Sets the translations `(dx, dy)`.
    pub fn set_translate(mut self, dx: isize, dy: isize) -> Self {
        self.dx = dx;
        self.dy = dy;
        self
    }

    /// Sets the transformation.
    pub fn set_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Sets the symmetry.
    pub fn set_symmetry(mut self, symmetry: Symmetry) -> Self {
        self.symmetry = symmetry;
        self
    }

    /// Sets the search order.
    pub fn set_search_order<T: Into<Option<SearchOrder>>>(mut self, search_order: T) -> Self {
        self.search_order = search_order.into();
        self
    }

    /// Sets how to choose a state for an unknown cell.
    pub fn set_new_state(mut self, new_state: NewState) -> Self {
        self.new_state = new_state;
        self
    }

    /// Sets the maximal number of living cells.
    pub fn set_max_cell_count<T: Into<Option<usize>>>(mut self, max_cell_count: T) -> Self {
        self.max_cell_count = max_cell_count.into();
        self
    }

    /// Sets whether to force the first row/column to be nonempty.
    pub fn set_non_empty_front(mut self, non_empty_front: bool) -> Self {
        self.non_empty_front = non_empty_front;
        self
    }

    /// Sets whether to automatically reduce the `max_cell_count`
    /// when a result is found.
    pub fn set_reduce_max(mut self, reduce_max: bool) -> Self {
        self.reduce_max = reduce_max;
        self
    }

    /// Sets the rule string.
    pub fn set_rule_string<S: ToString>(mut self, rule_string: S) -> Self {
        self.rule_string = rule_string.to_string();
        self
    }

    /// Sets the diagonal width.
    pub fn set_diagonal_width<T: Into<Option<isize>>>(mut self, diagonal_width: T) -> Self {
        self.diagonal_width = diagonal_width.into();
        self
    }

    /// Automatically determines the search order if `search_order` is `None`.
    ///
    /// The result will never be `SearchOrder::Diagonal`.
    pub(crate) fn auto_search_order(&self) -> SearchOrder {
        self.search_order.unwrap_or_else(|| {
            let (width, height) = match self.symmetry {
                Symmetry::D2Row => (self.width, (self.height + 1) / 2),
                Symmetry::D2Col => ((self.width + 1) / 2, self.height),
                _ => (self.width, self.height),
            };
            match width.cmp(&height) {
                Ordering::Greater => SearchOrder::ColumnFirst,
                Ordering::Less => SearchOrder::RowFirst,
                Ordering::Equal => {
                    if self.dx.abs() >= self.dy.abs() {
                        SearchOrder::ColumnFirst
                    } else {
                        SearchOrder::RowFirst
                    }
                }
            }
        })
    }

    /// Applies the transformation and translation to a coord.
    pub(crate) fn translate(&self, coord: Coord) -> Coord {
        let (mut x, mut y, mut t) = coord;
        while t < 0 {
            t += self.period;
            let (new_x, new_y) = match self.transform {
                Transform::Id => (x, y),
                Transform::Rotate90 => (self.height - 1 - y, x),
                Transform::Rotate180 => (self.width - 1 - x, self.height - 1 - y),
                Transform::Rotate270 => (y, self.width - 1 - x),
                Transform::FlipRow => (x, self.height - 1 - y),
                Transform::FlipCol => (self.width - 1 - x, y),
                Transform::FlipDiag => (y, x),
                Transform::FlipAntidiag => (self.height - 1 - y, self.width - 1 - x),
            };
            x = new_x - self.dx;
            y = new_y - self.dy;
        }
        while t >= self.period {
            t -= self.period;
            x += self.dx;
            y += self.dy;
            let (new_x, new_y) = match self.transform {
                Transform::Id => (x, y),
                Transform::Rotate90 => (y, self.width - 1 - x),
                Transform::Rotate180 => (self.width - 1 - x, self.height - 1 - y),
                Transform::Rotate270 => (self.height - 1 - y, x),
                Transform::FlipRow => (x, self.height - 1 - y),
                Transform::FlipCol => (self.width - 1 - x, y),
                Transform::FlipDiag => (y, x),
                Transform::FlipAntidiag => (self.height - 1 - y, self.width - 1 - x),
            };
            x = new_x;
            y = new_y;
        }
        (x, y, t)
    }

    /// Creates a new world from the configuration.
    /// Returns an error if the rule string is invalid.
    pub fn world(&self) -> Result<Box<dyn Search>, Error> {
        if self.width <= 0 || self.height <= 0 || self.period <= 0 {
            return Err(Error::NonPositiveError);
        }
        if let Some(diagonal_width) = self.diagonal_width {
            if diagonal_width <= 0 {
                return Err(Error::NonPositiveError);
            }
        }
        if (self.symmetry.square_world()
            || self.transform.square_world()
            || self.search_order == Some(SearchOrder::Diagonal))
            && self.width != self.height
        {
            return Err(Error::SquareWorldError);
        }
        if let Ok(rule) = self.rule_string.parse::<Life>() {
            Ok(Box::new(World::new(&self, rule)))
        } else if let Ok(rule) = self.rule_string.parse::<NtLife>() {
            Ok(Box::new(World::new(&self, rule)))
        } else if let Ok(rule) = self.rule_string.parse::<LifeGen>() {
            if rule.gen() > 2 {
                Ok(Box::new(World::new(&self, rule)))
            } else {
                let rule = rule.non_gen();
                Ok(Box::new(World::new(&self, rule)))
            }
        } else {
            let rule = self.rule_string.parse::<NtLifeGen>()?;
            if rule.gen() > 2 {
                Ok(Box::new(World::new(&self, rule)))
            } else {
                let rule = rule.non_gen();
                Ok(Box::new(World::new(&self, rule)))
            }
        }
    }
}
