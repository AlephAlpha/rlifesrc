use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cell::{Cell, RefCell};
use std::fmt::{Display, Error, Formatter};
use std::ops::Index;
use std::str::FromStr;
use State::{Alive, Dead};

// 细胞状态
#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Dead,
    Alive,
}

impl Distribution<State> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> State {
        match rng.gen_range(0, 2) {
            0 => Dead,
            _ => Alive,
        }
    }
}

// 所有的细胞保存在一个向量中，细胞的 id 等于其在向量中的位置
pub type CellId = usize;

// D 表示邻域的状态
pub struct LifeCell<D> {
    // 细胞的 id
    pub id: CellId,
    // 细胞自身的状态
    pub state: Cell<Option<State>>,
    // 细胞邻域的状态
    pub desc: Cell<D>,
    // 细胞的状态是否由别的细胞决定
    pub free: Cell<bool>,
    // 同一位置上一代的细胞
    pub pred: Cell<Option<CellId>>,
    // 同一位置下一代的细胞
    pub succ: Cell<Option<CellId>>,
    // 细胞的邻域
    pub nbhd: Cell<[Option<CellId>; 8]>,
    // 与此细胞对称（因此状态一致）的细胞
    pub sym: RefCell<Vec<CellId>>,
}

impl<D: Desc> LifeCell<D> {
    pub fn new(id: CellId, state: Option<State>, free: bool) -> Self {
        let desc = Cell::new(D::new(state));
        let state = Cell::new(state);
        let free = Cell::new(free);
        let pred = Default::default();
        let succ = Default::default();
        let nbhd = Default::default();
        let sym = Default::default();
        LifeCell {
            id,
            state,
            desc,
            free,
            pred,
            succ,
            nbhd,
            sym,
        }
    }
}

// 邻域的状态应该满足一个 trait
pub trait Desc: Copy {
    // 通过一个细胞的状态生成一个默认的邻域
    fn new(state: Option<State>) -> Self;

    // 改变一个细胞的状态时处理其邻域中所有细胞的邻域状态
    fn set_nbhd<R: Rule<Desc = Self>>(
        world: &World<Self, R>,
        cell: &LifeCell<Self>,
        old_state: Option<State>,
        state: Option<State>,
    );
}

// 把规则写成一个 Trait，方便以后支持更多的规则
pub trait Rule: Sized {
    type Desc: Desc;

    // 规则是否是 B0
    fn b0(&self) -> bool;

    // 由一个细胞及其邻域的状态得到其后一代的状态
    fn transition(&self, state: Option<State>, desc: Self::Desc) -> Option<State>;

    // 由一个细胞的邻域以及其后一代的状态，决定其本身的状态
    fn implication(&self, desc: Self::Desc, succ_state: State) -> Option<State>;

    // 由一个细胞本身、邻域以及其后一代的状态，改变其邻域中某些未知细胞的状态
    // 并把改变了值的细胞放到 set_table 中
    fn consistify_nbhd(
        &self,
        cell: &LifeCell<Self::Desc>,
        world: &World<Self::Desc, Self>,
        desc: Self::Desc,
        state: Option<State>,
        succ_state: State,
        set_table: &mut Vec<CellId>,
    );
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

impl FromStr for Symmetry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C1" => Ok(Symmetry::C1),
            "C2" => Ok(Symmetry::C2),
            "C4" => Ok(Symmetry::C4),
            "D2|" => Ok(Symmetry::D2Row),
            "D2-" => Ok(Symmetry::D2Column),
            "D2\\" => Ok(Symmetry::D2Diag),
            "D2/" => Ok(Symmetry::D2Antidiag),
            "D4+" => Ok(Symmetry::D4Ortho),
            "D4X" => Ok(Symmetry::D4Diag),
            "D8" => Ok(Symmetry::D8),
            _ => Err(String::from("invalid symmetry")),
        }
    }
}

// 横座标，纵座标，时间
type Coord = (isize, isize, isize);

// 世界，暂时只适用于 Life-Like 的规则
pub struct World<D: Desc, R: Rule<Desc = D>> {
    pub width: isize,
    pub height: isize,
    pub period: isize,
    pub rule: R,

    // 搜索顺序是先行后列还是先列后行
    // 通过比较行数和列数的大小来自动决定
    pub column_first: bool,

    // 搜索范围内的所有细胞的列表
    cells: Vec<LifeCell<D>>,

    // 公用的搜索范围外的死细胞
    dead_cell: LifeCell<D>,
}

impl<D: Desc, R: Rule<Desc = D>> World<D, R> {
    pub fn new(
        (width, height, period): Coord,
        dx: isize,
        dy: isize,
        symmetry: Symmetry,
        rule: R,
        column_first: Option<bool>,
    ) -> Self {
        // 自动决定搜索顺序
        let column_first = match column_first {
            Some(c) => c,
            None => {
                let (width, height) = match symmetry {
                    Symmetry::D2Row => ((width + 1) / 2, height),
                    Symmetry::D2Column => (width, (height + 1) / 2),
                    _ => (width, height),
                };
                if width == height {
                    dx.abs() >= dy.abs()
                } else {
                    width > height
                }
            }
        };

        let b0 = rule.b0();

        let mut cells = Vec::with_capacity(((width + 2) * (height + 2) * period) as usize);

        // 先全部填上死细胞；如果是 B0 的规则，则在奇数代填上活细胞
        for id in 0..(width + 2) * (height + 2) * period {
            let t = id % period;
            let state = if b0 && t % 2 == 1 { Alive } else { Dead };
            cells.push(LifeCell::new(id as usize, Some(state), false));
        }

        // 用不到 dead_cell 的 id，随便设一个值
        let dead_cell = LifeCell::new(0, Some(Dead), false);

        let life = World {
            width,
            height,
            period,
            rule,
            column_first,
            cells,
            dead_cell,
        };

        // 先设定细胞的邻域
        // 注意：对于范围边缘的细胞，邻域可能指向不存在的细胞！
        let neighbors = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        for x in -1..=width {
            for y in -1..=height {
                for t in 0..period {
                    let id = life.find_cell((x, y, t)).unwrap();
                    let cell = &life.cells[id];
                    let mut nbhd = cell.nbhd.get();
                    for (i, (nx, ny)) in neighbors.iter().enumerate() {
                        nbhd[i] = life.find_cell((x + nx, y + ny, t));
                    }
                    cell.nbhd.set(nbhd)
                }
            }
        }

        // 再给范围内的细胞添加别的信息
        for x in -1..=width {
            for y in -1..=height {
                for t in 0..period {
                    let id = life.find_cell((x, y, t)).unwrap();
                    let cell = &life.cells[id];

                    // 默认的细胞状态
                    let default = if b0 && t % 2 == 1 { Alive } else { Dead };

                    // 用 set 设置细胞状态
                    if 0 <= x && x < width && 0 <= y && y < height {
                        life.set_cell(cell, None, true);
                    }

                    // 设定前一代；若前一代不在范围内则把此细胞设为 default
                    if t != 0 {
                        cell.pred.set(life.find_cell((x, y, t - 1)));
                    } else {
                        let pred = life.find_cell((x - dx, y - dy, period - 1));
                        if pred.is_some() {
                            cell.pred.set(pred);
                        } else if 0 <= x && x < width && 0 <= y && y < height {
                            life.set_cell(cell, Some(default), false);
                        }
                    }

                    // 设定后一代；若后一代不在范围内则不设
                    if t != period - 1 {
                        cell.succ.set(life.find_cell((x, y, t + 1)));
                    } else {
                        let succ = life.find_cell((x + dx, y + dy, 0));
                        if succ.is_some() {
                            cell.succ.set(succ);
                        }
                    }

                    // 设定对称的细胞；若对称的细胞不在范围内则把此细胞设为 default
                    let sym_coords = match symmetry {
                        Symmetry::C1 => vec![],
                        Symmetry::C2 => vec![(width - 1 - x, height - 1 - y, t)],
                        Symmetry::C4 => vec![
                            (y, width - 1 - x, t),
                            (width - 1 - x, height - 1 - y, t),
                            (height - 1 - y, x, t),
                        ],
                        Symmetry::D2Row => vec![(width - 1 - x, y, t)],
                        Symmetry::D2Column => vec![(x, height - 1 - y, t)],
                        Symmetry::D2Diag => vec![(y, x, t)],
                        Symmetry::D2Antidiag => vec![(height - 1 - y, width - 1 - x, t)],
                        Symmetry::D4Ortho => vec![
                            (width - 1 - x, y, t),
                            (x, height - 1 - y, t),
                            (width - 1 - x, height - 1 - y, t),
                        ],
                        Symmetry::D4Diag => vec![
                            (y, x, t),
                            (height - 1 - y, width - 1 - x, t),
                            (width - 1 - x, height - 1 - y, t),
                        ],
                        Symmetry::D8 => vec![
                            (y, width - 1 - x, t),
                            (height - 1 - y, x, t),
                            (width - 1 - x, y, t),
                            (x, height - 1 - y, t),
                            (y, x, t),
                            (height - 1 - y, width - 1 - x, t),
                            (width - 1 - x, height - 1 - y, t),
                        ],
                    };
                    for coord in sym_coords {
                        let sym = life.find_cell(coord);
                        if 0 <= coord.0 && coord.0 < width && 0 <= coord.1 && coord.1 < height {
                            cell.sym.borrow_mut().push(sym.unwrap());
                        } else if 0 <= x && x < width && 0 <= y && y < height {
                            life.set_cell(cell, Some(default), false);
                        }
                    }
                }
            }
        }

        life
    }

    // 通过坐标查找细胞
    fn find_cell(&self, coord: Coord) -> Option<CellId> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = if self.column_first {
                ((x + 1) * (self.height + 2) + y + 1) * self.period + t
            } else {
                ((y + 1) * (self.width + 2) + x + 1) * self.period + t
            };
            Some(index as usize)
        } else {
            None
        }
    }

    // 设定一个细胞的值，并处理其邻域中所有细胞的邻域状态
    pub fn set_cell(&self, cell: &LifeCell<D>, state: Option<State>, free: bool) {
        let old_state = cell.state.get();
        cell.state.set(state);
        cell.free.set(free);
        D::set_nbhd(&self, &cell, old_state, state);
    }

    // 显示某一代的整个世界
    pub fn display_gen(&self, t: isize) -> String {
        let mut str = String::new();
        let t = t % self.period;
        for y in 0..self.height {
            for x in 0..self.width {
                let state = self[self.find_cell((x, y, t))].state.get();
                let s = match state {
                    Some(Dead) => '.',
                    Some(Alive) => 'O',
                    None => '?',
                };
                str.push(s);
            }
            str.push('\n');
        }
        str
    }

    // 获取一个未知的细胞
    pub fn get_unknown(&self) -> Option<&LifeCell<D>> {
        self.cells.iter().find(|cell| cell.state.get().is_none())
    }

    // 确保搜出来的图样非空，而且最小周期不小于指定的周期
    pub fn nontrivial(&self) -> bool {
        let nonzero = self
            .cells
            .iter()
            .step_by(self.period as usize)
            .any(|c| c.state.get() != Some(Dead));
        nonzero
            && (self.period == 1
                || (1..self.period).all(|t| {
                    self.period % t != 0
                        || self
                            .cells
                            .chunks(self.period as usize)
                            .any(|c| c[0].state.get() != c[t as usize].state.get())
                }))
    }
}

// 通过 Index 来由 id 获取细胞
impl<D: Desc, R: Rule<Desc = D>> Index<CellId> for World<D, R> {
    type Output = LifeCell<D>;

    fn index(&self, id: CellId) -> &Self::Output {
        &self.cells[id]
    }
}

impl<D: Desc, R: Rule<Desc = D>> Index<Option<CellId>> for World<D, R> {
    type Output = LifeCell<D>;

    fn index(&self, id: Option<CellId>) -> &Self::Output {
        match id {
            Some(id) => &self.cells[id],
            None => &self.dead_cell,
        }
    }
}

impl<D: Desc, R: Rule<Desc = D>> Display for World<D, R> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for y in 0..self.height {
            for x in 0..self.width {
                let state = self[self.find_cell((x, y, 0))].state.get();
                let s = match state {
                    Some(Dead) => '.',
                    Some(Alive) => 'O',
                    None => '?',
                };
                write!(f, "{}", s)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
