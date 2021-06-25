//! Configurations related to the
//! [dihedral group _D_<sub>8</sub>](https://en.wikipedia.org/wiki/Examples_of_groups#dihedral_group_of_order_8).
//!
//! 8 different transformations correspond to 8 elements of _D_<sub>8</sub>.
//!
//! 10 different symmetries correspond to 10 subgroups of _D_<sub>8</sub>.

use super::{Config, Coord};
use educe::Educe;
use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    matches,
    ops::Mul,
    str::FromStr,
    vec,
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
/// and some are only valid when the world has no diagonal width.
#[derive(Clone, Copy, Debug, Educe, PartialEq, Eq, Hash)]
#[educe(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Transform {
    /// `Id`.
    ///
    /// Identity transformation.
    #[educe(Default)]
    Id,
    /// `R90`.
    ///
    /// 90° rotation counterclockwise.
    ///
    /// Requires the world to be square and have no diagonal width.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "R90")))]
    #[cfg_attr(feature = "serde", serde(alias = "R90"))]
    Rotate90,
    /// `R180`.
    ///
    /// 180° rotation counterclockwise.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "R180")))]
    #[cfg_attr(feature = "serde", serde(alias = "R180"))]
    Rotate180,
    /// `R270`.
    ///
    /// 270° rotation counterclockwise.
    ///
    /// Requires the world to be square and have no diagonal width.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "R270")))]
    #[cfg_attr(feature = "serde", serde(alias = "R270"))]
    Rotate270,
    /// `F-`.
    ///
    /// Reflection across the middle row.
    ///
    /// Requires the world to have no diagonal width.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "F-")))]
    #[cfg_attr(feature = "serde", serde(alias = "F-"))]
    FlipRow,
    /// `F|`.
    ///
    /// Reflection across the middle column.
    ///
    /// Requires the world to have no diagonal width.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "F|")))]
    #[cfg_attr(feature = "serde", serde(alias = "F|"))]
    FlipCol,
    /// `F\`.
    ///
    /// Reflection across the diagonal.
    ///
    /// Requires the world to be square.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "F\\")))]
    #[cfg_attr(feature = "serde", serde(alias = "F\\"))]
    FlipDiag,
    /// `F/`.
    ///
    /// Reflection across the antidiagonal.
    ///
    /// Requires the world to be square.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "F/")))]
    #[cfg_attr(feature = "serde", serde(alias = "F/"))]
    FlipAntidiag,
}

impl FromStr for Transform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Id" => Ok(Self::Id),
            "R90" => Ok(Self::Rotate90),
            "R180" => Ok(Self::Rotate180),
            "R270" => Ok(Self::Rotate270),
            "F-" => Ok(Self::FlipRow),
            "F|" => Ok(Self::FlipCol),
            "F\\" => Ok(Self::FlipDiag),
            "F/" => Ok(Self::FlipAntidiag),
            _ => Err(String::from("Invalid Transform")),
        }
    }
}

impl Display for Transform {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            Self::Id => "Id",
            Self::Rotate90 => "R90",
            Self::Rotate180 => "R180",
            Self::Rotate270 => "R270",
            Self::FlipRow => "F-",
            Self::FlipCol => "F|",
            Self::FlipDiag => "F\\",
            Self::FlipAntidiag => "F/",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Id, Self::Id)
            | (Self::Rotate90, Self::Rotate270)
            | (Self::Rotate180, Self::Rotate180)
            | (Self::Rotate270, Self::Rotate90)
            | (Self::FlipRow, Self::FlipRow)
            | (Self::FlipCol, Self::FlipCol)
            | (Self::FlipDiag, Self::FlipDiag)
            | (Self::FlipAntidiag, Self::FlipAntidiag) => Self::Id,
            (Self::Id, Self::Rotate90)
            | (Self::Rotate90, Self::Id)
            | (Self::Rotate180, Self::Rotate270)
            | (Self::Rotate270, Self::Rotate180)
            | (Self::FlipRow, Self::FlipAntidiag)
            | (Self::FlipCol, Self::FlipDiag)
            | (Self::FlipDiag, Self::FlipRow)
            | (Self::FlipAntidiag, Self::FlipCol) => Self::Rotate90,
            (Self::Id, Self::Rotate180)
            | (Self::Rotate90, Self::Rotate90)
            | (Self::Rotate180, Self::Id)
            | (Self::Rotate270, Self::Rotate270)
            | (Self::FlipRow, Self::FlipCol)
            | (Self::FlipCol, Self::FlipRow)
            | (Self::FlipDiag, Self::FlipAntidiag)
            | (Self::FlipAntidiag, Self::FlipDiag) => Self::Rotate180,
            (Self::Id, Self::Rotate270)
            | (Self::Rotate90, Self::Rotate180)
            | (Self::Rotate180, Self::Rotate90)
            | (Self::Rotate270, Self::Id)
            | (Self::FlipRow, Self::FlipDiag)
            | (Self::FlipCol, Self::FlipAntidiag)
            | (Self::FlipDiag, Self::FlipCol)
            | (Self::FlipAntidiag, Self::FlipRow) => Self::Rotate270,
            (Self::Id, Self::FlipRow)
            | (Self::Rotate90, Self::FlipAntidiag)
            | (Self::Rotate180, Self::FlipCol)
            | (Self::Rotate270, Self::FlipDiag)
            | (Self::FlipRow, Self::Id)
            | (Self::FlipCol, Self::Rotate180)
            | (Self::FlipDiag, Self::Rotate90)
            | (Self::FlipAntidiag, Self::Rotate270) => Self::FlipRow,
            (Self::Id, Self::FlipCol)
            | (Self::Rotate90, Self::FlipDiag)
            | (Self::Rotate180, Self::FlipRow)
            | (Self::Rotate270, Self::FlipAntidiag)
            | (Self::FlipRow, Self::Rotate180)
            | (Self::FlipCol, Self::Id)
            | (Self::FlipDiag, Self::Rotate270)
            | (Self::FlipAntidiag, Self::Rotate90) => Self::FlipCol,
            (Self::Id, Self::FlipDiag)
            | (Self::Rotate90, Self::FlipRow)
            | (Self::Rotate180, Self::FlipAntidiag)
            | (Self::Rotate270, Self::FlipCol)
            | (Self::FlipRow, Self::Rotate270)
            | (Self::FlipCol, Self::Rotate90)
            | (Self::FlipDiag, Self::Id)
            | (Self::FlipAntidiag, Self::Rotate180) => Self::FlipDiag,
            (Self::Id, Self::FlipAntidiag)
            | (Self::Rotate90, Self::FlipCol)
            | (Self::Rotate180, Self::FlipDiag)
            | (Self::Rotate270, Self::FlipRow)
            | (Self::FlipRow, Self::Rotate90)
            | (Self::FlipCol, Self::Rotate270)
            | (Self::FlipDiag, Self::Rotate180)
            | (Self::FlipAntidiag, Self::Id) => Self::FlipAntidiag,
        }
    }
}

impl Transform {
    /// Whether this transformation requires the world to be square.
    ///
    /// Returns `true` for `R90`, `R270`, `F\` and `F/`.
    pub const fn require_square_world(self) -> bool {
        !self.is_in(Symmetry::D4Ortho)
    }

    /// Whether this transformation requires the world to have no diagonal width.
    ///
    /// Returns `true` for `R90`, `R270`, `F-` and `F|`.
    pub const fn require_no_diagonal_width(self) -> bool {
        !self.is_in(Symmetry::D4Diag)
    }

    /// The order of this transformation in the symmetry group.
    pub const fn order(self) -> u8 {
        match self {
            Self::Id => 1,
            Self::Rotate90 | Self::Rotate270 => 4,
            _ => 2,
        }
    }

    /// The inverse of this transformation.
    pub const fn inverse(self) -> Self {
        match self {
            Self::Rotate90 => Self::Rotate270,
            Self::Rotate270 => Self::Rotate90,
            x => x,
        }
    }

    /// Whether the transformation is a member of the symmetry group,
    /// i.e., whether patterns with this symmetry are invariant under
    /// this transformation.
    pub const fn is_in(self, sym: Symmetry) -> bool {
        matches!(
            (self, sym),
            (Self::Id, _)
                | (_, Symmetry::D8)
                | (Self::Rotate90, Symmetry::C4)
                | (Self::Rotate180, Symmetry::C2)
                | (Self::Rotate180, Symmetry::C4)
                | (Self::Rotate180, Symmetry::D4Ortho)
                | (Self::Rotate180, Symmetry::D4Diag)
                | (Self::Rotate270, Symmetry::C4)
                | (Self::FlipRow, Symmetry::D2Row)
                | (Self::FlipRow, Symmetry::D4Ortho)
                | (Self::FlipCol, Symmetry::D2Col)
                | (Self::FlipCol, Symmetry::D4Ortho)
                | (Self::FlipDiag, Symmetry::D2Diag)
                | (Self::FlipDiag, Symmetry::D4Diag)
                | (Self::FlipAntidiag, Symmetry::D2Antidiag)
                | (Self::FlipAntidiag, Symmetry::D4Diag),
        )
    }

    /// Apply the transformation on a coordinate.
    pub const fn act_on(self, coord: Coord, width: i32, height: i32) -> Coord {
        let (x, y, t) = coord;
        match self {
            Self::Id => (x, y, t),
            Self::Rotate90 => (y, width - 1 - x, t),
            Self::Rotate180 => (width - 1 - x, height - 1 - y, t),
            Self::Rotate270 => (height - 1 - y, x, t),
            Self::FlipRow => (x, height - 1 - y, t),
            Self::FlipCol => (width - 1 - x, y, t),
            Self::FlipDiag => (y, x, t),
            Self::FlipAntidiag => (height - 1 - y, width - 1 - x, t),
        }
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
/// Some of the symmetries are only valid when the world is square,
/// and some are only valid when the world has no diagonal width.
#[derive(Clone, Copy, Debug, Educe, PartialEq, Eq, Hash)]
#[educe(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Symmetry {
    /// `C1`.
    ///
    /// No symmetry at all.
    #[educe(Default)]
    C1,
    /// `C2`.
    ///
    /// Symmetry under 180° rotation.
    C2,
    /// `C4`.
    ///
    /// Symmetry under 90° rotation.
    ///
    /// Requires the world to be square and have no diagonal width.
    C4,
    /// `D2-`.
    ///
    /// Symmetry under reflection across the middle row.
    ///
    /// Requires the world to have no diagonal width.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "D2-")))]
    #[cfg_attr(feature = "serde", serde(alias = "D2-"))]
    D2Row,
    /// `D2|`.
    ///
    /// Symmetry under reflection across the middle column.
    ///
    /// Requires the world to have no diagonal width.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "D2|")))]
    #[cfg_attr(feature = "serde", serde(alias = "D2|"))]
    D2Col,
    /// `D2\`.
    ///
    /// Symmetry under reflection across the diagonal.
    ///
    /// Requires the world to be square.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "D2\\")))]
    #[cfg_attr(feature = "serde", serde(alias = "D2\\"))]
    D2Diag,
    /// `D2/`.
    ///
    /// Symmetry under reflection across the antidiagonal.
    ///
    /// Requires the world to be square.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "D2/")))]
    #[cfg_attr(feature = "serde", serde(alias = "D2/"))]
    D2Antidiag,
    /// `D4+`.
    ///
    /// Symmetry under reflections across the middle row
    /// and the middle column.
    ///
    /// Requires the world to have no diagonal width.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "D4+")))]
    #[cfg_attr(feature = "serde", serde(alias = "D4+"))]
    D4Ortho,
    /// `D4X`.
    ///
    /// Symmetry under reflections across the diagonal
    /// and the antidiagonal.
    ///
    /// Requires the world to be square.
    #[cfg_attr(feature = "serde", serde(rename(serialize = "D4X")))]
    #[cfg_attr(feature = "serde", serde(alias = "D4X"))]
    D4Diag,
    /// `D8`.
    ///
    /// Symmetry under all 8 transformations.
    ///
    /// Requires the world to be square and have no diagonal width.
    D8,
}

impl FromStr for Symmetry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C1" => Ok(Self::C1),
            "C2" => Ok(Self::C2),
            "C4" => Ok(Self::C4),
            "D2-" => Ok(Self::D2Row),
            "D2|" => Ok(Self::D2Col),
            "D2\\" => Ok(Self::D2Diag),
            "D2/" => Ok(Self::D2Antidiag),
            "D4+" => Ok(Self::D4Ortho),
            "D4X" => Ok(Self::D4Diag),
            "D8" => Ok(Self::D8),
            _ => Err(String::from("invalid Self")),
        }
    }
}

impl Display for Symmetry {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            Self::C1 => "C1",
            Self::C2 => "C2",
            Self::C4 => "C4",
            Self::D2Row => "D2-",
            Self::D2Col => "D2|",
            Self::D2Diag => "D2\\",
            Self::D2Antidiag => "D2/",
            Self::D4Ortho => "D4+",
            Self::D4Diag => "D4X",
            Self::D8 => "D8",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

impl PartialOrd for Symmetry {
    /// We say that symmetry `a` is smaller than symmetry `b`,
    /// when the symmetry group of `a` is a subgroup of that of `b`,
    /// i.e., all patterns with symmetry `b` also have symmetry `a`.
    ///
    /// For example, `Symmetry::C1` is smaller than all other symmetries.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.is_subgroup_of(*other) {
            Some(Ordering::Less)
        } else if other.is_subgroup_of(*self) {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl Symmetry {
    /// Whether the symmetry group of `self` is a subgroup of that of `other`,
    /// i.e., all patterns with symmetry `other` also have symmetry `self`.
    ///
    /// For example, the symmetry group of `Symmetry::C1` is a subgroup of
    /// that of all other symmetries.
    pub const fn is_subgroup_of(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::C1, _)
                | (_, Self::D8)
                | (Self::C2, Self::C2)
                | (Self::C2, Self::C4)
                | (Self::C2, Self::D4Ortho)
                | (Self::C2, Self::D4Diag)
                | (Self::C4, Self::C4)
                | (Self::D2Row, Self::D2Row)
                | (Self::D2Row, Self::D4Ortho)
                | (Self::D2Col, Self::D2Col)
                | (Self::D2Col, Self::D4Ortho)
                | (Self::D2Diag, Self::D2Diag)
                | (Self::D2Diag, Self::D4Diag)
                | (Self::D2Antidiag, Self::D2Antidiag)
                | (Self::D2Antidiag, Self::D4Diag)
                | (Self::D4Ortho, Self::D4Ortho)
                | (Self::D4Diag, Self::D4Diag)
        )
    }

    /// Whether this symmetry requires the world to be square.
    ///
    /// Returns `true` for `C4`, `D2\`, `D2/`, `D4X` and `D8`.
    pub fn require_square_world(self) -> bool {
        !self.is_subgroup_of(Self::D4Ortho)
    }

    /// Whether this transformation requires the world to have no diagonal width.
    ///
    /// Returns `true` for `C4`, `D2-`, `D2|`, `D4+` and `D8`.
    pub fn require_no_diagonal_width(self) -> bool {
        !self.is_subgroup_of(Self::D4Diag)
    }

    /// Transformations contained in the symmetry group.
    pub fn members(self) -> Vec<Transform> {
        match self {
            Self::C1 => vec![Transform::Id],
            Self::C2 => vec![Transform::Id, Transform::Rotate180],
            Self::C4 => vec![
                Transform::Id,
                Transform::Rotate90,
                Transform::Rotate180,
                Transform::Rotate270,
            ],
            Self::D2Row => vec![Transform::Id, Transform::FlipRow],
            Self::D2Col => vec![Transform::Id, Transform::FlipCol],
            Self::D2Diag => vec![Transform::Id, Transform::FlipDiag],
            Self::D2Antidiag => vec![Transform::Id, Transform::FlipAntidiag],
            Self::D4Ortho => vec![
                Transform::Id,
                Transform::FlipRow,
                Transform::FlipCol,
                Transform::Rotate180,
            ],
            Self::D4Diag => vec![
                Transform::Id,
                Transform::FlipDiag,
                Transform::FlipAntidiag,
                Transform::Rotate180,
            ],
            Self::D8 => vec![
                Transform::Id,
                Transform::Rotate90,
                Transform::Rotate180,
                Transform::Rotate270,
                Transform::FlipRow,
                Transform::FlipCol,
                Transform::FlipDiag,
                Transform::FlipAntidiag,
            ],
        }
    }

    /// A list of coset representatives,
    /// seeing the symmetry group as a subgroup of _D_<sub>8</sub>.
    ///
    /// The first element in the result is always [`Transform::Id`].
    pub fn cosets(self) -> Vec<Transform> {
        match self {
            Self::C1 => vec![
                Transform::Id,
                Transform::Rotate90,
                Transform::Rotate180,
                Transform::Rotate270,
                Transform::FlipRow,
                Transform::FlipCol,
                Transform::FlipDiag,
                Transform::FlipAntidiag,
            ],
            Self::C2 => vec![
                Transform::Id,
                Transform::Rotate90,
                Transform::FlipRow,
                Transform::FlipDiag,
            ],
            Self::C4 => vec![Transform::Id, Transform::FlipRow],
            Self::D2Row => vec![
                Transform::Id,
                Transform::FlipCol,
                Transform::FlipDiag,
                Transform::FlipAntidiag,
            ],
            Self::D2Col => vec![
                Transform::Id,
                Transform::FlipRow,
                Transform::FlipDiag,
                Transform::FlipAntidiag,
            ],
            Self::D2Diag => vec![
                Transform::Id,
                Transform::FlipRow,
                Transform::FlipCol,
                Transform::FlipAntidiag,
            ],
            Self::D2Antidiag => vec![
                Transform::Id,
                Transform::FlipRow,
                Transform::FlipCol,
                Transform::FlipDiag,
            ],
            Self::D4Ortho => vec![Transform::Id, Transform::FlipDiag],
            Self::D4Diag => vec![Transform::Id, Transform::FlipRow],
            Self::D8 => vec![Transform::Id],
        }
    }
}

impl Config {
    /// Applies the transformation and translation to a coord.
    pub(crate) const fn translate(&self, coord: Coord) -> Coord {
        let mut coord = coord;
        while coord.2 < 0 {
            coord = self
                .transform
                .inverse()
                .act_on(coord, self.width, self.height);
            coord.0 -= self.dx;
            coord.1 -= self.dy;
            coord.2 += self.period;
        }
        while coord.2 >= self.period {
            coord.0 += self.dx;
            coord.1 += self.dy;
            coord.2 -= self.period;
            coord = self.transform.act_on(coord, self.width, self.height);
        }
        coord
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::collections::HashSet;

    const ALL_TRANSFORM: [Transform; 8] = [
        Transform::Id,
        Transform::Rotate90,
        Transform::Rotate180,
        Transform::Rotate270,
        Transform::FlipRow,
        Transform::FlipCol,
        Transform::FlipDiag,
        Transform::FlipAntidiag,
    ];

    const ALL_SYMMETRY: [Symmetry; 10] = [
        Symmetry::C1,
        Symmetry::C2,
        Symmetry::C4,
        Symmetry::D2Col,
        Symmetry::D2Row,
        Symmetry::D2Diag,
        Symmetry::D2Antidiag,
        Symmetry::D4Diag,
        Symmetry::D4Ortho,
        Symmetry::D8,
    ];

    #[test]
    fn test_sym_tran_names() {
        for sym in ALL_SYMMETRY {
            assert!(Symmetry::from_str(&sym.to_string()) == Ok(sym))
        }
        for trans in ALL_TRANSFORM {
            assert!(Transform::from_str(&trans.to_string()) == Ok(trans))
        }
    }

    #[test]
    fn test_symmetry_group_member() {
        for sym in ALL_SYMMETRY {
            let members = sym.members();
            for tran in ALL_TRANSFORM {
                assert_eq!(tran.is_in(sym), members.contains(&tran));
            }
        }
    }

    #[test]
    fn test_symmetry_subgroup() {
        for sym in ALL_SYMMETRY {
            for sub_sym in ALL_SYMMETRY {
                let is_subgroup = sub_sym.members().into_iter().all(|tran| tran.is_in(sym));
                assert_eq!(sub_sym <= sym, is_subgroup);
            }
        }
    }

    #[test]
    fn test_symmetry_coset() {
        let group = ALL_TRANSFORM.iter().copied().collect::<HashSet<_>>();
        for sym in ALL_SYMMETRY {
            let all_cosets = sym
                .cosets()
                .into_iter()
                .flat_map(|coset| sym.members().into_iter().map(move |elem| elem * coset))
                .collect::<HashSet<_>>();
            assert_eq!(all_cosets, group);
        }
    }

    #[test]
    fn test_transform_inverse() {
        let width = 16;
        let height = 16;
        let mut rng = thread_rng();
        for _ in 0..10 {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            let coord = (x, y, 0);

            for tran in ALL_TRANSFORM {
                assert_eq!(
                    coord,
                    tran.inverse()
                        .act_on(tran.act_on(coord, width, height), width, height),
                    "{} ^ -1 != {}",
                    tran,
                    tran.inverse()
                )
            }
        }
    }

    #[test]
    fn test_transform_mul() {
        let width = 16;
        let height = 16;
        let mut rng = thread_rng();
        for _ in 0..10 {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            let coord = (x, y, 0);

            for tran0 in ALL_TRANSFORM {
                for tran1 in ALL_TRANSFORM {
                    assert_eq!(
                        (tran0 * tran1).act_on(coord, width, height),
                        tran1.act_on(tran0.act_on(coord, width, height), width, height),
                        "{} * {} != {}",
                        tran0,
                        tran1,
                        tran0 * tran1
                    )
                }
            }
        }
    }

    #[test]
    fn test_world_condition() {
        for tran in ALL_TRANSFORM {
            assert_eq!(
                tran.require_square_world(),
                matches!(
                    tran,
                    Transform::Rotate90
                        | Transform::Rotate270
                        | Transform::FlipDiag
                        | Transform::FlipAntidiag
                )
            );
            assert_eq!(
                tran.require_no_diagonal_width(),
                matches!(
                    tran,
                    Transform::Rotate90
                        | Transform::Rotate270
                        | Transform::FlipRow
                        | Transform::FlipCol
                )
            );
        }
        for sym in ALL_SYMMETRY {
            assert_eq!(
                sym.require_square_world(),
                matches!(
                    sym,
                    Symmetry::C4
                        | Symmetry::D2Diag
                        | Symmetry::D2Antidiag
                        | Symmetry::D4Diag
                        | Symmetry::D8
                )
            );
            assert_eq!(
                sym.require_no_diagonal_width(),
                matches!(
                    sym,
                    Symmetry::C4
                        | Symmetry::D2Row
                        | Symmetry::D2Col
                        | Symmetry::D4Ortho
                        | Symmetry::D8
                )
            );
        }
    }
}
