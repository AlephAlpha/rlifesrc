//! Totalistic life-like rules.

use crate::{
    cells::{Alive, Dead, LifeCell, State},
    rules::Rule,
    world::World,
};
use ca_rules::ParseLife;

#[derive(Clone, Copy)]
/// The neighborhood descriptor.
///
/// It counts the number of dead cells and living cells in the neighborhood,
/// write them as two 4-bit integers, and concatenate these two integers
/// into a `u8`.
///
/// For example, if a cell has 4 dead neighbors, 3 living neighbors,
/// 1 unknown neighbors, the the neighborhood descriptor is `0x43`.
pub struct NbhdDesc(u8);

/// A struct to store the results of `transition` and `implication`.
#[derive(Clone, Copy, Default)]
struct Implication {
    dead: Option<State>,
    alive: Option<State>,
    none: Option<State>,
}

/// Totalistic life-like rules.
///
/// The struct will not store the definition of the rule itself,
/// but the results of `transition` and `implication`.
pub struct Life {
    b0: bool,
    trans_table: [Implication; 256],
    impl_table: [Option<State>; 512],
    impl_nbhd_table: [Implication; 512],
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
    fn init_trans_table(b: Vec<u8>, s: Vec<u8>) -> [Implication; 256] {
        let mut trans_table: [Implication; 256] = [Default::default(); 256];

        // Fills in the positions of the neighborhood descriptors
        // that have no unknown neighbors.
        for alives in 0..=8 {
            let nbhd = ((8 - alives) << 4) | alives;
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
        for unknowns in 1..=8 {
            for alives in 0..=8 - unknowns {
                let nbhd = ((8 - alives - unknowns) << 4) | alives;
                let nbhd0 = ((8 - alives - unknowns + 1) << 4) | alives;
                let nbhd1 = ((8 - alives - unknowns) << 4) | (alives + 1);
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
    fn init_impl_table(trans_table: &[Implication; 256]) -> [Option<State>; 512] {
        let mut impl_table = [Default::default(); 512];

        for unknowns in 0..=8 {
            for alives in 0..=8 - unknowns {
                let nbhd = ((8 - alives - unknowns) << 4) | alives;
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
    fn init_impl_nbhd_table(trans_table: &[Implication; 256]) -> [Implication; 512] {
        let mut impl_nbhd_table: [Implication; 512] = [Default::default(); 512];

        for unknowns in 1..=8 {
            for alives in 0..=8 - unknowns {
                let nbhd = ((8 - alives - unknowns) << 4) | alives;
                let nbhd0 = ((8 - alives - unknowns + 1) << 4) | alives;
                let nbhd1 = ((8 - alives - unknowns) << 4) | (alives + 1);
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
                        impl_nbhd_table[index].dead = Some(Dead);
                    } else if !possibly_dead && possibly_alive {
                        impl_nbhd_table[index].dead = Some(Alive);
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
                        impl_nbhd_table[index].alive = Some(Dead);
                    } else if !possibly_dead && possibly_alive {
                        impl_nbhd_table[index].alive = Some(Alive);
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
                        impl_nbhd_table[index].none = Some(Dead);
                    } else if !possibly_dead && possibly_alive {
                        impl_nbhd_table[index].none = Some(Alive);
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
    ) -> Option<State> {
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

    pub fn parse_rule(input: &str) -> Result<Self, String> {
        ParseLife::parse_rule(input).map_err(|e| e.to_string())
    }
}

impl Rule for Life {
    type Desc = NbhdDesc;

    fn b0(&self) -> bool {
        self.b0
    }
    fn new_desc(state: Option<State>) -> Self::Desc {
        match state {
            Some(Dead) => NbhdDesc(0x80),
            Some(Alive) => NbhdDesc(0x08),
            None => NbhdDesc(0x00),
        }
    }

    fn update_desc(cell: &LifeCell<Self>, old_state: Option<State>, state: Option<State>) {
        let old_state_num = match old_state {
            Some(Dead) => 0x10,
            Some(Alive) => 0x01,
            None => 0x00,
        };
        let state_num = match state {
            Some(Dead) => 0x10,
            Some(Alive) => 0x01,
            None => 0x00,
        };
        for &neigh in cell.nbhd.iter() {
            let neigh = neigh.unwrap();
            let mut desc = neigh.desc.get();
            desc.0 -= old_state_num;
            desc.0 += state_num;
            neigh.desc.set(desc);
        }
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
        cell: &LifeCell<'a, Self>,
        world: &World<'a, Self>,
        desc: Self::Desc,
        state: Option<State>,
        succ_state: State,
        stack: &mut Vec<&'a LifeCell<'a, Self>>,
    ) {
        if let Some(state) = self.implication_nbhd(state, desc, succ_state) {
            for &neigh in cell.nbhd.iter() {
                if let Some(neigh) = neigh {
                    if neigh.state.get().is_none() {
                        world.set_cell(neigh, Some(state), false);
                        stack.push(neigh);
                    }
                }
            }
        }
    }
}

/// A parser for the rule.
impl ParseLife for Life {
    fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
        Self::new(b, s)
    }
}
