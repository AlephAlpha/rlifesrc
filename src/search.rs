// 由于搞不定所有权之类的东西，只好 Copy 一切
#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Dead,
    Alive,
    Unknown,
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub state: State,   // 细胞状态
    pub free: bool,     // 此状态是否取决于其它细胞的状态
}

// 写成一个 Trait，方便以后支持更多的规则
// Ix 代表细胞的索引，由细胞的位置和时间决定
pub trait World<Ix: Copy> {
    // 用一个类型来记录邻域的状态
    type NbhdState;

    fn get_cell(&self, ix: Ix) -> Cell;
    fn set_cell(&mut self, ix: Ix, cell: Cell);

    // 细胞的邻域
    fn neighbors(&self, ix: Ix) -> Vec<Ix>;
    // 同一位置前一代的细胞
    fn pred(&self, ix: Ix) -> Ix;
    // 同一位置后一代的细胞
    fn succ(&self, ix: Ix) -> Ix;

    // 从这个细胞开始搜索
    fn first(&self) -> Ix;
    // 要搜索的下一个细胞；若是搜完了则返回 None
    fn next(&self, ix: Ix) -> Option<Ix>;

    // 从邻域的列表得到邻域的状态
    fn nbhd_state(&self, neighbors: Vec<Ix>) -> Self::NbhdState;
    // 由一个细胞及其邻域的状态得到其后一代的状态
    fn transit(cell: Cell, nbhd: &Self::NbhdState) -> State;

    // 由一个细胞本身、邻域以及其后一代的状态，决定其本身或者邻域中某些未知细胞的状态
    // 返回两个值，一个表示本身的状态，另一个表示邻域中未知细胞的状态
    // 这样写并不好扩展到 non-totalistic 的规则的情形，不过以后再说吧
    fn implic(cell: Cell, nbhd: &Self::NbhdState, succ: Cell) -> (State, State);

    // 输出图样
    fn display(&self);
}

// 搜索时除了世界本身的状态，还需要记录别的一些信息。
pub struct Search<W: World<Ix>, Ix: Copy> {
    pub world: W,
    // 存放在搜索过程中设定了值的细胞
    set_table: Vec<Ix>,
    // 下一个要检验其状态的细胞，详见 proceed 函数
    next_set: usize,
}

impl<W: World<Ix>, Ix: Copy> Search<W, Ix> {
    pub fn new(world: W) -> Search<W, Ix> {
        let set_table = Vec::new();   // 可能给它设一个 capacity 会比较好，比如说世界的大小
        Search {world, set_table, next_set: 0}
    }

    // 只有细胞原本的状态为未知时才改变细胞的状态；若原本的状态和新的状态矛盾则返回 false
    // 并且把细胞记录到 set_table 中
    fn put_cell(&mut self, ix: Ix, cell: Cell) -> bool {
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
    fn consistify(&mut self, ix: Ix) -> bool {
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
    fn consistify10(&mut self, ix: Ix) -> bool {
        let succ = self.world.succ(ix);
        let mut cells = self.world.neighbors(succ);
        cells.push(succ);
        cells.push(ix);
        cells.iter().all(|&i| self.consistify(i))
    }

    // 把所有能确定的细胞确定下来
    fn proceed(&mut self) -> bool {
        while self.next_set < self.set_table.len() {
            if !self.consistify10(self.set_table[self.next_set]) {
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

    // 获取一个未知的细胞
    fn get_unknown(&mut self) -> Option<Ix> {
        let mut ix = self.world.first();
        loop {
            if let State::Unknown = self.world.get_cell(ix).state {
                return Some(ix);
            } else if let Some(next) = self.world.next(ix) {
                ix = next;
            } else {
                return None;
            }
        }
    }

    // 最终搜索函数
    pub fn search(&mut self) -> bool {
        if let None = self.get_unknown() {
            if !self.backup() {
                return false;
            }
        }
        while self.go() {
            if let Some(ix) = self.get_unknown() {
                self.put_cell(ix, Cell {state: State::Dead, free: true});
            } else {
                return true;
            }
        }
        false
    }

    pub fn display(&self) {
        self.world.display();
    }
}