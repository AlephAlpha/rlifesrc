use crate::world::{State, Cell, World};
use crate::world::State::{Dead, Alive};

// 先实现生命游戏，以后再修改以满足其它规则
pub struct Life {
    width: isize,
    height: isize,
    period: isize,
    dx: isize,
    dy: isize,
    symmetry: Symmetry,

    // 搜索顺序是先行后列还是先列后行
    // 通过比较行数和列数的大小来自动决定
    col_first: bool,

    // 搜索范围内的所有细胞的列表
    cells: Vec<Cell<NbhdDesc>>,
}

// 横座标，纵座标，时间
type Index = (isize, isize, isize);

// 邻域的状态
#[derive(Clone, Copy)]
pub struct NbhdDesc {
    state: Option<State>,
    alives: u8,
    deads: u8,
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
        let size = (width + 2) * (height + 2) * period;
        let cells = Vec::with_capacity(size as usize);
        let col_first = {
            let (width, height) = match symmetry {
                Symmetry::D2Row => (width / 2, height),
                Symmetry::D2Column => (width, height / 2),
                _ => (width, height),
            };
            width > height
        };
        let mut life = Life {width, height, period, dx, dy, symmetry, col_first, cells};
        for _ in 0..size {
            let state = Some(Dead);
            let desc = NbhdDesc {state, alives: 0, deads: 8};
            life.cells.push(Cell {free: false, desc});
        }
        for x in 0..width {
            for y in 0..height {
                for t in 0..period {
                    life.set_cell((x, y, t), None, true);
                }
            }
        }
        life
    }

    fn inside(&self, ix: Index) -> bool {
        ix.0 >= 0 && ix.0 < self.width && ix.1 >= 0 && ix.1 < self.height
    }

    fn index(&self, ix: Index) -> usize {
        let index = if self.col_first {
            ((ix.0 + 1) * (self.height + 2) + ix.1 + 1) * self.period + ix.2
        } else {
            ((ix.1 + 1) * (self.width + 2) + ix.0 + 1) * self.period + ix.2
        };
        index as usize
    }

    fn to_index(&self, i: usize) -> Index {
        let i = i as isize;
        let (j, k);
        if self.col_first {
            j = i / self.period;
            k = j / (self.height + 2);
        } else {
            k = i / self.period;
            j = k / (self.width + 2);
        }
        (k % (self.width + 2) - 1, j % (self.height + 2) - 1, i % self.period)
    }
}

impl World<Index> for Life {
    type NbhdDesc = NbhdDesc;

    fn size(&self) -> usize {
        (self.width * self.height * self.period) as usize
    }

    fn get_state(&self, ix: Index) -> Option<State> {
        if self.inside(ix) {
            self.cells[self.index(ix)].desc.state
        } else {
            Some(Dead)
        }
    }

    fn get_free(&self, ix: Index) -> bool {
        self.cells[self.index(ix)].free
    }

    fn set_cell(&mut self, ix: Index, state: Option<State>, free: bool) {
        let index = self.index(ix);
        let old_state = self.cells[index].desc.state;
        self.cells[index].desc.state = state;
        self.cells[index].free = free;
        for &n in self.neighbors(ix).iter() {
            let index = self.index(n);
            match old_state {
                Some(Dead) => self.cells[index].desc.deads -= 1,
                Some(Alive) => self.cells[index].desc.alives -= 1,
                None => (),
            };
            match state {
                Some(Dead) => self.cells[index].desc.deads += 1,
                Some(Alive) => self.cells[index].desc.alives += 1,
                None => (),
            };
        }
    }

    fn neighbors(&self, ix: Index) -> [Index; 8] {
        [(ix.0 - 1, ix.1 - 1, ix.2),
            (ix.0 - 1, ix.1, ix.2),
            (ix.0 - 1, ix.1 + 1, ix.2),
            (ix.0, ix.1 - 1, ix.2),
            (ix.0, ix.1 + 1, ix.2),
            (ix.0 + 1, ix.1 - 1, ix.2),
            (ix.0 + 1, ix.1, ix.2),
            (ix.0 + 1, ix.1 + 1, ix.2)]
    }

    fn get_desc(&self, ix: Index) -> Self::NbhdDesc {
        if ix.0 >= -1 && ix.0 <= self.width && ix.1 >= -1 && ix.1 <= self.height {
            self.cells[self.index(ix)].desc
        } else {
            NbhdDesc {state: Some(Dead), alives: 0, deads: 8}
        }
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

    fn get_unknown(&self) -> Option<Index> {
        self.cells.iter().position(|cell| cell.desc.state.is_none())
            .map(|i| self.to_index(i))
    }

    // 仅适用于生命游戏
    // 这些条件是从 lifesrc 抄来的
    fn transition(nbhd: &Self::NbhdDesc) -> Option<State> {
        let state = nbhd.state;
        let alives = nbhd.alives;
        let deads = nbhd.deads;
        match state {
            Some(Dead) => if deads > 5 || alives > 3 {
                Some(Dead)
            } else if alives == 3 && deads == 5 {
                Some(Alive)
            } else {
                None
            },
            Some(Alive) => if deads > 6 || alives > 3 {
                Some(Dead)
            } else if (alives == 2 && (deads == 5 || deads == 6)) ||
                      (alives == 3 && deads == 5) {
                Some(Alive)
            } else {
                None
            },
            None => if deads > 6 || alives > 3 {
                Some(Dead)
            } else if alives == 3 && deads == 5 {
                Some(Alive)
            } else {
                None
            },
        }
    }

    // 从 lifesrc 抄来的
    fn implication(nbhd: &Self::NbhdDesc, succ_state: State) -> Option<State> {
        let alives = nbhd.alives;
        let deads = nbhd.deads;
        match (succ_state, alives, deads) {
            (Dead, 2, 6) => Some(Dead),
            (Dead, 2, 5) => Some(Dead),
            (Alive, _, 6) => Some(Alive),
            _ => None,
        }
    }

    fn implication_nbhd(nbhd: &Self::NbhdDesc, succ_state: State) -> Option<State> {
        let state = nbhd.state;
        let alives = nbhd.alives;
        let deads = nbhd.deads;
        match (state, succ_state, alives, deads) {
            (Some(Dead), Dead, 2, 5) => Some(Dead),
            (Some(Dead), Alive, _, 5) => Some(Alive),
            (Some(Alive), Dead, 2, 4) => Some(Alive),
            (Some(Alive), Dead, 1, 5) => Some(Dead),
            (Some(Alive), Dead, 1, 6) => Some(Dead),
            (Some(Alive), Alive, _, 6) => Some(Alive),
            (None, Dead, 2, 5) => Some(Dead),
            (None, Alive, _, 6) => Some(Alive),
            _ => None,
        }
    }

    fn subperiod(&self) -> bool {
        (1..self.period).all(|t| self.period % t != 0
            || (0..self.height).any(|y|
                (0..self.width).any(|x|
                    self.get_state((x, y, 0)) != self.get_state((x, y, t)))))
    }

    fn display(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let s = match self.get_state((x, y, 0)) {
                    Some(Dead) => ".",
                    Some(Alive) => "o",
                    None => "?",
                };
                print!("{}", s);
            }
            println!("");
        }
    }
}
