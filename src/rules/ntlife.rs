//! Non-totalistic life-like rules.

use crate::{
    cells::{Alive, Dead, LifeCell, State},
    rules::{Desc, Rule},
    world::World,
};
pub use ca_rules::ParseNtLife;

#[derive(Clone, Copy, Default)]
/// The neighborhood descriptor.
///
/// It is a 16-bit integer, where the first 8 bits tell whether a neighbor
/// is dead, while the last 8 bits tell whether a neighbor is alive.
///
/// For example, the following neighborhood is described by
/// `0b_01001000_10010001`.
///
/// ```plaintext
/// O . ?
/// O _ .
/// ? ? O
/// ```
pub struct NbhdDesc(u16);

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        match state {
            Some(Dead) => NbhdDesc(0xff00),
            Some(Alive) => NbhdDesc(0x00ff),
            None => NbhdDesc(0x0000),
        }
    }

    fn update_desc(cell: &LifeCell<Self>, old_state: Option<State>, state: Option<State>) {
        let change_num = match (state, old_state) {
            (Some(Dead), Some(Alive)) => 0x0101,
            (Some(Alive), Some(Dead)) => 0x0101,
            (Some(Dead), None) | (None, Some(Dead)) => 0x0100,
            (Some(Alive), None) | (None, Some(Alive)) => 0x0001,
            _ => 0x0000,
        };
        for (i, &neigh) in cell.nbhd.iter().rev().enumerate() {
            let neigh = neigh.unwrap();
            let mut desc = neigh.desc.get();
            desc.0 ^= change_num << i;
            neigh.desc.set(desc);
        }
    }
}

/// A struct to store the results of `transition` and `implication`.
#[derive(Clone, Copy, Default)]
struct Implication<T> {
    dead: T,
    alive: T,
    none: T,
}

/// Non-totalistic life-like rules.
///
/// The struct will not store the definition of the rule itself,
/// but the results of `transition` and `implication`.
pub struct Life {
    b0: bool,
    trans_table: Box<[Implication<Option<State>>; 65536]>,
    impl_table: Box<[Option<State>; 65536 * 2]>,
    impl_nbhd_table: Box<[Implication<NbhdDesc>; 65536 * 2]>,
}

impl Life {
    /// Constructs a new rule from the `b` and `s` data.
    pub fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
        let b0 = b.contains(&0);

        let trans_table = Self::init_trans_table(b, s);
        let impl_table = Self::init_impl_table(&trans_table);
        let impl_nbhd_table = Self::init_impl_nbhd_table(&trans_table);

        Life {
            b0,
            trans_table,
            impl_table,
            impl_nbhd_table,
        }
    }

    /// Generates the transition table.
    fn init_trans_table(b: Vec<u8>, s: Vec<u8>) -> Box<[Implication<Option<State>>; 65536]> {
        let mut trans_table: Box<[Implication<Option<State>>; 65536]> =
            Box::new([Default::default(); 65536]);

        // Fills in the positions of the neighborhood descriptors
        // that have no unknown neighbors.
        for alives in 0..=0xff {
            let nbhd = ((0xff & !alives) << 8) | alives;
            let alives = alives as u8;
            trans_table[nbhd].dead = if b.contains(&alives) {
                Some(Alive)
            } else {
                Some(Dead)
            };
            trans_table[nbhd].alive = if s.contains(&alives) {
                Some(Alive)
            } else {
                Some(Dead)
            };
            trans_table[nbhd].none = if b.contains(&alives) && s.contains(&alives) {
                Some(Alive)
            } else if !b.contains(&alives) && !s.contains(&alives) {
                Some(Dead)
            } else {
                None
            };
        }

        // Fills in the other positions.
        for unknowns in 1usize..=0xff {
            // `n` is the largest power of two smaller than `unknowns`.
            let n = unknowns.next_power_of_two() >> usize::from(!unknowns.is_power_of_two());
            for alives in (0..=0xff).filter(|a| a & unknowns == 0) {
                let nbhd = ((0xff & !alives & !unknowns) << 8) | alives;
                let nbhd0 = ((0xff & !alives & !unknowns | n) << 8) | alives;
                let nbhd1 = ((0xff & !alives & !unknowns) << 8) | alives | n;
                let trans0 = trans_table[nbhd0];
                let trans1 = trans_table[nbhd1];
                if trans0.dead == trans1.dead {
                    trans_table[nbhd].dead = trans0.dead;
                }
                if trans0.alive == trans1.alive {
                    trans_table[nbhd].alive = trans0.alive;
                }
                if trans0.none == trans1.none {
                    trans_table[nbhd].none = trans0.none;
                }
            }
        }

        trans_table
    }

    /// Generates the implication table.
    fn init_impl_table(
        trans_table: &[Implication<Option<State>>; 65536],
    ) -> Box<[Option<State>; 65536 * 2]> {
        let mut impl_table = Box::new([Default::default(); 65536 * 2]);

        for unknowns in 0..=0xff {
            for alives in (0..=0xff).filter(|a| a & unknowns == 0) {
                let nbhd = ((0xff & !alives & !unknowns) << 8) | alives;
                for (i, &succ) in [Dead, Alive].iter().enumerate() {
                    let index = nbhd * 2 + i;
                    let possibly_dead = match trans_table[nbhd].dead {
                        Some(state) => state == succ,
                        None => true,
                    };
                    let possibly_alive = match trans_table[nbhd].alive {
                        Some(state) => state == succ,
                        None => true,
                    };
                    if possibly_dead && !possibly_alive {
                        impl_table[index] = Some(Dead);
                    } else if !possibly_dead && possibly_alive {
                        impl_table[index] = Some(Alive);
                    }
                }
            }
        }

        impl_table
    }

    /// Generates the neighborhood implication table.
    fn init_impl_nbhd_table(
        trans_table: &[Implication<Option<State>>; 65536],
    ) -> Box<[Implication<NbhdDesc>; 65536 * 2]> {
        let mut impl_nbhd_table: Box<[Implication<NbhdDesc>; 65536 * 2]> =
            Box::new([Default::default(); 65536 * 2]);

        for unknowns in 1usize..=0xff {
            // `n` runs through all the non-zero binary digits of `unknowns`.
            for n in (0..8).map(|i| 1 << i).filter(|n| unknowns & n != 0) {
                for alives in 0..=0xff {
                    let nbhd = ((0xff & !alives & !unknowns) << 8) | alives;
                    let nbhd0 = ((0xff & !alives & !unknowns | n) << 8) | alives;
                    let nbhd1 = ((0xff & !alives & !unknowns) << 8) | alives | n;
                    let trans0 = trans_table[nbhd0];
                    let trans1 = trans_table[nbhd1];
                    for (i, &succ) in [Dead, Alive].iter().enumerate() {
                        let index = nbhd * 2 + i;

                        let possibly_dead = match trans0.dead {
                            Some(state) => state == succ,
                            None => true,
                        };
                        let possibly_alive = match trans1.dead {
                            Some(state) => state == succ,
                            None => true,
                        };
                        if possibly_dead && !possibly_alive {
                            impl_nbhd_table[index].dead.0 |= (n << 8) as u16;
                        } else if !possibly_dead && possibly_alive {
                            impl_nbhd_table[index].dead.0 |= n as u16;
                        }

                        let possibly_dead = match trans0.alive {
                            Some(state) => state == succ,
                            None => true,
                        };
                        let possibly_alive = match trans1.alive {
                            Some(state) => state == succ,
                            None => true,
                        };
                        if possibly_dead && !possibly_alive {
                            impl_nbhd_table[index].alive.0 |= (n << 8) as u16;
                        } else if !possibly_dead && possibly_alive {
                            impl_nbhd_table[index].alive.0 |= n as u16;
                        }

                        let possibly_dead = match trans0.none {
                            Some(state) => state == succ,
                            None => true,
                        };
                        let possibly_alive = match trans1.none {
                            Some(state) => state == succ,
                            None => true,
                        };
                        if possibly_dead && !possibly_alive {
                            impl_nbhd_table[index].none.0 |= (n << 8) as u16;
                        } else if !possibly_dead && possibly_alive {
                            impl_nbhd_table[index].none.0 |= n as u16;
                        }
                    }
                }
            }
        }

        impl_nbhd_table
    }

    /// Implicates states of some neighbors using the
    /// neighborhood implication table.
    fn implication_nbhd(
        &self,
        state: Option<State>,
        desc: NbhdDesc,
        succ_state: State,
    ) -> NbhdDesc {
        let index = desc.0 as usize * 2
            + match succ_state {
                Dead => 0,
                Alive => 1,
            };
        let implication = self.impl_nbhd_table[index];
        match state {
            Some(Dead) => implication.dead,
            Some(Alive) => implication.alive,
            None => implication.none,
        }
    }
}

impl Rule for Life {
    type Desc = NbhdDesc;

    fn b0(&self) -> bool {
        self.b0
    }

    fn transition(&self, state: Option<State>, desc: NbhdDesc) -> Option<State> {
        let transition = self.trans_table[desc.0 as usize];
        match state {
            Some(Dead) => transition.dead,
            Some(Alive) => transition.alive,
            None => transition.none,
        }
    }

    fn implication(&self, desc: NbhdDesc, succ_state: State) -> Option<State> {
        let index = desc.0 as usize * 2
            + match succ_state {
                Dead => 0,
                Alive => 1,
            };
        self.impl_table[index]
    }

    fn consistify_nbhd<'a>(
        &self,
        cell: &LifeCell<'a, NbhdDesc>,
        world: &World<'a, NbhdDesc, Self>,
        desc: NbhdDesc,
        state: Option<State>,
        succ_state: State,
        stack: &mut Vec<&'a LifeCell<'a, Self::Desc>>,
    ) {
        let nbhd_states = self.implication_nbhd(state, desc, succ_state).0;
        if nbhd_states != 0 {
            for (i, &neigh) in cell.nbhd.iter().enumerate() {
                let state = match nbhd_states >> i & 0x0101 {
                    0x0001 => Alive,
                    0x0100 => Dead,
                    _ => continue,
                };
                if let Some(neigh) = neigh {
                    world.set_cell(neigh, Some(state), false);
                    stack.push(neigh);
                }
            }
        }
    }
}

/// A parser for the rule.
impl ParseNtLife for Life {
    fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
        Life::new(b, s)
    }
}
