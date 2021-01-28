//! Search orders.

use crate::{
    cells::Coord,
    config::{Config, Symmetry, Transform},
};
use std::{iter::Iterator, matches};

#[cfg(any(feature = "serde", doc))]
use serde::{Deserialize, Serialize};

/// The order to find a new unknown cell.
#[cfg_attr(feature = "typetag", typetag::serde)]
pub trait SearchOrder {
    /// An iterator over cell coordinates, which defines
    /// the search order __in the reversed order__.
    fn iter(&self, config: &Config) -> Box<dyn Iterator<Item = Coord>>;

    /// Determines if a cell is in the 'front' according to this
    /// search order.
    ///
    /// When [`non_empty_front`](Config::set_non_empty_front) in [`Config`]
    /// is set, rlifesrc will ensure that at least one cell in the front
    /// is not dead.
    ///
    /// The default implementation returns `true` for all cells.
    #[allow(unused_variables)]
    fn is_front(&self, config: &Config, rule_is_b0: bool, coord: Coord) -> bool {
        true
    }
}

/// Searches all cells of one row before going to the next row.
///
/// ```plaintext
/// 123
/// 456
/// 789
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RowFirst;

/// The iterator for [`RowFirst`].

#[derive(Clone, Debug)]
pub struct RowFirstIter {
    /// Width.
    width: isize,
    /// Height.
    height: isize,
    /// Period.
    period: isize,
    /// The current cell. Become `None` after the iteration.
    current_cell: Option<Coord>,
    /// Whether to only search half of the world.
    half: bool,
}

impl Iterator for RowFirstIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y, t) = self.current_cell?;
        let x_start = if self.half { self.width / 2 } else { 0 };
        self.current_cell = if t == 0 {
            if y == 0 {
                if x == x_start {
                    None
                } else {
                    Some((x - 1, self.height - 1, self.period - 1))
                }
            } else {
                Some((x, y - 1, self.period - 1))
            }
        } else {
            Some((x, y, t - 1))
        };
        Some((x, y, t))
    }
}

#[cfg_attr(feature = "typetag", typetag::serde)]
impl SearchOrder for RowFirst {
    fn iter(&self, config: &Config) -> Box<dyn Iterator<Item = Coord>> {
        let half = config.symmetry > Symmetry::D2Col;
        let current_cell = Some((config.width - 1, config.height - 1, config.period - 1));
        Box::new(RowFirstIter {
            width: config.width,
            height: config.height,
            period: config.period,
            half,
            current_cell,
        })
    }

    fn is_front(&self, config: &Config, rule_is_b0: bool, (x, y, t): Coord) -> bool {
        config.symmetry > Symmetry::D2Col
            || !matches!(config.transform, Transform::Id | Transform::FlipCol)
            || config.diagonal_width.is_some()
            || if !rule_is_b0 && config.dx == 0 && config.dy >= 0 {
                y == (config.dy - 1).max(0) && t == 0 && x >= config.width / 2
            } else {
                y == 0
            }
    }
}
