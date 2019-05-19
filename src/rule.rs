use crate::world::State;
use crate::world::Cell;
use crate::world::World;

// 先实现生命游戏，以后再修改以满足其它规则
pub struct Life {
    width: isize,
    height: isize,
    period: isize,
    dx: isize,  // 每周期平移的列数
    dy: isize,  // 每周期平移的行数
    symmetry: Symmetry,

    // 搜索范围内的所有细胞的列表
    cells: Vec<Cell>,

    // 一个共用的死细胞
    dead: Cell,

    // 以后再添加别的内容，比如说对称性
}

// 横座标，纵座标，时间
type Index = (isize, isize, isize);

// 邻域的状态，两个数加起来不能超过 8
// 有点想像 lifesrc 一样用一个字节来保持邻域状态，不过那样看起来太不直观
pub struct NbhdState {
    alives: u8,     // 活细胞的个数
    deads: u8,   // 未知细胞的个数
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

impl Life {
    pub fn new(width: isize, height: isize, period: isize, dx: isize, dy: isize,
               symmetry: Symmetry) -> Life {
        let size = width * height * period;
        let mut cells = Vec::with_capacity(size as usize);
        let dead = Cell {state: State::Dead, free: false};

        for _ in 0..size {
            cells.push(Cell {state: State::Unknown, free: true});
        }

        Life {width, height, period, dx, dy, symmetry, cells, dead}
    }

    // 细胞是否在范围之内
    #[inline]
    fn includes(&self, ix: Index) -> bool {
         return ix.0 >= 0 && ix.0 < self.width &&
                ix.1 >= 0 && ix.1 < self.height &&
                ix.2 >= 0 && ix.2 < self.period
    }

    #[inline]
    fn index(&self, ix: Index) -> usize {
        ((ix.0 * self.height + ix.1) * self.period + ix.2) as usize
    }

    #[inline]
    fn to_index(&self, i: usize) -> Index {
        let i = i as isize;
        let j = i / self.period;
        let k = j / self.height;
        (k % self.width, j % self.height, i % self.period)
    }
}

impl World<Index> for Life {
    type NbhdState = NbhdState;

    fn size(&self) -> usize {
        (self.width * self.height * self.period) as usize
    }

    fn get_cell(&self, ix: Index) -> Cell {
        if self.includes(ix) {
            self.cells[self.index(ix)]
        } else {
            self.dead
        }
    }

    fn set_cell(&mut self, ix: Index, cell: Cell) {
        let index = self.index(ix);
        if self.includes(ix) {
            self.cells[index] = cell;
        }
    }

    fn neighbors(&self, ix: Index) -> Vec<Index> {
        vec![(ix.0 - 1, ix.1 - 1, ix.2),
             (ix.0 - 1, ix.1, ix.2),
             (ix.0 - 1, ix.1 + 1, ix.2),
             (ix.0, ix.1 - 1, ix.2),
             (ix.0, ix.1 + 1, ix.2),
             (ix.0 + 1, ix.1 - 1, ix.2),
             (ix.0 + 1, ix.1, ix.2),
             (ix.0 + 1, ix.1 + 1, ix.2),]
    }

    fn nbhd_state(&self, neighbors: Vec<Index>) -> Self::NbhdState {
        let mut alives = 0;
        let mut unknowns = 0;
        for n in neighbors {
            match self.get_cell(n).state {
                State::Alive => alives += 1,
                State::Dead => (),
                State::Unknown => unknowns += 1,
            }
        }
        let deads = 8 - alives - unknowns;
        NbhdState {alives, deads}
    }

    fn pred(&self, ix: Index) -> Index {
        if ix.2 == 0 {
            (ix.0 - self.dx, ix.1 - self.dy, self.period - 1)
        } else {
            (ix.0, ix.1, ix.2 - 1)
        }
    }

    fn succ(&self, ix: Index) -> Index {
        if ix.2 == self.period - 1 {
            (ix.0 + self.dx, ix.1 + self.dy, 0)
        } else {
            (ix.0, ix.1, ix.2 + 1)
        }
    }

    fn sym(&self, ix: Index) -> Vec<Index> {
        match &self.symmetry {
            Symmetry::C1 => vec![],
            Symmetry::C2 => vec![(self.width - 1 - ix.0, self.height - 1 - ix.1, ix.2)],
            Symmetry::C4 => vec![(ix.1, self.width - 1 - ix.0, ix.2),
                                 (self.width - 1 - ix.0, self.height - 1 - ix.1, ix.2),
                                 (self.height - 1 - ix.1, ix.0, ix.2)],
            Symmetry::D2Row => vec![(self.width - 1 - ix.0, ix.1, ix.2)],
            Symmetry::D2Column => vec![(ix.0, self.height - 1 - ix.1, ix.2)],
            Symmetry::D2Diag => vec![(ix.1, ix.0, ix.2)],
            Symmetry::D2Antidiag => vec![(self.height - 1 - ix.1, self.width - 1 - ix.0, ix.2)],
            Symmetry::D4Ortho => vec![(self.width - 1 - ix.0, ix.1, ix.2),
                                      (ix.0, self.height - 1 - ix.1, ix.2),
                                      (self.width - 1 - ix.0, self.height - 1 - ix.1, ix.2)],
            Symmetry::D4Diag => vec![(ix.1, ix.0, ix.2),
                                     (self.height - 1 - ix.1, self.width - 1 - ix.0, ix.2),
                                     (self.width - 1 - ix.0, self.height - 1 - ix.1, ix.2)],
            Symmetry::D8 => vec![(ix.1, self.width - 1 - ix.0, ix.2),
                                 (self.height - 1 - ix.1, ix.0, ix.2),
                                 (self.width - 1 - ix.0, ix.1, ix.2),
                                 (ix.0, self.height - 1 - ix.1, ix.2),
                                 (ix.1, ix.0, ix.2),
                                 (self.height - 1 - ix.1, self.width - 1 - ix.0, ix.2),
                                 (self.width - 1 - ix.0, self.height - 1 - ix.1, ix.2)],
        }
    }

    // 搜索顺序不太好决定……先随便按顺序搜，以后慢慢调整
    fn get_unknown(&self) -> Option<Index> {
        self.cells.iter().position(|&cell| cell.state == State::Unknown)
            .map(|i| self.to_index(i))
    }

    // 仅适用于生命游戏
    // 这些条件是从 lifesrc 抄来的
    fn transit(cell:Cell, nbhd: &Self::NbhdState) -> State {
        let alives = nbhd.alives;
        let deads = nbhd.deads;
        match cell.state {
            State::Dead => if deads > 5 || alives > 3 {
                State::Dead
            } else if alives == 3 && deads == 5 {
                State::Alive
            } else {
                State::Unknown
            },
            State::Alive => if deads > 6 || alives > 3 {
                State::Dead
            } else if (alives == 2 && (deads == 5 || deads == 6)) ||
                      (alives == 3 && deads == 5) {
                State::Alive
            } else {
                State::Unknown
            },
            State::Unknown => if deads > 6 || alives > 3 {
                State::Dead
            } else if alives == 3 && deads == 5 {
                State::Alive
            } else {
                State::Unknown
            },
        }
    }

    // 从 lifesrc 抄来的
    fn implic(cell: Cell, nbhd: &Self::NbhdState, succ: Cell) -> (State, State) {
        let alives = nbhd.alives;
        let deads = nbhd.deads;
        match (cell.state, succ.state) {
            (State::Dead, State::Dead) => if alives == 2 && deads == 5 {
                (State::Unknown, State::Dead)
            } else {
                (State::Unknown, State::Unknown)
            },
            (State::Dead, State::Alive) => if deads == 5 {
                (State::Unknown, State::Alive)
            } else {
                (State::Unknown, State::Unknown)
            },
            (State::Alive, State::Dead) => if alives == 2 && deads == 4 {
                (State::Unknown, State::Alive)
            } else if alives == 1 && (deads == 5 || deads == 6) {
                (State::Unknown, State::Dead)
            } else {
                (State::Unknown, State::Unknown)
            },
            (State::Alive, State::Alive) => if deads == 6 {
                (State::Unknown, State::Alive)
            } else {
                (State::Unknown, State::Unknown)
            },
            (State::Unknown, State::Dead) => if alives == 2 && deads == 6 {
                (State::Dead, State::Unknown)
            } else if alives == 2 && deads == 5 {
                (State::Dead, State::Dead)
            } else {
                (State::Unknown, State::Unknown)
            },
            (State::Unknown, State::Alive) => if alives == 2 && deads == 6 {
                (State::Alive, State::Unknown)
            } else if deads == 6 {
                (State::Alive, State::Alive)
            } else {
                (State::Unknown, State::Unknown)
            },
            _ => (State::Unknown, State::Unknown),
        }
    }

    fn display(&self) {
        for t in 0..self.period {
            println!("Generation {}:", t);
            for y in 0..self.height {
                for x in 0..self.width {
                    let s = match self.get_cell((x, y, t)).state {
                        State::Dead => ".",
                        State::Alive => "o",
                        State::Unknown => "?",
                    };
                    print!("{}", s);
                }
                println!("");
            }
        }
    }
}

