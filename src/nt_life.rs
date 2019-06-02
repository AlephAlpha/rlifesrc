use std::rc::Rc;
use std::str::FromStr;
use crate::world::{State, Desc, Rule, LifeCell, RcCell, WeakCell};

#[derive(Clone, Copy, Default, PartialEq, Debug)]
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
        let (trans_table, impl_table, impl_nbhd_table) = Self::to_tables(b, s);
        NTLife {b0, trans_table, impl_table, impl_nbhd_table}
    }

    // 在邻域没有未知细胞的情形下推导下一代的状态
    // 这里 nbhd 是由八个邻域状态组成的二进制数
    fn next_state(b: &Vec<u8>, s: &Vec<u8>, state: Option<State>, nbhd: u8) -> Option<State> {
        match state {
            Some(State::Dead) => {
                if b.contains(&nbhd) {
                    Some(State::Alive)
                } else {
                    Some(State::Dead)
                }
            },
            Some(State::Alive) => {
                if s.contains(&nbhd) {
                    Some(State::Alive)
                } else {
                    Some(State::Dead)
                }
            },
            None => {
                if b.contains(&nbhd) && s.contains(&nbhd) {
                    Some(State::Alive)
                } else if b.contains(&nbhd) || s.contains(&nbhd) {
                    None
                } else {
                    Some(State::Dead)
                }
            },
        }
    }

    // 由一个细胞及其邻域的状态得到其后一代的状态
    // 这里 nbhd 是由八个邻域状态通过前面的 to_num 函数得到的结果
    fn to_trans(b: &Vec<u8>, s: &Vec<u8>, state: Option<State>, nbhd: usize)
        -> Option<State> {
        let knowns = (nbhd >> 8 | nbhd) & 0xff;
        let all_nbhds = (0..256).filter_map(|n| {
            if n & knowns == 0 {
                Some((n | nbhd) as u8)
            } else {
                None
            }
        });
        let always_dead = all_nbhds.clone().all(|n| {
            Self::next_state(b, s, state, n) == Some(State::Dead)
        });
        let always_alive = all_nbhds.clone().all(|n| {
            Self::next_state(b, s, state, n) == Some(State::Alive)
        });
        if always_alive {
            Some(State::Alive)
        } else if always_dead {
            Some(State::Dead)
        } else {
            None
        }
    }

    // 由一个细胞的邻域及其后一代的状态，决定其本身的状态
    fn to_impl(b: &Vec<u8>, s: &Vec<u8>, nbhd: usize, succ_state: State)
        -> Option<State> {
        let possibly_dead = match Self::to_trans(b, s, Some(State::Dead), nbhd) {
            Some(succ) => succ == succ_state,
            None => true,
        };
        let possibly_alive = match Self::to_trans(b, s, Some(State::Alive), nbhd) {
            Some(succ) => succ == succ_state,
            None => true,
        };
        if possibly_dead && !possibly_alive {
            Some(State::Dead)
        } else if !possibly_dead && possibly_alive {
            Some(State::Alive)
        } else {
            None
        }
    }

    // 由一个细胞本身、邻域以及其后一代的状态，决定其域中未知细胞的状态
    // 对于 non-totalistic 的规则，这个很难办……以后再写
    fn to_impl_nbhd(_b: &Vec<u8>, _s: &Vec<u8>, _state: Option<State>, _nbhd: usize,
        _succ_state: State) -> NbhdDesc {
        NbhdDesc::new(None)
    }

    // 计算以上推导结果，保存在三个数组中
    fn to_tables(b: Vec<u8>, s: Vec<u8>)
        -> (Box<[Implication<Option<State>>; 65536]>,
            Box<[Option<State>; 65536 * 2]>,
            Box<[Implication<NbhdDesc>; 65536 * 2]>) {
        let mut trans_table = Box::new([Default::default(); 65536]);
        let mut impl_table = Box::new([Default::default(); 65536 * 2]);
        let mut impl_nbhd_table = Box::new([Default::default(); 65536 * 2]);
        for nbhd in 0..65536 {
            if nbhd >> 8 & nbhd != 0 {
                continue;
            }
            trans_table[nbhd] = Implication {
                dead: Self::to_trans(&b, &s, Some(State::Dead), nbhd),
                alive: Self::to_trans(&b, &s, Some(State::Alive), nbhd),
                none: Self::to_trans(&b, &s, None, nbhd),
            };
            for (i, &succ_state) in [State::Dead, State::Alive].iter().enumerate() {
                let index = nbhd * 2 + i;
                impl_table[index] = Self::to_impl(&b, &s, nbhd, succ_state);
                impl_nbhd_table[index] = Implication {
                    dead: Self::to_impl_nbhd(&b, &s, Some(State::Dead), nbhd, succ_state),
                    alive: Self::to_impl_nbhd(&b, &s, Some(State::Alive), nbhd, succ_state),
                    none: Self::to_impl_nbhd(&b, &s, None, nbhd, succ_state),
                };
            }
        }
        (trans_table, impl_table, impl_nbhd_table)
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
