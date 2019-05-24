use std::rc::{Rc, Weak};
use std::collections::HashMap;
use crate::world::{State, Desc, LifeCell, World};
use crate::world::State::{Dead, Alive};

// 横座标，纵座标，时间
type Index = (isize, isize, isize);

// 邻域的状态，state 表示细胞本身的状态，后两个数加起来不能超过 8
#[derive(Clone, Copy)]
pub struct NbhdDesc {
    state: Option<State>,
    alives: u8,
    deads: u8,
}

impl Desc for NbhdDesc {
    fn new(state: Option<State>) -> Self {
        NbhdDesc {state, alives: 0, deads: 8}
    }

    fn state(&self) -> Option<State> {
        self.state
    }

    fn transition(&self) -> Option<State> {
        let alives = self.alives;
        let deads = self.deads;
        match self.state {
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

    fn implication(&self, succ_state: State) -> Option<State> {
        match (succ_state, self.alives, self.deads) {
            (Dead, 2, 6) => Some(Dead),
            (Dead, 2, 5) => Some(Dead),
            (Alive, _, 6) => Some(Alive),
            _ => None,
        }
    }

    fn implication_nbhd(&self, succ_state: State) -> Option<State> {
        match (self.state, succ_state, self.alives, self.deads) {
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

    // 搜索范围内的所有细胞的列表
    cells: Vec<Rc<LifeCell<NbhdDesc>>>,

    // 搜索范围之外的辅助细胞
    aux_cells: HashMap<Index, Rc<LifeCell<NbhdDesc>>>,
}

impl Life {
    pub fn new(width: isize, height: isize, period: isize,
        dx: isize, dy: isize, symmetry: Symmetry) -> Self {
        let size = (width + 2) * (height + 2) * period;
        let neighbors = [(-1,-1), (-1,0), (-1,1), (0,-1), (0,1), (1,-1), (1,0), (1,1)];
        let mut cells = Vec::with_capacity(size as usize);
        let aux_cells = HashMap::new();
        for _ in 0..size {
            cells.push(Rc::new(LifeCell::new(Some(Dead), false)));
        }
        let mut life = Life {width, height, period, cells, aux_cells};

        // 给范围内的细胞添加各种信息
        for x in -1..width + 1 {
            for y in -1..height + 1 {
                for t in 0..period {
                    let cell = life.find_cell((x, y, t)).upgrade().unwrap();

                    // 设定前一代；若前一代不在范围内则添加相应的辅助细胞
                    if t != 0 {
                        *cell.pred.borrow_mut() = life.find_cell((x, y, t - 1));
                    } else {
                        let pred_ix = (x - dx, y - dy, period - 1);
                        let pred_weak = life.find_cell(pred_ix);
                        if pred_weak.upgrade().is_some() {
                            *cell.pred.borrow_mut() = pred_weak;
                        } else {
                            let pred = Rc::new(LifeCell::new(Some(Dead), false));
                            life.aux_cells.insert(pred_ix, pred.clone());
                            *cell.pred.borrow_mut() = Rc::downgrade(&pred);
                            *pred.succ.borrow_mut() = Rc::downgrade(&cell);
                        }
                    }

                    // 设定后一代；若后一代不在范围内则添加相应的辅助细胞
                    if t != period - 1 {
                        *cell.succ.borrow_mut() = life.find_cell((x, y, t + 1));
                    } else {
                        let succ_ix = (x + dx, y + dy, 0);
                        let succ_weak = life.find_cell(succ_ix);
                        if succ_weak.upgrade().is_some() {
                            *cell.succ.borrow_mut() = succ_weak;
                        } else {
                            let succ = Rc::new(LifeCell::new(Some(Dead), false));
                            life.aux_cells.insert(succ_ix, succ.clone());
                            *cell.succ.borrow_mut() = Rc::downgrade(&succ);
                            *succ.pred.borrow_mut() = Rc::downgrade(&cell);
                        }
                    }

                    // 设定对称的细胞
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
                            let sym = Rc::new(LifeCell::new(Some(Dead), false));
                            life.aux_cells.insert(ix, sym.clone());
                            cell.sym.borrow_mut().push(Rc::downgrade(&sym));
                        }
                    }

                    // 设定邻域
                    for (nx, ny) in neighbors.iter() {
                        let neigh_weak = life.find_cell((x + nx, y + ny, t));
                        if neigh_weak.upgrade().is_some() {
                            cell.nbhd.borrow_mut().push(neigh_weak);
                        }
                    }
                }
            }
        }

        // 辅助细胞添加邻域信息就够了
        for (ix, cell) in &life.aux_cells {
            for (nx, ny) in neighbors.iter() {
                let neigh_weak = life.find_cell((ix.0 + nx, ix.1 + ny, ix.2));
                if neigh_weak.upgrade().is_some() {
                    cell.nbhd.borrow_mut().push(neigh_weak);
                }
            }
        }

        // 最后才用 set_cell 设置细胞状态
        for x in 0..width {
            for y in 0..height {
                for t in 0..period {
                    let cell = life.find_cell((x, y, t)).upgrade().unwrap();
                    Life::set_cell(&cell, None, true);
                }
            }
        }

        life
    }

    fn find_cell(&self, ix: Index) -> Weak<LifeCell<NbhdDesc>> {
        let (x, y, t) = ix;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = ((x + 1) * (self.height + 2) + y + 1) * self.period + t;
            Rc::downgrade(&self.cells[index as usize])
        } else {
            self.aux_cells.get(&ix).map(Rc::downgrade).unwrap_or_default()
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
        if old_state == state {
            return ();
        }
        let mut desc = cell.desc.get();
        desc.state = state;
        cell.desc.set(desc);
        cell.free.set(free);
        for neigh in cell.nbhd.borrow().iter() {
            let neigh = neigh.upgrade().unwrap();
            let mut desc = neigh.desc.get();
            match old_state {
                Some(Dead) => desc.deads -= 1,
                Some(Alive) => desc.alives -= 1,
                None => (),
            };
            match state {
                Some(Dead) => desc.deads += 1,
                Some(Alive) => desc.alives += 1,
                None => (),
            };
            neigh.desc.set(desc);
        }
    }

    fn get_unknown(&self) -> Weak<LifeCell<NbhdDesc>> {
        self.cells.iter().find(|cell| cell.state().is_none())
            .map(Rc::downgrade).unwrap_or_default()
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
