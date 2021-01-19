//! The world.

use crate::{
    cells::{CellRef, Coord, LifeCell, State, DEAD},
    config::{Config, SearchOrder, Symmetry, Transform},
    error::Error,
    rules::Rule,
    search::{Reason, SetCell},
};
use std::mem;

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
    // So the unsafe codes below are actually safe.
    cells: Vec<LifeCell<'a, R>>,

    /// Number of known living cells in each generation.
    ///
    /// For Generations rules, dying cells are not counted.
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

    /// The position of the next cell to be examined in the [`set_stack`](#structfield.set_stack).
    ///
    /// See [`proceed`](Self::proceed) for details.
    pub(crate) check_index: usize,

    /// The starting point to look for an unknown cell.
    ///
    /// There must be no unknown cell before this cell.
    pub(crate) next_unknown: Option<CellRef<'a, R>>,
}

impl<'a, R: Rule> World<'a, R> {
    /// Creates a new world from the configuration and the rule.
    pub fn new(config: &Config, rule: R) -> Self {
        let search_order = config.auto_search_order();

        let size = ((config.width + 2) * (config.height + 2) * config.period) as usize;
        let mut cells = Vec::with_capacity(size);

        // Whether to consider only the first generation of the front.
        let front_gen0 = !rule.has_b0()
            && config.diagonal_width.is_none()
            && match search_order {
                SearchOrder::ColumnFirst => {
                    config.dy == 0
                        && config.dx >= 0
                        && (config.transform == Transform::Id
                            || config.transform == Transform::FlipRow)
                }
                SearchOrder::RowFirst => {
                    config.dx == 0
                        && config.dy >= 0
                        && (config.transform == Transform::Id
                            || config.transform == Transform::FlipCol)
                }
                SearchOrder::Diagonal => false,
            };

        // Whether to consider only half of the first generation of the front.
        let front_half = config.diagonal_width.is_none()
            && match config.symmetry {
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
                    let state = if rule.has_b0() {
                        State(t as usize % rule.gen())
                    } else {
                        DEAD
                    };
                    let succ_state = if rule.has_b0() {
                        State((t as usize + 1) % rule.gen())
                    } else {
                        DEAD
                    };
                    let mut cell = LifeCell::new((x, y, t), state, succ_state);
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
                        SearchOrder::Diagonal => {
                            if x == 0 || y == 0 {
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
            cell_count: vec![0; config.period as usize],
            front_cell_count: 0,
            conflicts: 0,
            set_stack: Vec::with_capacity(size),
            check_index: 0,
            next_unknown: None,
        }
        .init_nbhd()
        .init_pred_succ()
        .init_sym()
        .init_state()
        .init_search_order()
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
    fn init_pred_succ(mut self) -> Self {
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
                        let pred = self.find_cell(self.config.translate((x, y, t - 1)));
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
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = self.find_cell(self.config.translate((x, y, t + 1)));
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
    fn init_sym(mut self) -> Self {
        for x in -1..=self.config.width {
            for y in -1..=self.config.height {
                for t in 0..self.config.period {
                    let cell_ptr = self.find_cell_mut((x, y, t)).unwrap();
                    let cell = self.find_cell((x, y, t)).unwrap();

                    let sym_coords = match self.config.symmetry {
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
    /// or are marked as known in [`init_pred_succ`](Self::init_pred_succ)
    /// or [`init_sym`](Self::init_sym).
    fn init_state(mut self) -> Self {
        for x in 0..self.config.width {
            for y in 0..self.config.height {
                if let Some(d) = self.config.diagonal_width {
                    if (x - y).abs() >= d {
                        continue;
                    }
                }
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

    /// Set the [`next`](LifeCell#structfield.next) of a cell to be
    /// [`next_unknown`](#structfield.next_unknown) and set
    /// [`next_unknown`](#structfield.next_unknown) to be this cell.
    fn set_next(&mut self, coord: Coord) {
        if let Some(cell) = self.find_cell(coord) {
            if cell.state.get().is_none() {
                let next = mem::replace(&mut self.next_unknown, Some(cell));
                let cell_ptr = self.find_cell_mut(coord).unwrap();
                unsafe {
                    let cell = cell_ptr.as_mut().unwrap();
                    cell.next = next;
                }
            }
        }
    }

    /// Sets the search order.
    fn init_search_order(mut self) -> Self {
        for coord in self.config.search_order_iter() {
            self.set_next(coord);
        }
        self
    }

    /// Finds a cell by its coordinates. Returns a [`CellRef`].
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

    /// Sets the [`state`](LifeCell#structfield.state) of a cell,
    /// push it to the [`set_stack`](#structfield.set_stack),
    /// and update the neighborhood descriptor of its neighbors.
    ///
    /// The original state of the cell must be unknown.
    ///
    /// Return `false` if the number of living cells exceeds the
    /// [`max_cell_count`](#structfield.max_cell_count) or the front becomes empty.
    pub(crate) fn set_cell(&mut self, cell: CellRef<'a, R>, state: State, reason: Reason) -> bool {
        cell.state.set(Some(state));
        let mut result = true;
        cell.update_desc(Some(state), true);
        if state == !cell.background {
            self.cell_count[cell.coord.2 as usize] += 1;
            if let Some(max) = self.config.max_cell_count {
                if self.cell_count() > max {
                    result = false;
                }
            }
        }
        if cell.is_front && state == cell.background {
            self.front_cell_count -= 1;
            if self.config.non_empty_front && self.front_cell_count == 0 {
                result = false;
            }
        }
        self.set_stack.push(SetCell::new(cell, reason));
        result
    }

    /// Clears the [`state`](LifeCell#structfield.state) of a cell,
    /// and update the neighborhood descriptor of its neighbors.
    pub(crate) fn clear_cell(&mut self, cell: CellRef<'a, R>) {
        let old_state = cell.state.take();
        if old_state != None {
            cell.update_desc(old_state, false);
            if old_state == Some(!cell.background) {
                self.cell_count[cell.coord.2 as usize] -= 1;
            }
            if cell.is_front && old_state == Some(cell.background) {
                self.front_cell_count += 1;
            }
        }
    }

    /// Gets a references to the first unknown cell since [`next_unknown`](#structfield.next_unknown).
    pub(crate) fn get_unknown(&mut self) -> Option<CellRef<'a, R>> {
        while let Some(cell) = self.next_unknown {
            if cell.state.get().is_none() {
                return Some(cell);
            } else {
                self.next_unknown = cell.next;
            }
        }
        None
    }

    /// Tests if the result is "uninteresting".
    pub(crate) fn is_uninteresting(&self) -> bool {
        self.is_trivial() || self.is_subperiod_oscillator() || self.is_subperiod_spaceship()
    }

    /// Tests if the result is trivial.
    fn is_trivial(&self) -> bool {
        self.cell_count[0] == 0
    }

    /// Tests if the result is an oscillator of subperiod.
    fn is_subperiod_oscillator(&self) -> bool {
        self.config.period > 1
            && (1..self.config.period).any(|t| {
                self.config.period % t == 0
                    && self
                        .cells
                        .chunks(self.config.period as usize)
                        .all(|c| c[0].state.get() == c[t as usize].state.get())
            })
    }

    /// Tests if the result is a spaceship of subperiod.
    fn is_subperiod_spaceship(&self) -> bool {
        (self.config.dx != 0 || self.config.dy != 0)
            && (1..self.config.period).any(|f| {
                self.config.period % f == 0
                    && self.config.dx % f == 0
                    && self.config.dy % f == 0
                    && {
                        let t = self.config.period / f;
                        let dx = self.config.dx / f;
                        let dy = self.config.dy / f;
                        self.cells
                            .iter()
                            .step_by(self.config.period as usize)
                            .all(|c| {
                                let (x, y, _) = c.coord;
                                c.state.get()
                                    == self.find_cell((x - dx, y - dy, t)).map_or_else(
                                        || self.find_cell((x, y, t)).map(|c1| c1.background),
                                        |c1| c1.state.get(),
                                    )
                            })
                    }
            })
    }

    /// Gets the state of a cell. Returns `Err(())` if there is no such cell.
    pub fn get_cell_state(&self, coord: Coord) -> Result<Option<State>, Error> {
        self.find_cell(self.config.translate(coord))
            .map(|cell| cell.state.get())
            .ok_or(Error::GetCellError(coord))
    }

    /// Minimum number of known living cells in all generation.
    ///
    /// For Generations rules, dying cells are not counted.
    pub(crate) fn cell_count(&self) -> usize {
        *self.cell_count.iter().min().unwrap()
    }
}
