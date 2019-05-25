use std::cell::{Cell, RefCell};
use std::rc::Weak;

// 细胞状态
#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Dead,
    Alive,
}

// 邻域的状态也应该满足一个 trait
pub trait Desc {
    fn new(state: Option<State>) -> Self;

    // 从邻域的状态还原出细胞本身的状态，None 表示未知
    fn state(&self) -> Option<State>;
}

// 改名 LifeCell 以免和 std::cell::Cell 混淆
// NbhdDesc 包含了细胞邻域的状态和本身的状态
pub struct LifeCell<NbhdDesc: Desc + Copy> {
    // 细胞自身和邻域的状态
    pub desc: Cell<NbhdDesc>,
    // 细胞的状态是否由别的细胞决定
    pub free: Cell<bool>,
    // 同一位置上一代的细胞
    pub pred: RefCell<Weak<LifeCell<NbhdDesc>>>,
    // 同一位置下一代的细胞
    pub succ: RefCell<Weak<LifeCell<NbhdDesc>>>,
    // 细胞的邻域
    pub nbhd: RefCell<Vec<Weak<LifeCell<NbhdDesc>>>>,
    // 与此细胞对称（因此状态一致）的细胞
    pub sym: RefCell<Vec<Weak<LifeCell<NbhdDesc>>>>,
}

impl<NbhdDesc: Desc + Copy> LifeCell<NbhdDesc> {
    pub fn new(state: Option<State>, free: bool) -> Self {
        let desc = Cell::new(NbhdDesc::new(state));
        let free = Cell::new(free);
        let pred = RefCell::new(Weak::new());
        let succ = RefCell::new(Weak::new());
        let nbhd = RefCell::new(vec![]);
        let sym = RefCell::new(vec![]);
        LifeCell {desc, free, pred, succ, nbhd, sym}
    }

    // 获取一个细胞的状态
    pub fn state(&self) -> Option<State> {
        self.desc.get().state()
    }
}

// 写成一个 Trait，方便以后支持更多的规则
// Index 代表细胞的索引，由细胞的位置和时间决定
pub trait World<NbhdDesc: Desc + Copy> {
    // 世界的大小，即所有回合的细胞总数
    fn size(&self) -> usize;

    // 获取一个未知的细胞
    fn get_unknown(&self) -> Weak<LifeCell<NbhdDesc>>;

    // 设定一个细胞的值，并处理其邻域中所有细胞的邻域状态
    fn set_cell(cell: &LifeCell<NbhdDesc>, state: Option<State>, free: bool);

    // 由一个细胞及其邻域的状态得到其后一代的状态
    fn transition(&self, desc: NbhdDesc) -> Option<State>;

    // 由一个细胞本身、邻域以及其后一代的状态，决定其本身或者邻域中某些未知细胞的状态
    // implication表示本身的状态，implication_nbhd表示邻域中未知细胞的状态
    // 这样写并不好扩展到 non-totalistic 的规则的情形，不过以后再说吧
    fn implication(&self, desc: NbhdDesc, succ_state: State) -> Option<State>;
    fn implication_nbhd(&self, desc: NbhdDesc, succ_state: State) -> Option<State>;

    // 确保搜振荡子不会搜出静物，或者周期比指定的要小的振荡子
    fn subperiod(&self) -> bool;

    // 输出图样
    fn display(&self);
}
