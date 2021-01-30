//! Configurations related to the
//! [dihedral group _D_<sub>8</sub>](https://en.wikipedia.org/wiki/Examples_of_groups#dihedral_group_of_order_8).
//!
//! 8 different transformations correspond to 8 elements of _D_<sub>8</sub>.
//!
//! 10 different symmetries correspond to 10 subgroups of _D_<sub>8</sub>.

use super::{Config, Coord};
use derivative::Derivative;
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    matches,
    str::FromStr,
};

#[cfg(any(feature = "serde", doc))]
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
                | Transform::FlipAntidiag,
        )
    }

    /// Whether the transformation is a member of the symmetry group,
    /// i.e., whether patterns with this symmetry are invariant under
    /// this transformation.
    pub fn is_in(self, sym: Symmetry) -> bool {
        matches!(
            (self, sym),
            (Transform::Id, _)
            | (_, Symmetry::D8)
            | (Transform::Rotate90, Symmetry::C4)
            | (Transform::Rotate180, Symmetry::C2)
            | (Transform::Rotate180, Symmetry::C4)
            | (Transform::Rotate180, Symmetry::D4Ortho)
            | (Transform::Rotate180, Symmetry::D4Diag)
            | (Transform::Rotate270, Symmetry::C4)
            | (Transform::FlipRow, Symmetry::D2Row)
            | (Transform::FlipRow, Symmetry::D4Ortho)
            | (Transform::FlipCol, Symmetry::D2Col)
            | (Transform::FlipCol, Symmetry::D4Ortho)
            | (Transform::FlipDiag, Symmetry::D2Diag)
            | (Transform::FlipDiag, Symmetry::D4Diag)
            | (Transform::FlipAntidiag, Symmetry::D2Antidiag)
            | (Transform::FlipAntidiag, Symmetry::D4Diag),
        )
    }
}

/// Symmetries of the pattern.
///
/// For each symmetry, its [symmetry group](https://en.wikipedia.org/wiki/Symmetry_group)
/// is a subgroup of the dihedral group _D_<sub>8</sub>.
/// 10 different symmetries correspond to 10 subgroups of _D_<sub>8</sub>.
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

impl PartialOrd for Symmetry {
    /// We say that symmetry `a` is smaller than symmetry `b`,
    /// when the symmetry group of `a` is a subgroup of that of `b`,
    /// i.e., all patterns with symmetry `b` also have symmetry `a`.
    ///
    /// For example, `Symmetry::C1` is smaller than all other symmetries.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        match (self, other) {
            (Symmetry::C1, _)
            | (_, Symmetry::D8)
            | (Symmetry::C2, Symmetry::C4)
            | (Symmetry::C2, Symmetry::D4Ortho)
            | (Symmetry::C2, Symmetry::D4Diag)
            | (Symmetry::D2Row, Symmetry::D4Ortho)
            | (Symmetry::D2Col, Symmetry::D4Ortho)
            | (Symmetry::D2Diag, Symmetry::D4Diag)
            | (Symmetry::D2Antidiag, Symmetry::D4Diag) => Some(Ordering::Less),
            (Symmetry::D8, _)
            | (_, Symmetry::C1)
            | (Symmetry::C4, Symmetry::C2)
            | (Symmetry::D4Ortho, Symmetry::C2)
            | (Symmetry::D4Diag, Symmetry::C2)
            | (Symmetry::D4Ortho, Symmetry::D2Row)
            | (Symmetry::D4Ortho, Symmetry::D2Col)
            | (Symmetry::D4Diag, Symmetry::D2Diag)
            | (Symmetry::D4Diag, Symmetry::D2Antidiag) => Some(Ordering::Greater),
            _ => None,
        }
    }
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
    /// Whether this symmetry requires the world to be square.
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

impl Config {
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
}
