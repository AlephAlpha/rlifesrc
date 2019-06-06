use std::rc::Rc;
use crate::cell::{State, Desc, LifeCell, RcCell, WeakCell};
use crate::cell::State::{Dead, Alive};
use crate::world::Rule;

#[derive(Clone, Copy)]
// 邻域的细胞统计
// 由于死细胞和活细胞的数量都不超过 8，可以一起放到一个字节中
// 0x01 代表活，0x10 代表死
pub struct NbhdDesc(u8);

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        match state {
            Some(Dead) => NbhdDesc(0x80),
            Some(Alive) => NbhdDesc(0x08),
            None => NbhdDesc(0x00),
        }
    }

    fn set_nbhd(cell: &LifeCell<Self>, old_state: Option<State>, state: Option<State>) {
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
        for neigh in cell.nbhd.borrow().iter() {
            let neigh = neigh.upgrade().unwrap();
            let mut desc = neigh.desc.get();
            desc.0 -= old_state_num;
            desc.0 += state_num;
            neigh.desc.set(desc);
        }
    }
}

// 用一个结构体来放 transition 和 implication 的结果
#[derive(Clone, Copy, Default)]
struct Implication {
    dead: Option<State>,
    alive: Option<State>,
    none: Option<State>,
}

// 规则，其中不提供规则本身的数据，只保存 transition 和 implication 的结果
pub struct Life {
    b0: bool,
    trans_table: [Implication; 256],
    impl_table: [Option<State>; 512],
    impl_nbhd_table: [Implication; 512],
}

impl Life {
    pub fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
        let b0 = b.contains(&0);

        let mut trans_table: [Implication; 256] = [Default::default(); 256];

        // 先把 trans_table 中没有未知细胞的地方填上
        for alives in 0..9 {
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

        // 然后根据未知细胞的情况，一个一个来
        for unknowns in 1usize..9 {
            for alives in 0..9 - unknowns {
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

        let mut impl_table = [Default::default(); 512];
        for unknowns in 0..9 {
            for alives in 0..9 - unknowns {
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

        let mut impl_nbhd_table: [Implication; 512] = [Default::default(); 512];
        for unknowns in 1..9 {
            for alives in 0..9 - unknowns {
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
        Life {b0, trans_table, impl_table, impl_nbhd_table}
    }

    fn implication_nbhd(&self, state: Option<State>, desc: NbhdDesc, succ_state: State)
        -> Option<State> {
        let index = desc.0 as usize * 2 + match succ_state {
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
        let index = desc.0 as usize * 2 + match succ_state {
            Dead => 0,
            Alive => 1,
        };
        self.impl_table[index]
    }

    fn consistify_nbhd(&self, cell: &RcCell<NbhdDesc>, desc: NbhdDesc, state: Option<State>,
        succ_state: State, set_table: &mut Vec<WeakCell<NbhdDesc>>) {
        if let Some(state) = self.implication_nbhd(state, desc, succ_state) {
            for neigh in cell.nbhd.borrow().iter() {
                if let Some(neigh) = neigh.upgrade() {
                    if neigh.state.get().is_none() {
                        neigh.set(Some(state), false);
                        set_table.push(Rc::downgrade(&neigh));
                    }
                }
            }
        }
    }
}
