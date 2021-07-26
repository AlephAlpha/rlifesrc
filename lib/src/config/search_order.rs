//! Configurations related to the the search order.

use super::{Config, Coord, Symmetry};
use auto_enums::auto_enum;
use std::{borrow::Cow, cmp::Ordering};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The order to find a new unknown cell.
///
/// It will always search all generations of one cell
/// before going to another cell.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

    /// Specify the search order by a vector of coordinates.
    ///
    /// This vector should cover every cell in the search range,
    /// and should not have any duplication, otherwise rlifesrc
    /// would give a wrong result.
    FromVec(Vec<Coord>),
}

impl Config {
    /// Automatically determines the search order if `search_order` is `None`.
    pub(crate) fn auto_search_order(&self) -> Cow<SearchOrder> {
        if let Some(search_order) = &self.search_order {
            Cow::Borrowed(search_order)
        } else {
            let (width, height) = match self.symmetry {
                Symmetry::D2Row => (self.width, (self.height + 1) / 2),
                Symmetry::D2Col => ((self.width + 1) / 2, self.height),
                _ => (self.width, self.height),
            };
            let search_order = match width.cmp(&height) {
                Ordering::Greater => SearchOrder::ColumnFirst,
                Ordering::Less => SearchOrder::RowFirst,
                Ordering::Equal => {
                    if self.diagonal_width.is_some()
                        && 2 * self.diagonal_width.unwrap() <= self.width
                    {
                        SearchOrder::Diagonal
                    } else if self.dx.abs() >= self.dy.abs() {
                        SearchOrder::ColumnFirst
                    } else {
                        SearchOrder::RowFirst
                    }
                }
            };
            Cow::Owned(search_order)
        }
    }

    /// Generates an iterator over cells coordinates from the search order.
    #[auto_enum(Iterator)]
    pub(crate) fn search_order_iter(
        &self,
        search_order: &SearchOrder,
    ) -> impl Iterator<Item = Coord> {
        let width = self.width;
        let height = self.height;
        let period = self.period;
        let x_start = if self.symmetry >= Symmetry::D2Col {
            self.width / 2
        } else {
            0
        };
        let y_start = if self.symmetry >= Symmetry::D2Row {
            self.height / 2
        } else {
            0
        };
        match search_order {
            SearchOrder::ColumnFirst => (0..width).rev().flat_map(move |x| {
                (y_start..height)
                    .rev()
                    .flat_map(move |y| (0..period).rev().map(move |t| (x, y, t)))
            }),
            SearchOrder::RowFirst => (0..height).rev().flat_map(move |y| {
                (x_start..width)
                    .rev()
                    .flat_map(move |x| (0..period).rev().map(move |t| (x, y, t)))
            }),
            #[nested]
            SearchOrder::Diagonal => {
                if self.symmetry >= Symmetry::D2Diag {
                    (0..width)
                        .rev()
                        .flat_map(move |d| {
                            ((width + d + 1) / 2..width).rev().flat_map(move |x| {
                                (0..period).rev().map(move |t| (x, width + d - x, t))
                            })
                        })
                        .chain((0..width).rev().flat_map(move |d| {
                            ((d + 1) / 2..=d)
                                .rev()
                                .flat_map(move |x| (0..period).rev().map(move |t| (x, d - x, t)))
                        }))
                } else {
                    (0..width)
                        .rev()
                        .flat_map(move |d| {
                            (d + 1..width).rev().flat_map(move |x| {
                                (0..period).rev().map(move |t| (x, width + d - x, t))
                            })
                        })
                        .chain((0..width).rev().flat_map(move |d| {
                            (0..=d)
                                .rev()
                                .flat_map(move |x| (0..period).rev().map(move |t| (x, d - x, t)))
                        }))
                }
            }
            SearchOrder::FromVec(vec) => vec.clone().into_iter().rev(),
        }
    }

    /// Generates a closure to determine whether a cell is in the front.
    ///
    /// Return `None` when we should not force the front to be nonempty,
    /// or there isn't a well-defined 'front'.
    pub(crate) fn fn_is_front(
        &self,
        rule_is_b0: bool,
        rule_symmetry: Symmetry,
        search_order: &SearchOrder,
    ) -> Option<Box<dyn Fn(Coord) -> bool>> {
        let dx = self.dx;
        let dy = self.dy;
        let width = self.width;
        let height = self.height;
        if !self.known_cells.is_empty() {
            return None;
        }

        match search_order {
            SearchOrder::RowFirst => {
                if self.symmetry <= Symmetry::D2Col
                    && self.transform.is_in(Symmetry::D2Col)
                    && self.diagonal_width.is_none()
                {
                    if !rule_is_b0 && dx == 0 && dy >= 0 {
                        if rule_symmetry >= Symmetry::D2Col {
                            Some(Box::new(move |(x, y, t)| {
                                y == (dy - 1).max(0) && t == 0 && x <= width / 2
                            }))
                        } else {
                            Some(Box::new(move |(_, y, t)| y == (dy - 1).max(0) && t == 0))
                        }
                    } else {
                        Some(Box::new(|(_, y, _)| y == 0))
                    }
                } else {
                    None
                }
            }
            SearchOrder::ColumnFirst => {
                if self.symmetry <= Symmetry::D2Row
                    && self.transform.is_in(Symmetry::D2Row)
                    && self.diagonal_width.is_none()
                {
                    if !rule_is_b0 && dx >= 0 && dy == 0 {
                        if rule_symmetry >= Symmetry::D2Row {
                            Some(Box::new(move |(x, y, t)| {
                                x == (dx - 1).max(0) && t == 0 && y <= height / 2
                            }))
                        } else {
                            Some(Box::new(move |(x, _, t)| x == (dx - 1).max(0) && t == 0))
                        }
                    } else {
                        Some(Box::new(|(x, _, _)| x == 0))
                    }
                } else {
                    None
                }
            }
            SearchOrder::Diagonal => {
                if self.symmetry <= Symmetry::D2Diag && self.transform.is_in(Symmetry::D2Diag) {
                    if !rule_is_b0 && dx >= 0 && dx == dy && self.width == self.height {
                        if rule_symmetry >= Symmetry::D2Diag {
                            Some(Box::new(move |(x, _, t)| x == (dx - 1).max(0) && t == 0))
                        } else {
                            Some(Box::new(move |(x, y, t)| {
                                x == (dx - 1).max(0) && y == (dy - 1).max(0) && t == 0
                            }))
                        }
                    } else {
                        Some(Box::new(|(x, y, _)| x == 0 || y == 0))
                    }
                } else {
                    None
                }
            }
            SearchOrder::FromVec(_) => None,
        }
    }
}
