use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use State::{Dead, Alive};

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

pub type WeakCell<D> = Weak<LifeCell<D>>;
pub type RcCell<D> = Rc<LifeCell<D>>;

// D 表示邻域的状态
pub struct LifeCell<D> {
    // 细胞自身的状态
    pub state: Cell<Option<State>>,
    // 细胞邻域的状态
    pub desc: Cell<D>,
    // 细胞的状态是否由别的细胞决定
    pub free: Cell<bool>,
    // 同一位置上一代的细胞
    pub pred: RefCell<WeakCell<D>>,
    // 同一位置下一代的细胞
    pub succ: RefCell<WeakCell<D>>,
    // 细胞的邻域
    pub nbhd: RefCell<[WeakCell<D>; 8]>,
    // 与此细胞对称（因此状态一致）的细胞
    pub sym: RefCell<Vec<WeakCell<D>>>,
}

impl<D: Desc> LifeCell<D> {
    pub fn new(state: Option<State>, free: bool) -> Self {
        let desc = Cell::new(D::new(state));
        let state = Cell::new(state);
        let free = Cell::new(free);
        let pred = Default::default();
        let succ = Default::default();
        let nbhd = Default::default();
        let sym = Default::default();
        LifeCell {state, desc, free, pred, succ, nbhd, sym}
    }

    // 设定一个细胞的值，并处理其邻域中所有细胞的邻域状态
    pub fn set(&self, state: Option<State>, free: bool) {
        let old_state = self.state.get();
        self.state.set(state);
        self.free.set(free);
        D::set_nbhd(&self, old_state, state);
    }
}

// 邻域的状态应该满足一个 trait
pub trait Desc: Copy {
    // 通过一个细胞的状态生成一个默认的邻域
    fn new(state: Option<State>) -> Self;

    // 改变一个细胞的状态时处理其邻域中所有细胞的邻域状态
    fn set_nbhd(cell: &LifeCell<Self>, old_state: Option<State>, state: Option<State>);
}
