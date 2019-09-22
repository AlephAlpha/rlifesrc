//! The search process.

use crate::{
    cells::{Alive, Dead, LifeCell, State},
    rules::{Desc, Rule},
    world::World,
};
use NewState::{Choose, Random, Smart};

#[cfg(feature = "stdweb")]
use serde::{Deserialize, Serialize};

/// Search status.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
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

/// How to choose a state for an unknown cell.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
pub enum NewState {
    /// Chooses the given state.
    Choose(State),
    /// Random. The probability of either state is 1/2.
    Random,
    /// Chooses `Alive` for cells on the first row / column,
    /// and `Dead` for other cells.
    ///
    /// It is not smart at all, but I can't find a better name.
    Smart,
}

/// The search.
///
/// In addition to the world itself,
/// we need to record some other information during the search.
pub struct Search<'a, D: Desc, R: 'a + Rule<Desc = D>> {
    /// The world.
    pub world: World<'a, D, R>,
    /// How to choose a state for an unknown cell.
    new_state: NewState,
    /// A stack to records the cells whose values are set during the search.
    ///
    /// The cells in this table always have known states.
    ///
    /// It is used in the backtracking.
    stack: Vec<&'a LifeCell<'a, D>>,
    /// The position in the `stack` of the next cell to be examined.
    ///
    /// See `proceed` for details.
    next_set: usize,
    /// The number of living cells must not exceed this number.
    ///
    /// `None` means that there is no limit for the cell count.
    max_cell_count: Option<u32>,
}

impl<'a, D: Desc, R: 'a + Rule<Desc = D>> Search<'a, D, R> {
    /// Construct a new search.
    pub fn new(world: World<'a, D, R>, new_state: NewState, max_cell_count: Option<u32>) -> Self {
        let size = (world.width * world.height * world.period) as usize;
        let stack = Vec::with_capacity(size);
        Search {
            world,
            new_state,
            stack,
            next_set: 0,
            max_cell_count,
        }
    }

    /// Consistifies a cell.
    ///
    /// Examines the state and the neighborhood descriptor of the cell,
    /// and makes sure that it can validly produce the cell in the next
    /// generation. If possible, determines the states of some of the
    /// cells involved.
    ///
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    fn consistify(&mut self, cell: &'a LifeCell<'a, D>) -> bool {
        let succ = cell.succ.unwrap();
        let state = cell.state.get();
        let desc = cell.desc.get();

        // Examines the cell,
        // and determines the state of the cell in the next generation.
        if let Some(new_state) = self.world.rule.transition(state, desc) {
            if let Some(succ_state) = succ.state.get() {
                if new_state != succ_state {
                    return false;
                }
            } else {
                self.world.set_cell(succ, Some(new_state), false);
                self.stack.push(succ);
            }
        }

        if let Some(succ_state) = succ.state.get() {
            // Determines the state of the current cell.
            if state.is_none() {
                if let Some(state) = self.world.rule.implication(desc, succ_state) {
                    self.world.set_cell(cell, Some(state), false);
                    self.stack.push(cell);
                }
            }

            // Determines the states of some neighbors of the cell.
            self.world.rule.consistify_nbhd(
                &cell,
                &self.world,
                desc,
                state,
                succ_state,
                &mut self.stack,
            );
        }
        true
    }

    /// Consistifies a cell, its eight neighbors,
    /// and the cell in the last generation.
    ///
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    fn consistify10(&mut self, cell: &'a LifeCell<'a, D>) -> bool {
        self.consistify(cell) && {
            let pred = cell.pred.unwrap();
            self.consistify(pred) && {
                cell.nbhd
                    .iter()
                    .all(|&neigh_id| self.consistify(neigh_id.unwrap()))
            }
        }
    }

    /// Deduces all the consequences by `consistify` and symmetry.
    ///
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    fn proceed(&mut self) -> bool {
        while self.next_set < self.stack.len() {
            // Tests if the number of living cells exceeds the `max_cell_count`.
            if let Some(max) = self.max_cell_count {
                if self.world.cell_count.get() > max {
                    return false;
                }
            }

            let cell = self.stack[self.next_set];
            let state = cell.state.get().unwrap();

            // Determines some cells by symmetry.
            for &sym in cell.sym.iter() {
                if let Some(old_state) = sym.state.get() {
                    if state != old_state {
                        return false;
                    }
                } else {
                    self.world.set_cell(sym, Some(state), false);
                    self.stack.push(sym);
                }
            }

            // Determines some cells by `consistify`.
            if !self.consistify10(cell) {
                return false;
            }

            self.next_set += 1;
        }
        true
    }

    /// Backtracks to the last time when a free unknown cell is set,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if it backtracks successfully,
    /// `false` if it goes back to the time before the first cell is set.
    fn backup(&mut self) -> bool {
        self.next_set = self.stack.len();
        while self.next_set > 0 {
            self.next_set -= 1;
            let cell = self.stack[self.next_set];
            self.stack.pop();
            if cell.free.get() {
                let state = match cell.state.get().unwrap() {
                    Dead => Alive,
                    Alive => Dead,
                };
                self.world.set_cell(cell, Some(state), false);
                self.stack.push(cell);
                return true;
            } else {
                self.world.set_cell(cell, None, true);
            }
        }
        false
    }

    /// Keep proceeding and backtracking,
    /// until there are no more cells to examine (and returns `true`),
    /// or the backtracking goes back to the time before the first cell is set
    /// (and returns `false`).
    ///
    /// It also records the number of steps it has walked in the parameter
    /// `step`. A step consists of a `proceed` and a `backup`.
    fn go(&mut self, step: &mut u32) -> bool {
        loop {
            *step += 1;
            if self.proceed() {
                return true;
            } else if !self.backup() {
                return false;
            }
        }
    }

    /// The search function.
    ///
    /// Returns `Found` if a result is found,
    /// `None` if such pattern does not exist,
    /// `Searching` if the number of steps exceeds `max_step`
    /// and no results are found.
    pub fn search(&mut self, max_step: Option<u32>) -> Status {
        let mut step_count = 0;
        if self.world.get_unknown().is_none() && !self.backup() {
            return Status::None;
        }
        while self.go(&mut step_count) {
            if let Some(cell) = self.world.get_unknown() {
                let state = match self.new_state {
                    Choose(state) => state,
                    Random => rand::random(),
                    Smart => {
                        if cell.first_col {
                            Alive
                        } else {
                            Dead
                        }
                    }
                };
                self.world.set_cell(cell, Some(state), true);

                // `stack` requires the lifetime of `cell` to be as long as `'a`,
                // so we have to use `unsafe`.
                unsafe {
                    let cell: *const LifeCell<_> = cell;
                    self.stack.push(cell.as_ref().unwrap());
                }

                if let Some(max) = max_step {
                    if step_count > max {
                        return Status::Searching;
                    }
                }
            } else if self.world.nontrivial() {
                return Status::Found;
            } else if !self.backup() {
                return Status::None;
            }
        }
        Status::None
    }
}

/// A trait for `Search`.
///
/// So that we can switch between different rule types using trait objects.
pub trait TraitSearch {
    /// The search function.
    ///
    /// Returns `Found` if a result is found,
    /// `None` if such pattern does not exist,
    /// `Searching` if the number of steps exceeds `max_step`
    /// and no results are found.
    fn search(&mut self, max_step: Option<u32>) -> Status;

    /// Display the whole world in some generation.
    ///
    /// * **Dead** cells are represented by `.`;
    /// * **Living** cells are represented by `O`;
    /// * **Unknown** cells are represented by `?`.
    fn display_gen(&self, t: isize) -> String;

    /// Period of the pattern.
    fn period(&self) -> isize;

    /// Number of known living cells in the first generation.
    fn cell_count(&self) -> u32;
}

/// The `TraitSearch` trait is implemented for every `Search`.
impl<'a, D: Desc, R: Rule<Desc = D>> TraitSearch for Search<'a, D, R> {
    fn search(&mut self, max_step: Option<u32>) -> Status {
        self.search(max_step)
    }

    fn display_gen(&self, t: isize) -> String {
        self.world.display_gen(t)
    }

    fn period(&self) -> isize {
        self.world.period
    }

    fn cell_count(&self) -> u32 {
        self.world.cell_count.get()
    }
}
