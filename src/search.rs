use crate::world::State::{Alive, Dead};
use crate::world::{CellId, Desc, Rule, State, World};
use NewState::{Choose, FirstRandomThenDead, Random};

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

// 如何给未知细胞选取状态
#[derive(Clone, Copy, PartialEq)]
pub enum NewState {
    // 就选 Dead 或 Alive
    Choose(State),
    // 随机
    Random,
    // 细胞的 id 小于指定的值时随机选取，其余的选 Dead
    FirstRandomThenDead(CellId),
}

// 搜索时除了世界本身，还需要记录别的一些信息。
pub struct Search<D: Desc, R: Rule<Desc = D>> {
    pub world: World<D, R>,
    // 搜索时给未知细胞选取的状态，None 表示随机
    new_state: NewState,
    // 存放在搜索过程中设定了状态的细胞
    set_table: Vec<CellId>,
    // 下一个要检验其状态的细胞，详见 proceed 函数
    next_set: usize,
}

impl<D: Desc, R: Rule<Desc = D>> Search<D, R> {
    pub fn new(world: World<D, R>, new_state: NewState) -> Search<D, R> {
        let size = (world.width * world.height * world.period) as usize;
        let set_table = Vec::with_capacity(size);
        let new_state = match new_state {
            FirstRandomThenDead(_) => {
                let id = if world.column_first {
                    2 * (world.height + 2) * world.period
                } else {
                    2 * (world.width + 2) * world.period
                };
                FirstRandomThenDead(id as usize)
            }
            new_state => new_state,
        };
        Search {
            world,
            new_state,
            set_table,
            next_set: 0,
        }
    }

    // 确保由一个细胞本身和邻域能得到其后一代，由此确定一些未知细胞的状态
    fn consistify(&mut self, cell_id: CellId) -> bool {
        let cell = &self.world[cell_id];
        let succ_id = cell.succ.get();
        let succ = &self.world[succ_id];
        let state = cell.state.get();
        let desc = cell.desc.get();
        if let Some(new_state) = self.world.rule.transition(state, desc) {
            if let Some(succ_state) = succ.state.get() {
                if new_state != succ_state {
                    return false;
                }
            } else {
                self.world.set_cell(succ, Some(new_state), false);
                self.set_table.push(succ_id.unwrap());
            }
        }
        if let Some(succ_state) = succ.state.get() {
            if state.is_none() {
                if let Some(state) = self.world.rule.implication(desc, succ_state) {
                    self.world.set_cell(cell, Some(state), false);
                    self.set_table.push(cell_id);
                }
            }
            self.world.rule.consistify_nbhd(
                &cell,
                &self.world,
                desc,
                state,
                succ_state,
                &mut self.set_table,
            );
        }
        true
    }

    // consistify 一个细胞前一代，本身，以及邻域中的所有细胞
    fn consistify10(&mut self, cell_id: CellId) -> bool {
        self.consistify(cell_id) && {
            let cell = &self.world[cell_id];
            let pred_id = cell.pred.get().unwrap();
            self.consistify(pred_id) && {
                let cell = &self.world[cell_id];
                cell.nbhd
                    .get()
                    .iter()
                    .all(|&neigh_id| self.consistify(neigh_id.unwrap()))
            }
        }
    }

    // 通过 consistify 和对称性把所有能确定的细胞确定下来
    fn proceed(&mut self) -> bool {
        while self.next_set < self.set_table.len() {
            let cell_id = self.set_table[self.next_set];
            let cell = &self.world[cell_id];
            let state = cell.state.get().unwrap();
            for &sym_id in cell.sym.borrow().iter() {
                let sym = &self.world[sym_id];
                if let Some(old_state) = sym.state.get() {
                    if state != old_state {
                        return false;
                    }
                } else {
                    self.world.set_cell(sym, Some(state), false);
                    self.set_table.push(sym_id);
                }
            }
            if !self.consistify10(cell_id) {
                return false;
            }
            self.next_set += 1;
        }
        true
    }

    // 恢复到上一次设定自由的未知细胞的值之前，并切换细胞的状态
    fn backup(&mut self) -> bool {
        self.next_set = self.set_table.len();
        while self.next_set > 0 {
            self.next_set -= 1;
            let cell_id = self.set_table[self.next_set];
            let cell = &self.world[cell_id];
            self.set_table.pop();
            if cell.free.get() {
                let state = match cell.state.get().unwrap() {
                    Dead => Alive,
                    Alive => Dead,
                };
                self.world.set_cell(cell, Some(state), false);
                self.set_table.push(cell_id);
                return true;
            } else {
                self.world.set_cell(cell, None, true);
            }
        }
        false
    }

    // 走；不对就退回来，换一下细胞的状态，再走，如此下去
    fn go(&mut self, step: &mut usize) -> bool {
        loop {
            *step += 1;
            if self.proceed() {
                return true;
            } else if !self.backup() {
                return false;
            }
        }
    }

    // 最终搜索函数
    pub fn search(&mut self, max_step: Option<usize>) -> Status {
        let mut step_count = 0;
        if self.world.get_unknown().is_none() && !self.backup() {
            return Status::None;
        }
        while self.go(&mut step_count) {
            if let Some(cell) = self.world.get_unknown() {
                let state = match self.new_state {
                    Choose(state) => state,
                    Random => rand::random(),
                    FirstRandomThenDead(id) => {
                        if cell.id < id {
                            rand::random()
                        } else {
                            Dead
                        }
                    }
                };
                self.world.set_cell(cell, Some(state), true);
                self.set_table.push(cell.id);
                if let Some(max) = max_step {
                    if step_count > max {
                        return Status::Searching;
                    }
                }
            } else if self.world.nontrivial() {
                return Status::Found;
            } else if !self.backup() {
                return Status::None;
            }
        }
        Status::None
    }
}
