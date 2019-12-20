//! The world.

use crate::{
    cells::{CellRef, Coord, LifeCell, State, ALIVE, DEAD},
    config::{Config, SearchOrder, Symmetry, Transform},
    rules::Rule,
    search::{Reason, SetCell},
};
use std::fmt::Write;

/// The world.
pub struct World<'a, R: Rule> {
    /// World configuration.
    pub(crate) config: Config,

    /// The rule of the cellular automaton.
    pub(crate) rule: R,

    /// A vector that stores all the cells in the search range.
    ///
    /// This vector will not be moved after its creation.
    /// All the cells will live throughout the lifetime of the world.
    // So the unsafe code below is actually safe.
    cells: Vec<LifeCell<'a, R>>,

    /// A list of references to cells sorted by the search order.
    ///
    /// Used to find unknown cells.
    search_list: Vec<CellRef<'a, R>>,

    /// Number of known living cells in each generation.
    pub(crate) cell_count: Vec<usize>,

    /// Number of unknown or living cells on the first row or column.
    pub(crate) front_cell_count: usize,

    /// Number of conflicts during the search.
    pub(crate) conflicts: u64,

    /// A stack to record the cells whose values are set during the search.
    ///
    /// The cells in this stack always have known states.
    ///
    /// It is used in the backtracking.
    pub(crate) set_stack: Vec<SetCell<'a, R>>,

    /// The position in the `set_stack` of the next cell to be examined.
    ///
    /// See `proceed` for details.
    pub(crate) check_index: usize,

    /// The starting position in the `search_list` to look for an unknown cell.
    ///
    /// Cells before this position are all known.
    pub(crate) search_index: usize,
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

        // Whether to consider only the first generation of the front.
        let front_gen0 = match search_order {
            SearchOrder::ColumnFirst => {
                config.dy == 0
                    && config.dx >= 0
                    && (config.transform == Transform::Id || config.transform == Transform::FlipRow)
            }
            SearchOrder::RowFirst => {
                config.dx == 0
                    && config.dy >= 0
                    && (config.transform == Transform::Id || config.transform == Transform::FlipCol)
            }
        };

        // Whether to consider only half of the first generation of the front.
        let front_half = match config.symmetry {
            Symmetry::D2Diag | Symmetry::D2Antidiag | Symmetry::D4Diag => false,
            _ => front_gen0,
        };

        // Fills the vector with dead cells,
        // and checks whether it is on the first row or column.
        //
        // If the rule contains `B0`, then fills the odd generations
        // with living cells instead.
        for x in -1..=config.width {
            for y in -1..=config.height {
                for t in 0..config.period {
                    let state = if rule.has_b0() && t % 2 == 1 {
                        ALIVE
                    } else {
                        DEAD
                    };
                    let mut cell = LifeCell::new((x, y, t), state, rule.has_b0());
                    match search_order {
                        SearchOrder::ColumnFirst => {
                            if front_gen0 {
                                if x == (config.dx - 1).max(0)
                                    && t == 0
                                    && (!front_half || 2 * y < config.height)
                                {
                                    cell.is_front = true
                                }
                            } else if x == 0 {
                                cell.is_front = true
                            }
                        }
                        SearchOrder::RowFirst => {
                            if front_gen0 {
                                if y == (config.dy - 1).max(0)
                                    && t == 0
                                    && (!front_half || 2 * x < config.width)
                                {
                                    cell.is_front = true
                                }
                            } else if y == 0 {
                                cell.is_front = true
                            }
                        }
                    }
                    cells.push(cell);
                }
            }
        }

        World {
            config: config.clone(),
            rule,
            cells,
            search_list: Vec::with_capacity(size),
            cell_count: vec![0; config.period as usize],
            front_cell_count: 0,
            conflicts: 0,
            set_stack: Vec::with_capacity(size),
            check_index: 0,
            search_index: 0,
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
        for x in -1..=self.config.width {
            for y in -1..=self.config.height {
                for t in 0..self.config.period {
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
    /// then marks the current cell as known.
    ///
    /// If the successor is out of the search range,
    /// then sets it to `None`.
    fn init_pred_succ(mut self, dx: isize, dy: isize, transform: Transform) -> Self {
        for x in -1..=self.config.width {
            for y in -1..=self.config.height {
                for t in 0..self.config.period {
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
                            Transform::Rotate90 => (self.config.height - 1 - y, x),
                            Transform::Rotate180 => {
                                (self.config.width - 1 - x, self.config.height - 1 - y)
                            }
                            Transform::Rotate270 => (y, self.config.width - 1 - x),
                            Transform::FlipRow => (x, self.config.height - 1 - y),
                            Transform::FlipCol => (self.config.width - 1 - x, y),
                            Transform::FlipDiag => (y, x),
                            Transform::FlipAntidiag => {
                                (self.config.height - 1 - y, self.config.width - 1 - x)
                            }
                        };
                        let pred = self.find_cell((new_x - dx, new_y - dy, self.config.period - 1));
                        if pred.is_some() {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.pred = pred;
                            }
                        } else if 0 <= x
                            && x < self.config.width
                            && 0 <= y
                            && y < self.config.height
                            && !self.set_stack.iter().any(|s| s.cell == cell)
                        {
                            self.set_stack.push(SetCell::new(cell, Reason::Deduce));
                        }
                    }

                    if t != self.config.period - 1 {
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = self.find_cell((x, y, t + 1));
                        }
                    } else {
                        let (x, y) = (x + dx, y + dy);
                        let (new_x, new_y) = match transform {
                            Transform::Id => (x, y),
                            Transform::Rotate90 => (y, self.config.width - 1 - x),
                            Transform::Rotate180 => {
                                (self.config.width - 1 - x, self.config.height - 1 - y)
                            }
                            Transform::Rotate270 => (self.config.height - 1 - y, x),
                            Transform::FlipRow => (x, self.config.height - 1 - y),
                            Transform::FlipCol => (self.config.width - 1 - x, y),
                            Transform::FlipDiag => (y, x),
                            Transform::FlipAntidiag => {
                                (self.config.height - 1 - y, self.config.width - 1 - x)
                            }
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
    /// then  marks the current cell as known.
    fn init_sym(mut self, symmetry: Symmetry) -> Self {
        for x in -1..=self.config.width {
            for y in -1..=self.config.height {
                for t in 0..self.config.period {
                    let cell_ptr = self.find_cell_mut((x, y, t)).unwrap();
                    let cell = self.find_cell((x, y, t)).unwrap();

                    let sym_coords = match symmetry {
                        Symmetry::C1 => vec![],
                        Symmetry::C2 => {
                            vec![(self.config.width - 1 - x, self.config.height - 1 - y, t)]
                        }
                        Symmetry::C4 => vec![
                            (y, self.config.width - 1 - x, t),
                            (self.config.width - 1 - x, self.config.height - 1 - y, t),
                            (self.config.height - 1 - y, x, t),
                        ],
                        Symmetry::D2Row => vec![(x, self.config.height - 1 - y, t)],
                        Symmetry::D2Col => vec![(self.config.width - 1 - x, y, t)],
                        Symmetry::D2Diag => vec![(y, x, t)],
                        Symmetry::D2Antidiag => {
                            vec![(self.config.height - 1 - y, self.config.width - 1 - x, t)]
                        }
                        Symmetry::D4Ortho => vec![
                            (self.config.width - 1 - x, y, t),
                            (x, self.config.height - 1 - y, t),
                            (self.config.width - 1 - x, self.config.height - 1 - y, t),
                        ],
                        Symmetry::D4Diag => vec![
                            (y, x, t),
                            (self.config.height - 1 - y, self.config.width - 1 - x, t),
                            (self.config.width - 1 - x, self.config.height - 1 - y, t),
                        ],
                        Symmetry::D8 => vec![
                            (y, self.config.width - 1 - x, t),
                            (self.config.height - 1 - y, x, t),
                            (self.config.width - 1 - x, y, t),
                            (x, self.config.height - 1 - y, t),
                            (y, x, t),
                            (self.config.height - 1 - y, self.config.width - 1 - x, t),
                            (self.config.width - 1 - x, self.config.height - 1 - y, t),
                        ],
                    };
                    for coord in sym_coords {
                        if 0 <= coord.0
                            && coord.0 < self.config.width
                            && 0 <= coord.1
                            && coord.1 < self.config.height
                        {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.sym.push(self.find_cell(coord).unwrap());
                            }
                        } else if 0 <= x
                            && x < self.config.width
                            && 0 <= y
                            && y < self.config.height
                            && !self.set_stack.iter().any(|s| s.cell == cell)
                        {
                            self.set_stack.push(SetCell::new(cell, Reason::Deduce));
                        }
                    }
                }
            }
        }
        self
    }

    /// Sets states for the cells.
    ///
    /// All cells are set to unknown unless they are on the boundary,
    /// or are marked as known in `init_pred_succ` or `init_sym`.
    fn init_state(mut self) -> Self {
        for x in 0..self.config.width {
            for y in 0..self.config.height {
                for t in 0..self.config.period {
                    let cell = self.find_cell((x, y, t)).unwrap();
                    if !self.set_stack.iter().any(|s| s.cell == cell) {
                        self.clear_cell(cell);
                    }
                }
            }
        }
        self
    }

    /// Sets the search order.
    fn init_search_order(mut self, search_order: SearchOrder) -> Self {
        match search_order {
            SearchOrder::ColumnFirst => {
                for x in 0..self.config.width {
                    for y in 0..self.config.height {
                        for t in 0..self.config.period {
                            let cell = self.find_cell((x, y, t)).unwrap();
                            self.search_list.push(cell);
                        }
                    }
                }
            }
            SearchOrder::RowFirst => {
                for y in 0..self.config.height {
                    for x in 0..self.config.width {
                        for t in 0..self.config.period {
                            let cell = self.find_cell((x, y, t)).unwrap();
                            self.search_list.push(cell);
                        }
                    }
                }
            }
        }
        self
    }

    /// Finds a cell by its coordinates. Returns a `CellRef`.
    pub(crate) fn find_cell(&self, coord: Coord) -> Option<CellRef<'a, R>> {
        let (x, y, t) = coord;
        if x >= -1
            && x <= self.config.width
            && y >= -1
            && y <= self.config.height
            && t >= 0
            && t < self.config.period
        {
            let index = ((x + 1) * (self.config.height + 2) + y + 1) * self.config.period + t;
            let cell = &self.cells[index as usize];
            Some(cell.borrow())
        } else {
            None
        }
    }

    /// Finds a cell by its coordinates. Returns a mutable pointer.
    fn find_cell_mut(&mut self, coord: Coord) -> Option<*mut LifeCell<'a, R>> {
        let (x, y, t) = coord;
        if x >= -1
            && x <= self.config.width
            && y >= -1
            && y <= self.config.height
            && t >= 0
            && t < self.config.period
        {
            let index = ((x + 1) * (self.config.height + 2) + y + 1) * self.config.period + t;
            Some(&mut self.cells[index as usize])
        } else {
            None
        }
    }

    /// Sets the `state` of a cell, push it to the `set_stack`,
    /// and update the neighborhood descriptor of its neighbors.
    ///
    /// The original state of the cell must be unknown.
    ///
    /// Return `false` if the number of living cells exceeds the `max_cell_count`
    /// or the front becomes empty.
    pub(crate) fn set_cell(&mut self, cell: CellRef<'a, R>, state: State, reason: Reason) -> bool {
        cell.state.set(Some(state));
        let mut result = true;
        cell.update_desc(None, Some(state));
        if state == ALIVE {
            self.cell_count[cell.coord.2 as usize] += 1;
            if let Some(max) = self.config.max_cell_count {
                if *self.cell_count.iter().min().unwrap() > max {
                    result = false;
                }
            }
        }
        if cell.is_front && state == DEAD {
            self.front_cell_count -= 1;
            if self.config.non_empty_front && self.front_cell_count == 0 {
                result = false;
            }
        }
        self.set_stack.push(SetCell::new(cell, reason));
        result
    }

    /// Clears the `state` of a cell,
    /// and update the neighborhood descriptor of its neighbors.
    pub(crate) fn clear_cell(&mut self, cell: CellRef<'a, R>) {
        let old_state = cell.state.take();
        if old_state != None {
            cell.update_desc(old_state, None);
            if old_state == Some(ALIVE) {
                self.cell_count[cell.coord.2 as usize] -= 1;
            }
            if cell.is_front && old_state == Some(DEAD) {
                self.front_cell_count += 1;
            }
        }
    }

    /// Displays the whole world in some generation.
    ///
    /// Uses a mix of [Plaintext](https://www.conwaylife.com/wiki/Plaintext) and
    /// [RLE](https://www.conwaylife.com/wiki/Rle) format.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** cells are represented by `o` for rules with 2 states,
    ///   `A` for rules with more states;
    /// * **Dying** cells are represented by uppercase letters starting from `B`;
    /// * **Unknown** cells are represented by `?`.
    pub(crate) fn display_gen(&self, t: isize) -> String {
        let mut str = String::new();
        writeln!(
            str,
            "x = {}, y = {}, rule = {}",
            self.config.width, self.config.height, self.config.rule_string
        )
        .unwrap();
        let t = t % self.config.period;
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let state = self.find_cell((x, y, t)).unwrap().state.get();
                match state {
                    Some(DEAD) => str.push('.'),
                    Some(ALIVE) => {
                        if self.rule.gen() == 2 {
                            str.push('o')
                        } else {
                            str.push('A')
                        }
                    }
                    Some(State(i)) => str.push((b'A' + i as u8 - 1) as char),
                    _ => str.push('?'),
                };
            }
            if y == self.config.height - 1 {
                str.push('!')
            } else {
                str.push('$')
            };
            str.push('\n');
        }
        str
    }

    /// Gets a references to the first unknown cell since `index` in the `search_list`.
    pub(crate) fn get_unknown(&self, index: usize) -> Option<(usize, CellRef<'a, R>)> {
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
        self.cell_count[0] > 0
            && (1..self.config.period).all(|t| {
                self.config.period % t != 0
                    || self
                        .cells
                        .chunks(self.config.period as usize)
                        .any(|c| c[0].state.get() != c[t as usize].state.get())
            })
    }
}
