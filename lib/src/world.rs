//! The world.

use crate::{
    cells::{Alive, Dead, LifeCell, State},
    config::{Config, NewState, SearchOrder, Symmetry, Transform},
    rules::Rule,
    search::{Reason, SetCell},
};

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
    pub(crate) gen0_cell_count: u32,

    /// Number of unknown or living cells in the first generation.
    pub(crate) front_cell_count: u32,

    /// How to choose a state for an unknown cell.
    pub(crate) new_state: NewState,

    /// A stack to records the cells whose values are set during the search.
    ///
    /// The cells in this table always have known states.
    ///
    /// It is used in the backtracking.
    pub(crate) set_stack: Vec<SetCell<'a, R>>,

    /// The position in the `set_stack` of the next cell to be examined.
    ///
    /// See `proceed` for details.
    pub(crate) check_index: usize,

    /// The position in the `search_list` of the last decided cell.
    pub(crate) search_index: usize,

    /// The number of living cells in the 0th generation must not exceed
    /// this number.
    ///
    /// `None` means that there is no limit for the cell count.
    pub(crate) max_cell_count: Option<u32>,

    /// Whether to force the first row/column to be nonempty.
    ///
    /// Here 'front' means the first row or column to search,
    /// according to the search order.
    pub(crate) non_empty_front: bool,
}

impl<'a, R: Rule> World<'a, R> {
    /// Creates a new world from the configuration and the rule.
    ///
    /// In rules that contain `B0`, cells outside the search range are
    /// considered `Dead` in even generations, `Alive` in odd generations.
    /// In other rules, all cells outside the search range are `Dead`.
    ///
    /// After the last generation, the pattern will return to
    /// the first generation, applying the transformation first,
    /// and then the translation defined by `dx` and `dy`.
    pub fn new(config: &Config, rule: R) -> Self {
        let search_order = config.auto_search_order();

        let size = ((config.width + 2) * (config.height + 2) * config.period) as usize;
        let mut cells = Vec::with_capacity(size);

        // Fills the vector with dead cells.
        // If the rule contains `B0`, then fills the odd generations
        // with living cells instead.
        for x in -1..=config.width {
            for y in -1..=config.height {
                for t in 0..config.period {
                    let state = if rule.b0() && t % 2 == 1 { Alive } else { Dead };
                    let mut cell = LifeCell::new(state, rule.b0());
                    if t == 0 {
                        cell.is_gen0 = true;
                    }
                    match search_order {
                        SearchOrder::ColumnFirst => {
                            if x == 0 {
                                cell.is_front = true
                            }
                        }
                        SearchOrder::RowFirst => {
                            if y == 0 {
                                cell.is_front = true
                            }
                        }
                    }
                    cells.push(cell);
                }
            }
        }

        World {
            width: config.width,
            height: config.height,
            period: config.period,
            rule,
            cells,
            search_list: Vec::with_capacity(size),
            gen0_cell_count: 0,
            front_cell_count: 0,
            new_state: config.new_state,
            set_stack: Vec::with_capacity(size),
            check_index: 0,
            search_index: 0,
            max_cell_count: config.max_cell_count,
            non_empty_front: config.non_empty_front,
        }
        .init_nbhd()
        .init_pred_succ(config.dx, config.dy, config.transform)
        .init_sym(config.symmetry)
        .init_state()
        .init_search_order(search_order)
    }

    /// Links the cells to their neighbors.
    ///
    /// Note that for cells on the edges of the search range,
    /// some neighbors might point to `None`.
    fn init_nbhd(mut self) -> Self {
        const NBHD: [(isize, isize); 8] = [
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
                    let cell_ptr = self.find_cell_mut((x, y, t)).unwrap();
                    for (i, (nx, ny)) in NBHD.iter().enumerate() {
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.nbhd[i] = self.find_cell((x + nx, y + ny, t));
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
                    let cell_ptr = self.find_cell_mut((x, y, t)).unwrap();
                    let cell = self.find_cell((x, y, t)).unwrap();

                    if t != 0 {
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.pred = self.find_cell((x, y, t - 1));
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
                        if pred.is_some() {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.pred = pred;
                            }
                        } else if 0 <= x && x < self.width && 0 <= y && y < self.height {
                            // Temperately marks its state as `None`.
                            // Will restore in `init_state`.
                            cell.state.set(None);
                        }
                    }

                    if t != self.period - 1 {
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = self.find_cell((x, y, t + 1));
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
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = self.find_cell((new_x, new_y, 0));
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
                    let cell_ptr = self.find_cell_mut((x, y, t)).unwrap();
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
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.sym.push(self.find_cell(coord).unwrap());
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
    fn init_state(mut self) -> Self {
        for x in 0..self.width {
            for y in 0..self.height {
                for t in 0..self.period {
                    let cell = self.find_cell((x, y, t)).unwrap();
                    if cell.state.get().is_some() {
                        self.clear_cell(cell);
                    } else {
                        cell.state.set(Some(cell.background));
                    }
                }
            }
        }
        self
    }

    /// Sets the search order.
    ///
    /// This method will be called only once, inside `World::new`.
    fn init_search_order(mut self, search_order: SearchOrder) -> Self {
        match search_order {
            SearchOrder::ColumnFirst => {
                for x in 0..self.width {
                    for y in 0..self.height {
                        for t in 0..self.period {
                            let cell = self.find_cell((x, y, t)).unwrap();
                            self.search_list.push(cell);
                        }
                    }
                }
            }
            SearchOrder::RowFirst => {
                for y in 0..self.height {
                    for x in 0..self.width {
                        for t in 0..self.period {
                            let cell = self.find_cell((x, y, t)).unwrap();
                            self.search_list.push(cell);
                        }
                    }
                }
            }
        }
        self
    }

    /// Finds a cell by its coordinates. Returns a reference that lives
    /// as long as the world.
    fn find_cell(&self, coord: Coord) -> Option<&'a LifeCell<'a, R>> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = ((x + 1) * (self.height + 2) + y + 1) * self.period + t;
            let cell = &self.cells[index as usize];
            unsafe { (cell as *const LifeCell<'a, R>).as_ref() }
        } else {
            None
        }
    }

    /// Finds a cell by its coordinates. Returns a mutable pointer.
    fn find_cell_mut(&mut self, coord: Coord) -> Option<*mut LifeCell<'a, R>> {
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
    pub(crate) fn set_cell(&mut self, cell: &'a LifeCell<'a, R>, state: State, reason: Reason) {
        let old_state = cell.state.replace(Some(state));
        if old_state != Some(state) {
            cell.update_desc(old_state, Some(state));
            if cell.is_gen0 {
                match (state, old_state) {
                    (Alive, Some(Alive)) => (),
                    (Alive, _) => self.gen0_cell_count += 1,
                    (_, Some(Alive)) => self.gen0_cell_count -= 1,
                    _ => (),
                }
            }
            if cell.is_front {
                match (state, old_state) {
                    (Dead, Some(Dead)) => (),
                    (Dead, _) => self.front_cell_count -= 1,
                    (_, Some(Dead)) => self.front_cell_count += 1,
                    _ => (),
                }
            }
        }
        self.set_stack.push(SetCell::new(cell, reason));
    }

    /// Clears the `state` of a cell,
    /// and update the neighborhood descriptor of its neighbors.
    pub(crate) fn clear_cell(&mut self, cell: &'a LifeCell<'a, R>) {
        let old_state = cell.state.take();
        if old_state != None {
            cell.update_desc(old_state, None);
            if cell.is_gen0 && old_state == Some(Alive) {
                self.gen0_cell_count -= 1;
            }
            if cell.is_front && old_state == Some(Dead) {
                self.front_cell_count += 1;
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
        self.gen0_cell_count > 0
            && (1..self.period).all(|t| {
                self.period % t != 0
                    || self
                        .cells
                        .chunks(self.period as usize)
                        .any(|c| c[0].state.get() != c[t as usize].state.get())
            })
    }
}
