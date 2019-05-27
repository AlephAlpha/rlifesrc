use std::rc::{Rc, Weak};
use crate::world::{State, Desc, LifeCell, World};
use crate::world::State::{Dead, Alive};

// 搜索状态
pub enum Status {
    // 已找到
    Found,
    // 无结果
    None,
    // 还在找
    Searching,
}

// 搜索时除了世界本身，还需要记录别的一些信息。
pub struct Search<W: World<NbhdDesc>, NbhdDesc: Desc + Copy> {
    pub world: W,
    // 搜索时给未知细胞选取的状态是否随机
    random: bool,
    // 存放在搜索过程中设定了状态的细胞
    set_table: Vec<Weak<LifeCell<NbhdDesc>>>,
    // 下一个要检验其状态的细胞，详见 proceed 函数
    next_set: usize,
    // 每搜索若干步就暂停一下
    step: Option<usize>,
    // 每搜索若干步就暂停一下
    step_count: usize,
}

impl<W: World<NbhdDesc>, NbhdDesc: Desc + Copy> Search<W, NbhdDesc> {
    pub fn new(world: W, random: bool, step: Option<usize>) -> Search<W, NbhdDesc> {
        let set_table = Vec::with_capacity(world.size());
        Search {world, random, set_table, next_set: 0, step, step_count: 0}
    }

    // 只有细胞原本的状态为未知时才改变细胞的状态，并且把细胞记录到 set_table 中
    fn put_cell(&mut self, cell: &Rc<LifeCell<NbhdDesc>>, state: State) -> Result<(), ()> {
        if let Some(old_state) = cell.state() {
            if state == old_state {
                Ok(())
            } else {
                Err(())
            }
        } else {
            W::set_cell(cell, Some(state), false);
            self.set_table.push(Rc::downgrade(cell));
            Ok(())
        }
    }

    // 确保由一个细胞前一代的邻域能得到这一代的状态
    // 由此确定一些未知细胞的状态
    fn consistify(&mut self, cell: &Rc<LifeCell<NbhdDesc>>) -> Result<(), ()> {
        let pred = cell.pred.borrow().upgrade().unwrap();
        let desc = pred.desc.get();
        if let Some(state) = self.world.transition(desc) {
            self.put_cell(cell, state)?;
        }
        if let Some(state) = cell.state() {
            if let Some(state) = self.world.implication(desc, state) {
                self.put_cell(&pred, state)?;
            }
            if let Some(state) = self.world.implication_nbhd(desc, state) {
                for neigh in pred.nbhd.borrow().iter() {
                    if let Some(neigh) = neigh.upgrade() {
                        if neigh.state().is_none() {
                            self.put_cell(&neigh, state)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // consistify 一个细胞本身，后一代，以及后一代的邻域中的所有细胞
    fn consistify10(&mut self, cell: &Rc<LifeCell<NbhdDesc>>) -> Result<(), ()> {
        let succ = cell.succ.borrow().upgrade().unwrap();
        self.consistify(cell)?;
        self.consistify(&succ)?;
        for neigh in succ.nbhd.borrow().iter() {
            if let Some(neigh) = neigh.upgrade() {
                self.consistify(&neigh)?;
            }
        }
        Ok(())
    }

    // 通过 consistify 和对称性把所有能确定的细胞确定下来
    fn proceed(&mut self) -> Result<(), ()> {
        while self.next_set < self.set_table.len() {
            let cell = self.set_table[self.next_set].upgrade().unwrap();
            let state = cell.state().unwrap();
            for sym in cell.sym.borrow().iter() {
                if let Some(sym) = sym.upgrade() {
                    self.put_cell(&sym, state)?;
                }
            }
            self.consistify10(&cell)?;
            self.next_set += 1;
        }
        Ok(())
    }

    // 恢复到上一次设定自由的未知细胞的值之前，并切换细胞的状态
    fn backup(&mut self) -> Result<(), ()> {
        self.next_set = self.set_table.len();
        while self.next_set > 0 {
            self.next_set -= 1;
            let cell = self.set_table[self.next_set].upgrade().unwrap();
            self.set_table.pop();
            if cell.free.get() {
                let state = match cell.state().unwrap() {
                    Dead => Alive,
                    Alive => Dead,
                };
                W::set_cell(&cell, Some(state), false);
                self.set_table.push(Rc::downgrade(&cell));
                return Ok(());
            } else {
                W::set_cell(&cell, None, true);
            }
        }
        Err(())
    }

    // 走；不对就退回来，换一下细胞的状态，再走，如此下去
    fn go(&mut self) -> Result<(), ()> {
        loop {
            if self.proceed().is_ok() {
                return Ok(());
            } else {
                self.backup()?;
            }
        }
    }

    // 最终搜索函数
    pub fn search(&mut self) -> Status {
        if let None = self.world.get_unknown().upgrade() {
            if self.backup().is_err() {
                return Status::None;
            }
        }
        while self.go().is_ok() {
            self.step_count += 1;
            if let Some(cell) = self.world.get_unknown().upgrade() {
                let state = if self.random {
                    rand::random()
                } else {
                    Dead
                };
                W::set_cell(&cell, Some(state), true);
                self.set_table.push(Rc::downgrade(&cell));
                if Some(self.step_count) >= self.step {
                    self.step_count = 0;
                    return Status::Searching;
                }
            } else if self.world.nontrivial() {
                self.step_count = 0;
                return Status::Found;
            } else {
                if self.backup().is_err() {
                    self.step_count = 0;
                    return Status::None;
                }
            }
        }
        self.step_count = 0;
        Status::None
    }
}
