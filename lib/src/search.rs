//! The search process.

use crate::{
    cells::{CellRef, State},
    config::{Config, NewState},
    rules::Rule,
    world::World,
};

#[cfg(feature = "serialize")]
use crate::save::WorldSer;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Search status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Status {
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
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub(crate) enum Reason {
    /// Decides the state of a cell by choice,
    /// and remembers its position in the `search_list` of the world.
    Decide(usize),

    /// Determines the state of a cell by other cells.
    Deduce,
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
                    let state = !cell.state.get().unwrap();
                    self.clear_cell(cell);
                    if self.set_cell(cell, state, Reason::Deduce) {
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
    /// will be resetted in each `search`.
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
                NewState::Choose(State::Dead) => cell.background,
                NewState::Choose(State::Alive) => !cell.background,
                NewState::Random => rand::random(),
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
}

/// A trait for `World`.
///
/// So that we can switch between different rule types using trait objects.
pub trait Search {
    /// The search function.
    ///
    /// Returns `Found` if a result is found,
    /// `None` if such pattern does not exist,
    /// `Searching` if the number of steps exceeds `max_step`
    /// and no results are found.
    fn search(&mut self, max_step: Option<u64>) -> Status;

    /// Displays the whole world in some generation.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** cells are represented by `O`;
    /// * **Unknown** cells are represented by `?`.
    fn display_gen(&self, t: isize) -> String;

    /// World configuration.
    fn config(&self) -> &Config;

    /// Number of known living cells in some generation.
    fn cell_count_gen(&self, t: isize) -> usize;

    /// Minumum number of known living cells in all generation.
    fn cell_count(&self) -> usize;

    /// Number of conflicts during the search.
    fn conflicts(&self) -> u64;

    /// Set the max cell counts.
    ///
    /// Currently this is the only parameter that you can change
    /// during the search.
    fn set_max_cell_count(&mut self, max_cell_count: Option<usize>);

    #[cfg(feature = "serialize")]
    /// Saves the world as a `WorldSer`,
    /// which can be easily serialized.
    fn ser(&self) -> WorldSer;
}

/// The `Search` trait is implemented for every `World`.
impl<'a, R: Rule> Search for World<'a, R> {
    fn search(&mut self, max_step: Option<u64>) -> Status {
        self.search(max_step)
    }

    fn display_gen(&self, t: isize) -> String {
        self.display_gen(t)
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn cell_count_gen(&self, t: isize) -> usize {
        self.cell_count[t as usize]
    }

    fn cell_count(&self) -> usize {
        *self.cell_count.iter().min().unwrap()
    }

    fn conflicts(&self) -> u64 {
        self.conflicts
    }

    fn set_max_cell_count(&mut self, max_cell_count: Option<usize>) {
        self.config.max_cell_count = max_cell_count;
        if let Some(max) = self.config.max_cell_count {
            while self.cell_count() > max {
                if !self.backup() {
                    break;
                }
            }
        }
    }

    #[cfg(feature = "serialize")]
    fn ser(&self) -> WorldSer {
        self.ser()
    }
}
