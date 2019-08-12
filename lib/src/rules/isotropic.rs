use super::super::world::*;
use ca_rules::neighborhood;
use ca_rules::ParseBSRules;

#[derive(Clone, Copy, Default)]
// 邻域的八个细胞的状态
// 16位的二进制数，前8位中的 1 表示死细胞，后8位中的 1 表示活细胞
pub struct NbhdDesc(u16);

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        match state {
            Some(Dead) => NbhdDesc(0xff00),
            Some(Alive) => NbhdDesc(0x00ff),
            None => NbhdDesc(0x0000),
        }
    }

    fn set_nbhd<R: Rule<Desc = Self>>(
        world: &World<Self, R>,
        cell: &LifeCell<Self>,
        old_state: Option<State>,
        state: Option<State>,
    ) {
        let change_num = match (state, old_state) {
            (Some(Dead), Some(Alive)) => 0x0101,
            (Some(Alive), Some(Dead)) => 0x0101,
            (Some(Dead), None) | (None, Some(Dead)) => 0x0100,
            (Some(Alive), None) | (None, Some(Alive)) => 0x0001,
            _ => 0x0000,
        };
        for (i, &neigh_id) in cell.nbhd.get().iter().rev().enumerate() {
            let neigh = &world[neigh_id.unwrap()];
            let mut desc = neigh.desc.get();
            desc.0 ^= change_num << i;
            neigh.desc.set(desc);
        }
    }
}

// 用一个结构体来放 transition 和 implication 的结果
#[derive(Clone, Copy, Default)]
struct Implication<T> {
    dead: T,
    alive: T,
    none: T,
}

// Non-totalistic 的规则
// 不提供规则本身的数据，只保存 transition 和 implication 的结果
pub struct Life {
    b0: bool,
    trans_table: Box<[Implication<Option<State>>; 65536]>,
    impl_table: Box<[Option<State>; 65536 * 2]>,
    impl_nbhd_table: Box<[Implication<NbhdDesc>; 65536 * 2]>,
}

impl Life {
    pub fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
        let b0 = b.contains(&0);

        let trans_table = Self::init_trans_table(b, s);
        let impl_table = Self::init_impl_table(&trans_table);
        let impl_nbhd_table = Self::init_impl_trans_table(&trans_table);

        Life {
            b0,
            trans_table,
            impl_table,
            impl_nbhd_table,
        }
    }

    fn init_trans_table(b: Vec<u8>, s: Vec<u8>) -> Box<[Implication<Option<State>>; 65536]> {
        let mut trans_table: Box<[Implication<Option<State>>; 65536]> =
            Box::new([Default::default(); 65536]);

        // 先把 trans_table 中没有未知细胞的地方填上
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

        // 然后根据未知细胞的情况，一个一个来
        for unknowns in 1usize..=0xff {
            // n 是 unknowns 写成二进制时最高的一位
            // 于是处理 unknowns 时 unknowns - n 一定已经处理过了
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

    fn init_impl_trans_table(
        trans_table: &[Implication<Option<State>>; 65536],
    ) -> Box<[Implication<NbhdDesc>; 65536 * 2]> {
        let mut impl_nbhd_table: Box<[Implication<NbhdDesc>; 65536 * 2]> =
            Box::new([Default::default(); 65536 * 2]);

        for unknowns in 1usize..=0xff {
            // n 取遍 unknowns 写成二进制时所有非零的位
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

    fn consistify_nbhd(
        &self,
        cell: &LifeCell<NbhdDesc>,
        world: &World<NbhdDesc, Self>,
        desc: NbhdDesc,
        state: Option<State>,
        succ_state: State,
        set_table: &mut Vec<CellId>,
    ) {
        let nbhd_states = self.implication_nbhd(state, desc, succ_state).0;
        if nbhd_states != 0 {
            for (i, &neigh_id) in cell.nbhd.get().iter().enumerate() {
                let state = match nbhd_states >> i & 0x0101 {
                    0x0001 => Alive,
                    0x0100 => Dead,
                    _ => continue,
                };
                if let Some(neigh_id) = neigh_id {
                    let neigh = &world[neigh_id];
                    world.set_cell(neigh, Some(state), false);
                    set_table.push(neigh_id);
                }
            }
        }
    }
}

impl ParseBSRules for Life {
    type Neighborhood = neighborhood::Isotropic;

    fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
        Life::new(b, s)
    }
}
