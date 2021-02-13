//! The search process.
use crate::{
    cells::{CellRef, State},
    config::NewState,
    rules::Rule,
    world::World,
};
use derivative::Derivative;
use rand::{thread_rng, Rng};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Search status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Status {
    /// Initial status. Waiting to start.
    Initial,
    /// A result is found.
    Found,
    /// Such pattern does not exist.
    None,
    /// Still searching.
    Searching,
    /// Paused.
    Paused,
}

/// Reasons for setting a cell.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub(crate) enum Reason<'a, R: Rule> {
    /// Known before the search starts,
    Known,

    /// Decides the state of a cell by choice.
    Decide,

    /// Deduced from the rule when constitifying another cell.
    Rule(CellRef<'a, R>),

    /// Deduced from symmetry.
    Sym(CellRef<'a, R>),

    /// Deduced from other cells or conflicts.
    ///
    /// A general reason used as a fallback.
    Deduced,

    /// Deduced from a learnt clause.
    Clause(Vec<CellRef<'a, R>>),

    /// Tries another state of a cell when the original state
    /// leads to a conflict.
    ///
    /// Remembers the number of remaining states to try.
    ///
    /// Only used in Generations rules.
    TryAnother(usize),
}

impl<'a, R: Rule> Reason<'a, R> {
    /// Cells involved in the reason.
    fn cells(self, set_cell: CellRef<'a, R>) -> Vec<CellRef<'a, R>> {
        match self {
            Reason::Rule(cell) => {
                let mut cells = Vec::with_capacity(10);
                if set_cell != cell {
                    cells.push(cell);
                }
                if let Some(succ) = cell.succ {
                    if set_cell != succ {
                        cells.push(succ);
                    }
                }
                for i in 0..8 {
                    if let Some(neigh) = cell.nbhd[i] {
                        if set_cell != neigh {
                            cells.push(neigh);
                        }
                    }
                }
                cells
            }
            Reason::Sym(cell) => vec![cell],
            Reason::Clause(clause) => clause,
            _ => Vec::new(),
        }
    }
}

/// Records the cells whose values are set and their reasons.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub(crate) struct SetCell<'a, R: Rule> {
    /// The set cell.
    pub(crate) cell: CellRef<'a, R>,

    /// The reason for setting a cell.
    pub(crate) reason: Reason<'a, R>,
}

impl<'a, R: Rule> SetCell<'a, R> {
    /// Get a reference to the set cell.
    pub(crate) fn new(cell: CellRef<'a, R>, reason: Reason<'a, R>) -> Self {
        SetCell { cell, reason }
    }
}

/// Reasons for a conflict.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
enum ConflReason<'a, R: Rule> {
    /// Deduced from the rule when constitifying another cell.
    Rule(CellRef<'a, R>),

    /// Deduced from symmetry.
    Sym(CellRef<'a, R>, CellRef<'a, R>),

    /// Deduced from other conditions.
    ///
    /// A general reason used as a fallback.
    Deduced,
}

impl<'a, R: Rule> ConflReason<'a, R> {
    /// Cells involved in the reason.
    fn cells(self) -> Vec<CellRef<'a, R>> {
        match self {
            ConflReason::Rule(cell) => {
                let mut cells = Vec::with_capacity(10);
                cells.push(cell);
                if let Some(succ) = cell.succ {
                    cells.push(succ);
                }
                for i in 0..8 {
                    if let Some(neigh) = cell.nbhd[i] {
                        cells.push(neigh);
                    }
                }
                cells
            }
            ConflReason::Sym(cell, sym) => vec![cell, sym],
            _ => Vec::new(),
        }
    }
}

impl<'a, R: Rule> World<'a, R> {
    /// Consistifies a cell.
    ///
    /// Examines the state and the neighborhood descriptor of the cell,
    /// and makes sure that it can validly produce the cell in the next
    /// generation. If possible, determines the states of some of the
    /// cells involved.
    ///
    /// If there is a conflict, returns its reason.
    fn consistify(&mut self, cell: CellRef<'a, R>) -> Result<(), ConflReason<'a, R>> {
        if Rule::consistify(self, cell) {
            Ok(())
        } else {
            Err(ConflReason::Rule(cell))
        }
    }

    /// Consistifies a cell, its neighbors, and its predecessor.
    ///
    /// If there is a conflict, returns its reason.
    fn consistify10(&mut self, cell: CellRef<'a, R>) -> Result<(), ConflReason<'a, R>> {
        self.consistify(cell)?;

        if let Some(pred) = cell.pred {
            self.consistify(pred)?;
        }
        for &neigh in cell.nbhd.iter() {
            if let Some(neigh) = neigh {
                self.consistify(neigh)?;
            }
        }
        Ok(())
    }

    /// Deduces all the consequences by [`consistify`](Self::consistify) and symmetry.
    ///
    /// If there is a conflict, returns its reason.
    fn proceed(&mut self) -> Result<(), ConflReason<'a, R>> {
        while self.check_index < self.set_stack.len() as u32 {
            let cell = self.set_stack[self.check_index as usize].cell;
            let state = cell.state.get().unwrap();

            // Determines some cells by symmetry.
            for &sym in cell.sym.iter() {
                if let Some(old_state) = sym.state.get() {
                    if state != old_state {
                        return Err(ConflReason::Sym(cell, sym));
                    }
                } else if !self.set_cell(sym, state, Reason::Sym(cell)) {
                    return Err(ConflReason::Deduced);
                }
            }

            // Determines some cells by `consistify`.
            self.consistify10(cell)?;

            self.check_index += 1;
        }
        Ok(())
    }

    /// Retreats to the last time when a unknown cell is decided by choice,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    fn retreat(&mut self) -> bool {
        while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
            match reason {
                Reason::Decide => {
                    let state;
                    let reason;
                    if R::IS_GEN {
                        let State(j) = cell.state.get().unwrap();
                        state = State((j + 1) % self.rule.gen());
                        reason = Reason::TryAnother(self.rule.gen() - 2);
                    } else {
                        state = !cell.state.get().unwrap();
                        reason = Reason::Deduced;
                    }

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.level -= 1;
                    self.clear_cell(cell);
                    if self.set_cell(cell, state, reason) {
                        return true;
                    }
                }
                Reason::TryAnother(n) => {
                    let State(j) = cell.state.get().unwrap();
                    let state = State((j + 1) % self.rule.gen());
                    let reason = if n == 1 {
                        Reason::Deduced
                    } else {
                        Reason::TryAnother(n - 1)
                    };

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.level -= 1;
                    self.clear_cell(cell);
                    if self.set_cell(cell, state, reason) {
                        return true;
                    }
                }
                Reason::Known => {
                    break;
                }
                _ => {
                    self.clear_cell(cell);
                }
            }
        }
        self.set_stack.clear();
        self.check_index = 0;
        self.next_unknown = None;
        false
    }

    /// Retreats to the last time when a unknown cell is assumed,
    /// analyzes the conflict during the backtracking, and
    /// backjumps if possible.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    fn analyze(&mut self, reason: Vec<CellRef<'a, R>>) -> bool {
        if reason.is_empty() || R::IS_GEN {
            return self.retreat();
        }
        let mut max_level = 0;
        let mut counter = 0;
        let mut learnt = Vec::new();
        for reason_cell in reason {
            if reason_cell.state.get().is_some() {
                let level = reason_cell.level.get();
                if level == self.level {
                    if !reason_cell.seen.get() {
                        counter += 1;
                        reason_cell.seen.set(true);
                    }
                } else {
                    max_level = max_level.max(level);
                    learnt.push(reason_cell);
                }
            }
        }
        while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
            match reason {
                Reason::Decide => {
                    let state = !cell.state.get().unwrap();
                    let reason = Reason::Clause(learnt);

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.level -= 1;
                    self.clear_cell(cell);
                    return self.set_cell(cell, state, reason) || self.retreat();
                }
                Reason::TryAnother(_) => unreachable!(),
                Reason::Deduced => {
                    self.clear_cell(cell);
                    return self.retreat();
                }
                Reason::Known => {
                    break;
                }
                _ => {
                    if cell.seen.get() {
                        self.clear_cell(cell);
                        counter -= 1;
                        for reason_cell in reason.cells(cell) {
                            if reason_cell.state.get().is_some() {
                                let level = reason_cell.level.get();
                                if level == self.level {
                                    if !reason_cell.seen.get() {
                                        counter += 1;
                                        reason_cell.seen.set(true);
                                    }
                                } else {
                                    max_level = max_level.max(level);
                                    learnt.push(reason_cell);
                                }
                            }
                        }

                        if counter == 0 {
                            while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
                                if matches!(reason, Reason::Decide | Reason::TryAnother(_)) {
                                    self.clear_cell(cell);
                                    self.next_unknown = cell.next;
                                    self.level -= 1;
                                    if self.level == max_level {
                                        return self.analyze(learnt);
                                    }
                                } else {
                                    self.clear_cell(cell);
                                }
                            }
                            break;
                        }
                    } else {
                        self.clear_cell(cell);
                    }
                }
            }
        }
        self.set_stack.clear();
        self.check_index = 0;
        self.next_unknown = None;
        false
    }

    /// Keeps proceeding and backtracking,
    /// until there are no more cells to examine (and returns `true`),
    /// or the backtracking goes back to the time before the first cell is set
    /// (and returns `false`).
    ///
    /// It also records the number of steps it has walked in the parameter
    /// `step`. A step consists of a [`proceed`](Self::proceed) and a [`retreat`](Self::retreat).
    fn go(&mut self, step: &mut u64) -> bool {
        loop {
            *step += 1;
            match self.proceed() {
                Ok(()) => return true,
                Err(reason) => {
                    self.conflicts += 1;
                    let is_fail = if self.config.backjump {
                        !self.analyze(reason.cells())
                    } else {
                        !self.retreat()
                    };
                    if is_fail {
                        return false;
                    }
                }
            }
        }
    }

    /// Deduces all cells that could be deduced before the first decision.
    pub(crate) fn presearch(mut self) -> Self {
        loop {
            if self.proceed().is_ok() {
                self.set_stack.clear();
                self.check_index = 0;
                return self;
            } else {
                self.conflicts += 1;
                if !self.retreat() {
                    return self;
                }
            }
        }
    }

    /// Makes a decision.
    ///
    /// Chooses an unknown cell, assigns a state for it,
    /// and push a reference to it to the [`set_stack`](#structfield.set_stack).
    ///
    /// Returns `None` is there is no unknown cell,
    /// `Some(false)` if the new state leads to an immediate conflict.
    fn decide(&mut self) -> Option<bool> {
        if let Some(cell) = self.get_unknown() {
            self.next_unknown = cell.next;
            let state = match self.config.new_state {
                NewState::ChooseDead => cell.background,
                NewState::ChooseAlive => !cell.background,
                NewState::Random => State(thread_rng().gen_range(0..self.rule.gen())),
            };
            Some(self.set_cell(cell, state, Reason::Decide))
        } else {
            None
        }
    }

    /// The search function.
    ///
    /// Returns [`Status::Found`] if a result is found,
    /// [`Status::None`] if such pattern does not exist,
    /// [`Status::Searching`] if the number of steps exceeds `max_step`
    /// and no results are found.
    pub fn search(&mut self, max_step: Option<u64>) -> Status {
        let mut step_count = 0;
        if self.next_unknown.is_none() && !self.retreat() {
            return Status::None;
        }
        while self.go(&mut step_count) {
            if let Some(result) = self.decide() {
                if !result && !self.retreat() {
                    return Status::None;
                }
            } else if !self.is_boring() {
                if self.config.reduce_max {
                    self.config.max_cell_count = Some(self.cell_count() - 1);
                }
                return Status::Found;
            } else if !self.retreat() {
                return Status::None;
            }

            if let Some(max) = max_step {
                if step_count > max {
                    return Status::Searching;
                }
            }
        }
        Status::None
    }

    /// Set the max cell counts.
    pub(crate) fn set_max_cell_count(&mut self, max_cell_count: Option<u32>) {
        self.config.max_cell_count = max_cell_count;
        if let Some(max) = self.config.max_cell_count {
            while self.cell_count() > max {
                if !self.retreat() {
                    break;
                }
            }
        }
    }
}
