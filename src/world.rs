use std::cell::RefCell;
use std::rc::{Rc,Weak};

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Dead,
    Alive,
}

pub struct Cell {
    pub state: Option<State>,
    pub free: bool,     // 此细胞的状态是否取决于其它细胞的状态
    pub pred: Weak<RefCell<Cell>>,
    pub succ: Weak<RefCell<Cell>>,
    pub nbhd: Vec<Weak<RefCell<Cell>>>,
    pub sym: Vec<Weak<RefCell<Cell>>>,
}

impl Cell {
    pub fn new(state: Option<State>, free: bool) -> Self {
        let pred = Weak::new();
        let succ = Weak::new();
        let nbhd = vec![];
        let sym = vec![];
        Cell {state, free, pred, succ, nbhd, sym}
    }

    pub fn new_rc(state: Option<State>, free: bool) -> Rc<RefCell<Cell>> {
        Rc::new(RefCell::new(Cell::new(state, free)))
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
    fn get_unknown(&self) -> Option<Rc<RefCell<Cell>>>;

    // 一个细胞邻域的状态
    fn get_desc(cell: &Cell) -> Self::NbhdDesc;

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
