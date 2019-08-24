use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cell::Cell;
use std::fmt::{Debug, Error, Formatter};
use std::str::FromStr;
pub use State::{Alive, Dead};

#[cfg(feature = "stdweb")]
use serde::{Deserialize, Serialize};

/// 细胞状态
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
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

/// 细胞
///
/// `D` 表示邻域的状态。
pub struct LifeCell<'a, D: Desc> {
    /// 细胞自身的状态
    pub state: Cell<Option<State>>,
    /// 细胞邻域的状态
    pub desc: Cell<D>,
    /// 细胞的状态是否由别的细胞决定
    pub free: Cell<bool>,
    /// 同一位置上一代的细胞
    pub pred: Option<&'a LifeCell<'a, D>>,
    /// 同一位置下一代的细胞
    pub succ: Option<&'a LifeCell<'a, D>>,
    /// 细胞的邻域
    pub nbhd: [Option<&'a LifeCell<'a, D>>; 8],
    /// 与此细胞对称（因此状态一致）的细胞
    pub sym: Vec<&'a LifeCell<'a, D>>,
    /// 此细胞是否在第一代
    pub first_gen: bool,
    /// 此细胞是否在第一行/第一列
    pub first_col: bool,
}

impl<'a, D: Desc> LifeCell<'a, D> {
    pub fn new(state: Option<State>, free: bool, first_gen: bool, first_col: bool) -> Self {
        LifeCell {
            state: Cell::new(state),
            desc: Cell::new(D::new(state)),
            free: Cell::new(free),
            pred: Default::default(),
            succ: Default::default(),
            nbhd: Default::default(),
            sym: Default::default(),
            first_gen,
            first_col,
        }
    }
}

/// 邻域的状态
pub trait Desc: Copy {
    /// 通过一个细胞的状态生成一个默认的邻域
    fn new(state: Option<State>) -> Self;

    /// 改变一个细胞的状态时处理其邻域中所有细胞的邻域状态
    fn set_nbhd(cell: &LifeCell<Self>, old_state: Option<State>, state: Option<State>);
}

/// 规则
pub trait Rule: Sized {
    type Desc: Desc;

    /// 规则是否包含 B0
    ///
    /// 也就是说，如果一个细胞邻域中全部细胞都是死的，此细胞下一代会不会活过来。
    fn b0(&self) -> bool;

    /// 由一个细胞及其邻域的状态得到其后一代的状态
    fn transition(&self, state: Option<State>, desc: Self::Desc) -> Option<State>;

    /// 由一个细胞的邻域以及其后一代的状态，决定其本身的状态
    fn implication(&self, desc: Self::Desc, succ_state: State) -> Option<State>;

    /// 由一个细胞本身、邻域以及其后一代的状态，改变其邻域中某些未知细胞的状态，
    /// 并把改变了值的细胞放到 `set_table` 中。
    fn consistify_nbhd<'a>(
        &self,
        cell: &LifeCell<'a, Self::Desc>,
        world: &World<'a, Self::Desc, Self>,
        desc: Self::Desc,
        state: Option<State>,
        succ_state: State,
        set_table: &mut Vec<&'a LifeCell<'a, Self::Desc>>,
    );
}

/// 变换。旋转或翻转
///
/// 8 个不同的值，对应二面体群 D8 的 8 个元素。
///
/// `Id` 表示恒等变换。
///
/// `R` 表示旋转（Rotate）， 后面的数字表示逆时针旋转的角度。
///
/// `F` 表示翻转（Flip）， 后面的符号表示翻转的轴线。
///
/// 注意有些变换仅在世界是正方形时才有意义。
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
pub enum Transform {
    Id,
    Rotate90,
    Rotate180,
    Rotate270,
    FlipRow,
    FlipColumn,
    FlipDiag,
    FlipAntidiag,
}

impl FromStr for Transform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Id" => Ok(Transform::Id),
            "R90" => Ok(Transform::Rotate90),
            "R180" => Ok(Transform::Rotate180),
            "R270" => Ok(Transform::Rotate270),
            "F|" => Ok(Transform::FlipRow),
            "F-" => Ok(Transform::FlipColumn),
            "F\\" => Ok(Transform::FlipDiag),
            "F/" => Ok(Transform::FlipAntidiag),
            _ => Err(String::from("invalid Transform")),
        }
    }
}

impl Debug for Transform {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Transform::Id => "Id",
            Transform::Rotate90 => "R90",
            Transform::Rotate180 => "R180",
            Transform::Rotate270 => "R270",
            Transform::FlipRow => "F|",
            Transform::FlipColumn => "F-",
            Transform::FlipDiag => "F\\",
            Transform::FlipAntidiag => "F/",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::Id
    }
}

impl Transform {
    /// 此变换是否要求世界是正方形
    pub fn square_world(self) -> bool {
        match self {
            Transform::Rotate90
            | Transform::Rotate270
            | Transform::FlipDiag
            | Transform::FlipAntidiag => true,
            _ => false,
        }
    }
}

/// 对称性
///
/// 10 个不同的值，对应二面体群 D8 的 10 个子群。
///
/// 这些符号的含义和 Oscar Cunningham 的 Logic Life Search 一样。
///
/// 注意有些对称性仅在世界是正方形时才有意义。
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "stdweb", derive(Serialize, Deserialize))]
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

impl Debug for Symmetry {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Symmetry::C1 => "C1",
            Symmetry::C2 => "C2",
            Symmetry::C4 => "C4",
            Symmetry::D2Row => "D2|",
            Symmetry::D2Column => "D2-",
            Symmetry::D2Diag => "D2\\",
            Symmetry::D2Antidiag => "D2/",
            Symmetry::D4Ortho => "D4+",
            Symmetry::D4Diag => "D4X",
            Symmetry::D8 => "D8",
        };
        write!(f, "{}", s)?;
        Ok(())
    }
}

impl Default for Symmetry {
    fn default() -> Self {
        Symmetry::C1
    }
}

impl Symmetry {
    /// 此对称性是否要求世界是正方形
    pub fn square_world(self) -> bool {
        match self {
            Symmetry::C4
            | Symmetry::D2Diag
            | Symmetry::D2Antidiag
            | Symmetry::D4Diag
            | Symmetry::D8 => true,
            _ => false,
        }
    }
}

/// 细胞的坐标
///
/// （横座标，纵座标，时间）
pub type Coord = (isize, isize, isize);

/// 世界
pub struct World<'a, D: Desc, R: 'a + Rule<Desc = D>> {
    /// 宽度
    pub width: isize,
    /// 高度
    pub height: isize,
    /// 周期
    pub period: isize,
    /// 规则
    pub rule: R,

    /// 搜索顺序是先行后列还是先列后行
    ///
    /// 通过比较行数和列数的大小来自动决定。
    pub column_first: bool,

    /// 搜索范围内的所有细胞的列表
    ///
    /// 此列表在创建了世界之后不会再动，里边所有的细胞都会活到寿终正寝之时，
    /// 因此后面的 unsafe 是安全的。
    cells: Vec<LifeCell<'a, D>>,

    /// 公用的搜索范围外的死细胞
    ///
    /// 如果一个细胞的下一代超出了搜索范围，其 `succ` 会设为 `dead_cell`。
    dead_cell: LifeCell<'a, D>,

    /// 已知的活细胞个数
    pub cell_count: Cell<u32>,
}

impl<'a, D: Desc, R: 'a + Rule<Desc = D>> World<'a, D, R> {
    /// 新建一个世界
    pub fn new(
        (width, height, period): Coord,
        dx: isize,
        dy: isize,
        transform: Transform,
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

        let mut cells = Vec::with_capacity(((width + 2) * (height + 2) * period) as usize);

        // 先全部填上死细胞；如果是 B0 的规则，则在奇数代填上活细胞
        let (w, h) = if column_first {
            (width, height)
        } else {
            (height, width)
        };
        for x in -1..=w {
            for _y in -1..=h {
                for t in 0..period {
                    let state = if rule.b0() && t % 2 == 1 { Alive } else { Dead };
                    let first_gen = t == 0;
                    let first_col = x == 0;
                    cells.push(LifeCell::new(Some(state), false, first_gen, first_col));
                }
            }
        }

        let dead_cell = LifeCell::new(Some(Dead), false, false, false);

        let cell_count = Cell::new(0);

        let mut world = World {
            width,
            height,
            period,
            rule,
            column_first,
            cells,
            dead_cell,
            cell_count,
        };

        world.init(dx, dy, transform, symmetry);
        world
    }

    /// 初始化整个世界
    fn init(&mut self, dx: isize, dy: isize, transform: Transform, symmetry: Symmetry) {
        // 先设定细胞的邻域，以便后面 `set_cell`
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
        for x in -1..=self.width {
            for y in -1..=self.height {
                for t in 0..self.period {
                    let cell_ptr: *mut _ = self.find_cell_mut((x, y, t)).unwrap();
                    for (i, (nx, ny)) in neighbors.iter().enumerate() {
                        if let Some(neigh) = self.find_cell((x + nx, y + ny, t)) {
                            let neigh: *const _ = neigh;
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.nbhd[i] = neigh.as_ref();
                            }
                        }
                    }
                }
            }
        }

        // 再给范围内的细胞添加别的信息
        for x in -1..=self.width {
            for y in -1..=self.height {
                for t in 0..self.period {
                    let cell_ptr: *mut _ = self.find_cell_mut((x, y, t)).unwrap();
                    let cell = self.find_cell((x, y, t)).unwrap();

                    // 默认的细胞状态
                    let default = if self.rule.b0() && t % 2 == 1 {
                        Alive
                    } else {
                        Dead
                    };

                    // 用 `set` 设置细胞状态
                    if 0 <= x && x < self.width && 0 <= y && y < self.height {
                        self.set_cell(cell, None, true);
                    }

                    // 设定前一代；若前一代不在范围内则把原细胞的状态设为 `default`
                    if t != 0 {
                        let pred: *const _ = self.find_cell((x, y, t - 1)).unwrap();
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.pred = pred.as_ref();
                        }
                    } else {
                        let (new_x, new_y) = match transform {
                            Transform::Id => (x, y),
                            Transform::Rotate90 => (self.height - 1 - y, x),
                            Transform::Rotate180 => (self.width - 1 - x, self.height - 1 - y),
                            Transform::Rotate270 => (y, self.width - 1 - x),
                            Transform::FlipRow => (self.width - 1 - x, y),
                            Transform::FlipColumn => (x, self.height - 1 - y),
                            Transform::FlipDiag => (y, x),
                            Transform::FlipAntidiag => (self.height - 1 - y, self.width - 1 - x),
                        };
                        let pred = self.find_cell((new_x - dx, new_y - dy, self.period - 1));
                        if let Some(pred) = pred {
                            let pred: *const _ = pred;
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.pred = pred.as_ref();
                            }
                        } else if 0 <= x && x < self.width && 0 <= y && y < self.height {
                            self.set_cell(cell, Some(default), false);
                        }
                    }

                    // 设定后一代；若后一代不在范围内则设其 `succ` 为 `dead_cell`
                    if t != self.period - 1 {
                        let succ: *const _ = self.find_cell((x, y, t + 1)).unwrap();
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = succ.as_ref();
                        }
                    } else {
                        let (new_x, new_y) = match transform {
                            Transform::Id => (x, y),
                            Transform::Rotate90 => (y, self.width - 1 - x),
                            Transform::Rotate180 => (self.width - 1 - x, self.height - 1 - y),
                            Transform::Rotate270 => (self.height - 1 - y, x),
                            Transform::FlipRow => (self.width - 1 - x, y),
                            Transform::FlipColumn => (x, self.height - 1 - y),
                            Transform::FlipDiag => (y, x),
                            Transform::FlipAntidiag => (self.height - 1 - y, self.width - 1 - x),
                        };
                        let succ = self.find_cell((new_x + dx, new_y + dy, 0));
                        let succ: *const _ = if let Some(succ) = succ {
                            succ
                        } else {
                            &self.dead_cell
                        };
                        unsafe {
                            let cell = cell_ptr.as_mut().unwrap();
                            cell.succ = succ.as_ref();
                        }
                    }

                    // 设定对称的细胞；若对称的细胞不在范围内则把原细胞的状态设为 `default`
                    let sym_coords = match symmetry {
                        Symmetry::C1 => vec![],
                        Symmetry::C2 => vec![(self.width - 1 - x, self.height - 1 - y, t)],
                        Symmetry::C4 => vec![
                            (y, self.width - 1 - x, t),
                            (self.width - 1 - x, self.height - 1 - y, t),
                            (self.height - 1 - y, x, t),
                        ],
                        Symmetry::D2Row => vec![(self.width - 1 - x, y, t)],
                        Symmetry::D2Column => vec![(x, self.height - 1 - y, t)],
                        Symmetry::D2Diag => vec![(y, x, t)],
                        Symmetry::D2Antidiag => vec![(self.height - 1 - y, self.width - 1 - x, t)],
                        Symmetry::D4Ortho => vec![
                            (self.width - 1 - x, y, t),
                            (x, self.height - 1 - y, t),
                            (self.width - 1 - x, self.height - 1 - y, t),
                        ],
                        Symmetry::D4Diag => vec![
                            (y, x, t),
                            (self.height - 1 - y, self.width - 1 - x, t),
                            (self.width - 1 - x, self.height - 1 - y, t),
                        ],
                        Symmetry::D8 => vec![
                            (y, self.width - 1 - x, t),
                            (self.height - 1 - y, x, t),
                            (self.width - 1 - x, y, t),
                            (x, self.height - 1 - y, t),
                            (y, x, t),
                            (self.height - 1 - y, self.width - 1 - x, t),
                            (self.width - 1 - x, self.height - 1 - y, t),
                        ],
                    };
                    for coord in sym_coords {
                        if 0 <= coord.0
                            && coord.0 < self.width
                            && 0 <= coord.1
                            && coord.1 < self.height
                        {
                            let sym: *const _ = self.find_cell(coord).unwrap();
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.sym.push(sym.as_ref().unwrap());
                            }
                        } else if 0 <= x && x < self.width && 0 <= y && y < self.height {
                            self.set_cell(cell, Some(default), false);
                        }
                    }
                }
            }
        }
    }

    /// 通过坐标查找细胞
    fn find_cell(&self, coord: Coord) -> Option<&LifeCell<'a, D>> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = if self.column_first {
                ((x + 1) * (self.height + 2) + y + 1) * self.period + t
            } else {
                ((y + 1) * (self.width + 2) + x + 1) * self.period + t
            };
            Some(&self.cells[index as usize])
        } else {
            None
        }
    }

    /// 通过坐标查找细胞
    fn find_cell_mut(&mut self, coord: Coord) -> Option<&mut LifeCell<'a, D>> {
        let (x, y, t) = coord;
        if x >= -1 && x <= self.width && y >= -1 && y <= self.height {
            let index = if self.column_first {
                ((x + 1) * (self.height + 2) + y + 1) * self.period + t
            } else {
                ((y + 1) * (self.width + 2) + x + 1) * self.period + t
            };
            Some(&mut self.cells[index as usize])
        } else {
            None
        }
    }

    /// 设定一个细胞的值，并处理其邻域中所有细胞的邻域状态
    pub fn set_cell(&self, cell: &LifeCell<D>, state: Option<State>, free: bool) {
        let old_state = cell.state.replace(state);
        cell.free.set(free);
        D::set_nbhd(&cell, old_state, state);
        if cell.first_gen {
            match (state, old_state) {
                (Some(Alive), Some(Alive)) => (),
                (Some(Alive), _) => self.cell_count.set(self.cell_count.get() + 1),
                (_, Some(Alive)) => self.cell_count.set(self.cell_count.get() - 1),
                _ => (),
            }
        }
    }

    /// 显示某一代的整个世界
    pub fn display_gen(&self, t: isize) -> String {
        let mut str = String::new();
        let t = t % self.period;
        for y in 0..self.height {
            for x in 0..self.width {
                let state = self.find_cell((x, y, t)).unwrap().state.get();
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

    /// 获取一个未知的细胞
    pub fn get_unknown(&self) -> Option<&LifeCell<'a, D>> {
        self.cells.iter().find(|cell| cell.state.get().is_none())
    }

    /// 确保搜出来的图样非空，而且最小周期不小于指定的周期
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
