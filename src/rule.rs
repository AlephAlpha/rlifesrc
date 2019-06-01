use std::str::FromStr;
use crate::world::{State, Desc, LifeCell, Rule};

// 邻域的状态
#[derive(Clone, Copy)]
pub struct NbhdDesc {
    // 细胞本身的状态
    state: Option<State>,
    // 邻域的细胞统计
    // 由于死细胞和活细胞的数量都不超过 8，可以一起放到一个字节中
    // 0x01 代表活，0x10 代表死
    count: u8,
}

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        let count = match state {
            Some(State::Dead) => 0x80,
            Some(State::Alive) => 0x08,
            None => 0x00,
        };
        NbhdDesc {state, count}
    }

    fn state(&self) -> Option<State> {
        self.state
    }

    fn set_cell(cell: &LifeCell<NbhdDesc>, state: Option<State>, free: bool) {
        let old_state = cell.state();
        let mut desc = cell.desc.get();
        desc.state = state;
        cell.desc.set(desc);
        cell.free.set(free);
        for neigh in cell.nbhd.borrow().iter() {
            let neigh = neigh.upgrade().unwrap();
            let mut desc = neigh.desc.get();
            match old_state {
                Some(State::Dead) => desc.count -= 0x10,
                Some(State::Alive) => desc.count -= 0x01,
                None => (),
            };
            match state {
                Some(State::Dead) => desc.count += 0x10,
                Some(State::Alive) => desc.count += 0x01,
                None => (),
            };
            neigh.desc.set(desc);
        }
    }
}

// 用一个结构体来放 transition 和 implication 的结果
#[derive(Clone, Copy, Default)]
pub struct Implication {
    dead: Option<State>,
    alive: Option<State>,
    none: Option<State>,
}

// 规则，其中不提供规则本身的数据，只保存 transition 和 implication 的结果
pub struct LifeLike {
    b0: bool,
    trans_table: [Implication; 256],
    impl_table: [Option<State>; 512],
    impl_nbhd_table: [Implication; 512],
}

impl FromStr for LifeLike {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let err = Err(String::from("not a Life-like rule"));
        match chars.next() {
            Some('b') => (),
            Some('B') => (),
            _ => return err,
        }
        let b: Vec<_> = chars.clone().take_while(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap() as u8).collect();
        let mut chars = chars.skip_while(|c| c.is_ascii_digit());
        match chars.next() {
            Some('s') => (),
            Some('S') => (),
            Some('/') => {
                match chars.next() {
                    Some('s') => (),
                    Some('S') => (),
                    _ => return err,
                }
            },
            _ => return err,
        }
        let s: Vec<_> = chars.clone().take_while(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap() as u8).collect();
        let mut chars = chars.skip_while(|c| c.is_ascii_digit());
        if chars.next().is_some() || b.contains(&9) || s.contains(&9) {
            err
        } else {
            Ok(LifeLike::new(b, s))
        }
    }
}

impl LifeLike {
    fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
        let b0 = b.contains(&0);
        let (trans_table, impl_table, impl_nbhd_table) = Self::to_tables(b, s);
        LifeLike {b0, trans_table, impl_table, impl_nbhd_table}
    }

    // 在邻域没有未知细胞的情形下推导下一代的状态
    fn next_state(b: &Vec<u8>, s: &Vec<u8>, state: Option<State>, alives: u8) -> Option<State> {
        match state {
            Some(State::Dead) => {
                if b.contains(&alives) {
                    Some(State::Alive)
                } else {
                    Some(State::Dead)
                }
            },
            Some(State::Alive) => {
                if s.contains(&alives) {
                    Some(State::Alive)
                } else {
                    Some(State::Dead)
                }
            },
            None => {
                if b.contains(&alives) && s.contains(&alives) {
                    Some(State::Alive)
                } else if b.contains(&alives) || s.contains(&alives) {
                    None
                } else {
                    Some(State::Dead)
                }
            },
        }
    }

    // 由一个细胞及其邻域的状态得到其后一代的状态
    fn to_trans(b: &Vec<u8>, s: &Vec<u8>, state: Option<State>, alives: u8, deads: u8)
        -> Option<State> {
        let unknowns = 8 - alives - deads;
        let always_dead = (0..unknowns + 1).all(|i| {
            Self::next_state(b, s, state, alives + i) == Some(State::Dead)
        });
        let always_alive = (0..unknowns + 1).all(|i| {
            Self::next_state(b, s, state, alives + i) == Some(State::Alive)
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
    fn to_impl(b: &Vec<u8>, s: &Vec<u8>, alives: u8, deads: u8, succ_state: State)
        -> Option<State> {
        let possibly_dead = match Self::to_trans(b, s, Some(State::Dead), alives, deads) {
            Some(succ) => succ == succ_state,
            None => true,
        };
        let possibly_alive = match Self::to_trans(b, s, Some(State::Alive), alives, deads) {
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
    fn to_impl_nbhd(b: &Vec<u8>, s: &Vec<u8>, state: Option<State>, alives: u8, deads: u8,
        succ_state: State) -> Option<State> {
        let unknowns = 8 - alives - deads;
        let must_be_dead = (1..unknowns + 1).all(|i| {
            match Self::next_state(b, s, state, alives + i) {
                Some(succ) => succ != succ_state,
                None => false,
            }
        });
        let must_be_alive = (0..unknowns).all(|i| {
            match Self::next_state(b, s, state, alives + i) {
                Some(succ) => succ != succ_state,
                None => false,
            }
        });
        if must_be_dead && !must_be_alive {
            Some(State::Dead)
        } else if !must_be_dead && must_be_alive {
            Some(State::Alive)
        } else {
            None
        }
    }

    // 计算以上推导结果，保存在三个数组中
    fn to_tables(b: Vec<u8>, s: Vec<u8>)
        -> ([Implication; 256], [Option<State>; 512], [Implication; 512]) {
        let mut trans_table = [Default::default(); 256];
        let mut impl_table = [Default::default(); 512];
        let mut impl_nbhd_table = [Default::default(); 512];
        for alives in 0..9 {
            for deads in 0..9 - alives {
                let count = (alives * 0x01 + deads * 0x10) as usize;
                trans_table[count] = Implication {
                    dead: Self::to_trans(&b, &s, Some(State::Dead), alives, deads),
                    alive: Self::to_trans(&b, &s, Some(State::Alive), alives, deads),
                    none: Self::to_trans(&b, &s, None, alives, deads),
                };
                for (i, &succ_state) in [State::Dead, State::Alive].iter().enumerate() {
                    let index = count * 2 + i;
                    impl_table[index] = Self::to_impl(&b, &s, alives, deads, succ_state);
                    impl_nbhd_table[index] = Implication {
                        dead: Self::to_impl_nbhd(&b, &s, Some(State::Dead),
                            alives, deads, succ_state),
                        alive: Self::to_impl_nbhd(&b, &s, Some(State::Alive),
                            alives, deads, succ_state),
                        none: Self::to_impl_nbhd(&b, &s, None, alives, deads, succ_state),
                    };
                }
            }
        }
        (trans_table, impl_table, impl_nbhd_table)
    }
}

impl Rule<NbhdDesc> for LifeLike {
    fn b0(&self) -> bool {
        self.b0
    }

    fn transition(&self, desc: NbhdDesc) -> Option<State> {
        let transition = self.trans_table[desc.count as usize];
        match desc.state {
            Some(State::Dead) => transition.dead,
            Some(State::Alive) => transition.alive,
            None => transition.none,
        }
    }

    fn implication(&self, desc: NbhdDesc, succ_state: State) -> Option<State> {
        let index = desc.count as usize * 2 + match succ_state {
            State::Dead => 0,
            State::Alive => 1,
        };
        self.impl_table[index]
    }

    fn implication_nbhd(&self, desc: NbhdDesc, succ_state: State) -> Option<State> {
        let index = desc.count as usize * 2 + match succ_state {
            State::Dead => 0,
            State::Alive => 1,
        };
        let implication = self.impl_nbhd_table[index];
        match desc.state {
            Some(State::Dead) => implication.dead,
            Some(State::Alive) => implication.alive,
            None => implication.none,
        }
    }
}
