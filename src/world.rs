//! The world, symmetries, transformations, and other information.

use crate::{
    cells::{Alive, Dead, LifeCell, State},
    rules::Rule,
    syms_trans::{Symmetry, Transform},
};
use std::cell::Cell;

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

    /// A vector that stores all the cells in the search range.
    ///
    /// The vector will not be moved after it is created.
    /// All the cells will live throughout the lifetime of the world.
    // So the unsafe code below is actually safe.
    cells: Vec<LifeCell<'a, R>>,

    /// A list of references of cells sorted by the search order.search
    ///
    /// Used to find unknown cells.
    search_list: Vec<&'a LifeCell<'a, R>>,

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
        for x in -1..=width {
            for y in -1..=height {
                for t in 0..period {
                    let state = if rule.b0() && t % 2 == 1 { Alive } else { Dead };
                    let mut cell = LifeCell::new(state, rule.b0());
                    if t == 0 {
                        cell.is_gen0 = true;
                    }
                    if (column_first && x == 0) || (!column_first && y == 0) {
                        cell.is_front = true;
                    }
                    cells.push(cell);
                }
            }
        }

        let search_list = Vec::new();

        let gen0_cell_count = Cell::new(0);
        let front_cell_count = Cell::new(0);

        World {
            width,
            height,
            period,
            rule,
            cells,
            search_list,
            gen0_cell_count,
            front_cell_count,
        }
        .init_nbhd()
        .init_pred_succ(dx, dy, transform)
        .init_sym(symmetry)
        .init_state()
        .init_search_order(column_first)
    }

    /// Links the cells to their neighbors.
    ///
    /// Note that for cells on the edges of the search range,
    /// some neighbors might point to `None`.
    fn init_nbhd(mut self) -> Self {
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
                                cell.nbhd[i] = neigh.extend_life();
                            }
                        }
                    }
                }
            }
        }
        self
    }

    /// Links a cell to its predecessor and successor.
    ///
    /// If the predecessor is out of the search range,
    /// then sets the state of the current cell to `default`.
    ///
    /// If the successor is out of the search range,
    /// then sets it to `None`.
    fn init_pred_succ(mut self, dx: isize, dy: isize, transform: Transform) -> Self {
        for x in -1..=self.width {
            for y in -1..=self.height {
                for t in 0..self.period {
                    let cell_ptr: *mut _ = self.find_cell_mut((x, y, t)).unwrap();
                    let cell = self.find_cell((x, y, t)).unwrap();

                    if t != 0 {
                        let pred = self.find_cell((x, y, t - 1)).unwrap();
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.pred = pred.extend_life();
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
                                cell.pred = pred.extend_life();
                            }
                        } else if 0 <= x && x < self.width && 0 <= y && y < self.height {
                            // Temperately marks its state as `None`.
                            // Will restore in `init_state`.
                            cell.state.set(None);
                        }
                    }

                    if t != self.period - 1 {
                        let succ = self.find_cell((x, y, t + 1)).unwrap();
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = succ.extend_life();
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
                        if let Some(succ) = succ {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.succ = succ.extend_life();
                            }
                        }
                    }
                }
            }
        }
        self
    }

    /// Links a cell to the symmetric cells.
    ///
    /// If some symmetric cell is out of the search range,
    /// then sets the current cell to `default`.
    fn init_sym(mut self, symmetry: Symmetry) -> Self {
        for x in -1..=self.width {
            for y in -1..=self.height {
                for t in 0..self.period {
                    let cell_ptr: *mut _ = self.find_cell_mut((x, y, t)).unwrap();
                    let cell = self.find_cell((x, y, t)).unwrap();

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
                                cell.sym.push(sym.extend_life().unwrap());
                            }
                        } else if 0 <= x && x < self.width && 0 <= y && y < self.height {
                            // Temperately marks its state as `None`.
                            // Will restore in `init_state`.
                            cell.state.set(None);
                        }
                    }
                }
            }
        }
        self
    }

    /// Sets states for the cells.
    fn init_state(self) -> Self {
        for x in 0..self.width {
            for y in 0..self.height {
                for t in 0..self.period {
                    let cell = self.find_cell((x, y, t)).unwrap();
                    if cell.state.get().is_some() {
                        self.set_cell(cell, None);
                    } else {
                        cell.state.set(Some(cell.default_state));
                    }
                }
            }
        }
        self
    }

    /// Sets the search order.
    ///
    /// This method will be called only once, inside `World::new`.
    fn init_search_order(mut self, column_first: bool) -> Self {
        if column_first {
            for x in 0..self.width {
                for y in 0..self.height {
                    for t in 0..self.period {
                        let cell = self.find_cell((x, y, t)).unwrap();
                        let cell = unsafe { cell.extend_life().unwrap() };
                        self.search_list.push(cell);
                    }
                }
            }
        } else {
            for y in 0..self.height {
                for x in 0..self.width {
                    for t in 0..self.period {
                        let cell = self.find_cell((x, y, t)).unwrap();
                        let cell = unsafe { cell.extend_life().unwrap() };
                        self.search_list.push(cell);
                    }
                }
            }
        }
        self
    }

    /// Finds a cell by its coordinates. Returns a reference.
    fn find_cell(&self, coord: Coord) -> Option<&LifeCell<'a, R>> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = ((x + 1) * (self.height + 2) + y + 1) * self.period + t;
            Some(&self.cells[index as usize])
        } else {
            None
        }
    }

    /// Finds a cell by its coordinates. Returns a mutable reference.
    fn find_cell_mut(&mut self, coord: Coord) -> Option<&mut LifeCell<'a, R>> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = ((x + 1) * (self.height + 2) + y + 1) * self.period + t;
            Some(&mut self.cells[index as usize])
        } else {
            None
        }
    }

    /// Sets the `state` of a cell,
    /// and update the neighborhood descriptor of its neighbors.
    pub(crate) fn set_cell(&self, cell: &LifeCell<'a, R>, state: Option<State>) {
        let old_state = cell.state.replace(state);
        if old_state != state {
            cell.update_desc(old_state, state);
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

    /// Get a references to the first unknown cell since `index` in the `search_list`.
    pub(crate) fn get_unknown(&self, index: usize) -> Option<(usize, &'a LifeCell<'a, R>)> {
        self.search_list[index..]
            .iter()
            .enumerate()
            .find_map(|(i, cell)| {
                if cell.state.get().is_none() {
                    Some((i + index, *cell))
                } else {
                    None
                }
            })
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
