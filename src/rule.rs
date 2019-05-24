use std::rc::{Rc, Weak};
use std::cell::Cell;
use crate::world::{State, Desc, LifeCell, World};
use crate::world::State::{Dead, Alive};

// 横座标，纵座标，时间
type Index = (isize, isize, isize);

// 邻域的状态，state 表示细胞本身的状态，后两个数加起来不能超过 8
pub struct NbhdDesc {
    state: Cell<Option<State>>,
    alives: Cell<u8>,
    deads: Cell<u8>,
}

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        let state = Cell::new(state);
        let alives = Cell::new(0);
        let deads = Cell::new(8);
        NbhdDesc {state, alives, deads}
    }

    fn state(&self) -> Option<State> {
        self.state.get()
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

// 先实现生命游戏，以后再修改以满足其它规则
pub struct Life {
    width: isize,
    height: isize,
    period: isize,

    // 搜索顺序是先行后列还是先列后行
    // 通过比较行数和列数的大小来自动决定
    col_first: bool,

    // 搜索范围内的所有细胞的列表
    cells: Vec<Rc<LifeCell<NbhdDesc>>>,
}

impl Life {
    pub fn new(width: isize, height: isize, period: isize,
        dx: isize, dy: isize, symmetry: Symmetry) -> Self {
        let size = (width + 2) * (height + 2) * period;
        let neighbors = [(-1,-1), (-1,0), (-1,1), (0,-1), (0,1), (1,-1), (1,0), (1,1)];
        let col_first = {
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
        let mut cells = Vec::with_capacity(size as usize);
        for _ in 0..size {
            cells.push(Rc::new(LifeCell::new(Some(Dead), false)));
        }
        let life = Life {width, height, period, col_first, cells};

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

                    // 用 set_cell 设置细胞状态
                    if 0 <= x && x < width && 0 <= y && y < height {
                        Life::set_cell(&cell, None, true);
                    }

                    // 设定前一代；若前一代不在范围内则把此细胞设为 Dead
                    if t != 0 {
                        *cell.pred.borrow_mut() = life.find_cell((x, y, t - 1));
                    } else {
                        let pred_ix = (x - dx, y - dy, period - 1);
                        let pred_weak = life.find_cell(pred_ix);
                        if pred_weak.upgrade().is_some() {
                            *cell.pred.borrow_mut() = pred_weak;
                        } else {
                            Life::set_cell(&cell, Some(Dead), false);
                        }
                    }

                    // 设定后一代；若后一代不在范围内则把此细胞设为 Dead
                    if t != period - 1 {
                        *cell.succ.borrow_mut() = life.find_cell((x, y, t + 1));
                    } else {
                        let succ_ix = (x + dx, y + dy, 0);
                        let succ_weak = life.find_cell(succ_ix);
                        if succ_weak.upgrade().is_some() {
                            *cell.succ.borrow_mut() = succ_weak;
                        } else {
                            Life::set_cell(&cell, Some(Dead), false);
                        }
                    }

                    // 设定对称的细胞；若对称的细胞不在范围内则把此细胞设为 Dead
                    let sym_ix = match symmetry {
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
                    for ix in sym_ix {
                        let sym_weak = life.find_cell(ix);
                        if sym_weak.upgrade().is_some() {
                            cell.sym.borrow_mut().push(sym_weak);
                        } else {
                            Life::set_cell(&cell, Some(Dead), false);
                        }
                    }
                }
            }
        }

        life
    }

    fn find_cell(&self, ix: Index) -> Weak<LifeCell<NbhdDesc>> {
        let (x, y, t) = ix;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = if self.col_first {
                ((x + 1) * (self.height + 2) + y + 1) * self.period + t
            } else {
                ((y + 1) * (self.width + 2) + x + 1) * self.period + t
            };
            Rc::downgrade(&self.cells[index as usize])
        } else {
            Weak::new()
        }
    }

    fn get_state(&self, ix: Index) -> Option<State> {
        match self.find_cell(ix).upgrade() {
            Some(cell) => cell.state(),
            None => Some(Dead),
        }
    }
}

impl World<NbhdDesc> for Life {
    fn size(&self) -> usize {
        (self.width * self.height * self.period) as usize
    }

    fn set_cell(cell: &LifeCell<NbhdDesc>, state: Option<State>, free: bool) {
        let old_state = cell.state();
        cell.desc.state.set(state);
        cell.free.set(free);
        for neigh in cell.nbhd.borrow().iter() {
            let neigh = neigh.upgrade().unwrap();
            let mut deads = neigh.desc.deads.get();
            let mut alives = neigh.desc.alives.get();
            match old_state {
                Some(Dead) => deads -= 1,
                Some(Alive) => alives -= 1,
                None => (),
            };
            match state {
                Some(Dead) => deads += 1,
                Some(Alive) => alives += 1,
                None => (),
            };
            neigh.desc.deads.set(deads);
            neigh.desc.alives.set(alives);
        }
    }

    fn get_unknown(&self) -> Weak<LifeCell<NbhdDesc>> {
        self.cells.iter().find(|cell| cell.state().is_none())
            .map(Rc::downgrade).unwrap_or_default()
    }

    fn transition(desc: &NbhdDesc) -> Option<State> {
        let alives = desc.alives.get();
        let deads = desc.deads.get();
        match desc.state() {
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

    fn implication(desc: &NbhdDesc, succ_state: State) -> Option<State> {
        match (succ_state, desc.alives.get(), desc.deads.get()) {
            (Dead, 2, 6) => Some(Dead),
            (Dead, 2, 5) => Some(Dead),
            (Alive, _, 6) => Some(Alive),
            _ => None,
        }
    }

    fn implication_nbhd(desc: &NbhdDesc, succ_state: State) -> Option<State> {
        match (desc.state(), succ_state, desc.alives.get(), desc.deads.get()) {
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
        (1..self.period).all(|t|
            self.period % t != 0 || (0..self.height).any(|y| (0..self.width).any(|x|
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
