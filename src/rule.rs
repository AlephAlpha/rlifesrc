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
}

// 横座标，纵座标，时间
type Index = (isize, isize, isize);

// 邻域的状态，state 表示细胞本身的状态，后两个数加起来不能超过 8
// 有点想像 lifesrc 一样用一个字节来保持邻域状态，不过试过之后发现并没有更快
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
        let size = width * height * period;
        let mut cells = Vec::with_capacity(size as usize);

        for _ in 0..size {
            cells.push(Cell {state: None, free: true});
        }

        Life {width, height, period, dx, dy, symmetry, cells}
    }

    fn inside(&self, ix: Index) -> bool {
         return ix.0 >= 0 && ix.0 < self.width &&
                ix.1 >= 0 && ix.1 < self.height &&
                ix.2 >= 0 && ix.2 < self.period
    }

    fn index(&self, ix: Index) -> usize {
        ((ix.0 * self.height + ix.1) * self.period + ix.2) as usize
    }

    fn to_index(&self, i: usize) -> Index {
        let i = i as isize;
        let j = i / self.period;
        let k = j / self.height;
        (k % self.width, j % self.height, i % self.period)
    }
}

impl World<Index> for Life {
    type NbhdDesc = NbhdDesc;

    fn size(&self) -> usize {
        (self.width * self.height * self.period) as usize
    }

    fn get_state(&self, ix: Index) -> Option<State> {
        if self.inside(ix) {
            self.cells[self.index(ix)].state
        } else {
            Some(State::Dead)
        }
    }

    fn get_free(&self, ix: Index) -> bool {
        if self.inside(ix) {
            self.cells[self.index(ix)].free
        } else {
            false
        }
    }

    fn set_cell(&mut self, ix: Index, state: Option<State>, free: bool) {
        let index = self.index(ix);
        if self.inside(ix) {
            self.cells[index].state = state;
            self.cells[index].free = free;
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
             (ix.0 + 1, ix.1 + 1, ix.2)]
    }

    fn get_desc(&self, ix: Index) -> Self::NbhdDesc {
        let mut alives = 0;
        let mut unknowns = 0;
        for n in self.neighbors(ix) {
            match self.get_state(n) {
                Some(State::Alive) => alives += 1,
                None => unknowns += 1,
                _ => (),
            }
        }
        let deads = 8 - alives - unknowns;
        let state = self.get_state(ix);
        NbhdDesc {state, alives, deads}
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
        self.cells.iter().position(|cell| cell.state.is_none())
            .map(|i| self.to_index(i))
    }

    // 仅适用于生命游戏
    // 这些条件是从 lifesrc 抄来的
    fn transition(nbhd: &Self::NbhdDesc) -> Option<State> {
        let state = nbhd.state;
        let alives = nbhd.alives;
        let deads = nbhd.deads;
        match state {
            Some(State::Dead) => if deads > 5 || alives > 3 {
                Some(State::Dead)
            } else if alives == 3 && deads == 5 {
                Some(State::Alive)
            } else {
                None
            },
            Some(State::Alive) => if deads > 6 || alives > 3 {
                Some(State::Dead)
            } else if (alives == 2 && (deads == 5 || deads == 6)) ||
                      (alives == 3 && deads == 5) {
                Some(State::Alive)
            } else {
                None
            },
            None => if deads > 6 || alives > 3 {
                Some(State::Dead)
            } else if alives == 3 && deads == 5 {
                Some(State::Alive)
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
            (State::Dead, 2, 6) => Some(State::Dead),
            (State::Dead, 2, 5) => Some(State::Dead),
            (State::Alive, _, 6) => Some(State::Alive),
            _ => None,
        }
    }

    fn implication_nbhd(nbhd: &Self::NbhdDesc, succ_state: State) -> Option<State> {
        let state = nbhd.state;
        let alives = nbhd.alives;
        let deads = nbhd.deads;
        match (state, succ_state, alives, deads) {
            (Some(State::Dead), State::Dead, 2, 5) => Some(State::Dead),
            (Some(State::Dead), State::Alive, _, 5) => Some(State::Alive),
            (Some(State::Alive), State::Dead, 2, 4) => Some(State::Alive),
            (Some(State::Alive), State::Dead, 1, 5) => Some(State::Dead),
            (Some(State::Alive), State::Dead, 1, 6) => Some(State::Dead),
            (Some(State::Alive), State::Alive, _, 6) => Some(State::Alive),
            (None, State::Dead, 2, 5) => Some(State::Dead),
            (None, State::Alive, _, 6) => Some(State::Alive),
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
                    Some(State::Dead) => ".",
                    Some(State::Alive) => "o",
                    None => "?",
                };
                print!("{}", s);
            }
            println!("");
        }
    }
}
