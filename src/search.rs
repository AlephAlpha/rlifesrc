use std::rc::{Rc, Weak};
use crate::world::{State, Desc, LifeCell, Rule, World};
use crate::world::State::{Dead, Alive};

// 搜索状态
pub enum Status {
    // 已找到
    Found,
    // 无结果
    None,
    // 还在找
    Searching,
    // 暂停
    Paused,
}

// 搜索时除了世界本身，还需要记录别的一些信息。
pub struct Search<D: Desc, R: Rule<D>> {
    pub world: World<D, R>,
    // 搜索时给未知细胞选取的状态，None 表示随机
    new_state: Option<State>,
    // 存放在搜索过程中设定了状态的细胞
    set_table: Vec<Weak<LifeCell<D>>>,
    // 下一个要检验其状态的细胞，详见 proceed 函数
    next_set: usize,
}

impl<D: Desc, R: Rule<D>> Search<D, R> {
    pub fn new(world: World<D, R>, new_state: Option<State>) -> Search<D, R> {
        let set_table = Vec::with_capacity(world.size());
        Search {world, new_state, set_table, next_set: 0}
    }

    // 改变细胞的状态，并且把细胞记录到 set_table 中
    fn set_cell(&mut self, cell: &Rc<LifeCell<D>>, state: State) {
        D::set_cell(cell, Some(state), false);
        self.set_table.push(Rc::downgrade(cell));
    }

    // 只有细胞原本的状态为未知时才 set_cell
    fn check_cell(&mut self, cell: &Rc<LifeCell<D>>, state: State) -> Result<(), ()> {
        if let Some(old_state) = cell.state() {
            if state == old_state {
                Ok(())
            } else {
                Err(())
            }
        } else {
            self.set_cell(cell, state);
            Ok(())
        }
    }

    // consistify 一个细胞本身，后一代，以及后一代的邻域中的所有细胞
    fn consistify10(&mut self, cell: &Rc<LifeCell<D>>) -> Result<(), ()> {
        let succ = cell.succ.borrow().upgrade().unwrap();
        self.world.rule.consistify(cell, &mut self.set_table)?;
        self.world.rule.consistify(&succ, &mut self.set_table)?;
        for neigh in succ.nbhd.borrow().iter() {
            if let Some(neigh) = neigh.upgrade() {
                self.world.rule.consistify(&neigh, &mut self.set_table)?;
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
                    self.check_cell(&sym, state)?;
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
                self.set_cell(&cell, state);
                return Ok(());
            } else {
                D::set_cell(&cell, None, true);
            }
        }
        Err(())
    }

    // 走；不对就退回来，换一下细胞的状态，再走，如此下去
    fn go(&mut self, step: &mut usize) -> Result<(), ()> {
        loop {
            *step += 1;
            if self.proceed().is_ok() {
                return Ok(());
            } else {
                self.backup()?;
            }
        }
    }

    // 最终搜索函数
    pub fn search(&mut self, max_step: Option<usize>) -> Status {
        let mut step_count = 0;
        if let None = self.world.get_unknown().upgrade() {
            if self.backup().is_err() {
                return Status::None;
            }
        }
        while self.go(&mut step_count).is_ok() {
            if let Some(cell) = self.world.get_unknown().upgrade() {
                let state = match self.new_state {
                    Some(state) => state,
                    None => rand::random(),
                };
                D::set_cell(&cell, Some(state), true);
                self.set_table.push(Rc::downgrade(&cell));
                if let Some(max) = max_step {
                    if step_count > max {
                        return Status::Searching;
                    }
                }
            } else if self.world.nontrivial() {
                return Status::Found;
            } else {
                if self.backup().is_err() {
                    return Status::None;
                }
            }
        }
        Status::None
    }
}
