use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Dead,
    Alive,
}

// 改名 LifeCell 以免和 std::cell::Cell 混淆
pub struct LifeCell {
    pub state: Cell<Option<State>>,
    pub free: Cell<bool>,
    pub pred: RefCell<Weak<LifeCell>>,
    pub succ: RefCell<Weak<LifeCell>>,
    pub nbhd: RefCell<Vec<Weak<LifeCell>>>,
    pub sym: RefCell<Vec<Weak<LifeCell>>>,
}

impl LifeCell {
    pub fn new(state: Option<State>, free: bool) -> Self {
        let state = Cell::new(state);
        let free = Cell::new(free);
        let pred = RefCell::new(Weak::new());
        let succ = RefCell::new(Weak::new());
        let nbhd = RefCell::new(vec![]);
        let sym = RefCell::new(vec![]);
        LifeCell {state, free, pred, succ, nbhd, sym}
    }

    pub fn new_rc(state: Option<State>, free: bool) -> Rc<LifeCell> {
        Rc::new(LifeCell::new(state, free))
    }
}

// 写成一个 Trait，方便以后支持更多的规则
// Index 代表细胞的索引，由细胞的位置和时间决定
pub trait World {
    // 用一个类型来记录邻域的状态
    type NbhdDesc;

    // 世界的大小，即所有回合的细胞总数
    fn size(&self) -> usize;

    // 获取一个未知的细胞
    fn get_unknown(&self) -> Option<Rc<LifeCell>>;

    // 一个细胞邻域的状态
    fn get_desc(cell: &LifeCell) -> Self::NbhdDesc;

    // 由一个细胞及其邻域的状态得到其后一代的状态
    fn transition(desc: &Self::NbhdDesc) -> Option<State>;

    // 由一个细胞本身、邻域以及其后一代的状态，决定其本身或者邻域中某些未知细胞的状态
    // implication表示本身的状态，implication_nbhd表示邻域中未知细胞的状态
    // 这样写并不好扩展到 non-totalistic 的规则的情形，不过以后再说吧
    fn implication(nbhd: &Self::NbhdDesc, succ_state: State) -> Option<State>;
    fn implication_nbhd(nbhd: &Self::NbhdDesc, succ_state: State) -> Option<State>;

    // 确保搜振荡子不会搜出静物，或者周期比指定的要小的振荡子
    fn subperiod(&self) -> bool;

    // 输出图样
    fn display(&self);
}
