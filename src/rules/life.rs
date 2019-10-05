//! Totalistic life-like rules.

use crate::{
    cells::{Alive, Dead, LifeCell, State},
    rules::Rule,
    world::World,
};
use bitflags::bitflags;
use ca_rules::ParseLife;

const DEAD: usize = 0b10;
const ALIVE: usize = 0b01;

#[derive(Clone, Copy, Debug, PartialEq)]
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
pub struct NbhdDesc(usize);

bitflags! {
    /// Flags to imply the state of a cell and its neighbors.
    struct ImplFlags: u8 {
        /// A conflict is detected.
        const CONFLICT = 0b_0000_0001;

        /// The successor must be alive.
        const SUCC_ALIVE = 0b_0000_0100;

        /// The successor must be dead.
        const SUCC_DEAD = 0b_0000_1000;

        /// The cell itself must be alive.
        const SELF_ALIVE = 0b_0001_0000;

        /// The cell itself must be dead.
        const SELF_DEAD = 0b_0010_0000;

        /// All unknown neighbors must be alive.
        const NBHD_ALIVE = 0b_0100_0000;

        /// All unknown neighbors must be dead.
        const NBHD_DEAD = 0b_1000_0000;
    }
}

impl Default for ImplFlags {
    fn default() -> Self {
        ImplFlags::empty()
    }
}

/// Totalistic life-like rules.
///
/// The struct will not store the definition of the rule itself,
/// but the results of `transition` and `implication`.
pub struct Life {
    /// Whether the rule contains `B0`.
    b0: bool,

    /// An array of actions for all neighborhood descriptors.
    impl_table: [ImplFlags; 1 << 12],
}

impl Life {
    /// Constructs a new rule from the `b` and `s` data.
    pub fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
        let b0 = b.contains(&0);

        let impl_table = [ImplFlags::empty(); 1 << 12];

        Life { b0, impl_table }
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
            self.impl_table[desc | DEAD] |= if b.contains(&alives) {
                ImplFlags::SUCC_ALIVE
            } else {
                ImplFlags::SUCC_DEAD
            };
            self.impl_table[desc | ALIVE] |= if s.contains(&alives) {
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

                for &state in [DEAD, ALIVE, 0].iter() {
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
            for &state in [DEAD, ALIVE, 0].iter() {
                let desc = nbhd_state << 4 | state;

                if self.impl_table[desc].contains(ImplFlags::SUCC_ALIVE) {
                    self.impl_table[desc | DEAD << 2] = ImplFlags::CONFLICT;
                } else if self.impl_table[desc].contains(ImplFlags::SUCC_DEAD) {
                    self.impl_table[desc | ALIVE << 2] = ImplFlags::CONFLICT;
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

                for &succ_state in [DEAD, ALIVE].iter() {
                    let flag = if succ_state == DEAD {
                        ImplFlags::SUCC_ALIVE | ImplFlags::CONFLICT
                    } else {
                        ImplFlags::SUCC_DEAD | ImplFlags::CONFLICT
                    };

                    let possibly_dead = !self.impl_table[desc | DEAD].intersects(flag);
                    let possibly_alive = !self.impl_table[desc | ALIVE].intersects(flag);

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

                for &succ_state in [DEAD, ALIVE].iter() {
                    let flag = if succ_state == DEAD {
                        ImplFlags::SUCC_ALIVE | ImplFlags::CONFLICT
                    } else {
                        ImplFlags::SUCC_DEAD | ImplFlags::CONFLICT
                    };

                    let index = desc | succ_state << 2;

                    for &state in [DEAD, ALIVE, 0].iter() {
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

    pub fn parse_rule(input: &str) -> Result<Self, String> {
        ParseLife::parse_rule(input).map_err(|e| e.to_string())
    }
}

impl Rule for Life {
    type Desc = NbhdDesc;

    fn b0(&self) -> bool {
        self.b0
    }

    fn new_desc(state: State, succ_state: State) -> Self::Desc {
        let nbhd_state = match state {
            Dead => 0x80,
            Alive => 0x08,
        };
        let succ_state = match succ_state {
            Dead => DEAD,
            Alive => ALIVE,
        };
        let state = match state {
            Dead => DEAD,
            Alive => ALIVE,
        };
        NbhdDesc(nbhd_state << 4 | succ_state << 2 | state)
    }

    fn update_desc(cell: &LifeCell<Self>, old_state: Option<State>, state: Option<State>) {
        let old_state_num = match old_state {
            Some(Dead) => 0x10,
            Some(Alive) => 0x01,
            None => 0,
        };
        let state_num = match state {
            Some(Dead) => 0x10,
            Some(Alive) => 0x01,
            None => 0,
        };
        for &neigh in cell.nbhd.iter() {
            let neigh = neigh.unwrap();
            let mut desc = neigh.desc.get();
            desc.0 -= old_state_num << 4;
            desc.0 += state_num << 4;
            neigh.desc.set(desc);
        }

        let change_num = match (state, old_state) {
            (Some(Dead), Some(Alive)) | (Some(Alive), Some(Dead)) => 0b11,
            (Some(Dead), None) | (None, Some(Dead)) => 0b10,
            (Some(Alive), None) | (None, Some(Alive)) => 0b01,
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

    fn consistify<'a>(
        &self,
        cell: &'a LifeCell<'a, Self>,
        world: &World<'a, Self>,
        set_stack: &mut Vec<&'a LifeCell<'a, Self>>,
    ) -> bool {
        let flags = self.impl_table[cell.desc.get().0];

        if flags.contains(ImplFlags::CONFLICT) {
            return false;
        }

        let succ_state = if flags.contains(ImplFlags::SUCC_DEAD) {
            Some(Dead)
        } else if flags.contains(ImplFlags::SUCC_ALIVE) {
            Some(Alive)
        } else {
            None
        };
        if let Some(succ_state) = succ_state {
            let succ = cell.succ.unwrap();
            world.set_cell(succ, Some(succ_state), false);
            set_stack.push(succ);
            return true;
        }

        let state = if flags.contains(ImplFlags::SELF_DEAD) {
            Some(Dead)
        } else if flags.contains(ImplFlags::SELF_ALIVE) {
            Some(Alive)
        } else {
            None
        };
        if let Some(state) = state {
            world.set_cell(cell, Some(state), false);
            set_stack.push(cell);
        }

        let nbhd_state = if flags.contains(ImplFlags::NBHD_DEAD) {
            Some(Dead)
        } else if flags.contains(ImplFlags::NBHD_ALIVE) {
            Some(Alive)
        } else {
            None
        };
        if let Some(nbhd_state) = nbhd_state {
            for &neigh in cell.nbhd.iter() {
                if let Some(neigh) = neigh {
                    if neigh.state.get().is_none() {
                        world.set_cell(neigh, Some(nbhd_state), false);
                        set_stack.push(neigh);
                    }
                }
            }
        }

        true
    }
}

/// A parser for the rule.
impl ParseLife for Life {
    fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
        Self::new(b, s)
    }
}
