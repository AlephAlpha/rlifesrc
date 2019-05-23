use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::collections::HashMap;
use crate::world::State;
use crate::world::Cell;
use crate::world::World;

// 先实现生命游戏，以后再修改以满足其它规则
pub struct Life {
    width: isize,
    height: isize,
    period: isize,
    symmetry: Symmetry,

    // 搜索范围内的所有细胞的列表
    cells: Vec<Rc<RefCell<Cell>>>,

    // 搜索范围之外的辅助细胞
    aux_cells: HashMap<Index, Rc<RefCell<Cell>>>,
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
    pub fn new(width: isize, height: isize, period: isize,
        dx: isize, dy: isize, symmetry: Symmetry) -> Self {
        let size = (width + 2) * (height + 2) * period;
        let neighbors = [(-1,-1), (-1,0), (-1,1), (0,-1), (0,1), (1,-1), (1,0), (1,1)];
        let mut cells = Vec::with_capacity(size as usize);
        let aux_cells = HashMap::new();
        for _ in 0..size {
            cells.push(Cell::new_rc(Some(State::Dead), false));
        }
        let mut life = Life {width, height, period, symmetry, cells, aux_cells};

        // 给范围内的细胞添加各种等信息
        for x in -1..width + 1 {
            for y in -1..height + 1 {
                for t in 0..period {
                    let cell_rc = life.find_cell((x, y, t)).unwrap().upgrade().unwrap();
                    let mut cell = cell_rc.borrow_mut();

                    // 设定细胞状态
                    if x >= 0 && x < width && y >= 0 && y < height {
                        cell.state = None;
                        cell.free = true;
                    }

                    // 设定前一代；若前一代不在范围内则添加相应的辅助细胞
                    if t != 0 {
                        cell.pred = life.find_cell((x, y, t - 1)).unwrap();
                    } else {
                        let pred_ix = (x - dx, y - dy, period - 1);
                        match life.find_cell(pred_ix) {
                            Some(pred) => cell.pred = pred,
                            None => {
                                let pred = Cell::new_rc(Some(State::Dead), false);
                                life.aux_cells.insert(pred_ix, pred.clone());
                                cell.pred = Rc::downgrade(&pred);
                                pred.borrow_mut().succ = Rc::downgrade(&cell_rc);
                            },
                        }
                    }

                    // 设定后一代；若后一代不在范围内则添加相应的辅助细胞
                    if t != period - 1 {
                        cell.succ = life.find_cell((x, y, t + 1)).unwrap();
                    } else {
                        let succ_ix = (x + dx, y + dy, 0);
                        match life.find_cell(succ_ix) {
                            Some(succ) => cell.succ = succ,
                            None => {
                                let succ = Cell::new_rc(Some(State::Dead), false);
                                life.aux_cells.insert(succ_ix, succ.clone());
                                cell.succ = Rc::downgrade(&succ);
                                succ.borrow_mut().pred = Rc::downgrade(&cell_rc);
                            },
                        }
                    }

                    // 设定对称的细胞
                    let sym_ix = match life.symmetry {
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
                        if let Some(sym) = life.find_cell(ix) {
                            cell.sym.push(sym)
                        }
                    }

                    // 设定邻域
                    for (nx, ny) in neighbors.iter() {
                        if let Some(neigh) = life.find_cell((x + nx, y + ny, t)) {
                            cell.nbhd.push(neigh)
                        }
                    }
                }
            }
        }

        // 辅助细胞添加邻域信息就够了
        for (ix, cell_rc) in &life.aux_cells {
            let mut cell = cell_rc.borrow_mut();
            for (nx, ny) in neighbors.iter() {
                if let Some(neigh) = life.find_cell((ix.0 + nx, ix.1 + ny, ix.2)) {
                    cell.nbhd.push(neigh)
                }
            }
        }
        life
    }

    fn find_cell(&self, ix: Index) -> Option<Weak<RefCell<Cell>>> {
        let (x, y, t) = ix;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = ((x + 1) * (self.height + 2) + y + 1) * self.period + t;
            Some(Rc::downgrade(&self.cells[index as usize]))
        } else {
            self.aux_cells.get(&ix).map(Rc::downgrade)
        }
    }

    fn get_state(&self, ix: Index) -> Option<State> {
        match self.find_cell(ix).and_then(|c| c.upgrade()) {
            Some(cell) => cell.borrow().state,
            None => Some(State::Dead),
        }
    }
}

impl World for Life {
    type NbhdDesc = NbhdDesc;

    fn size(&self) -> usize {
        (self.width * self.height * self.period) as usize
    }

    fn get_desc(cell: &Cell) -> Self::NbhdDesc {
        let state = cell.state;
        let mut alives = 0;
        let mut unknowns = 0;
        for neigh in &cell.nbhd {
            match neigh.upgrade().unwrap().borrow().state {
                Some(State::Alive) => alives += 1,
                None => unknowns += 1,
                _ => (),
            }
        }
        let deads = 8 - alives - unknowns;
        NbhdDesc {state, alives, deads}
    }

    fn transition(desc: &Self::NbhdDesc) -> Option<State> {
        let alives = desc.alives;
        let deads = desc.deads;
        match desc.state {
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

    fn implication(desc: &Self::NbhdDesc, succ_state: State) -> Option<State> {
        match (succ_state, desc.alives, desc.deads) {
            (State::Dead, 2, 6) => Some(State::Dead),
            (State::Dead, 2, 5) => Some(State::Dead),
            (State::Alive, _, 6) => Some(State::Alive),
            _ => None,
        }
    }

    fn implication_nbhd(desc: &Self::NbhdDesc, succ_state: State) -> Option<State> {
        match (desc.state, succ_state, desc.alives, desc.deads) {
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

    fn get_unknown(&self) -> Option<Rc<RefCell<Cell>>> {
        self.cells.iter().find(|cell| cell.borrow().state.is_none()).map(Rc::clone)
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
