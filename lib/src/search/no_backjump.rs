//! The search process.
use crate::{
    cells::{CellRef, State},
    config::NewState,
    rules::Rule,
    search::{Reason, SetCell, Status},
    world::World,
};
use rand::{thread_rng, Rng};

#[cfg(feature = "serde")]
use crate::{error::Error, save::ReasonSer};

/// Reasons for setting a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReasonNoBackjump {
    /// Known before the search starts,
    Known,

    /// Decides the state of a cell by choice.
    Decide,

    /// Determines the state of a cell by other cells.
    Deduce,

    /// Tries another state of a cell when the original state
    /// leads to a conflict.
    ///
    /// Remembers the number of remaining states to try.
    ///
    /// Only used in Generations rules.
    TryAnother(usize),
}

impl<'a, R: Rule + 'a> Reason<'a, R> for ReasonNoBackjump {
    const KNOWN: Self = ReasonNoBackjump::Known;
    const DECIDED: Self = ReasonNoBackjump::Decide;

    fn from_cell(_cell: CellRef<'a, R>) -> Self {
        ReasonNoBackjump::Deduce
    }

    fn is_decided(&self) -> bool {
        matches!(
            self,
            ReasonNoBackjump::Decide | ReasonNoBackjump::TryAnother(_)
        )
    }

    fn search(world: &mut World<'a, R, Self>, max_step: Option<u64>) -> Status {
        world.search(max_step)
    }

    fn retreat(world: &mut World<'a, R, Self>) -> bool {
        world.retreat()
    }

    fn presearch(world: World<'a, R, Self>) -> World<'a, R, Self> {
        world.presearch()
    }

    #[cfg(feature = "serde")]
    fn ser(&self) -> ReasonSer {
        match self {
            ReasonNoBackjump::Known => ReasonSer::Known,
            ReasonNoBackjump::Decide => ReasonSer::Decide,
            ReasonNoBackjump::Deduce => ReasonSer::Deduce,
            ReasonNoBackjump::TryAnother(n) => ReasonSer::TryAnother(*n),
        }
    }

    #[cfg(feature = "serde")]
    fn deser(ser: &ReasonSer, _world: &World<'a, R, Self>) -> Result<Self, Error> {
        Ok(match *ser {
            ReasonSer::Known => ReasonNoBackjump::Known,
            ReasonSer::Decide => ReasonNoBackjump::Decide,
            ReasonSer::TryAnother(n) => ReasonNoBackjump::TryAnother(n),
            _ => ReasonNoBackjump::Deduce,
        })
    }
}

impl<'a, R: Rule> World<'a, R, ReasonNoBackjump> {
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
            && cell.nbhd.iter().all(|&neigh| {
                if let Some(neigh) = neigh {
                    self.consistify(neigh)
                } else {
                    true
                }
            })
    }

    /// Deduces all the consequences by [`consistify`](Self::consistify) and symmetry.
    ///
    /// Returns `false` if there is a conflict,
    /// `true` if the cells are consistent.
    fn proceed(&mut self) -> bool {
        while self.check_index < self.set_stack.len() as u32 {
            let cell = self.set_stack[self.check_index as usize].cell;
            let state = cell.state.get().unwrap();

            // Determines some cells by symmetry.
            for &sym in cell.sym.iter() {
                if let Some(old_state) = sym.state.get() {
                    if state != old_state {
                        return false;
                    }
                } else if !self.set_cell(sym, state, ReasonNoBackjump::Deduce) {
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

    /// Retreats to the last time when a unknown cell is decided by choice,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    fn retreat(&mut self) -> bool {
        while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
            match reason {
                ReasonNoBackjump::Decide => {
                    let state;
                    let reason;
                    if R::IS_GEN {
                        let State(j) = cell.state.get().unwrap();
                        state = State((j + 1) % self.rule.gen());
                        reason = ReasonNoBackjump::TryAnother(self.rule.gen() - 2);
                    } else {
                        state = !cell.state.get().unwrap();
                        reason = ReasonNoBackjump::Deduce;
                    }

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.clear_cell(cell);
                    if self.set_cell(cell, state, reason) {
                        return true;
                    }
                }
                ReasonNoBackjump::TryAnother(n) => {
                    let State(j) = cell.state.get().unwrap();
                    let state = State((j + 1) % self.rule.gen());
                    let reason = if n == 1 {
                        ReasonNoBackjump::Deduce
                    } else {
                        ReasonNoBackjump::TryAnother(n - 1)
                    };

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.clear_cell(cell);
                    if self.set_cell(cell, state, reason) {
                        return true;
                    }
                }
                ReasonNoBackjump::Known => {
                    break;
                }
                ReasonNoBackjump::Deduce => {
                    self.clear_cell(cell);
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
            if self.proceed() {
                return true;
            } else {
                self.conflicts += 1;
                if !self.retreat() {
                    return false;
                }
            }
        }
    }

    /// Deduces all cells that could be deduced before the first decision.
    pub(crate) fn presearch(mut self) -> Self {
        loop {
            if self.proceed() {
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
            Some(self.set_cell(cell, state, ReasonNoBackjump::Decide))
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
}
