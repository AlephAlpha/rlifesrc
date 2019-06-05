use std::rc::Rc;
use crate::world::{State, Desc,  Rule, World, RcCell, WeakCell};
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
    set_table: Vec<WeakCell<D>>,
    // 下一个要检验其状态的细胞，详见 proceed 函数
    next_set: usize,
}

impl<D: Desc, R: Rule<D>> Search<D, R> {
    pub fn new(world: World<D, R>, new_state: Option<State>) -> Search<D, R> {
        let size = (world.width * world.height * world.period) as usize;
        let set_table = Vec::with_capacity(size);
        Search {world, new_state, set_table, next_set: 0}
    }

    // 由一个细胞本身，前一代，以及前一代的邻域，确保没有矛盾，并确定一些未知细胞的状态
    fn consistify(&mut self, cell: &RcCell<D>) -> Result<(), ()> {
        let pred = cell.pred.borrow().upgrade().unwrap();
        let pred_state = pred.state.get();
        let desc = pred.desc.get();
        if let Some(state) = self.world.rule.transition(pred_state, desc) {
            if let Some(old_state) = cell.state.get() {
                if state != old_state {
                    return Err(())
                }
            } else {
                cell.set(Some(state), false);
                self.set_table.push(Rc::downgrade(cell));
            }
        }
        if let Some(state) = cell.state.get() {
            if pred.state.get().is_none() {
                if let Some(state) = self.world.rule.implication(desc, state) {
                    pred.set(Some(state), false);
                    self.set_table.push(Rc::downgrade(&pred));
                }
            }
            self.world.rule.consistify_nbhd(&pred, desc, pred_state,
                state, &mut self.set_table);
        }
        Ok(())
    }

    // consistify 一个细胞本身，后一代，以及后一代的邻域中的所有细胞
    fn consistify10(&mut self, cell: &RcCell<D>) -> Result<(), ()> {
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
            let state = cell.state.get().unwrap();
            for sym in cell.sym.borrow().iter() {
                if let Some(sym) = sym.upgrade() {
                    if let Some(old_state) = sym.state.get() {
                        if state != old_state {
                            return Err(())
                        }
                    } else {
                        sym.set(Some(state), false);
                        self.set_table.push(Rc::downgrade(&sym));
                    }
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
                let state = match cell.state.get().unwrap() {
                    Dead => Alive,
                    Alive => Dead,
                };
                cell.set(Some(state), false);
                self.set_table.push(Rc::downgrade(&cell));
                return Ok(());
            } else {
                cell.set(None, true);
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
                cell.set(Some(state), true);
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
