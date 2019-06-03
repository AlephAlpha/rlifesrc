use std::rc::Rc;
use std::str::FromStr;
use crate::world::{State, Desc, Rule, LifeCell, RcCell, WeakCell};

#[derive(Clone, Copy, Default)]
// 邻域的八个细胞的状态
pub struct NbhdDesc(u16);

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        match state {
            Some(State::Dead) => NbhdDesc(0xff00),
            Some(State::Alive) => NbhdDesc(0xff),
            None => NbhdDesc(0),
        }
    }

    fn set_nbhd(cell: &LifeCell<Self>, _: Option<State>, state: Option<State>) {
        let state_num = match state {
            Some(State::Dead) => 0x100,
            Some(State::Alive) => 1,
            None => 0,
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
pub struct NTLife {
    b0: bool,
    trans_table: Box<[Implication<Option<State>>; 65536]>,
    impl_table: Box<[Option<State>; 65536 * 2]>,
    impl_nbhd_table: Box<[Implication<NbhdDesc>; 65536 * 2]>,
}

impl FromStr for NTLife {
    type Err = String;

    // 太复杂，先总是返回一个值，以供测试
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        // let mut chars = s.chars();
        // let err = Err(String::from("not a Life-like rule"));
        // match chars.next() {
        //     Some('b') => (),
        //     Some('B') => (),
        //     _ => return err,
        // }
        // let b: Vec<_> = chars.clone().take_while(|c| c.is_ascii_digit())
        //     .map(|c| c.to_digit(10).unwrap() as u8).collect();
        // let mut chars = chars.skip_while(|c| c.is_ascii_digit());
        // match chars.next() {
        //     Some('s') => (),
        //     Some('S') => (),
        //     Some('/') => {
        //         match chars.next() {
        //             Some('s') => (),
        //             Some('S') => (),
        //             _ => return err,
        //         }
        //     },
        //     _ => return err,
        // }
        // let s: Vec<_> = chars.clone().take_while(|c| c.is_ascii_digit())
        //     .map(|c| c.to_digit(10).unwrap() as u8).collect();
        // let mut chars = chars.skip_while(|c| c.is_ascii_digit());
        // if chars.next().is_some() || b.contains(&9) || s.contains(&9) {
        //     err
        // } else {
        //     Ok(Life::new(b, s))
        // }
        let b = vec![7, 11, 13, 14, 19, 21, 22, 25, 26, 28, 35, 37, 38, 41, 42, 44, 49,
            50, 52, 56, 67, 69, 70, 73, 74, 76, 81, 82, 84, 88, 97, 98, 100, 104,
            112, 131, 133, 134, 137, 138, 140, 145, 146, 148, 152, 161, 162, 164,
            168, 176, 193, 194, 196, 200, 208, 224];
        let s = vec![3, 5, 6, 7, 9, 10, 11, 12, 13, 14, 17, 18, 19, 20, 21, 22, 24, 25,
            26, 28, 33, 34, 35, 36, 37, 38, 40, 41, 42, 44, 48, 49, 50, 52, 56,
            65, 66, 67, 68, 69, 70, 72, 73, 74, 76, 80, 81, 82, 84, 88, 96, 97,
            98, 100, 104, 112, 129, 130, 131, 132, 133, 134, 136, 137, 138, 140,
            144, 145, 146, 148, 152, 160, 161, 162, 164, 168, 176, 192, 193, 194,
            196, 200, 208, 224];
        Ok(NTLife::new(b, s))
    }
}

impl NTLife {
    fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
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

        NTLife {b0, trans_table, impl_table, impl_nbhd_table}
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

impl Rule<NbhdDesc> for NTLife {
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
