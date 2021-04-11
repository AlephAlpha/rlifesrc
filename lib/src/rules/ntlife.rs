//! Non-totalistic Life-like rules.

use crate::{
    cells::{CellRef, State, ALIVE, DEAD},
    error::Error,
    rules::{private::Sealed, Rule},
    search::Reason,
    world::World,
};
use bitflags::bitflags;
use ca_rules::{ParseNtLife, ParseNtLifeGen};
use std::str::FromStr;

bitflags! {
    /// Flags to imply the state of a cell and its neighbors.
    #[derive(Default)]
    struct ImplFlags: u32 {
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

        /// The state of at least one unknown neighbor is implied.
        const NBHD = 0xffff << 6;
    }
}

/// The neighborhood descriptor.
///
/// It is a 20-bit integer of the form `0b_abcdefgh_ijklmnop_qr_st`,
/// where:
///
/// * `0b_ai`, `0b_bj`, ..., `0b_hp` are the states of the eight neighbors,
/// * `0b_qr` is the state of the successor.
/// * `0b_st` is the state of the cell itself.
/// * `0b_10` means dead,
/// * `0b_01` means alive,
/// * `0b_00` means unknown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NbhdDesc(u32);

/// Non-totalistic Life-like rules.
///
/// This includes any rule that can be converted to a non-totalistic
/// Life-like rule: isotropic non-totalistic rules,
/// non-isotropic rules, hexagonal rules, rules with von Neumann
/// neighborhoods, etc.
pub struct NtLife {
    /// Whether the rule contains `B0`.
    b0: bool,
    /// Whether the rule contains `S8`.
    s8: bool,
    /// An array of actions for all neighborhood descriptors.
    impl_table: Vec<ImplFlags>,
}

impl NtLife {
    /// Constructs a new rule from the `b` and `s` data.
    pub fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
        let b0 = b.contains(&0x00);
        let s8 = s.contains(&0xff);

        let impl_table = vec![ImplFlags::empty(); 1 << 20];

        Self { b0, s8, impl_table }
            .init_trans(b, s)
            .init_conflict()
            .init_impl()
            .init_impl_nbhd()
    }

    /// Deduces the implication for the successor.
    fn init_trans(mut self, b: Vec<u8>, s: Vec<u8>) -> Self {
        // Fills in the positions of the neighborhood descriptors
        // that have no unknown neighbors.
        for alives in 0..=0xff {
            let desc = (0xff & !alives) << 12 | alives << 4;
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

        // Fills in the other positions.
        for unknowns in 1usize..=0xff {
            // `n` is the largest power of two smaller than `unknowns`.
            let n = unknowns.next_power_of_two() >> usize::from(!unknowns.is_power_of_two());
            for alives in (0..=0xff).filter(|a| a & unknowns == 0) {
                let desc = (0xff & !alives & !unknowns) << 12 | alives << 4;
                let desc0 = (0xff & !alives & !unknowns | n) << 12 | alives << 4;
                let desc1 = (0xff & !alives & !unknowns) << 12 | (alives | n) << 4;

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
        for nbhd_state in 0..0xffff {
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
        for unknowns in 0..=0xff {
            for alives in (0..=0xff).filter(|a| a & unknowns == 0) {
                let desc = (0xff & !alives & !unknowns) << 12 | alives << 4;

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
        for unknowns in 1usize..=0xff {
            // `n` runs through all the non-zero binary digits of `unknowns`.
            for n in (0..8).map(|i| 1 << i).filter(|n| unknowns & n != 0) {
                for alives in 0..=0xff {
                    let desc = (0xff & !alives & !unknowns) << 12 | alives << 4;
                    let desc0 = (0xff & !alives & !unknowns | n) << 12 | alives << 4;
                    let desc1 = (0xff & !alives & !unknowns) << 12 | (alives | n) << 4;

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
                                self.impl_table[index | state] |=
                                    ImplFlags::from_bits((n.pow(2) << 7) as u32).unwrap();
                            } else if !possibly_dead && possibly_alive {
                                self.impl_table[index | state] |=
                                    ImplFlags::from_bits((n.pow(2) << 6) as u32).unwrap();
                            } else if !possibly_dead && !possibly_alive {
                                self.impl_table[index | state] = ImplFlags::CONFLICT;
                            }
                        }
                    }
                }
            }
        }

        self
    }
}

/// A parser for the rule.
impl ParseNtLife for NtLife {
    fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
        Self::new(b, s)
    }
}

impl FromStr for NtLife {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let rule: Self = ParseNtLife::parse_rule(input).map_err(Error::ParseRuleError)?;
        if rule.has_b0_s8() {
            Err(Error::B0S8Error)
        } else {
            Ok(rule)
        }
    }
}

impl Sealed for NtLife {}

impl Rule for NtLife {
    type Desc = NbhdDesc;
    const IS_GEN: bool = false;

    #[inline]
    fn has_b0(&self) -> bool {
        self.b0
    }

    #[inline]
    fn has_b0_s8(&self) -> bool {
        self.b0 && self.s8
    }

    #[inline]
    fn gen(&self) -> usize {
        2
    }

    #[inline]
    fn new_desc(state: State, succ_state: State) -> Self::Desc {
        let nbhd_state = match state {
            ALIVE => 0x00ff,
            _ => 0xff00,
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

    fn update_desc(cell: CellRef<Self>, state: Option<State>, _new: bool) {
        let nbhd_change_num = match state {
            Some(ALIVE) => 0x0001,
            Some(_) => 0x0100,
            _ => 0x0000,
        };
        for (i, &neigh) in cell.nbhd.iter().rev().enumerate() {
            let neigh = neigh.unwrap();
            let mut desc = neigh.desc.get();
            desc.0 ^= nbhd_change_num << i << 4;
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

    fn consistify<'a, RE: Reason<'a, Self>>(
        world: &mut World<'a, Self, RE>,
        cell: CellRef<'a, Self>,
    ) -> Result<(), RE::ConflReason> {
        let flags = world.rule.impl_table[cell.desc.get().0 as usize];

        if flags.is_empty() {
            return Ok(());
        }

        if flags.contains(ImplFlags::CONFLICT) {
            return Err(RE::confl_from_cell(cell));
        }

        if flags.intersects(ImplFlags::SUCC) {
            let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                DEAD
            } else {
                ALIVE
            };
            let succ = cell.succ.unwrap();
            return world.set_cell(succ, state, RE::from_cell(cell));
        }

        if flags.intersects(ImplFlags::SELF) {
            let state = if flags.contains(ImplFlags::SELF_DEAD) {
                DEAD
            } else {
                ALIVE
            };
            world.set_cell(cell, state, RE::from_cell(cell))?;
        }

        if flags.intersects(ImplFlags::NBHD) {
            {
                for (i, &neigh) in cell.nbhd.iter().enumerate() {
                    if flags.intersects(ImplFlags::from_bits(3 << (2 * i + 6)).unwrap()) {
                        if let Some(neigh) = neigh {
                            let state = if flags
                                .contains(ImplFlags::from_bits(1 << (2 * i + 7)).unwrap())
                            {
                                DEAD
                            } else {
                                ALIVE
                            };
                            world.set_cell(neigh, state, RE::from_cell(cell))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// The neighborhood descriptor.
///
/// Including a descriptor for the corresponding non-Generations rule,
/// and the states of the successor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NbhdDescGen(u32, Option<State>);

/// Non-totalistic Life-like Generations rules.
///
/// This includes any rule that can be converted to a non-totalistic
/// Life-like Generations rule.
pub struct NtLifeGen {
    /// Whether the rule contains `B0`.
    b0: bool,
    /// Whether the rule contains `S8`.
    s8: bool,
    /// Number of states.
    gen: usize,
    /// An array of actions for all neighborhood descriptors.
    impl_table: Vec<ImplFlags>,
}

impl NtLifeGen {
    /// Constructs a new rule from the `b` and `s` data
    /// and the number of states.
    pub fn new(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
        let life = NtLife::new(b, s);
        let impl_table = life.impl_table;
        Self {
            b0: life.b0,
            s8: life.s8,
            gen,
            impl_table,
        }
    }

    /// Converts to the corresponding non-Generations rule.
    pub fn non_gen(self) -> NtLife {
        NtLife {
            b0: self.b0,
            s8: self.s8,
            impl_table: self.impl_table,
        }
    }
}

/// A parser for the rule.
impl ParseNtLifeGen for NtLifeGen {
    fn from_bsg(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
        Self::new(b, s, gen)
    }
}

impl FromStr for NtLifeGen {
    type Err = Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let rule: Self = ParseNtLifeGen::parse_rule(input).map_err(Error::ParseRuleError)?;
        if rule.has_b0_s8() {
            Err(Error::B0S8Error)
        } else {
            Ok(rule)
        }
    }
}

impl Sealed for NtLifeGen {}

/// NOTE: This implementation does work when the number of states is 2.
impl Rule for NtLifeGen {
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
        let desc = NtLife::new_desc(state, succ_state);
        NbhdDescGen(desc.0, Some(succ_state))
    }

    fn update_desc(cell: CellRef<Self>, state: Option<State>, _new: bool) {
        let nbhd_change_num = match state {
            Some(ALIVE) => 0x0001,
            Some(_) => 0x0100,
            _ => 0x0000,
        };
        for (i, &neigh) in cell.nbhd.iter().rev().enumerate() {
            let neigh = neigh.unwrap();
            let mut desc = neigh.desc.get();
            desc.0 ^= nbhd_change_num << i << 4;
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
            desc.1 = if _new { state } else { None };
            pred.desc.set(desc);
        }
        let mut desc = cell.desc.get();
        desc.0 ^= change_num;
        cell.desc.set(desc);
    }

    fn consistify<'a, RE: Reason<'a, Self>>(
        world: &mut World<'a, Self, RE>,
        cell: CellRef<'a, Self>,
    ) -> Result<(), RE::ConflReason> {
        let desc = cell.desc.get();
        let flags = world.rule.impl_table[desc.0 as usize];
        let gen = world.rule.gen;

        match cell.state.get() {
            Some(DEAD) => {
                if let Some(State(j)) = desc.1 {
                    if j >= 2 {
                        return Err(RE::confl_from_cell(cell));
                    }
                }

                if flags.intersects(ImplFlags::SUCC) {
                    let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                        DEAD
                    } else {
                        ALIVE
                    };
                    let succ = cell.succ.unwrap();
                    return world.set_cell(succ, state, RE::from_cell(cell));
                }
            }
            Some(ALIVE) => {
                if let Some(State(j)) = desc.1 {
                    if j == 0 || j > 2 {
                        return Err(RE::confl_from_cell(cell));
                    }
                }
                if flags.intersects(ImplFlags::SUCC) {
                    let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                        State(2)
                    } else {
                        ALIVE
                    };
                    let succ = cell.succ.unwrap();
                    return world.set_cell(succ, state, RE::from_cell(cell));
                }
            }
            Some(State(i)) => {
                if let Some(State(j)) = desc.1 {
                    if j == (i + 1) % gen {
                        return Ok(());
                    } else {
                        return Err(RE::confl_from_cell(cell));
                    }
                } else {
                    let succ = cell.succ.unwrap();
                    return world.set_cell(succ, State((i + 1) % gen), RE::from_cell(cell));
                }
            }
            None => match desc.1 {
                Some(DEAD) => {
                    if flags.contains(ImplFlags::SELF_ALIVE) {
                        return world.set_cell(cell, State(gen - 1), RE::from_cell(cell));
                    } else {
                        return Ok(());
                    }
                }
                Some(ALIVE) => {
                    if flags.intersects(ImplFlags::SELF) {
                        let state = if flags.contains(ImplFlags::SELF_DEAD) {
                            DEAD
                        } else {
                            ALIVE
                        };
                        world.set_cell(cell, state, RE::from_cell(cell))?;
                    }
                }
                Some(State(j)) => {
                    return world.set_cell(cell, State(j - 1), RE::from_cell(cell));
                }
                None => return Ok(()),
            },
        }

        if flags.is_empty() {
            return Ok(());
        }

        if flags.contains(ImplFlags::CONFLICT) {
            return Err(RE::confl_from_cell(cell));
        }

        if flags.intersects(ImplFlags::NBHD) {
            for (i, &neigh) in cell.nbhd.iter().enumerate() {
                if flags.intersects(ImplFlags::from_bits(1 << (2 * i + 6)).unwrap()) {
                    if let Some(neigh) = neigh {
                        world.set_cell(neigh, ALIVE, RE::from_cell(cell))?;
                    }
                }
            }
        }

        Ok(())
    }
}
