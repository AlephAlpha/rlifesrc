//! The search process.
use crate::{
    cells::{CellRef, State},
    config::NewState,
    rules::Rule,
    world::World,
};
use rand::{thread_rng, Rng};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Search status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) enum Reason {
    /// Decides the state of a cell by choice,
    /// and remembers its position in the `search_list` of the world.
    Decide(usize),

    /// Determines the state of a cell by other cells.
    Deduce,

    /// Tries another state of a cell when the original state
    /// leads to a conflict.
    ///
    /// Remembers its position in the `search_list` of the world,
    /// and the number of remaining states to try.
    TryAnother(usize, usize),
}

/// Records the cells whose values are set and their reasons.
#[derive(Clone, Copy)]
pub(crate) struct SetCell<'a, R: Rule> {
    /// The set cell.
    pub(crate) cell: CellRef<'a, R>,

    /// The reason for setting a cell.
    pub(crate) reason: Reason,
}

impl<'a, R: Rule> SetCell<'a, R> {
    /// Get a reference to the set cell.
    pub(crate) fn new(cell: CellRef<'a, R>, reason: Reason) -> Self {
        SetCell { cell, reason }
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
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    fn consistify(&mut self, cell: CellRef<'a, R>) -> bool {
        Rule::consistify(self, cell)
    }

    /// Consistifies a cell, its neighbors, and its predecessor.
    ///
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    fn consistify10(&mut self, cell: CellRef<'a, R>) -> bool {
        self.consistify(cell)
            && {
                if let Some(pred) = cell.pred {
                    self.consistify(pred)
                } else {
                    true
                }
            }
            && cell
                .nbhd
                .iter()
                .all(|&neigh| self.consistify(neigh.unwrap()))
    }

    /// Deduces all the consequences by `consistify` and symmetry.
    ///
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    fn proceed(&mut self) -> bool {
        while self.check_index < self.set_stack.len() {
            let cell = self.set_stack[self.check_index].cell;
            let state = cell.state.get().unwrap();

            // Determines some cells by symmetry.
            for &sym in cell.sym.iter() {
                if let Some(old_state) = sym.state.get() {
                    if state != old_state {
                        return false;
                    }
                } else if !self.set_cell(sym, state, Reason::Deduce) {
                    return false;
                }
            }

            // Determines some cells by `consistify`.
            if !self.consistify10(cell) {
                return false;
            }

            self.check_index += 1;
        }
        true
    }

    /// Backtracks to the last time when a unknown cell is decided by choice,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if it backtracks successfully,
    /// `false` if it goes back to the time before the first cell is set.
    fn backup(&mut self) -> bool {
        while let Some(set_cell) = self.set_stack.pop() {
            let cell = set_cell.cell;
            match set_cell.reason {
                Reason::Decide(i) => {
                    self.check_index = self.set_stack.len();
                    self.search_index = i + 1;
                    if R::IS_GEN {
                        let State(j) = cell.state.get().unwrap();
                        let state = State((j + 1) % self.rule.gen());
                        self.clear_cell(cell);
                        if self.set_cell(cell, state, Reason::TryAnother(i, self.rule.gen() - 2)) {
                            return true;
                        }
                    } else {
                        let state = !cell.state.get().unwrap();
                        self.clear_cell(cell);
                        if self.set_cell(cell, state, Reason::Deduce) {
                            return true;
                        }
                    }
                }
                Reason::TryAnother(i, n) => {
                    self.check_index = self.set_stack.len();
                    self.search_index = i + 1;
                    let State(j) = cell.state.get().unwrap();
                    let state = State((j + 1) % self.rule.gen());
                    self.clear_cell(cell);
                    let reason = if n == 1 {
                        Reason::Deduce
                    } else {
                        Reason::TryAnother(i, n - 1)
                    };
                    if self.set_cell(cell, state, reason) {
                        return true;
                    }
                }
                Reason::Deduce => {
                    self.clear_cell(cell);
                }
            }
        }
        self.check_index = 0;
        self.search_index = 0;
        false
    }

    /// Keeps proceeding and backtracking,
    /// until there are no more cells to examine (and returns `true`),
    /// or the backtracking goes back to the time before the first cell is set
    /// (and returns `false`).
    ///
    /// It also records the number of steps it has walked in the parameter
    /// `step`. A step consists of a `proceed` and a `backup`.
    ///
    /// The difference between `step` and `self.steps` is that the former
    /// will be reset in each `search`.
    fn go(&mut self, step: &mut u64) -> bool {
        loop {
            *step += 1;
            if self.proceed() {
                return true;
            } else {
                self.conflicts += 1;
                if !self.backup() {
                    return false;
                }
            }
        }
    }

    /// Makes a decision.
    ///
    /// Chooses an unknown cell, assigns a state for it,
    /// and push a reference to it to the `set_stack`.
    ///
    /// Returns `None` is there is no unknown cell,
    /// `Some(false)` if the new state leads to an immediate conflict.
    fn decide(&mut self) -> Option<bool> {
        if let Some((i, cell)) = self.get_unknown(self.search_index) {
            self.search_index = i + 1;
            let state = match self.config.new_state {
                NewState::ChooseDead => cell.background,
                NewState::ChooseAlive => !cell.background,
                NewState::Random => State(thread_rng().gen_range(0, self.rule.gen())),
            };
            Some(self.set_cell(cell, state, Reason::Decide(i)))
        } else {
            None
        }
    }

    /// The search function.
    ///
    /// Returns `Found` if a result is found,
    /// `None` if such pattern does not exist,
    /// `Searching` if the number of steps exceeds `max_step`
    /// and no results are found.
    pub fn search(&mut self, max_step: Option<u64>) -> Status {
        let mut step_count = 0;
        if self.get_unknown(0).is_none() && !self.backup() {
            return Status::None;
        }
        while self.go(&mut step_count) {
            if let Some(result) = self.decide() {
                if !result && !self.backup() {
                    return Status::None;
                }
            } else if self.nontrivial() {
                if self.config.reduce_max {
                    self.config.max_cell_count = Some(self.cell_count() - 1);
                }
                return Status::Found;
            } else if !self.backup() {
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
    pub(crate) fn set_max_cell_count(&mut self, max_cell_count: Option<usize>) {
        self.config.max_cell_count = max_cell_count;
        if let Some(max) = self.config.max_cell_count {
            while self.cell_count() > max {
                if !self.backup() {
                    break;
                }
            }
        }
    }
}
