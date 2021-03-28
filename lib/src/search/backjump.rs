//! The search process, with backjumping.
use crate::{
    cells::{CellRef, State},
    rules::Rule,
    search::{Reason, SetCell},
    world::World,
};
use educe::Educe;

#[cfg(feature = "serde")]
use crate::{error::Error, save::ReasonSer};

#[cfg(doc)]
use crate::cells::LifeCell;

/// Reasons for setting a cell, with informations for backjumping.
#[derive(Educe)]
#[educe(Clone, Debug, PartialEq, Eq)]
pub enum ReasonBackjump<'a, R: Rule> {
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
    Deduce,

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

impl<'a, R: Rule> ReasonBackjump<'a, R> {
    /// Cells involved in the reason.
    fn cells(self) -> Vec<CellRef<'a, R>> {
        match self {
            ReasonBackjump::Rule(cell) => {
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
            ReasonBackjump::Sym(cell) => vec![cell],
            ReasonBackjump::Clause(clause) => clause,
            _ => unreachable!(),
        }
    }
}

impl<'a, R: Rule + 'a> Reason<'a, R> for ReasonBackjump<'a, R> {
    type ConflReason = ConflReason<'a, R>;
    const KNOWN: Self = Self::Known;
    const DECIDED: Self = Self::Decide;

    #[inline]
    fn from_cell(cell: CellRef<'a, R>) -> Self {
        Self::Rule(cell)
    }

    #[inline]
    fn from_sym(cell: CellRef<'a, R>) -> Self {
        Self::Sym(cell)
    }

    #[inline]
    fn is_decided(&self) -> bool {
        matches!(self, Self::Decide | Self::TryAnother(_))
    }

    #[inline]
    fn confl_from_cell(cell: CellRef<'a, R>) -> Self::ConflReason {
        ConflReason::Rule(cell)
    }

    #[inline]
    fn confl_from_sym(cell: CellRef<'a, R>, sym: CellRef<'a, R>) -> Self::ConflReason {
        ConflReason::Sym(cell, sym)
    }

    #[inline]
    fn init_front(mut world: World<'a, R, Self>) -> World<'a, R, Self> {
        for x in -1..=world.config.width {
            for y in -1..=world.config.height {
                if let Some(d) = world.config.diagonal_width {
                    if (x - y).abs() > d + 1 {
                        continue;
                    }
                }
                for t in 0..world.config.period {
                    if let Some(cell) = world.find_cell((x, y, t)) {
                        if cell.is_front {
                            world.front.push(cell);
                        }
                    }
                }
            }
        }
        world.front.shrink_to_fit();
        world
    }

    #[inline]
    fn set_cell(
        self,
        world: &mut World<'a, R, Self>,
        cell: CellRef<'a, R>,
        state: State,
    ) -> Result<(), Self::ConflReason> {
        world.set_cell_impl(cell, state, self)
    }

    #[inline]
    fn go(world: &mut World<'a, R, Self>, step: &mut u64) -> bool {
        world.go(step)
    }

    #[inline]
    fn retreat(world: &mut World<'a, R, Self>) -> bool {
        world.retreat_impl()
    }

    #[cfg(feature = "serde")]
    fn ser(&self) -> ReasonSer {
        match self {
            Self::Known => ReasonSer::Known,
            Self::Decide => ReasonSer::Decide,
            Self::Rule(cell) => ReasonSer::Rule(cell.coord),
            Self::Sym(cell) => ReasonSer::Sym(cell.coord),
            Self::Deduce => ReasonSer::Deduce,
            Self::Clause(c) => ReasonSer::Clause(c.iter().map(|cell| cell.coord).collect()),
            Self::TryAnother(n) => ReasonSer::TryAnother(*n),
        }
    }

    #[cfg(feature = "serde")]
    fn deser(ser: &ReasonSer, world: &World<'a, R, Self>) -> Result<Self, Error> {
        Ok(match *ser {
            ReasonSer::Known => Self::Known,
            ReasonSer::Decide => Self::Decide,
            ReasonSer::Rule(coord) => {
                Self::Rule(world.find_cell(coord).ok_or(Error::SetCellError(coord))?)
            }
            ReasonSer::Sym(coord) => {
                Self::Sym(world.find_cell(coord).ok_or(Error::SetCellError(coord))?)
            }
            ReasonSer::Deduce => Self::Deduce,
            ReasonSer::Clause(ref c) => {
                let mut clause = Vec::new();
                for &coord in c {
                    clause.push(world.find_cell(coord).ok_or(Error::SetCellError(coord))?);
                }
                Self::Clause(clause)
            }
            ReasonSer::TryAnother(n) => Self::TryAnother(n),
        })
    }
}

/// Reasons for a conflict.
#[derive(Educe)]
#[educe(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflReason<'a, R: Rule> {
    /// Conflict from the rule when constitifying another cell.
    Rule(CellRef<'a, R>),

    /// Conflict from symmetry.
    Sym(CellRef<'a, R>, CellRef<'a, R>),

    /// Conflict from non-empty-front condition.
    Front,

    /// Conflict from other conditions.
    ///
    /// A general reason used as a fallback.
    Deduce,
}

impl<'a, R: Rule> ConflReason<'a, R> {
    /// Whether this reason should be analyzed before retreating.
    fn should_analyze(&self) -> bool {
        !matches!(self, Self::Deduce)
    }
}

impl<'a, R: Rule> World<'a, R, ReasonBackjump<'a, R>> {
    /// Store the cells involved in the conflict reason into  [`self.learnt`](Self::learnt).
    fn learn_from_confl(&mut self, reason: ConflReason<'a, R>) {
        self.learnt.clear();
        match reason {
            ConflReason::Rule(cell) => {
                self.learnt.push(cell);
                if let Some(succ) = cell.succ {
                    self.learnt.push(succ);
                }
                for i in 0..8 {
                    if let Some(neigh) = cell.nbhd[i] {
                        self.learnt.push(neigh);
                    }
                }
            }
            ConflReason::Sym(cell, sym) => {
                self.learnt.push(cell);
                self.learnt.push(sym);
            }
            ConflReason::Front => self.learnt.extend_from_slice(&self.front),
            _ => unreachable!(),
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
    pub(crate) fn set_cell_impl(
        &mut self,
        cell: CellRef<'a, R>,
        state: State,
        reason: ReasonBackjump<'a, R>,
    ) -> Result<(), ConflReason<'a, R>> {
        cell.state.set(Some(state));
        let mut result = Ok(());
        cell.update_desc(Some(state), true);
        if state == !cell.background {
            self.cell_count[cell.coord.2 as usize] += 1;
            if let Some(max) = self.config.max_cell_count {
                if self.cell_count() > max {
                    result = Err(ConflReason::Deduce);
                }
            }
        }
        if cell.is_front && state == cell.background {
            self.front_cell_count -= 1;
            if self.non_empty_front && self.front_cell_count == 0 {
                result = Err(ConflReason::Front);
            }
        }
        if reason.is_decided() {
            self.level += 1;
        }
        cell.level.set(self.level);
        self.set_stack.push(SetCell::new(cell, reason));
        result
    }

    /// Retreats to the last time when a unknown cell is decided by choice,
    /// and switch that cell to the other state.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    fn retreat_impl(&mut self) -> bool {
        while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
            match reason {
                ReasonBackjump::Decide => {
                    let (state, reason) = if R::IS_GEN {
                        let State(j) = cell.state.get().unwrap();
                        (
                            State((j + 1) % self.rule.gen()),
                            ReasonBackjump::TryAnother(self.rule.gen() - 2),
                        )
                    } else {
                        (!cell.state.get().unwrap(), ReasonBackjump::Deduce)
                    };

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.level -= 1;
                    self.clear_cell(cell);
                    if self.set_cell_impl(cell, state, reason).is_ok() {
                        return true;
                    }
                }
                ReasonBackjump::TryAnother(n) => {
                    let State(j) = cell.state.get().unwrap();
                    let state = State((j + 1) % self.rule.gen());
                    let reason = if n == 1 {
                        ReasonBackjump::Deduce
                    } else {
                        ReasonBackjump::TryAnother(n - 1)
                    };

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.level -= 1;
                    self.clear_cell(cell);
                    if self.set_cell_impl(cell, state, reason).is_ok() {
                        return true;
                    }
                }
                ReasonBackjump::Known => {
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
    /// The reason of conflict must be stored in `self.learnt`
    /// before calling this method.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    fn analyze(&mut self) -> bool {
        if self.learnt.is_empty() {
            return self.retreat_impl();
        }
        let mut max_level = 0;
        let mut counter = 0;
        let mut i = 0;
        while i < self.learnt.len() {
            let reason_cell = self.learnt[i];
            if reason_cell.state.get().is_some() {
                let level = reason_cell.level.get();
                if level == self.level {
                    if !reason_cell.seen.get() {
                        counter += 1;
                        reason_cell.seen.set(true);
                    }
                } else if level > 0 {
                    max_level = max_level.max(level);
                    i += 1;
                    continue;
                }
            }
            self.learnt.swap_remove(i);
        }
        while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
            match reason {
                ReasonBackjump::Decide => {
                    let state = !cell.state.get().unwrap();
                    let mut learnt = self.learnt.clone();
                    learnt.shrink_to_fit();
                    let reason = ReasonBackjump::Clause(learnt);

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.level -= 1;
                    self.clear_cell(cell);
                    return self.set_cell_impl(cell, state, reason).is_ok() || self.retreat_impl();
                }
                ReasonBackjump::TryAnother(_) => unreachable!(),
                ReasonBackjump::Deduce => {
                    self.clear_cell(cell);
                    return self.retreat_impl();
                }
                ReasonBackjump::Known => {
                    break;
                }
                _ => {
                    if cell.seen.get() {
                        self.clear_cell(cell);
                        counter -= 1;
                        for reason_cell in reason.cells() {
                            if reason_cell.state.get().is_some() {
                                let level = reason_cell.level.get();
                                if level == self.level {
                                    if !reason_cell.seen.get() {
                                        counter += 1;
                                        reason_cell.seen.set(true);
                                    }
                                } else if level > 0 {
                                    max_level = max_level.max(level);
                                    self.learnt.push(reason_cell);
                                }
                            }
                        }

                        if counter == 0 {
                            if max_level == 0 {
                                break;
                            }
                            while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
                                if matches!(
                                    reason,
                                    ReasonBackjump::Decide | ReasonBackjump::TryAnother(_)
                                ) {
                                    self.clear_cell(cell);
                                    self.next_unknown = cell.next;
                                    self.level -= 1;
                                    if self.level == max_level {
                                        return self.analyze();
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
    /// `step`. A step consists of a [`proceed`](Self::proceed) and a [`analyze`](Self::analyze).
    fn go(&mut self, step: &mut u64) -> bool {
        loop {
            *step += 1;
            match self.proceed() {
                Ok(()) => return true,
                Err(reason) => {
                    self.conflicts += 1;
                    let failed = if reason.should_analyze() {
                        self.learn_from_confl(reason);
                        !self.analyze()
                    } else {
                        !self.retreat_impl()
                    };
                    if failed {
                        return false;
                    }
                }
            }
        }
    }
}
