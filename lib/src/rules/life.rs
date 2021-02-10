//! Totalistic Life-like rules.

use crate::{
    cells::{CellRef, State, ALIVE, DEAD},
    error::Error,
    rules::Rule,
    search::Reason,
    world::World,
};
use bitflags::bitflags;
use ca_rules::{ParseLife, ParseLifeGen};
use std::str::FromStr;

bitflags! {
    /// Flags to imply the state of a cell and its neighbors.
    #[derive(Default)]
    struct ImplFlags: u8 {
        /// A conflict is detected.
        const CONFLICT = 0b_0000_0001;

        /// The successor must be alive.
        const SUCC_ALIVE = 0b_0000_0100;

        /// The successor must be dead.
        const SUCC_DEAD = 0b_0000_1000;

        /// The state of the successor is implied.
        const SUCC = Self::SUCC_ALIVE.bits | Self::SUCC_DEAD.bits;

        /// The cell itself must be alive.
        const SELF_ALIVE = 0b_0001_0000;

        /// The cell itself must be dead.
        const SELF_DEAD = 0b_0010_0000;

        /// The state of the cell itself is implied.
        const SELF = Self::SELF_ALIVE.bits | Self::SELF_DEAD.bits;

        /// All unknown neighbors must be alive.
        const NBHD_ALIVE = 0b_0100_0000;

        /// All unknown neighbors must be dead.
        const NBHD_DEAD = 0b_1000_0000;

        /// The states of all unknown neighbors are implied.
        const NBHD = Self::NBHD_ALIVE.bits | Self::NBHD_DEAD.bits;
    }
}

/// The neighborhood descriptor.
///
/// It is a 12-bit integer of the form `0b_abcd_efgh_ij_kl`,
/// where:
///
/// * `0b_abcd` is the number of dead cells in the neighborhood.
/// * `0b_efgh` is the number of living cells in the neighborhood.
/// * `0b_ij` is the state of the successor.
/// * `0b_kl` is the state of the cell itself.
///
/// For `0b_ij` and `0b_kl`:
/// * `0b_10` means dead,
/// * `0b_01` means alive,
/// * `0b_00` means unknown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NbhdDesc(u16);

/// Totalistic Life-like rules.
pub struct Life {
    /// Whether the rule contains `B0`.
    b0: bool,
    /// Whether the rule contains `S8`.
    s8: bool,
    /// An array of actions for all neighborhood descriptors.
    impl_table: [ImplFlags; 1 << 12],
}

impl Life {
    /// Constructs a new rule from the `b` and `s` data.
    pub fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
        let b0 = b.contains(&0);
        let s8 = s.contains(&8);

        let impl_table = [ImplFlags::empty(); 1 << 12];

        Life { b0, s8, impl_table }
            .init_trans(b, s)
            .init_conflict()
            .init_impl()
            .init_impl_nbhd()
    }

    /// Deduces the implication for the successor.
    fn init_trans(mut self, b: Vec<u8>, s: Vec<u8>) -> Self {
        // Fills in the positions of the neighborhood descriptors
        // that have no unknown neighbors.
        for alives in 0..=8 {
            let desc = ((8 - alives) << 8) | alives << 4;
            let alives = alives as u8;
            self.impl_table[desc | 0b10] |= if b.contains(&alives) {
                ImplFlags::SUCC_ALIVE
            } else {
                ImplFlags::SUCC_DEAD
            };
            self.impl_table[desc | 0b01] |= if s.contains(&alives) {
                ImplFlags::SUCC_ALIVE
            } else {
                ImplFlags::SUCC_DEAD
            };
            self.impl_table[desc] |= if b.contains(&alives) && s.contains(&alives) {
                ImplFlags::SUCC_ALIVE
            } else if !b.contains(&alives) && !s.contains(&alives) {
                ImplFlags::SUCC_DEAD
            } else {
                ImplFlags::empty()
            };
        }

        // Fills in other positions.
        for unknowns in 1..=8 {
            for alives in 0..=8 - unknowns {
                let desc = (8 - alives - unknowns) << 8 | alives << 4;
                let desc0 = (8 - alives - unknowns + 1) << 8 | alives << 4;
                let desc1 = (8 - alives - unknowns) << 8 | (alives + 1) << 4;

                for state in 0..=2 {
                    let trans0 = self.impl_table[desc0 | state];

                    if trans0 == self.impl_table[desc1 | state] {
                        self.impl_table[desc | state] |= trans0;
                    }
                }
            }
        }

        self
    }

    /// Deduces the conflicts.
    fn init_conflict(mut self) -> Self {
        for nbhd_state in 0..0xff {
            for state in 0..=2 {
                let desc = nbhd_state << 4 | state;

                if self.impl_table[desc].contains(ImplFlags::SUCC_ALIVE) {
                    self.impl_table[desc | 0b10 << 2] = ImplFlags::CONFLICT;
                } else if self.impl_table[desc].contains(ImplFlags::SUCC_DEAD) {
                    self.impl_table[desc | 0b01 << 2] = ImplFlags::CONFLICT;
                }
            }
        }
        self
    }

    /// Deduces the implication for the cell itself.
    fn init_impl(mut self) -> Self {
        for unknowns in 0..=8 {
            for alives in 0..=8 - unknowns {
                let desc = (8 - alives - unknowns) << 8 | alives << 4;

                for succ_state in 1..=2 {
                    let flag = if succ_state == 0b10 {
                        ImplFlags::SUCC_ALIVE | ImplFlags::CONFLICT
                    } else {
                        ImplFlags::SUCC_DEAD | ImplFlags::CONFLICT
                    };

                    let possibly_dead = !self.impl_table[desc | 0b10].intersects(flag);
                    let possibly_alive = !self.impl_table[desc | 0b01].intersects(flag);

                    let index = desc | succ_state << 2;
                    if possibly_dead && !possibly_alive {
                        self.impl_table[index] |= ImplFlags::SELF_DEAD;
                    } else if !possibly_dead && possibly_alive {
                        self.impl_table[index] |= ImplFlags::SELF_ALIVE;
                    } else if !possibly_dead && !possibly_alive {
                        self.impl_table[index] = ImplFlags::CONFLICT;
                    }
                }
            }
        }

        self
    }

    ///  Deduces the implication for the neighbors.
    fn init_impl_nbhd(mut self) -> Self {
        for unknowns in 1..=8 {
            for alives in 0..=8 - unknowns {
                let desc = (8 - alives - unknowns) << 8 | alives << 4;
                let desc0 = (8 - alives - unknowns + 1) << 8 | alives << 4;
                let desc1 = (8 - alives - unknowns) << 8 | (alives + 1) << 4;

                for succ_state in 1..=2 {
                    let flag = if succ_state == 0b10 {
                        ImplFlags::SUCC_ALIVE | ImplFlags::CONFLICT
                    } else {
                        ImplFlags::SUCC_DEAD | ImplFlags::CONFLICT
                    };

                    let index = desc | succ_state << 2;

                    for state in 0..=2 {
                        let possibly_dead = !self.impl_table[desc0 | state].intersects(flag);
                        let possibly_alive = !self.impl_table[desc1 | state].intersects(flag);

                        if possibly_dead && !possibly_alive {
                            self.impl_table[index | state] |= ImplFlags::NBHD_DEAD;
                        } else if !possibly_dead && possibly_alive {
                            self.impl_table[index | state] |= ImplFlags::NBHD_ALIVE;
                        } else if !possibly_dead && !possibly_alive {
                            self.impl_table[index | state] = ImplFlags::CONFLICT;
                        }
                    }
                }
            }
        }

        self
    }
}

/// A parser for the rule.
impl ParseLife for Life {
    fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
        Self::new(b, s)
    }
}

impl FromStr for Life {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let rule: Life = ParseLife::parse_rule(input).map_err(Error::ParseRuleError)?;
        if rule.has_b0_s8() {
            Err(Error::B0S8Error)
        } else {
            Ok(rule)
        }
    }
}

impl Rule for Life {
    type Desc = NbhdDesc;
    const IS_GEN: bool = false;

    fn has_b0(&self) -> bool {
        self.b0
    }

    fn has_b0_s8(&self) -> bool {
        self.b0 && self.s8
    }

    fn gen(&self) -> usize {
        2
    }

    fn new_desc(state: State, succ_state: State) -> Self::Desc {
        let nbhd_state = match state {
            ALIVE => 0x08,
            _ => 0x80,
        };
        let succ_state = match succ_state {
            ALIVE => 0b01,
            _ => 0b10,
        };
        let state = match state {
            ALIVE => 0b01,
            _ => 0b10,
        };
        NbhdDesc(nbhd_state << 4 | succ_state << 2 | state)
    }

    fn update_desc(cell: CellRef<Self>, state: Option<State>, new: bool) {
        let state_num = match state {
            Some(ALIVE) => 0x01,
            Some(_) => 0x10,
            None => 0,
        };
        for &neigh in cell.nbhd.iter() {
            let neigh = neigh.unwrap();
            let mut desc = neigh.desc.get();
            if new {
                desc.0 += state_num << 4;
            } else {
                desc.0 -= state_num << 4;
            }
            neigh.desc.set(desc);
        }

        let change_num = match state {
            Some(ALIVE) => 0b01,
            Some(_) => 0b10,
            _ => 0,
        };
        if let Some(pred) = cell.pred {
            let mut desc = pred.desc.get();
            desc.0 ^= change_num << 2;
            pred.desc.set(desc);
        }
        let mut desc = cell.desc.get();
        desc.0 ^= change_num;
        cell.desc.set(desc);
    }

    fn consistify<'a>(world: &mut World<'a, Self>, cell: CellRef<'a, Self>) -> bool {
        let flags = world.rule.impl_table[cell.desc.get().0 as usize];

        if flags.is_empty() {
            return true;
        }

        if flags.contains(ImplFlags::CONFLICT) {
            return false;
        }

        if flags.intersects(ImplFlags::SUCC) {
            let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                DEAD
            } else {
                ALIVE
            };
            let succ = cell.succ.unwrap();
            return world.set_cell(succ, state, Reason::Rule(cell));
        }

        if flags.intersects(ImplFlags::SELF) {
            let state = if flags.contains(ImplFlags::SELF_DEAD) {
                DEAD
            } else {
                ALIVE
            };
            if !world.set_cell(cell, state, Reason::Rule(cell)) {
                return false;
            }
        }

        if flags.intersects(ImplFlags::NBHD) {
            {
                let state = if flags.contains(ImplFlags::NBHD_DEAD) {
                    DEAD
                } else {
                    ALIVE
                };
                for &neigh in cell.nbhd.iter() {
                    if let Some(neigh) = neigh {
                        if neigh.state.get().is_none()
                            && !world.set_cell(neigh, state, Reason::Rule(cell))
                        {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }
}

/// The neighborhood descriptor.
///
/// Including a descriptor for the corresponding non-Generations rule,
/// and the states of the successor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NbhdDescGen(u16, Option<State>);

/// Totalistic Life-like Generations rules.
pub struct LifeGen {
    /// Whether the rule contains `B0`.
    b0: bool,
    /// Whether the rule contains `S8`.
    s8: bool,
    /// Number of states.
    gen: usize,
    /// An array of actions for all neighborhood descriptors.
    impl_table: [ImplFlags; 1 << 12],
}

impl LifeGen {
    /// Constructs a new rule from the `b` and `s` data
    /// and the number of states.
    pub fn new(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
        let life = Life::new(b, s);
        let impl_table = life.impl_table;
        Self {
            b0: life.b0,
            s8: life.s8,
            gen,
            impl_table,
        }
    }

    /// Converts to the corresponding non-Generations rule.
    pub fn non_gen(self) -> Life {
        Life {
            b0: self.b0,
            s8: self.s8,
            impl_table: self.impl_table,
        }
    }
}

/// A parser for the rule.
impl ParseLifeGen for LifeGen {
    fn from_bsg(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
        Self::new(b, s, gen)
    }
}

impl FromStr for LifeGen {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let rule: LifeGen = ParseLifeGen::parse_rule(input).map_err(Error::ParseRuleError)?;
        if rule.has_b0_s8() {
            Err(Error::B0S8Error)
        } else {
            Ok(rule)
        }
    }
}

/// NOTE: This implementation does work when the number of states is 2.
impl Rule for LifeGen {
    type Desc = NbhdDescGen;
    const IS_GEN: bool = true;

    fn has_b0(&self) -> bool {
        self.b0
    }

    fn has_b0_s8(&self) -> bool {
        self.b0 && self.s8
    }

    fn gen(&self) -> usize {
        self.gen
    }

    fn new_desc(state: State, succ_state: State) -> Self::Desc {
        let desc = Life::new_desc(state, succ_state);
        NbhdDescGen(desc.0, Some(succ_state))
    }

    fn update_desc(cell: CellRef<Self>, state: Option<State>, new: bool) {
        {
            let state_num = match state {
                Some(ALIVE) => 0x01,
                Some(_) => 0x10,
                None => 0,
            };
            for &neigh in cell.nbhd.iter() {
                let neigh = neigh.unwrap();
                let mut desc = neigh.desc.get();
                if new {
                    desc.0 += state_num << 4;
                } else {
                    desc.0 -= state_num << 4;
                }
                neigh.desc.set(desc);
            }
        }
        let change_num = match state {
            Some(ALIVE) => 0b01,
            Some(_) => 0b10,
            _ => 0,
        };
        if let Some(pred) = cell.pred {
            let mut desc = pred.desc.get();
            desc.0 ^= change_num << 2;
            desc.1 = if new { state } else { None };
            pred.desc.set(desc);
        }
        let mut desc = cell.desc.get();
        desc.0 ^= change_num;
        cell.desc.set(desc);
    }

    fn consistify<'a>(world: &mut World<'a, Self>, cell: CellRef<'a, Self>) -> bool {
        let desc = cell.desc.get();
        let flags = world.rule.impl_table[desc.0 as usize];
        let gen = world.rule.gen;

        match cell.state.get() {
            Some(DEAD) => {
                if let Some(State(j)) = desc.1 {
                    if j >= 2 {
                        return false;
                    }
                }

                if flags.intersects(ImplFlags::SUCC) {
                    let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                        DEAD
                    } else {
                        ALIVE
                    };
                    let succ = cell.succ.unwrap();
                    return world.set_cell(succ, state, Reason::Rule(cell));
                }
            }
            Some(ALIVE) => {
                if let Some(State(j)) = desc.1 {
                    if j == 0 || j > 2 {
                        return false;
                    }
                }

                if flags.intersects(ImplFlags::SUCC) {
                    let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                        State(2)
                    } else {
                        ALIVE
                    };
                    let succ = cell.succ.unwrap();
                    return world.set_cell(succ, state, Reason::Rule(cell));
                }
            }
            Some(State(i)) => {
                if let Some(State(j)) = desc.1 {
                    return j == (i + 1) % gen;
                } else {
                    let succ = cell.succ.unwrap();
                    return world.set_cell(succ, State((i + 1) % gen), Reason::Rule(cell));
                }
            }
            None => match desc.1 {
                Some(DEAD) => {
                    if flags.contains(ImplFlags::SELF_ALIVE) {
                        return world.set_cell(cell, State(gen - 1), Reason::Rule(cell));
                    } else {
                        return true;
                    }
                }
                Some(ALIVE) => {
                    if flags.intersects(ImplFlags::SELF) {
                        let state = if flags.contains(ImplFlags::SELF_DEAD) {
                            DEAD
                        } else {
                            ALIVE
                        };
                        if !world.set_cell(cell, state, Reason::Rule(cell)) {
                            return false;
                        }
                    }
                }
                Some(State(j)) => {
                    return world.set_cell(cell, State(j - 1), Reason::Rule(cell));
                }
                None => return true,
            },
        }

        if flags.is_empty() {
            return true;
        }

        if flags.contains(ImplFlags::CONFLICT) {
            return false;
        }

        if flags.intersects(ImplFlags::NBHD_ALIVE) {
            for &neigh in cell.nbhd.iter() {
                if let Some(neigh) = neigh {
                    if neigh.state.get().is_none()
                        && !world.set_cell(neigh, ALIVE, Reason::Rule(cell))
                    {
                        return false;
                    }
                }
            }
        }

        true
    }
}
