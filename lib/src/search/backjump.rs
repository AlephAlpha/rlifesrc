//! The search process, with backjumping.
use crate::{
    cells::{CellRef, State},
    rules::Rule,
    search::{private::Sealed, Algorithm, Reason as TraitReason, SetCell},
    world::World,
};
use educe::Educe;
use typebool::False;

#[cfg(feature = "serde")]
use crate::{error::Error, save::ReasonSer};

#[cfg(doc)]
use crate::cells::LifeCell;

/// __(Experimental)__ Adding [Backjumping](https://en.wikipedia.org/wiki/Backjumping)
/// to the original lifesrc algorithm.
///
/// Backjumping will reduce the number of steps, but each step will takes
/// a much longer time. The current implementation is slower for most search,
/// only useful for large (e.g., 64x64) still lifes.
///
/// Currently it is only supported for non-Generations rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Backjump<'a, R: Rule> {
    /// The global decision level for assigning the cell state.
    pub(crate) level: u32,

    /// All cells in the front.
    pub(crate) front: Vec<CellRef<'a, R>>,

    /// A learnt clause.
    pub(crate) learnt: Vec<CellRef<'a, R>>,
}

impl<'a, R: Rule> Sealed for Backjump<'a, R> {}

impl<'a, R: Rule<IsGen = False> + 'a> Default for Backjump<'a, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, R: Rule<IsGen = False> + 'a> Algorithm<'a, R> for Backjump<'a, R> {
    type Reason = Reason<'a, R>;

    type ConflReason = ConflReason<'a, R>;

    fn new() -> Self {
        Self {
            level: 0,
            front: Vec::new(),
            learnt: Vec::new(),
        }
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
        world
            .algo_data
            .front
            .reserve((world.config.width + world.config.height) as usize);
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
                            world.algo_data.front.push(cell);
                        }
                    }
                }
            }
        }
        world.algo_data.front.shrink_to_fit();
        world
    }

    #[inline]
    fn set_cell(
        world: &mut World<'a, R, Self>,
        cell: CellRef<'a, R>,
        state: State,
        reason: Self::Reason,
    ) -> Result<(), Self::ConflReason> {
        world.set_cell_impl(cell, state, reason)
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
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    #[inline]
    fn deser_reason(world: &World<'a, R, Self>, ser: &ReasonSer) -> Result<Self::Reason, Error> {
        Ok(match *ser {
            ReasonSer::Known => Reason::Known,
            ReasonSer::Decide => Reason::Decide,
            ReasonSer::Rule(coord) => {
                Reason::Rule(world.find_cell(coord).ok_or(Error::SetCellError(coord))?)
            }
            ReasonSer::Sym(coord) => {
                Reason::Sym(world.find_cell(coord).ok_or(Error::SetCellError(coord))?)
            }
            ReasonSer::Deduce => Reason::Deduce,
            ReasonSer::Clause(ref c) => {
                let mut clause = Vec::new();
                for &coord in c {
                    clause.push(world.find_cell(coord).ok_or(Error::SetCellError(coord))?);
                }
                Reason::Clause(clause)
            }
            // Generations rules are not supported, so fallback to [`Reason::Deduce`].
            ReasonSer::TryAnother(_) => Reason::Deduce,
        })
    }
}

/// Reasons for setting a cell, with informations for backjumping.
#[derive(Educe)]
#[educe(Clone, Debug, PartialEq, Eq)]
pub enum Reason<'a, R: Rule> {
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
}

impl<'a, R: Rule> Reason<'a, R> {
    /// Cells involved in the reason.
    fn cells(self) -> Vec<CellRef<'a, R>> {
        match self {
            Reason::Rule(cell) => {
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
            Reason::Sym(cell) => vec![cell],
            Reason::Clause(clause) => clause,
            _ => unreachable!(),
        }
    }
}

impl<'a, R: Rule + 'a> TraitReason<'a, R> for Reason<'a, R> {
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
        matches!(self, Self::Decide)
    }

    #[cfg(feature = "serde")]
    #[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
    #[inline]
    fn ser(&self) -> ReasonSer {
        match self {
            Self::Known => ReasonSer::Known,
            Self::Decide => ReasonSer::Decide,
            Self::Rule(cell) => ReasonSer::Rule(cell.coord),
            Self::Sym(cell) => ReasonSer::Sym(cell.coord),
            Self::Deduce => ReasonSer::Deduce,
            Self::Clause(c) => ReasonSer::Clause(c.iter().map(|cell| cell.coord).collect()),
        }
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

impl<'a, R: Rule<IsGen = False>> World<'a, R, Backjump<'a, R>> {
    /// Store the cells involved in the conflict reason into  [`self.algo_data.learnt`](Backjump::learnt).
    fn learn_from_confl(&mut self, reason: ConflReason<'a, R>) {
        self.algo_data.learnt.clear();
        match reason {
            ConflReason::Rule(cell) => {
                self.algo_data.learnt.push(cell);
                if let Some(succ) = cell.succ {
                    self.algo_data.learnt.push(succ);
                }
                for i in 0..8 {
                    if let Some(neigh) = cell.nbhd[i] {
                        self.algo_data.learnt.push(neigh);
                    }
                }
            }
            ConflReason::Sym(cell, sym) => {
                self.algo_data.learnt.push(cell);
                self.algo_data.learnt.push(sym);
            }
            ConflReason::Front => self
                .algo_data
                .learnt
                .extend_from_slice(&self.algo_data.front),
            ConflReason::Deduce => unreachable!(),
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
        reason: Reason<'a, R>,
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
            self.algo_data.level += 1;
        }
        cell.level.set(self.algo_data.level);
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
                Reason::Decide => {
                    let state = !cell.state.get().unwrap();
                    let reason = Reason::Deduce;

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.algo_data.level -= 1;
                    self.clear_cell(cell);
                    if self.set_cell_impl(cell, state, reason).is_ok() {
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
    /// The reason of conflict must be stored in `self.algo_data.learnt`
    /// before calling this method.
    ///
    /// Returns `true` if successes,
    /// `false` if it goes back to the time before the first cell is set.
    fn analyze(&mut self) -> bool {
        if self.algo_data.learnt.is_empty() {
            return self.retreat_impl();
        }
        let mut max_level = 0;
        let mut counter = 0;
        let mut i = 0;
        while i < self.algo_data.learnt.len() {
            let reason_cell = self.algo_data.learnt[i];
            if reason_cell.state.get().is_some() {
                let level = reason_cell.level.get();
                if level == self.algo_data.level {
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
            self.algo_data.learnt.swap_remove(i);
        }
        while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
            match reason {
                Reason::Decide => {
                    let state = !cell.state.get().unwrap();
                    let mut learnt = self.algo_data.learnt.clone();
                    learnt.shrink_to_fit();
                    let reason = Reason::Clause(learnt);

                    self.check_index = self.set_stack.len() as u32;
                    self.next_unknown = cell.next;
                    self.algo_data.level -= 1;
                    self.clear_cell(cell);
                    return self.set_cell_impl(cell, state, reason).is_ok() || self.retreat_impl();
                }
                Reason::Deduce => {
                    self.clear_cell(cell);
                    return self.retreat_impl();
                }
                Reason::Known => {
                    break;
                }
                _ => {
                    let seen = cell.seen.get();
                    self.clear_cell(cell);
                    if seen {
                        counter -= 1;
                        for reason_cell in reason.cells() {
                            if reason_cell.state.get().is_some() {
                                let level = reason_cell.level.get();
                                if level == self.algo_data.level {
                                    if !reason_cell.seen.get() {
                                        counter += 1;
                                        reason_cell.seen.set(true);
                                    }
                                } else if level > 0 {
                                    max_level = max_level.max(level);
                                    self.algo_data.learnt.push(reason_cell);
                                }
                            }
                        }

                        if counter == 0 {
                            if max_level == 0 {
                                break;
                            }
                            while let Some(SetCell { cell, reason }) = self.set_stack.pop() {
                                self.clear_cell(cell);
                                if matches!(reason, Reason::Decide) {
                                    self.next_unknown = cell.next;
                                    self.algo_data.level -= 1;
                                    if self.algo_data.level == max_level {
                                        return self.analyze();
                                    }
                                }
                            }
                            break;
                        }
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
