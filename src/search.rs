use crate::world::*;
use NewState::{Choose, Random, Smart};

#[cfg(feature = "stdweb")]
use serde::{Deserialize, Serialize};

/// 搜索状态
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
pub enum Status {
    /// 已找到
    Found,
    /// 无结果
    None,
    /// 还在找
    Searching,
    /// 暂停
    Paused,
}

/// 如何给未知细胞选取状态
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
pub enum NewState {
    /// 就选 `Dead` 或 `Alive`
    Choose(State),
    /// 随机
    Random,
    /// 第一行/第一列的细胞选 `Alive`，其余的选 `Dead`
    ///
    /// 其实一点也不智能，不过我想不出别的名字了。
    Smart,
}

/// 搜索
///
/// 搜索时除了世界本身，还需要记录别的一些信息。
pub struct Search<'a, D: Desc, R: 'a + Rule<Desc = D>> {
    /// 世界
    pub world: World<'a, D, R>,
    /// 搜索时给未知细胞选取的状态
    new_state: NewState,
    /// 存放在搜索过程中设定了状态的细胞
    set_table: Vec<&'a LifeCell<'a, D>>,
    /// 下一个要检验其状态的细胞在 `set_table` 中的位置，详见 `proceed` 函数
    next_set: usize,
    /// 极大的活细胞个数
    max_cell_count: Option<u32>,
}

impl<'a, D: Desc, R: 'a + Rule<Desc = D>> Search<'a, D, R> {
    /// 新建搜索
    pub fn new(world: World<'a, D, R>, new_state: NewState, max_cell_count: Option<u32>) -> Self {
        let size = (world.width * world.height * world.period) as usize;
        let set_table = Vec::with_capacity(size);
        Search {
            world,
            new_state,
            set_table,
            next_set: 0,
            max_cell_count,
        }
    }

    /// 确保由一个细胞本身和邻域能得到其后一代，由此确定一些未知细胞的状态
    fn consistify(&mut self, cell: &'a LifeCell<'a, D>) -> bool {
        let succ = cell.succ.get().unwrap();
        let state = cell.state.get();
        let desc = cell.desc.get();
        if let Some(new_state) = self.world.rule.transition(state, desc) {
            if let Some(succ_state) = succ.state.get() {
                if new_state != succ_state {
                    return false;
                }
            } else {
                self.world.set_cell(succ, Some(new_state), false);
                self.set_table.push(succ);
            }
        }
        if let Some(succ_state) = succ.state.get() {
            if state.is_none() {
                if let Some(state) = self.world.rule.implication(desc, succ_state) {
                    self.world.set_cell(cell, Some(state), false);
                    self.set_table.push(cell);
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

    /// `consistify` 一个细胞前一代，本身，以及邻域中的所有细胞
    fn consistify10(&mut self, cell: &'a LifeCell<'a, D>) -> bool {
        self.consistify(cell) && {
            let pred = cell.pred.get().unwrap();
            self.consistify(pred) && {
                cell.nbhd
                    .get()
                    .iter()
                    .all(|&neigh_id| self.consistify(neigh_id.unwrap()))
            }
        }
    }

    /// 通过 `consistify` 和对称性把所有能确定的细胞确定下来
    fn proceed(&mut self) -> bool {
        while self.next_set < self.set_table.len() {
            if let Some(max) = self.max_cell_count {
                if self.world.cell_count.get() > max {
                    return false;
                }
            }
            let cell = self.set_table[self.next_set];
            let state = cell.state.get().unwrap();
            for &sym in cell.sym.borrow().iter() {
                if let Some(old_state) = sym.state.get() {
                    if state != old_state {
                        return false;
                    }
                } else {
                    self.world.set_cell(sym, Some(state), false);
                    self.set_table.push(sym);
                }
            }
            if !self.consistify10(cell) {
                return false;
            }
            self.next_set += 1;
        }
        true
    }

    /// 恢复到上一次设定自由的未知细胞的值之前，并切换细胞的状态
    fn backup(&mut self) -> bool {
        self.next_set = self.set_table.len();
        while self.next_set > 0 {
            self.next_set -= 1;
            let cell = self.set_table[self.next_set];
            self.set_table.pop();
            if cell.free.get() {
                let state = match cell.state.get().unwrap() {
                    Dead => Alive,
                    Alive => Dead,
                };
                self.world.set_cell(cell, Some(state), false);
                self.set_table.push(cell);
                return true;
            } else {
                self.world.set_cell(cell, None, true);
            }
        }
        false
    }

    /// 走；不对就退回来，换一下细胞的状态，再走，如此下去
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

    /// 最终搜索函数
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
                    Smart => {
                        if cell.first_col {
                            Alive
                        } else {
                            Dead
                        }
                    }
                };
                self.world.set_cell(cell, Some(state), true);

                // `set_table` 需要 `cell` 的生命周期至少是 `'a`，
                // 不用 `unsafe` 的话会导致生命周期冲突。
                unsafe {
                    let cell: *const LifeCell<_> = cell;
                    self.set_table.push(cell.as_ref().unwrap());
                }

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

/// 把 `Search` 写成一个 Trait，方便后面用 trait object 来切换不同类型的规则
///
/// 应该有更好的办法。但我能想到的就两种：
/// 一是直接在 `World` 和 `LifeCell` 里边用 trait object，但可能会影响速度
/// 二是把所有可能的规则对应的 `Search` 写成一个 Enum，但感觉好蠢
pub trait TraitSearch {
    fn search(&mut self, max_step: Option<usize>) -> Status;

    fn display_gen(&self, t: isize) -> String;

    fn period(&self) -> isize;
}

impl<'a, D: Desc, R: Rule<Desc = D>> TraitSearch for Search<'a, D, R> {
    /// 最终搜索函数
    fn search(&mut self, max_step: Option<usize>) -> Status {
        self.search(max_step)
    }

    /// 显示某一代的整个世界
    fn display_gen(&self, t: isize) -> String {
        self.world.display_gen(t)
    }

    /// 世界的周期
    fn period(&self) -> isize {
        self.world.period
    }
}
