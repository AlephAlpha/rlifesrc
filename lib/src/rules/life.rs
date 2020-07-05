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

impl_rule! {
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
    pub struct NbhdDesc(u16);

    /// Totalistic Life-like rules.
    pub struct Life {
        Parser: ParseLife,
        impl_table: [ImplFlags; 1 << 12],
    }

    /// Totalistic Life-like Generations rules.
    pub struct LifeGen {
        Parser: ParseLifeGen,
    }

    fn new_desc {
        ALIVE => 0x08,
        DEAD => 0x80,
    }

    fn update_desc(cell, state, new, change_num) {
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

    fn consistify<'a>(world, cell, flags) {
        let state = if flags.contains(ImplFlags::NBHD_DEAD) {
            DEAD
        } else {
            ALIVE
        };
        for &neigh in cell.nbhd.iter() {
            if let Some(neigh) = neigh {
                if neigh.state.get().is_none() && !world.set_cell(neigh, state, Reason::Deduce)
                {
                    return false;
                }
            }
        }
    }

    fn consistify_gen<'a>(world, cell, flags) {
        if flags.intersects(ImplFlags::NBHD_ALIVE) {
            for &neigh in cell.nbhd.iter() {
                if let Some(neigh) = neigh {
                    if neigh.state.get().is_none() && !world.set_cell(neigh, ALIVE, Reason::Deduce)
                    {
                        return false;
                    }
                }
            }
        }
    }
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
