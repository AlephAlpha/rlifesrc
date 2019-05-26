// 没想到这个文件写着写着变得这么长，不知道要不要拆成几个文件

use std::str::FromStr;
use std::rc::{Rc, Weak};
use crate::world::{State, Desc, LifeCell, World};
use crate::world::State::{Dead, Alive};

// 横座标，纵座标，时间
type Coord = (isize, isize, isize);

// 邻域的状态
#[derive(Clone, Copy)]
pub struct NbhdDesc {
    // 细胞本身的状态
    state: Option<State>,
    // 邻域的细胞统计，0x01 代表活，0x10 代表死
    count: u8,
}

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        let count = match state {
            Some(Dead) => 0x80,
            Some(Alive) => 0x08,
            None => 0x00,
        };
        NbhdDesc {state, count}
    }

    fn state(&self) -> Option<State> {
        self.state
    }
}

// 对称性
pub enum Symmetry {
    C1,
    C2,
    C4,
    D2Row,
    D2Column,
    D2Diag,
    D2Antidiag,
    D4Ortho,
    D4Diag,
    D8,
}

impl FromStr for Symmetry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C1" => Ok(Symmetry::C1),
            "C2" => Ok(Symmetry::C2),
            "C4" => Ok(Symmetry::C4),
            "D2|" => Ok(Symmetry::D2Row),
            "D2-" => Ok(Symmetry::D2Column),
            "D2\\" => Ok(Symmetry::D2Diag),
            "D2/" => Ok(Symmetry::D2Antidiag),
            "D4+" => Ok(Symmetry::D4Ortho),
            "D4X" => Ok(Symmetry::D4Diag),
            "D8" => Ok(Symmetry::D8),
            _ => Err(String::from("Invalid symmetry")),
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

// 规则，比如说 B3/S23 表示为 Rule {birth: vec![3], survive: vec![2, 3]}
// 不支持 B0 的规则
pub struct Rule {
    birth: Vec<u8>,
    survive: Vec<u8>,
}

impl FromStr for Rule {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match chars.next() {
            Some('b') => (),
            Some('B') => (),
            _ => return Err(String::from("Invalid rule")),
        }
        let birth: Vec<u8> = chars.clone().take_while(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap() as u8).collect();
        let mut chars = chars.skip_while(|c| c.is_ascii_digit());
        match chars.next() {
            Some('s') => (),
            Some('S') => (),
            Some('/') => {
                match chars.next() {
                    Some('s') => (),
                    Some('S') => (),
                    _ => return Err(String::from("Invalid rule")),
                }
            },
            _ => return Err(String::from("Invalid rule")),
        }
        let survive: Vec<u8> = chars.clone().take_while(|c| c.is_ascii_digit())
            .map(|c| c.to_digit(10).unwrap() as u8).collect();
        let mut chars = chars.skip_while(|c| c.is_ascii_digit());
        if chars.next().is_some() || birth.contains(&9) || survive.contains(&9) {
            Err(String::from("Invalid rule"))
        } else {
            Ok(Rule {birth, survive})
        }
    }
}

impl Rule {
    // 在邻域没有未知细胞的情形下推导下一代的状态
    fn next_state(&self, state: Option<State>, alives: u8) -> Option<State> {
        match state {
            Some(Dead) => {
                if self.birth.contains(&alives) {
                    Some(Alive)
                } else {
                    Some(Dead)
                }
            },
            Some(Alive) => {
                if self.survive.contains(&alives) {
                    Some(Alive)
                } else {
                    Some(Dead)
                }
            },
            None => {
                if self.birth.contains(&alives) && self.survive.contains(&alives) {
                    Some(Alive)
                } else if self.birth.contains(&alives) || self.survive.contains(&alives) {
                    None
                } else {
                    Some(Dead)
                }
            },
        }
    }

    // 计算 transition 的结果
    fn to_trans(&self, state: Option<State>, alives: u8, deads: u8) -> Option<State> {
        let unknowns = 8 - alives - deads;
        let always_dead = (0..unknowns + 1).all(|i| {
            self.next_state(state, alives + i) == Some(Dead)
        });
        let always_alive = (0..unknowns + 1).all(|i| {
            self.next_state(state, alives + i) == Some(Alive)
        });
        if always_alive {
            Some(Alive)
        } else if always_dead {
            Some(Dead)
        } else {
            None
        }
    }

    // 计算 implication 的结果
    fn to_impl(&self, alives: u8, deads: u8, succ_state: State) -> Option<State> {
        let possibly_dead = match self.to_trans(Some(Dead), alives, deads) {
            Some(succ) => succ == succ_state,
            None => true,
        };
        let possibly_alive = match self.to_trans(Some(Alive), alives, deads) {
            Some(succ) => succ == succ_state,
            None => true,
        };
        if possibly_dead && !possibly_alive {
            Some(Dead)
        } else if !possibly_dead && possibly_alive {
            Some(Alive)
        } else {
            None
        }
    }

    // 计算 implication_nbhd 的结果
    fn to_impl_nbhd(&self, state: Option<State>, alives: u8, deads: u8, succ_state: State)
        -> Option<State> {
        let unknowns = 8 - alives - deads;
        let must_be_dead = (1..unknowns + 1).all(|i| {
            match self.next_state(state, alives + i) {
                Some(succ) => succ != succ_state,
                None => false,
            }
        });
        let must_be_alive = (0..unknowns).all(|i| {
            match self.next_state(state, alives + i) {
                Some(succ) => succ != succ_state,
                None => false,
            }
        });
        if must_be_dead && !must_be_alive {
            Some(Dead)
        } else if !must_be_dead && must_be_alive {
            Some(Alive)
        } else {
            None
        }
    }

    // 计算这个规则的 transition 和 implication，保存在三个数组中
    fn implication_tables(&self)
        -> ([Implication; 256], [Option<State>; 512], [Implication; 512]) {
        let mut trans_table = [Default::default(); 256];
        let mut impl_table = [Default::default(); 512];
        let mut impl_nbhd_table = [Default::default(); 512];
        for alives in 0..9 {
            for deads in 0..9 - alives {
                let count = (alives * 0x01 + deads * 0x10) as usize;
                trans_table[count] = Implication {
                    dead: self.to_trans(Some(Dead), alives, deads),
                    alive: self.to_trans(Some(Alive), alives, deads),
                    none: self.to_trans(None, alives, deads),
                };
                for (i, &succ_state) in [Dead, Alive].iter().enumerate() {
                    let index = count * 2 + i;
                    impl_table[index] = self.to_impl(alives, deads, succ_state);
                    impl_nbhd_table[index] = Implication {
                        dead: self.to_impl_nbhd(Some(Dead), alives, deads, succ_state),
                        alive: self.to_impl_nbhd(Some(Alive), alives, deads, succ_state),
                        none: self.to_impl_nbhd(None, alives, deads, succ_state),
                    };
                }
            }
        }
        (trans_table, impl_table, impl_nbhd_table)
    }
}

// 生命游戏
pub struct Life {
    width: isize,
    height: isize,
    period: isize,

    // 搜索顺序是先行后列还是先列后行
    // 通过比较行数和列数的大小来自动决定
    column_first: bool,

    // 搜索范围内的所有细胞的列表
    cells: Vec<Rc<LifeCell<NbhdDesc>>>,

    // 保存 transition 和 implication 的结果
    trans_table: [Implication; 256],
    impl_table: [Option<State>; 512],
    impl_nbhd_table: [Implication; 512],
}

impl Life {
    pub fn new(width: isize, height: isize, period: isize,
        dx: isize, dy: isize, symmetry: Symmetry, rule: Rule) -> Self {
        let neighbors = [(-1,-1), (-1,0), (-1,1), (0,-1), (0,1), (1,-1), (1,0), (1,1)];
        let column_first = {
            let (width, height) = match symmetry {
                Symmetry::D2Row => ((width + 1) / 2, height),
                Symmetry::D2Column => (width, (height + 1) / 2),
                _ => (width, height),
            };
            if width == height {
                dx >= dy
            } else {
                width > height
            }
        };

        let b0 = rule.birth.contains(&0);

        let mut cells = Vec::with_capacity(((width + 2) * (height + 2) * period) as usize);

        // 先全部填上死细胞；如果时 B0 的规则，则在奇数代填上活细胞
        for _ in 0..(width + 2) * (height + 2) {
            for t in 0..period {
                let state = if b0 && t % 2 == 1 {
                    Alive
                } else {
                    Dead
                };
                cells.push(Rc::new(LifeCell::new(Some(state), false)));
            }
        }

        let (trans_table, impl_table, impl_nbhd_table) = rule.implication_tables();

        let life = Life {width, height, period, column_first, cells,
            trans_table, impl_table, impl_nbhd_table};

        // 先设定细胞的邻域
        for x in -1..width + 1 {
            for y in -1..height + 1 {
                for t in 0..period {
                    let cell = life.find_cell((x, y, t)).upgrade().unwrap();
                    for (nx, ny) in neighbors.iter() {
                        let neigh_weak = life.find_cell((x + nx, y + ny, t));
                        if neigh_weak.upgrade().is_some() {
                            cell.nbhd.borrow_mut().push(neigh_weak);
                        }
                    }
                }
            }
        }

        // 再给范围内的细胞添加别的信息
        for x in -1..width + 1 {
            for y in -1..height + 1 {
                for t in 0..period {
                    let cell = life.find_cell((x, y, t)).upgrade().unwrap();

                    // 默认的细胞状态
                    let default = if b0 && t % 2 == 1 {
                        Alive
                    } else {
                        Dead
                    };

                    // 用 set_cell 设置细胞状态
                    if 0 <= x && x < width && 0 <= y && y < height {
                        Life::set_cell(&cell, None, true);
                    }

                    // 设定前一代；若前一代不在范围内则把此细胞设为 default
                    if t != 0 {
                        *cell.pred.borrow_mut() = life.find_cell((x, y, t - 1));
                    } else {
                        let pred_coord = (x - dx, y - dy, period - 1);
                        let pred_weak = life.find_cell(pred_coord);
                        if pred_weak.upgrade().is_some() {
                            *cell.pred.borrow_mut() = pred_weak;
                        } else {
                            Life::set_cell(&cell, Some(default), false);
                        }
                    }

                    // 设定后一代；若后一代不在范围内则把此细胞设为 default
                    if t != period - 1 {
                        *cell.succ.borrow_mut() = life.find_cell((x, y, t + 1));
                    } else {
                        let succ_coord = (x + dx, y + dy, 0);
                        let succ_weak = life.find_cell(succ_coord);
                        if succ_weak.upgrade().is_some() {
                            *cell.succ.borrow_mut() = succ_weak;
                        } else {
                            Life::set_cell(&cell, Some(default), false);
                        }
                    }

                    // 设定对称的细胞；若对称的细胞不在范围内则把此细胞设为 default
                    let sym_coord = match symmetry {
                        Symmetry::C1 => vec![],
                        Symmetry::C2 => vec![(width - 1 - x, height - 1 - y, t)],
                        Symmetry::C4 => vec![(y, width - 1 - x, t),
                            (width - 1 - x, height - 1 - y, t),
                            (height - 1 - y, x, t)],
                        Symmetry::D2Row => vec![(width - 1 - x, y, t)],
                        Symmetry::D2Column => vec![(x, height - 1 - y, t)],
                        Symmetry::D2Diag => vec![(y, x, t)],
                        Symmetry::D2Antidiag => vec![(height - 1 - y, width - 1 - x, t)],
                        Symmetry::D4Ortho => vec![(width - 1 - x, y, t),
                            (x, height - 1 - y, t),
                            (width - 1 - x, height - 1 - y, t)],
                        Symmetry::D4Diag => vec![(y, x, t),
                            (height - 1 - y, width - 1 - x, t),
                            (width - 1 - x, height - 1 - y, t)],
                        Symmetry::D8 => vec![(y, width - 1 - x, t),
                            (height - 1 - y, x, t),
                            (width - 1 - x, y, t),
                            (x, height - 1 - y, t),
                            (y, x, t),
                            (height - 1 - y, width - 1 - x, t),
                            (width - 1 - x, height - 1 - y, t)],
                    };
                    for coord in sym_coord {
                        let sym_weak = life.find_cell(coord);
                        if sym_weak.upgrade().is_some() {
                            cell.sym.borrow_mut().push(sym_weak);
                        } else {
                            Life::set_cell(&cell, Some(default), false);
                        }
                    }
                }
            }
        }

        life
    }

    // 通过坐标查找细胞
    fn find_cell(&self, coord: Coord) -> Weak<LifeCell<NbhdDesc>> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = if self.column_first {
                ((x + 1) * (self.height + 2) + y + 1) * self.period + t
            } else {
                ((y + 1) * (self.width + 2) + x + 1) * self.period + t
            };
            Rc::downgrade(&self.cells[index as usize])
        } else {
            Weak::new()
        }
    }

    // 通过坐标给出细胞的状态；范围外的细胞状态默认为死
    fn get_state(&self, coord: Coord) -> Option<State> {
        match self.find_cell(coord).upgrade() {
            Some(cell) => cell.state(),
            None => Some(Dead),
        }
    }

    // 输出图样
    pub fn display(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let s = match self.get_state((x, y, 0)) {
                    Some(Dead) => ".",
                    Some(Alive) => "O",
                    None => "?",
                };
                print!("{}", s);
            }
            println!("");
        }
    }
}

impl World<NbhdDesc> for Life {
    fn size(&self) -> usize {
        (self.width * self.height * self.period) as usize
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
                Some(Dead) => desc.count -= 0x10,
                Some(Alive) => desc.count -= 0x01,
                None => (),
            };
            match state {
                Some(Dead) => desc.count += 0x10,
                Some(Alive) => desc.count += 0x01,
                None => (),
            };
            neigh.desc.set(desc);
        }
    }

    fn get_unknown(&self) -> Weak<LifeCell<NbhdDesc>> {
        self.cells.iter().find(|cell| cell.state().is_none())
            .map(Rc::downgrade).unwrap_or_default()
    }

    fn transition(&self, desc: NbhdDesc) -> Option<State> {
        let transition = self.trans_table[desc.count as usize];
        match desc.state {
            Some(Dead) => transition.dead,
            Some(Alive) => transition.alive,
            None => transition.none,
        }
    }

    fn implication(&self, desc: NbhdDesc, succ_state: State) -> Option<State> {
        let index = desc.count as usize * 2 + match succ_state {
            Dead => 0,
            Alive => 1,
        };
        self.impl_table[index]
    }

    fn implication_nbhd(&self, desc: NbhdDesc, succ_state: State) -> Option<State> {
        let index = desc.count as usize * 2 + match succ_state {
            Dead => 0,
            Alive => 1,
        };
        let implication = self.impl_nbhd_table[index];
        match desc.state {
            Some(Dead) => implication.dead,
            Some(Alive) => implication.alive,
            None => implication.none,
        }
    }

    fn subperiod(&self) -> bool {
        let nonzero = (0..self.height).any(|y|
            (0..self.width).any(|x|
                self.get_state((x, y, 0)) != Some(Dead)));
        nonzero && (self.period == 1 ||
            (1..self.period).all(|t|
                self.period % t != 0 || (0..self.height).any(|y|
                    (0..self.width).any(|x|
                        self.get_state((x, y, 0)) != self.get_state((x, y, t))))))
    }
}
