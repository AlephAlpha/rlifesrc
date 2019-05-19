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
// Index 代表细胞的索引，由细胞的位置和时间决定
pub trait World<Index: Copy> {
    // 用一个类型来记录邻域的状态
    type NbhdState;

    // 世界的大小，即所有回合的细胞总数
    fn size(&self) -> usize;

    fn get_cell(&self, ix: Index) -> Cell;
    fn set_cell(&mut self, ix: Index, cell: Cell);

    // 细胞的邻域
    fn neighbors(&self, ix: Index) -> Vec<Index>;
    // 同一位置前一代的细胞
    fn pred(&self, ix: Index) -> Index;
    // 同一位置后一代的细胞
    fn succ(&self, ix: Index) -> Index;

    // 按照对称性和一个细胞状态一致的所有细胞
    fn sym(&self, ix: Index) -> Vec<Index>;

    // 获取一个未知的细胞
    fn get_unknown(&self) -> Option<Index>;

    // 从邻域的列表得到邻域的状态
    fn nbhd_state(&self, neighbors: Vec<Index>) -> Self::NbhdState;
    // 由一个细胞及其邻域的状态得到其后一代的状态
    fn transit(cell: Cell, nbhd: &Self::NbhdState) -> State;

    // 由一个细胞本身、邻域以及其后一代的状态，决定其本身或者邻域中某些未知细胞的状态
    // 返回两个值，一个表示本身的状态，另一个表示邻域中未知细胞的状态
    // 这样写并不好扩展到 non-totalistic 的规则的情形，不过以后再说吧
    fn implic(cell: Cell, nbhd: &Self::NbhdState, succ: Cell) -> (State, State);

    // 确保搜振荡子不会搜出静物，或者周期比指定的要小的振荡子
    fn subperiod(&self) -> bool;

    // 输出图样
    fn display(&self);
}