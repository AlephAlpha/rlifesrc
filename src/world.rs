use std::cell::{Cell, RefCell};
use std::rc::Weak;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Dead,
    Alive,
}

// 邻域的状态也应该满足一个 trait
pub trait Desc {
    fn new(state: Option<State>) -> Self;

    // 从邻域的状态还原出细胞本身的状态
    fn state(&self) -> Option<State>;

    // 由一个细胞及其邻域的状态得到其后一代的状态
    fn transition(&self) -> Option<State>;

    // 由一个细胞本身、邻域以及其后一代的状态，决定其本身或者邻域中某些未知细胞的状态
    // implication表示本身的状态，implication_nbhd表示邻域中未知细胞的状态
    // 这样写并不好扩展到 non-totalistic 的规则的情形，不过以后再说吧
    fn implication(&self, succ_state: State) -> Option<State>;
    fn implication_nbhd(&self, succ_state: State) -> Option<State>;
}

// 改名 LifeCell 以免和 std::cell::Cell 混淆
pub struct LifeCell<N: Desc + Copy> {
    pub desc: Cell<N>,
    pub free: Cell<bool>,
    pub pred: RefCell<Weak<LifeCell<N>>>,
    pub succ: RefCell<Weak<LifeCell<N>>>,
    pub nbhd: RefCell<Vec<Weak<LifeCell<N>>>>,
    pub sym: RefCell<Vec<Weak<LifeCell<N>>>>,
}

impl<N: Desc + Copy> LifeCell<N> {
    pub fn new(state: Option<State>, free: bool) -> Self {
        let desc = Cell::new(N::new(state));
        let free = Cell::new(free);
        let pred = RefCell::new(Weak::new());
        let succ = RefCell::new(Weak::new());
        let nbhd = RefCell::new(vec![]);
        let sym = RefCell::new(vec![]);
        LifeCell {desc, free, pred, succ, nbhd, sym}
    }

    pub fn state(&self) -> Option<State> {
        self.desc.get().state()
    }
}

// 写成一个 Trait，方便以后支持更多的规则
// Index 代表细胞的索引，由细胞的位置和时间决定
pub trait World<N: Desc + Copy> {
    // 世界的大小，即所有回合的细胞总数
    fn size(&self) -> usize;

    // 获取一个未知的细胞
    fn get_unknown(&self) -> Weak<LifeCell<N>>;

    // 设定一个细胞的值，并处理其邻域中所有细胞的邻域状态
    fn set_cell(cell: &LifeCell<N>, state: Option<State>, free: bool);

    // 确保搜振荡子不会搜出静物，或者周期比指定的要小的振荡子
    fn subperiod(&self) -> bool;

    // 输出图样
    fn display(&self);
}
