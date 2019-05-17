use crate::search::State;
use crate::search::Cell;
use crate::search::World;

// 先实现生命游戏，以后再修改以满足其它规则
pub struct Life {
    width: isize,
    height: isize,
    period: isize,
    dx: isize,  // 每周期平移的列数
    dy: isize,  // 每周期平移的行数

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
    unknowns: u8,   // 未知细胞的个数
}

impl Life {
    pub fn new(width: isize, height: isize, period: isize, dx: isize, dy: isize) -> Life {
        let size = width * height * period;
        let mut cells = Vec::with_capacity(size as usize);
        let dead = Cell {state: State::Dead, free: false};

        for _ in 0..size {
            cells.push(Cell {state: State::Unknown, free: true});
        }

        Life {width, height, period, dx, dy, cells, dead}
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
}

impl World<Index> for Life {
    type NbhdState = NbhdState;

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
        let outside = ix.0 < -1 || ix.0 > self.width + 1 ||
                      ix.1 < -1 || ix.1 > self.height + 1 ||
                      ix.2 < 0 || ix.2 > self.period;
        if outside {
            Vec::new()
        } else {
            [(-1,-1), (-1,0), (-1,1), (0,-1), (0, 1), (1,-1), (1,0), (1,1)].iter()
                .map(|n| (ix.0 + n.0, ix.1 + n.1, ix.2)).collect()
        }
    }

    fn nbhd_state(&self, neighbors: Vec<Index>) -> Self::NbhdState {
        let mut alives = 0;
        let mut unknowns = 0;
        for n in neighbors {
            let cell = self.get_cell(n);
            match cell.state {
                State::Alive => alives += 1,
                State::Dead => (),
                State::Unknown => unknowns += 1
            }
        }
        NbhdState {alives, unknowns}
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

    // 搜索顺序不太好决定……先随便按顺序搜，以后慢慢调整
    fn first(&self) -> Index {
        (0, 0, 0)
    }
    fn next(&self, ix: Index) -> Option<Index> {
        if self.includes(ix) {
            if ix.2 == self.period - 1 {
                if ix.1 == self.height - 1 {
                    if ix.0 == self.width - 1 {
                        None
                    } else {
                        Some((ix.0 + 1, 0, 0))
                    }
                } else {
                    Some((ix.0, ix.1 + 1, 0))
                }
            } else {
                Some((ix.0, ix.1, ix.2 + 1))
            }
        } else {
            None
        }
    }

    // 仅适用于生命游戏
    // 这些条件是从 lifesrc 抄来的
    fn transit(cell:Cell, nbhd: Self::NbhdState) -> State {
        let alives = nbhd.alives;
        let deads = 8 - nbhd.alives - nbhd.unknowns;
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

    fn display(&self) {
        for t in 0..self.period {
            println!("Generation {}", t);
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

