extern crate stopwatch;
use stopwatch::Stopwatch;
use crate::world::State;
use crate::world::World;

// 搜索时除了世界本身的状态，还需要记录别的一些信息。
pub struct Search<W: World<Index>, Index: Copy> {
    world: W,
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
    fn put_cell(&mut self, ix: Index, state: State) -> Result<(), ()> {
        if let Some(old_state) = self.world.get_state(ix) {
            if state == old_state {
                Ok(())
            } else {
                Err(())
            }
        } else {
            self.world.set_cell(ix, Some(state), false);
            self.set_table.push(ix);
            Ok(())
        }
    }

    // 确保由一个细胞前一代的邻域能得到这一代的状态；若不能则返回 false
    fn consistify(&mut self, ix: Index) -> Result<(), ()> {
        let pred = self.world.pred(ix);
        let pred_nbhd = self.world.get_desc(pred);
        if let Some(state) = W::transition(&pred_nbhd) {
            self.put_cell(ix, state)?;
        }
        if let Some(state) = self.world.get_state(ix) {
            if let Some(state) = W::implication(&pred_nbhd, state) {
                self.put_cell(pred, state)?;
            }
            if let Some(state) = W::implication_nbhd(&pred_nbhd, state) {
                for i in self.world.neighbors(pred) {
                    if self.world.get_state(i).is_none() {
                        self.put_cell(i, state)?;
                    }
                }
            }
        }
        Ok(())
    }

    // consistify 一个细胞本身，后一代，以及后一代的邻域中的所有细胞
    fn consistify10(&mut self, ix: Index) -> Result<(), ()> {
        let succ = self.world.succ(ix);
        self.consistify(ix)?;
        self.consistify(succ)?;
        for i in self.world.neighbors(succ) {
            self.consistify(i)?;
        }
        Ok(())
    }

    // 把所有能确定的细胞确定下来
    fn proceed(&mut self) -> Result<(), ()> {
        while self.next_set < self.set_table.len() {
            let ix = self.set_table[self.next_set];
            let state = self.world.get_state(ix).unwrap();
            for i in self.world.sym(ix) {
                self.put_cell(i, state)?;
            }
            self.consistify10(ix)?;
            self.next_set += 1;
        }
        Ok(())
    }

    // 恢复到上一次设定自由的未知细胞的值之前，并切换细胞的状态
    fn backup(&mut self) -> Result<(), ()> {
        self.next_set = self.set_table.len();
        while self.next_set > 0 {
            self.next_set -= 1;
            let ix = self.set_table[self.next_set];
            self.set_table.pop();
            if self.world.get_free(ix) {
                let state = match self.world.get_state(ix).unwrap() {
                    State::Dead => State::Alive,
                    State::Alive => State::Dead,
                };
                self.world.set_cell(ix, Some(state), false);
                self.set_table.push(ix);
                return Ok(());
            } else {
                self.world.set_cell(ix, None, true);
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
    pub fn search(&mut self) -> Result<(), ()> {
        if self.time {
            self.stopwatch.restart();
        }
        if let None = self.world.get_unknown() {
            self.backup()?;
        }
        while self.go().is_ok() {
            if let Some(ix) = self.world.get_unknown() {
                self.world.set_cell(ix, Some(State::Dead), true);
                self.set_table.push(ix);
            } else if self.world.subperiod() {
                return Ok(());
            } else {
                self.backup()?;
            }
        }
        Err(())
    }

    pub fn display(&self) {
        self.world.display();
        if self.time {
            println!("Time taken: {}ms.", self.stopwatch.elapsed_ms());
        }
    }
}
