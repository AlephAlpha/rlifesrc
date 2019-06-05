use std::rc::Rc;
use crate::world::{State, Desc, Rule, LifeCell, RcCell, WeakCell};

#[derive(Clone, Copy, Default)]
// 邻域的八个细胞的状态
// 写成二进制，前八位中的 1 表示死细胞，后八位中的 1 表示活细胞
pub struct NbhdDesc(u16);

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        match state {
            Some(State::Dead) => NbhdDesc(0xff00),
            Some(State::Alive) => NbhdDesc(0x00ff),
            None => NbhdDesc(0x0000),
        }
    }

    fn set_nbhd(cell: &LifeCell<Self>, _: Option<State>, state: Option<State>) {
        let state_num = match state {
            Some(State::Dead) => 0x0100,
            Some(State::Alive) => 0x0001,
            None => 0x0000,
        };
        for (i, neigh) in cell.nbhd.borrow().iter().rev().enumerate() {
            let neigh = neigh.upgrade().unwrap();
            let mut desc = neigh.desc.get();
            desc.0 &= !(0x101 << i);
            desc.0 |= state_num << i;
            neigh.desc.set(desc);
        }
    }
}

// 用一个结构体来放 transition 和 implication 的结果
#[derive(Clone, Copy, Default)]
pub struct Implication<T> {
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


        let mut trans_table: Box<[Implication<Option<State>>; 65536]> =
            Box::new([Default::default(); 65536]);
            // 先把 trans_table 中没有未知细胞的地方填上
        for alives in 0..256 {
            let nbhd = ((0xff & !alives) << 8) | alives;
            let alives = alives as u8;
            trans_table[nbhd].dead = if b.contains(&alives) {
                Some(State::Alive)
            } else {
                Some(State::Dead)
            };
            trans_table[nbhd].alive = if s.contains(&alives) {
                Some(State::Alive)
            } else {
                Some(State::Dead)
            };
            trans_table[nbhd].none = if b.contains(&alives) && s.contains(&alives) {
                Some(State::Alive)
            } else if !b.contains(&alives) && !s.contains(&alives) {
                Some(State::Dead)
            } else {
                None
            };
        }
        // 然后根据未知细胞的情况，一个一个来
        for unknowns in 1usize..256 {
            // n 是 unknowns 写成二进制时最高的一位
            // 于是处理 unknowns 时 unknowns - n 一定已经处理过了
            let n = unknowns.next_power_of_two() >> !unknowns.is_power_of_two() as usize;
            for alives in (0..256).filter(|a| a & unknowns == 0) {
                let nbhd = ((0xff & !alives & !unknowns) << 8) | alives;
                let nbhd0 = ((0xff & !alives & !unknowns | n) << 8) | alives;
                let nbhd1 = ((0xff & !alives & !unknowns & !n) << 8) | alives | n;
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

        // impl_table 按顺序来就行
        let mut impl_table = Box::new([Default::default(); 65536 * 2]);
        for unknowns in 0..256 {
            for alives in 0..256 {
                let nbhd = ((0xff & !alives & !unknowns) << 8) | alives;
                for (i, &succ) in [State::Dead, State::Alive].iter().enumerate() {
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
                        impl_table[index] = Some(State::Dead);
                    } else if !possibly_dead && possibly_alive {
                        impl_table[index] = Some(State::Alive);
                    }
                }
            }
        }

        // 接下来是最难的 impl_nbhd_table
        // 不确定有没有写漏什么东西
        let mut impl_nbhd_table: Box<[Implication<NbhdDesc>; 65536 * 2]> =
            Box::new([Default::default(); 65536 * 2]);
        for unknowns in 0usize..256 {
            // n 取遍 unknowns 写成二进制时所有非零的位
            for n in (0..8).map(|i| 1 << i).filter(|n| unknowns & n != 0) {
                for alives in 0..256 {
                    let nbhd = ((0xff & !alives & !unknowns) << 8) | alives;
                    let nbhd0 = ((0xff & !alives & !unknowns & !n) << 8) | alives | n;
                    let nbhd1 = ((0xff & !alives & !unknowns | n) << 8) | alives;
                    let trans0 = trans_table[nbhd0];
                    let trans1 = trans_table[nbhd1];
                    for (i, &succ) in [State::Dead, State::Alive].iter().enumerate() {
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
                            impl_nbhd_table[index].dead.0 |= n as u16;
                        } else if !possibly_dead && possibly_alive {
                            impl_nbhd_table[index].dead.0 |= (n << 8) as u16;
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
                            impl_nbhd_table[index].alive.0 |= n as u16;
                        } else if !possibly_dead && possibly_alive {
                            impl_nbhd_table[index].alive.0 |= (n << 8) as u16;
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
                            impl_nbhd_table[index].none.0 |= n as u16;
                        } else if !possibly_dead && possibly_alive {
                            impl_nbhd_table[index].none.0 |= (n << 8) as u16;
                        }
                    }
                }
            }
        }

        Life {b0, trans_table, impl_table, impl_nbhd_table}
    }

    fn transition(&self, state: Option<State>, desc: NbhdDesc) -> Option<State> {
        let transition = self.trans_table[desc.0 as usize];
        match state {
            Some(State::Dead) => transition.dead,
            Some(State::Alive) => transition.alive,
            None => transition.none,
        }
    }

    fn implication(&self, desc: NbhdDesc, succ_state: State) -> Option<State> {
        let index = desc.0 as usize * 2 + match succ_state {
            State::Dead => 0,
            State::Alive => 1,
        };
        self.impl_table[index]
    }

    fn implication_nbhd(&self, state: Option<State>, desc: NbhdDesc, succ_state: State)
        -> NbhdDesc {
        let index = desc.0 as usize * 2 + match succ_state {
            State::Dead => 0,
            State::Alive => 1,
        };
        let implication = self.impl_nbhd_table[index];
        match state {
            Some(State::Dead) => implication.dead,
            Some(State::Alive) => implication.alive,
            None => implication.none,
        }
    }
}

impl Rule<NbhdDesc> for Life {
    fn b0(&self) -> bool {
        self.b0
    }

    fn consistify(&self, cell: &RcCell<NbhdDesc>,
        set_table: &mut Vec<WeakCell<NbhdDesc>>) -> Result<(), ()> {
        let pred = cell.pred.borrow().upgrade().unwrap();
        let pred_state = pred.state.get();
        let desc = pred.desc.get();
        if let Some(state) = self.transition(pred_state, desc) {
            if let Some(old_state) = cell.state.get() {
                if state != old_state {
                    return Err(())
                }
            } else {
                cell.set(Some(state), false);
                set_table.push(Rc::downgrade(cell));
            }
        }
        if let Some(state) = cell.state.get() {
            if pred.state.get().is_none() {
                if let Some(state) = self.implication(desc, state) {
                    pred.set(Some(state), false);
                    set_table.push(Rc::downgrade(&pred));
                }
            }
            let nbhd_states = self.implication_nbhd(pred_state, desc, state).0;
            if nbhd_states != 0 {
                for (i, neigh) in pred.nbhd.borrow().iter().enumerate() {
                    let state = match nbhd_states >> i & 0x101 {
                        1 => State::Alive,
                        0x101 => State::Dead,
                        _ => continue,
                    };
                    if let Some(neigh) = neigh.upgrade() {
                        if neigh.state.get().is_none() {
                            neigh.set(Some(state), false);
                            set_table.push(Rc::downgrade(&neigh));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
