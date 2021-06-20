//! The world.

use crate::{
    cells::{CellRef, Coord, LifeCell, State, ALIVE, DEAD},
    config::{Config, KnownCell, SearchOrder},
    rules::Rule,
    search::{Algorithm, Backjump, LifeSrc, Reason, SetCell},
};
use std::{convert::TryInto, fmt::Write, mem};
use typebool::{Bool, False};

/// The world.
pub struct World<R: Rule, A: Algorithm<R>> {
    /// World configuration.
    pub(crate) config: Config,

    /// The rule of the cellular automaton.
    pub(crate) rule: R,

    /// A vector that stores all the cells in the search range.
    ///
    /// This vector will not be moved after its creation.
    /// All the cells will live throughout the lifetime of the world.
    cells: Box<[LifeCell<R>]>,

    /// Number of known living cells in each generation.
    ///
    /// For Generations rules, dying cells are not counted.
    pub(crate) cell_count: Vec<u32>,

    /// Number of unknown or living cells on the first row or column.
    pub(crate) front_cell_count: u32,

    /// Number of conflicts during the search.
    pub(crate) conflicts: u64,

    /// A stack to record the cells whose values are set during the search.
    ///
    /// The cells in this stack always have known states.
    ///
    /// It is used in the backtracking.
    pub(crate) set_stack: Vec<SetCell<R, A>>,

    /// The position of the next cell to be examined in the [`set_stack`](#structfield.set_stack).
    ///
    /// See [`proceed`](Self::proceed) for details.
    pub(crate) check_index: u32,

    /// The starting point to look for an unknown cell.
    ///
    /// There must be no unknown cell before this cell.
    pub(crate) next_unknown: Option<CellRef<R>>,

    /// Whether to force the first row/column to be nonempty.
    ///
    /// Depending on the search order, the 'front' means:
    /// * the first row, when the search order is row first;
    /// * the first column, when the search order is column first;
    /// * the first row plus the first column, when the search order is diagonal.
    pub(crate) non_empty_front: bool,

    /// Other data used by the algorithm.
    pub(crate) algo_data: A,
}

impl<R: Rule> World<R, LifeSrc> {
    /// Creates a new world from the configuration and the rule.
    pub fn new_with_rule<A: Algorithm<R>>(config: &Config, rule: R) -> World<R, A> {
        let search_order = config.auto_search_order();

        let size = ((config.width + 2) * (config.height + 2) * config.period) as usize;
        let mut cells = Vec::with_capacity(size);
        let algo_data = A::new();

        let is_front = config.is_front_fn(rule.has_b0(), &search_order);

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
                        if t == config.period - 1 {
                            State(0)
                        } else {
                            State((t as usize + 1) % rule.gen())
                        }
                    } else {
                        DEAD
                    };
                    let mut cell = LifeCell::new((x, y, t), state, succ_state);
                    if let Some(is_front) = &is_front {
                        if is_front((x, y, t)) {
                            cell.is_front = true;
                        }
                    }
                    cells.push(cell);
                }
            }
        }

        World {
            config: config.clone(),
            rule,
            cells: cells.into_boxed_slice(),
            cell_count: vec![0; config.period as usize],
            front_cell_count: 0,
            conflicts: 0,
            set_stack: Vec::with_capacity(size),
            check_index: 0,
            next_unknown: None,
            non_empty_front: is_front.is_some(),
            algo_data,
        }
        .init_front()
        .init_border()
        .init_nbhd()
        .init_pred_succ()
        .init_sym()
        .init_state()
        .init_known_cells(&config.known_cells)
        .init_search_order(search_order.as_ref())
        .presearch()
    }

    /// Creates a new world from the configuration and the rule,
    /// using the [`LifeSrc`] algorithm.
    pub fn new_lifesrc(config: &Config, rule: R) -> Self {
        World::new_with_rule(config, rule)
    }
}

impl<R: Rule<IsGen = False>> World<R, Backjump<R>> {
    /// Creates a new world from the configuration and the rule,
    /// using the [`Backjump`] algorithm.
    pub fn new_backjump(config: &Config, rule: R) -> Self {
        World::new_with_rule(config, rule)
    }
}

impl<R: Rule, A: Algorithm<R>> World<R, A> {
    /// Initialize the list of cells in the front.
    fn init_front(self) -> Self {
        A::init_front(self)
    }

    /// Initialize the cells at the borders.
    fn init_border(mut self) -> Self {
        for x in -1..=self.config.width {
            for y in -1..=self.config.height {
                if let Some(d) = self.config.diagonal_width {
                    let abs = (x - y).abs();
                    if abs >= d {
                        if abs == d || abs == d + 1 {
                            for t in 0..self.config.period {
                                let cell = self.find_cell_ref((x, y, t)).unwrap();
                                self.set_stack.push(SetCell::new(cell, A::Reason::KNOWN));
                            }
                        }
                        continue;
                    }
                }
                for t in 0..self.config.period {
                    if x == -1 || x == self.config.width || y == -1 || y == self.config.height {
                        let cell = self.find_cell_ref((x, y, t)).unwrap();
                        self.set_stack.push(SetCell::new(cell, A::Reason::KNOWN));
                    }
                }
            }
        }
        self
    }

    /// Links the cells to their neighbors.
    ///
    /// Note that for cells on the edges of the search range,
    /// some neighbors might point to `None`.
    fn init_nbhd(mut self) -> Self {
        /// Relative positions of the neighbors.
        const NBHD: [(i32, i32); 8] = [
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
                if let Some(d) = self.config.diagonal_width {
                    if (x - y).abs() > d + 1 {
                        continue;
                    }
                }
                for t in 0..self.config.period {
                    let mut nbhd = Vec::with_capacity(NBHD.len());
                    for (nx, ny) in NBHD {
                        nbhd.push(self.find_cell_ref((x + nx, y + ny, t)));
                    }
                    let cell_mut = self.find_cell_mut((x, y, t)).unwrap();
                    cell_mut.nbhd = nbhd.try_into().unwrap();
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
                if let Some(d) = self.config.diagonal_width {
                    if (x - y).abs() > d + 1 {
                        continue;
                    }
                }
                for t in 0..self.config.period {
                    let cell = self.find_cell_ref((x, y, t)).unwrap();

                    if t != 0 {
                        let pred = self.find_cell_ref((x, y, t - 1));
                        let cell_mut = self.find_cell_mut((x, y, t)).unwrap();
                        cell_mut.pred = pred;
                    } else {
                        let pred = self.find_cell_ref(self.config.translate((x, y, t - 1)));
                        if pred.is_some() {
                            let cell_mut = self.find_cell_mut((x, y, t)).unwrap();
                            cell_mut.pred = pred;
                        } else if 0 <= x
                            && x < self.config.width
                            && 0 <= y
                            && y < self.config.height
                            && (self.config.diagonal_width.is_none()
                                || (x - y).abs() < self.config.diagonal_width.unwrap())
                            && !self.set_stack.iter().any(|s| s.cell == cell)
                        {
                            self.set_stack.push(SetCell::new(cell, A::Reason::KNOWN));
                        }
                    }

                    let succ = if t != self.config.period - 1 {
                        self.find_cell_ref((x, y, t + 1))
                    } else {
                        self.find_cell_ref(self.config.translate((x, y, t + 1)))
                    };
                    let cell_mut = self.find_cell_mut((x, y, t)).unwrap();
                    cell_mut.succ = succ;
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
                if let Some(d) = self.config.diagonal_width {
                    if (x - y).abs() > d + 1 {
                        continue;
                    }
                }
                for t in 0..self.config.period {
                    let cell = self.find_cell_ref((x, y, t)).unwrap();
                    let mut sym = Vec::with_capacity(8);

                    for transform in self.config.symmetry.members() {
                        let coord =
                            transform.act_on((x, y, t), self.config.width, self.config.height);
                        if 0 <= coord.0
                            && coord.0 < self.config.width
                            && 0 <= coord.1
                            && coord.1 < self.config.height
                            && (self.config.diagonal_width.is_none()
                                || (coord.0 - coord.1).abs() < self.config.diagonal_width.unwrap())
                        {
                            sym.push(self.find_cell_ref(coord).unwrap());
                        } else if 0 <= x
                            && x < self.config.width
                            && 0 <= y
                            && y < self.config.height
                            && (self.config.diagonal_width.is_none()
                                || (x - y).abs() < self.config.diagonal_width.unwrap())
                            && !self.set_stack.iter().any(|s| s.cell == cell)
                        {
                            self.set_stack.push(SetCell::new(cell, A::Reason::KNOWN));
                        }
                    }

                    let cell_mut = self.find_cell_mut((x, y, t)).unwrap();
                    cell_mut.sym = sym;
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
                    let cell = self.find_cell_ref((x, y, t)).unwrap();
                    if !self.set_stack.iter().any(|s| s.cell == cell) {
                        self.clear_cell(cell);
                    }
                }
            }
        }
        self
    }

    /// Sets the known cells.
    fn init_known_cells(mut self, known_cells: &[KnownCell]) -> Self {
        for &KnownCell { coord, state } in known_cells.iter() {
            if let Some(cell) = self.find_cell_ref(coord) {
                if cell.state.get().is_none() && state.0 < self.rule.gen() {
                    self.set_cell(cell, state, A::Reason::KNOWN).ok();
                }
            }
        }
        self
    }

    /// Set the [`next`](LifeCell#structfield.next) of a cell to be
    /// [`next_unknown`](#structfield.next_unknown) and set
    /// [`next_unknown`](#structfield.next_unknown) to be this cell.
    fn set_next(&mut self, coord: Coord) {
        if let Some(cell) = self.find_cell_ref(coord) {
            if cell.state.get().is_none() && cell.next.is_none() {
                let next = mem::replace(&mut self.next_unknown, Some(cell));
                let cell_mut = self.find_cell_mut(coord).unwrap();
                cell_mut.next = next;
            }
        }
    }

    /// Sets the search order.
    fn init_search_order(mut self, search_order: &SearchOrder) -> Self {
        for coord in self.config.search_order_iter(search_order) {
            self.set_next(coord);
        }
        self
    }

    /// Finds a cell by its coordinates.
    pub(crate) fn find_cell(&self, coord: Coord) -> Option<&LifeCell<R>> {
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
            Some(cell)
        } else {
            None
        }
    }

    /// Finds a cell by its coordinates. Returns a [`CellRef`].
    pub(crate) fn find_cell_ref(&self, coord: Coord) -> Option<CellRef<R>> {
        self.find_cell(coord).map(|cell| cell.borrow())
    }

    /// Finds a cell by its coordinates.
    ///
    /// Unlike [`find_cell`](Self::find_cell), it returns `None` when the cell
    /// is out of the [`diagonal_width`](Config#structfield.diagonal_width).
    fn find_cell_mut(&mut self, coord: Coord) -> Option<&mut LifeCell<R>> {
        let (x, y, t) = coord;
        if x >= -1
            && x <= self.config.width
            && y >= -1
            && y <= self.config.height
            && t >= 0
            && t < self.config.period
            && (self.config.diagonal_width.is_none()
                || (x - y).abs() <= self.config.diagonal_width.unwrap() + 1)
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
    pub(crate) fn set_cell(
        &mut self,
        cell: CellRef<R>,
        state: State,
        reason: A::Reason,
    ) -> Result<(), A::ConflReason> {
        A::set_cell(self, cell, state, reason)
    }

    /// Clears the [`state`](LifeCell#structfield.state) of a cell,
    /// and update the neighborhood descriptor of its neighbors.
    pub(crate) fn clear_cell(&mut self, cell: CellRef<R>) {
        cell.seen.set(false);
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
    pub(crate) fn get_unknown(&mut self) -> Option<CellRef<R>> {
        while let Some(cell) = self.next_unknown {
            if cell.state.get().is_none() {
                return Some(cell);
            } else {
                self.next_unknown = cell.next;
            }
        }
        None
    }

    /// Tests if the result is borling.
    pub(crate) fn is_boring(&self) -> bool {
        self.is_trivial()
            || self.is_stable()
            || (self.config.skip_subperiod && self.is_subperiodic())
            || (self.config.skip_subsymmetry && self.is_subsymmetric())
    }

    /// Tests if the result is trivial.
    fn is_trivial(&self) -> bool {
        self.cell_count[0] == 0
    }

    /// Tests if the result is stable.
    fn is_stable(&self) -> bool {
        self.config.period > 1
            && self
                .cells
                .chunks(self.config.period as usize)
                .all(|c| c[0].state.get() == c[1].state.get())
    }

    /// Tests if the fundamental period of the result is smaller than the given period.
    fn is_subperiodic(&self) -> bool {
        (2..=self.config.period).any(|f| {
            self.config.period % f == 0 && self.config.dx % f == 0 && self.config.dy % f == 0 && {
                let t = self.config.period / f;
                let dx = self.config.dx / f;
                let dy = self.config.dy / f;
                self.cells
                    .iter()
                    .step_by(self.config.period as usize)
                    .all(|c| {
                        let (x, y, _) = self.config.transform.act_on(
                            c.coord,
                            self.config.width,
                            self.config.height,
                        );
                        c.state.get() == self.get_cell_state((x - dx, y - dy, t))
                    })
            }
        })
    }

    /// Tests if the result is invariant under more transformations than
    /// required by the given symmetry.
    fn is_subsymmetric(&self) -> bool {
        let cosets = self.config.symmetry.cosets();
        self.cells
            .iter()
            .step_by(self.config.period as usize)
            .all(|c| {
                cosets.iter().skip(1).any(|t| {
                    let coord = t.act_on(c.coord, self.config.width, self.config.height);
                    c.state.get() == self.get_cell_state(coord)
                })
            })
    }

    /// Gets the state of a cell. Returns `Err(())` if there is no such cell.
    #[inline]
    pub fn get_cell_state(&self, coord: Coord) -> Option<State> {
        let (x, y, t) = self.config.translate(coord);
        self.find_cell_ref((x, y, t)).map_or_else(
            || self.find_cell_ref((0, 0, t)).map(|c1| c1.background),
            |c1| c1.state.get(),
        )
    }

    /// World configuration.
    #[inline]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Whether the rule is a Generations rule.
    #[inline]
    pub fn is_gen_rule(&self) -> bool {
        R::IsGen::VALUE
    }

    /// Whether the rule contains `B0`.
    ///
    /// In other words, whether a cell would become [`ALIVE`] in the next
    /// generation, if all its neighbors in this generation are dead.
    #[inline]
    pub fn is_b0_rule(&self) -> bool {
        self.rule.has_b0()
    }

    /// Number of known living cells in some generation.
    ///
    /// For Generations rules, dying cells are not counted.
    #[inline]
    pub fn cell_count_gen(&self, t: i32) -> u32 {
        self.cell_count[t as usize]
    }

    /// Minimum number of known living cells in all generation.
    ///
    /// For Generations rules, dying cells are not counted.
    #[inline]
    pub fn cell_count(&self) -> u32 {
        *self.cell_count.iter().min().unwrap()
    }

    /// Number of conflicts during the search.
    #[inline]
    pub fn conflicts(&self) -> u64 {
        self.conflicts
    }

    /// Set the max cell counts.
    ///
    /// Currently this is the only parameter that you can change
    /// during the search.
    #[inline]
    pub fn set_max_cell_count(&mut self, max_cell_count: Option<u32>) {
        self.config.max_cell_count = max_cell_count;
        if let Some(max) = self.config.max_cell_count {
            while self.cell_count() > max {
                if !self.retreat() {
                    break;
                }
            }
        }
    }

    /// Displays the whole world in some generation,
    /// in a mix of [Plaintext](https://conwaylife.com/wiki/Plaintext) and
    /// [RLE](https://conwaylife.com/wiki/Rle) format.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** cells are represented by `o` for rules with 2 states,
    ///   `A` for rules with more states;
    /// * **Dying** cells are represented by uppercase letters starting from `B`;
    /// * **Unknown** cells are represented by `?`;
    /// * Each line is ended with `$`;
    /// * The whole pattern is ended with `!`.
    pub fn rle_gen(&self, t: i32) -> String {
        let mut str = String::new();
        writeln!(
            str,
            "x = {}, y = {}, rule = {}",
            self.config().width,
            self.config().height,
            self.config().rule_string
        )
        .unwrap();
        for y in 0..self.config().height {
            for x in 0..self.config().width {
                let state = self.get_cell_state((x, y, t));
                match state {
                    Some(DEAD) => str.push('.'),
                    Some(ALIVE) => {
                        if self.is_gen_rule() {
                            str.push('A')
                        } else {
                            str.push('o')
                        }
                    }
                    Some(State(i)) => str.push((b'A' + i as u8 - 1) as char),
                    _ => str.push('?'),
                };
            }
            if y == self.config().height - 1 {
                str.push('!')
            } else {
                str.push('$')
            };
            str.push('\n');
        }
        str
    }

    /// Displays the whole world in some generation in
    /// [Plaintext](https://conwaylife.com/wiki/Plaintext) format.
    ///
    /// Do not use this for Generations rules.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** and **Dying** cells are represented by `o`;
    /// * **Unknown** cells are represented by `?`.
    pub fn plaintext_gen(&self, t: i32) -> String {
        let mut str = String::new();
        for y in 0..self.config().height {
            for x in 0..self.config().width {
                let state = self.get_cell_state((x, y, t));
                match state {
                    Some(DEAD) => str.push('.'),
                    Some(_) => str.push('o'),
                    None => str.push('?'),
                };
            }
            str.push('\n');
        }
        str
    }
}
