//! The world, symmetries, transformations, and other information.

use crate::{
    cells::{Alive, Dead, LifeCell, State},
    rules::Rule,
};
use std::{
    cell::Cell,
    fmt::{Debug, Error, Formatter},
    str::FromStr,
};

#[cfg(feature = "stdweb")]
use serde::{Deserialize, Serialize};

/// Transformations. Rotations and reflections.
///
/// 8 different values corresponds to 8 elements of the dihedral group
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
/// The transformation is applied _before_ the translation.
///
/// Some of the transformations are only valid when the world is square.
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
pub enum Transform {
    /// `Id`.
    ///
    /// Identity transformation.
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
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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

/// The default transformation is the `Id`.
impl Default for Transform {
    fn default() -> Self {
        Transform::Id
    }
}

impl Transform {
    /// Whether the transformation requires the world to be square.
    ///
    /// Returns `true` for `R90`, `R270`, `F\` and `F/`.
    pub fn square_world(self) -> bool {
        match self {
            Transform::Rotate90
            | Transform::Rotate270
            | Transform::FlipDiag
            | Transform::FlipAntidiag => true,
            _ => false,
        }
    }
}

/// Symmetries.
///
/// 10 different values corresponds to 10 subgroups of the dihedral group
/// _D_<sub>8</sub>.
///
/// The notation is stolen from Oscar Cunningham's
/// [Logic Life Search](https://github.com/OscarCunningham/logic-life-search).
///
/// `Id` is the identity transformation.
///
/// `R` means rotations around the center of the world.
/// The number after it is the counterclockwise rotation angle in degrees.
///
/// `F` means reflections (flips).
/// The symbol after it is the axis of reflection.
///
/// Some of the symmetries are only valid when the world is square.
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
pub enum Symmetry {
    /// `C1`.
    ///
    /// No symmetry at all.
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
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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

/// The default symmetry is the `C1`.
impl Default for Symmetry {
    fn default() -> Self {
        Symmetry::C1
    }
}

impl Symmetry {
    /// Whether the transformation requires the world to be square.
    ///
    /// Returns `true` for `C4`, `D2\`, `D2/`, `D4X` and `D8`.
    pub fn square_world(self) -> bool {
        match self {
            Symmetry::C4
            | Symmetry::D2Diag
            | Symmetry::D2Antidiag
            | Symmetry::D4Diag
            | Symmetry::D8 => true,
            _ => false,
        }
    }
}

/// The coordinates of a cell.
///
/// `(x-coordinate, y-coordinate, time)`.
/// All three coordinates are 0-indexed.
pub type Coord = (isize, isize, isize);

/// The world.
pub struct World<'a, R: Rule> {
    /// Width.
    pub(crate) width: isize,
    /// Height.
    pub(crate) height: isize,
    /// Period.
    pub(crate) period: isize,
    /// The rule of the cellular automaton.
    pub(crate) rule: R,

    /// Search order. Whether the search starts from columns.
    ///
    /// Automatically determined by the width and the height of the world.
    pub(crate) column_first: bool,

    /// A vector that stores all the cells in the search range.
    ///
    /// The vector will not be moved after it is created.
    /// All the cells will live throughout the lifetime of the world.
    // So the unsafe code below is actually safe.
    cells: Vec<LifeCell<'a, R>>,

    /// A shared dead cell outside the search range.
    ///
    /// If the next generation of a cell is out of the search range,
    /// its `succ` would be this `dead_cell`.
    dead_cell: LifeCell<'a, R>,

    /// A list of references of cells sorted by the search order.search
    ///
    /// Used to find unknown cells.
    pub(crate) search_list: Vec<&'a LifeCell<'a, R>>,

    /// Number of known living cells in the first generation.
    pub(crate) gen0_cell_count: Cell<u32>,

    /// Number of unknown or living cells in the first generation.
    pub(crate) front_cell_count: Cell<u32>,
}

impl<'a, R: Rule> World<'a, R> {
    /// Create a new world.
    ///
    /// The pattern has size `(width, height, period)`
    /// and symmetry `symmetry`.
    /// In rules that contain `B0`, cells outside the search range are
    /// considered `Dead` in even generations, `Alive` in odd generations.
    /// In other rules, all cells outside the search range are `Dead`.
    ///
    /// In a period, the pattern would transforms according to `transform`,
    /// and translates `(dx, dy)`.
    /// The transformation is applied _before_ the translation.
    pub fn new(
        (width, height, period): Coord,
        dx: isize,
        dy: isize,
        transform: Transform,
        symmetry: Symmetry,
        rule: R,
        column_first: Option<bool>,
    ) -> Self {
        // Determine the search order automatically if `column_first` is `None`.
        let column_first = column_first.unwrap_or_else(|| {
            let (width, height) = match symmetry {
                Symmetry::D2Row => (width, (height + 1) / 2),
                Symmetry::D2Col => ((width + 1) / 2, height),
                _ => (width, height),
            };
            if width == height {
                dx.abs() >= dy.abs()
            } else {
                width > height
            }
        });

        let mut cells = Vec::with_capacity(((width + 2) * (height + 2) * period) as usize);

        // Fill the vector with dead cells.
        // If the rule contains `B0`, then fill the odd generations
        // with living cells instead.
        let (w, h) = if column_first {
            (width, height)
        } else {
            (height, width)
        };
        for x in -1..=w {
            for _y in -1..=h {
                for t in 0..period {
                    let state = if rule.b0() && t % 2 == 1 { Alive } else { Dead };
                    let mut cell = LifeCell::new(state);
                    if t == 0 {
                        cell.is_gen0 = true;
                    }
                    if x == 0 {
                        cell.is_front = true;
                    }
                    cells.push(cell);
                }
            }
        }

        let dead_cell = LifeCell::default();
        let search_list = Vec::new();

        let gen0_cell_count = Cell::new(0);
        let front_cell_count = Cell::new(0);

        let mut world = World {
            width,
            height,
            period,
            rule,
            column_first,
            cells,
            dead_cell,
            search_list,
            gen0_cell_count,
            front_cell_count,
        };

        // Initializes the world.
        world.init(dx, dy, transform, symmetry);
        world
    }

    /// Initializes the world, and links the cells together.
    ///
    /// This method will be called only once, inside `World::new`.
    fn init(&mut self, dx: isize, dy: isize, transform: Transform, symmetry: Symmetry) {
        // First links a cell to its neighbors,
        // so that we can use `set_cell` afterwards.
        //
        // Note that for cells on the edges of the search range,
        // some neighbors might point to `None`.
        let neighbors = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        for x in -1..=self.width {
            for y in -1..=self.height {
                for t in 0..self.period {
                    let cell_ptr: *mut _ = self.find_cell_mut((x, y, t)).unwrap();
                    for (i, (nx, ny)) in neighbors.iter().enumerate() {
                        if let Some(neigh) = self.find_cell((x + nx, y + ny, t)) {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.nbhd[i] = self.lift(neigh);
                            }
                        }
                    }
                }
            }
        }

        // Sets other information.
        for x in -1..=self.width {
            for y in -1..=self.height {
                for t in 0..self.period {
                    let cell_ptr: *mut _ = self.find_cell_mut((x, y, t)).unwrap();
                    let cell = self.find_cell((x, y, t)).unwrap();

                    // Default state of a cell.
                    let default = if self.rule.b0() && t % 2 == 1 {
                        Alive
                    } else {
                        Dead
                    };

                    // Sets the state of a cell.
                    if 0 <= x && x < self.width && 0 <= y && y < self.height {
                        self.set_cell(cell, None, true);
                    }

                    // Links a cell to the cell in the last generation.
                    //
                    // If that cell is out of the search range,
                    // then sets the state of the current cell to `default`.
                    if t != 0 {
                        let pred = self.find_cell((x, y, t - 1)).unwrap();
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.pred = self.lift(pred);
                        }
                    } else {
                        let (new_x, new_y) = match transform {
                            Transform::Id => (x, y),
                            Transform::Rotate90 => (self.height - 1 - y, x),
                            Transform::Rotate180 => (self.width - 1 - x, self.height - 1 - y),
                            Transform::Rotate270 => (y, self.width - 1 - x),
                            Transform::FlipRow => (x, self.height - 1 - y),
                            Transform::FlipCol => (self.width - 1 - x, y),
                            Transform::FlipDiag => (y, x),
                            Transform::FlipAntidiag => (self.height - 1 - y, self.width - 1 - x),
                        };
                        let pred = self.find_cell((new_x - dx, new_y - dy, self.period - 1));
                        if let Some(pred) = pred {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.pred = self.lift(pred);
                            }
                        } else if 0 <= x && x < self.width && 0 <= y && y < self.height {
                            self.set_cell(cell, Some(default), false);
                        }
                    }

                    // Links a cell to the cell in the next generation.
                    //
                    // If that cell is out of the search range,
                    // then sets it to `dead_cell`.
                    if t != self.period - 1 {
                        let succ = self.find_cell((x, y, t + 1)).unwrap();
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = self.lift(succ);
                        }
                    } else {
                        let (x, y) = (x + dx, y + dy);
                        let (new_x, new_y) = match transform {
                            Transform::Id => (x, y),
                            Transform::Rotate90 => (y, self.width - 1 - x),
                            Transform::Rotate180 => (self.width - 1 - x, self.height - 1 - y),
                            Transform::Rotate270 => (self.height - 1 - y, x),
                            Transform::FlipRow => (x, self.height - 1 - y),
                            Transform::FlipCol => (self.width - 1 - x, y),
                            Transform::FlipDiag => (y, x),
                            Transform::FlipAntidiag => (self.height - 1 - y, self.width - 1 - x),
                        };
                        let succ = self.find_cell((new_x, new_y, 0));
                        let succ = succ.unwrap_or(&self.dead_cell);
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = self.lift(succ);
                        }
                    }

                    // Links a cell to the symmetric cells.
                    //
                    // If some symmetric cell is out of the search range,
                    // then sets the current cell to `default`.
                    let sym_coords = match symmetry {
                        Symmetry::C1 => vec![],
                        Symmetry::C2 => vec![(self.width - 1 - x, self.height - 1 - y, t)],
                        Symmetry::C4 => vec![
                            (y, self.width - 1 - x, t),
                            (self.width - 1 - x, self.height - 1 - y, t),
                            (self.height - 1 - y, x, t),
                        ],
                        Symmetry::D2Row => vec![(x, self.height - 1 - y, t)],
                        Symmetry::D2Col => vec![(self.width - 1 - x, y, t)],
                        Symmetry::D2Diag => vec![(y, x, t)],
                        Symmetry::D2Antidiag => vec![(self.height - 1 - y, self.width - 1 - x, t)],
                        Symmetry::D4Ortho => vec![
                            (self.width - 1 - x, y, t),
                            (x, self.height - 1 - y, t),
                            (self.width - 1 - x, self.height - 1 - y, t),
                        ],
                        Symmetry::D4Diag => vec![
                            (y, x, t),
                            (self.height - 1 - y, self.width - 1 - x, t),
                            (self.width - 1 - x, self.height - 1 - y, t),
                        ],
                        Symmetry::D8 => vec![
                            (y, self.width - 1 - x, t),
                            (self.height - 1 - y, x, t),
                            (self.width - 1 - x, y, t),
                            (x, self.height - 1 - y, t),
                            (y, x, t),
                            (self.height - 1 - y, self.width - 1 - x, t),
                            (self.width - 1 - x, self.height - 1 - y, t),
                        ],
                    };
                    for coord in sym_coords {
                        if 0 <= coord.0
                            && coord.0 < self.width
                            && 0 <= coord.1
                            && coord.1 < self.height
                        {
                            let sym = self.find_cell(coord).unwrap();
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.sym.push(self.lift(sym).unwrap());
                            }
                        } else if 0 <= x && x < self.width && 0 <= y && y < self.height {
                            self.set_cell(cell, Some(default), false);
                        }
                    }
                }
            }
        }

        for cell in self.cells.iter() {
            // Sets the next cell in the search order.
            if cell.state.get().is_none() && cell.free.get() {
                let cell = unsafe { self.lift(cell).unwrap() };
                self.search_list.push(cell);
            }
        }
    }

    /// Lift the lifetime of a reference to a cell to `'a`.
    ///
    /// Only safe to use after the creation of the world,
    /// and only when the cell is in the world.
    unsafe fn lift(&self, cell: &LifeCell<'a, R>) -> Option<&'a LifeCell<'a, R>> {
        let cell_ptr: *const _ = cell;
        cell_ptr.as_ref()
    }

    /// Finds a cell by its coordinates. Returns a reference.
    fn find_cell(&self, coord: Coord) -> Option<&LifeCell<'a, R>> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = if self.column_first {
                ((x + 1) * (self.height + 2) + y + 1) * self.period + t
            } else {
                ((y + 1) * (self.width + 2) + x + 1) * self.period + t
            };
            Some(&self.cells[index as usize])
        } else {
            None
        }
    }

    /// Finds a cell by its coordinates. Returns a mutable reference.
    fn find_cell_mut(&mut self, coord: Coord) -> Option<&mut LifeCell<'a, R>> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = if self.column_first {
                ((x + 1) * (self.height + 2) + y + 1) * self.period + t
            } else {
                ((y + 1) * (self.width + 2) + x + 1) * self.period + t
            };
            Some(&mut self.cells[index as usize])
        } else {
            None
        }
    }

    /// Sets the `state` and `free` of a cell,
    /// and update the neighborhood descriptor of its neighbors.
    pub(crate) fn set_cell(&self, cell: &LifeCell<R>, state: Option<State>, free: bool) {
        let old_state = cell.state.replace(state);
        cell.free.set(free);
        R::update_desc(&cell, old_state, state);
        if cell.is_gen0 {
            match (state, old_state) {
                (Some(Alive), Some(Alive)) => (),
                (Some(Alive), _) => self.gen0_cell_count.set(self.gen0_cell_count.get() + 1),
                (_, Some(Alive)) => self.gen0_cell_count.set(self.gen0_cell_count.get() - 1),
                _ => (),
            }
        }
        if cell.is_front {
            match (state, old_state) {
                (Some(Dead), Some(Dead)) => (),
                (Some(Dead), _) => self.front_cell_count.set(self.front_cell_count.get() - 1),
                (_, Some(Dead)) => self.front_cell_count.set(self.front_cell_count.get() + 1),
                _ => (),
            }
        }
    }

    /// Display the whole world in some generation.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** cells are represented by `O`;
    /// * **Unknown** cells are represented by `?`.
    pub(crate) fn display_gen(&self, t: isize) -> String {
        let mut str = String::new();
        let t = t % self.period;
        for y in 0..self.height {
            for x in 0..self.width {
                let state = self.find_cell((x, y, t)).unwrap().state.get();
                let s = match state {
                    Some(Dead) => '.',
                    Some(Alive) => 'O',
                    None => '?',
                };
                str.push(s);
            }
            str.push('\n');
        }
        str
    }

    /// Get a references to the first unknown cell in the `search_list`.
    pub(crate) fn get_unknown(&self) -> Option<&'a LifeCell<'a, R>> {
        self.search_list
            .iter()
            .find(|cell| cell.state.get().is_none())
            .copied()
    }

    /// Tests whether the world is nonempty,
    /// and whether the minimal period of the pattern equals to the given period.
    pub(crate) fn nontrivial(&self) -> bool {
        self.gen0_cell_count.get() > 0
            && (1..self.period).all(|t| {
                self.period % t != 0
                    || self
                        .cells
                        .chunks(self.period as usize)
                        .any(|c| c[0].state.get() != c[t as usize].state.get())
            })
    }
}
