extern crate stopwatch;
use stopwatch::Stopwatch;
use crate::world::State;
use crate::world::Cell;
use crate::world::World;

// 搜索时除了世界本身的状态，还需要记录别的一些信息。
pub struct Search<W: World<Index>, Index: Copy> {
    pub world: W,
    // 存放在搜索过程中设定了值的细胞
    set_table: Vec<Index>,
    // 下一个要检验其状态的细胞，详见 proceed 函数
    next_set: usize,
    // 是否计时
    time: bool,
    // 记录搜索时间
    stopwatch: Stopwatch,
}

impl<W: World<Index>, Index: Copy> Search<W, Index> {
    pub fn new(world: W, time: bool) -> Search<W, Index> {
        let set_table = Vec::with_capacity(world.size());
        let stopwatch = Stopwatch::new();
        Search {world, set_table, next_set: 0, time, stopwatch}
    }

    // 只有细胞原本的状态为未知时才改变细胞的状态；若原本的状态和新的状态矛盾则返回 false
    // 并且把细胞记录到 set_table 中
    fn put_cell(&mut self, ix: Index, cell: Cell) -> bool {
        let old_cell = self.world.get_cell(ix);
        match (old_cell.state, cell.state) {
            (_, State::Unknown) => true,
            (State::Unknown, _) => {
                self.world.set_cell(ix, cell);
                self.set_table.push(ix);
                true
            },
            _ => old_cell.state == cell.state,
        }
    }

    // 确保由一个细胞前一代的邻域能得到这一代的状态；若不能则返回 false
    fn consistify(&mut self, ix: Index) -> bool {
        // 先用 transit 来看这个细胞本来的状态
        let pred = self.world.pred(ix);
        let pred_cell = self.world.get_cell(pred);
        let pred_nbhd = self.world.nbhd_state(self.world.neighbors(pred));
        let state = W::transit(pred_cell, &pred_nbhd);
        if !self.put_cell(ix, Cell {state, free: false}) {
            return false;
        }

        // 如果上一步没有矛盾，就用 implic 来看前一代的邻域的状态
        let (state, state_nbhd) = W::implic(pred_cell, &pred_nbhd, self.world.get_cell(ix));
        if !self.put_cell(pred, Cell {state, free: false}) {
            return false;
        }
        self.world.neighbors(pred).iter().all(|&i| {
            if let State::Unknown = self.world.get_cell(i).state {
                self.put_cell(i, Cell {state: state_nbhd, free: false})
            } else {
                true
            }
        })
    }

    // consistify 一个细胞本身，后一代，以及后一代的邻域中的所有细胞
    fn consistify10(&mut self, ix: Index) -> bool {
        let succ = self.world.succ(ix);
        self.consistify(ix) && self.consistify(succ) &&
            self.world.neighbors(succ).iter().all(|&i| self.consistify(i))
    }

    // 把所有能确定的细胞确定下来
    fn proceed(&mut self) -> bool {
        while self.next_set < self.set_table.len() {
            let ix = self.set_table[self.next_set];
            let cell = self.world.get_cell(ix);
            if self.world.sym(ix).iter().any(|&i| !self.put_cell(i, cell))
                 || !self.consistify10(ix) {
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
            let ix = self.set_table[self.next_set];
            self.set_table.pop();
            if self.world.get_cell(ix).free {
                let state = match self.world.get_cell(ix).state {
                    State::Dead => State::Alive,
                    State::Alive => State::Dead,
                    State::Unknown => panic!("Something is wrong!"),
                };
                self.world.set_cell(ix, Cell {state, free: false});
                self.set_table.push(ix);
                return true;
            } else {
                self.world.set_cell(ix, Cell {state: State::Unknown, free: true});
            }
        }
        false
    }

    // 走；不对就退回来，换一下细胞的状态，再走，如此下去
    fn go(&mut self) -> bool {
        loop {
            if self.proceed() {
                return true;
            } else if !self.backup() {
                return false;
            }
        }
    }

    // 最终搜索函数
    pub fn search(&mut self) -> bool {
        if self.time {
            self.stopwatch.restart();
        }
        if let None = self.world.get_unknown() {
            if !self.backup() {
                return false;
            }
        }
        while self.go() {
            if let Some(ix) = self.world.get_unknown() {
                self.put_cell(ix, Cell {state: State::Dead, free: true});
            } else if self.world.subperiod() {
                return true;
            } else if !self.backup() {
                return false;
            }
        }
        false
    }

    pub fn display(&self) {
        self.world.display();
        if self.time {
            println!("Time taken: {}ms.", self.stopwatch.elapsed_ms());
        }
    }
}