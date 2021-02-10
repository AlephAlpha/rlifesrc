#![feature(prelude_import)]
//! __Rust Life Search__, or __rlifesrc__,
//! is a Game of Life pattern searcher written in Rust.
//!
//! The program is based on David Bell's
//! [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)
//! and Jason Summers's [WinLifeSearch](https://github.com/jsummers/winlifesearch/),
//! using [an algorithm invented by Dean Hickerson](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN).
//!
//! Compared to WinLifeSearch, rlifesrc is still slower, and lacks many important features.
//! But it supports non-totalistic Life-like rules.
//!
//! This is the library for rlifesrc. There is also a
//! [command-line tool with a TUI](https://github.com/AlephAlpha/rlifesrc/tree/master/tui)
//! and a [web app using WebAssembly](https://github.com/AlephAlpha/rlifesrc/tree/master/web).
//!
//! You can try the web app [here](https://alephalpha.github.io/rlifesrc/).
//!
//! # Example
//!
//! Finds the [25P3H1V0.1](https://conwaylife.com/wiki/25P3H1V0.1) spaceship.
//!
//! ```rust
//! use rlifesrc_lib::{Config, Status};
//!
//! // Configures the world.
//! let config = Config::new(16, 5, 3).set_translate(0, 1);
//!
//! // Creates the world.
//! let mut search = config.world().unwrap();
//!
//! // Searches and displays the generation 0 of the result.
//! if let Status::Found = search.search(None) {
//!     println!("{}", search.rle_gen(0))
//! }
//! ```
//!
//! Search result:
//!
//! ``` plaintext
//! x = 16, y = 5, rule = B3/S23
//! ........o.......$
//! .oo.ooo.ooo.....$
//! .oo....o..oo.oo.$
//! o..o.oo...o..oo.$
//! ............o..o!
//! ```
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
mod cells {
    //! Cells in the cellular automaton.
    use crate::rules::Rule;
    use derivative::Derivative;
    use std::{
        cell::Cell,
        fmt::{Debug, Error, Formatter},
        ops::{Deref, Not},
    };
    /// Possible states of a known cell.
    ///
    /// During the search, the state of a cell is represented by `Option<State>`,
    /// where `None` means that the state of the cell is unknown.
    pub struct State(pub usize);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for State {
        #[inline]
        fn clone(&self) -> State {
            {
                let _: ::core::clone::AssertParamIsClone<usize>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for State {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for State {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                State(ref __self_0_0) => {
                    let mut debug_trait_builder = f.debug_tuple("State");
                    let _ = debug_trait_builder.field(&&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for State {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for State {
        #[inline]
        fn eq(&self, other: &State) -> bool {
            match *other {
                State(ref __self_1_0) => match *self {
                    State(ref __self_0_0) => (*__self_0_0) == (*__self_1_0),
                },
            }
        }
        #[inline]
        fn ne(&self, other: &State) -> bool {
            match *other {
                State(ref __self_1_0) => match *self {
                    State(ref __self_0_0) => (*__self_0_0) != (*__self_1_0),
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for State {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for State {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<usize>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for State {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match *self {
                State(ref __self_0_0) => ::core::hash::Hash::hash(&(*__self_0_0), state),
            }
        }
    }
    /// The Dead state.
    pub const DEAD: State = State(0);
    /// The Alive state.
    pub const ALIVE: State = State(1);
    /// Flips the state.
    ///
    /// For Generations rules, the `not` of a dying state is [`ALIVE`].
    impl Not for State {
        type Output = State;
        fn not(self) -> Self::Output {
            match self {
                ALIVE => DEAD,
                _ => ALIVE,
            }
        }
    }
    /// The coordinates of a cell.
    ///
    /// `(x-coordinate, y-coordinate, time)`.
    /// All three coordinates are 0-indexed.
    pub type Coord = (i32, i32, i32);
    /// A cell in the cellular automaton.
    ///
    /// The name `LifeCell` is chosen to avoid ambiguity with
    /// [`std::cell::Cell`].
    pub struct LifeCell<'a, R: Rule> {
        /// The coordinates of a cell.
        pub coord: Coord,
        /// The background state of the cell.
        ///
        /// For rules without `B0`, it is always [`DEAD`].
        ///
        /// For rules with `B0`, the background changes periodically.
        /// For example, for non-Generations rules, it is [`DEAD`] on even generations,
        /// [`ALIVE`] on odd generations.
        pub(crate) background: State,
        /// The state of the cell.
        ///
        /// `None` means that the state of the cell is unknown.
        pub(crate) state: Cell<Option<State>>,
        /// The “neighborhood descriptors” of the cell.
        ///
        /// It describes the states of the cell itself, its neighbors,
        /// and its successor.
        pub(crate) desc: Cell<R::Desc>,
        /// The predecessor of the cell.
        ///
        /// The cell in the last generation at the same position.
        pub(crate) pred: Option<CellRef<'a, R>>,
        /// The successor of the cell.
        ///
        /// The cell in the next generation at the same position.
        pub(crate) succ: Option<CellRef<'a, R>>,
        /// The eight cells in the neighborhood.
        pub(crate) nbhd: [Option<CellRef<'a, R>>; 8],
        /// The cells in the same generation that must has the same state
        /// with this cell because of the symmetry.
        pub(crate) sym: Vec<CellRef<'a, R>>,
        /// The next cell to be searched when searching for an unknown cell.
        pub(crate) next: Option<CellRef<'a, R>>,
        /// Whether the cell is on the first row or column.
        ///
        /// Here the choice of row or column depends on the search order.
        pub(crate) is_front: bool,
    }
    impl<'a, R: Rule> LifeCell<'a, R> {
        /// Generates a new cell with background state, such that its neighborhood
        /// descriptor says that all neighboring cells also have the same state.
        ///
        /// `is_front` are set to `false`.
        pub(crate) fn new(coord: Coord, background: State, succ_state: State) -> Self {
            LifeCell {
                coord,
                background,
                state: Cell::new(Some(background)),
                desc: Cell::new(R::new_desc(background, succ_state)),
                pred: Default::default(),
                succ: Default::default(),
                nbhd: Default::default(),
                sym: Default::default(),
                next: Default::default(),
                is_front: false,
            }
        }
        /// Returns a [`CellRef`] from a [`LifeCell`].
        pub(crate) fn borrow(&self) -> CellRef<'a, R> {
            let cell = unsafe { (self as *const LifeCell<'a, R>).as_ref().unwrap() };
            CellRef { cell }
        }
    }
    impl<'a, R: Rule<Desc = D>, D: Copy + Debug> Debug for LifeCell<'a, R> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            f.debug_struct("LifeCell")
                .field("coord", &self.coord)
                .field("state", &self.state.get())
                .field("desc", &self.desc.get())
                .finish()
        }
    }
    /// A reference to a [`LifeCell`] which has the same lifetime as the cell
    /// it refers to.
    #[derivative(Clone(bound = ""), Copy(bound = ""))]
    pub struct CellRef<'a, R: Rule> {
        cell: &'a LifeCell<'a, R>,
    }
    #[allow(unused_qualifications)]
    impl<'a, R: Rule> ::std::clone::Clone for CellRef<'a, R> {
        fn clone(&self) -> Self {
            match *self {
                CellRef { cell: ref __arg_0 } => CellRef {
                    cell: (*__arg_0).clone(),
                },
            }
        }
    }
    #[allow(unused_qualifications)]
    impl<'a, R: Rule> ::std::marker::Copy for CellRef<'a, R> {}
    impl<'a, R: Rule> CellRef<'a, R> {
        /// Updates the neighborhood descriptors of all neighbors and the predecessor
        /// when the state of one cell is changed.
        ///
        /// Here `state` is the new state of the cell when `new` is true,
        /// the old state when `new` is false.
        pub(crate) fn update_desc(self, state: Option<State>, new: bool) {
            R::update_desc(self, state, new);
        }
    }
    impl<'a, R: Rule> PartialEq for CellRef<'a, R> {
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(self.cell, other.cell)
        }
    }
    impl<'a, R: Rule> Eq for CellRef<'a, R> {}
    impl<'a, R: Rule> Deref for CellRef<'a, R> {
        type Target = LifeCell<'a, R>;
        fn deref(&self) -> &Self::Target {
            self.cell
        }
    }
    impl<'a, R: Rule<Desc = D>, D: Copy + Debug> Debug for CellRef<'a, R> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            f.debug_struct("CellRef")
                .field("coord", &self.coord)
                .finish()
        }
    }
}
mod config {
    //! World configuration.
    use crate::{
        cells::{Coord, State},
        error::Error,
        rules::{Life, LifeGen, NtLife, NtLifeGen, Rule},
        traits::Search,
        world::World,
    };
    use derivative::Derivative;
    mod d8 {
        //! Configurations related to the
        //! [dihedral group _D_<sub>8</sub>](https://en.wikipedia.org/wiki/Examples_of_groups#dihedral_group_of_order_8).
        //!
        //! 8 different transformations correspond to 8 elements of _D_<sub>8</sub>.
        //!
        //! 10 different symmetries correspond to 10 subgroups of _D_<sub>8</sub>.
        use super::{Config, Coord};
        use derivative::Derivative;
        use std::{
            cmp::Ordering,
            fmt::{self, Display, Formatter},
            matches,
            ops::Mul,
            str::FromStr,
            vec,
        };
        /// Transformations (rotations and reflections) after the last generation
        /// in a period.
        ///
        /// After the last generation in a period, the pattern will return to
        /// the first generation, applying this transformation first,
        /// and then the translation defined by `dx` and `dy`.
        ///
        /// 8 different values correspond to 8 elements of the dihedral group
        /// _D_<sub>8</sub>.
        ///
        /// `Id` is the identity transformation.
        ///
        /// `R` means rotations around the center of the world.
        /// The number after it is the counterclockwise rotation angle in degrees.
        ///
        /// `F` means reflections (flips).
        /// The symbol after it is the axis of reflection.
        ///
        /// Some of the transformations are only valid when the world is square.
        /// and some are only valid when the world has no diagonal width.
        #[derivative(Default)]
        pub enum Transform {
            /// `Id`.
            ///
            /// Identity transformation.
            #[derivative(Default)]
            Id,
            /// `R90`.
            ///
            /// 90° rotation counterclockwise.
            ///
            /// Requires the world to be square and have no diagonal width.
            Rotate90,
            /// `R180`.
            ///
            /// 180° rotation counterclockwise.
            Rotate180,
            /// `R270`.
            ///
            /// 270° rotation counterclockwise.
            ///
            /// Requires the world to be square and have no diagonal width.
            Rotate270,
            /// `F-`.
            ///
            /// Reflection across the middle row.
            ///
            /// Requires the world to have no diagonal width.
            FlipRow,
            /// `F|`.
            ///
            /// Reflection across the middle column.
            ///
            /// Requires the world to have no diagonal width.
            FlipCol,
            /// `F\`.
            ///
            /// Reflection across the diagonal.
            ///
            /// Requires the world to be square.
            FlipDiag,
            /// `F/`.
            ///
            /// Reflection across the antidiagonal.
            ///
            /// Requires the world to be square.
            FlipAntidiag,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Transform {
            #[inline]
            fn clone(&self) -> Transform {
                {
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for Transform {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Transform {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Transform::Id,) => {
                        let mut debug_trait_builder = f.debug_tuple("Id");
                        debug_trait_builder.finish()
                    }
                    (&Transform::Rotate90,) => {
                        let mut debug_trait_builder = f.debug_tuple("Rotate90");
                        debug_trait_builder.finish()
                    }
                    (&Transform::Rotate180,) => {
                        let mut debug_trait_builder = f.debug_tuple("Rotate180");
                        debug_trait_builder.finish()
                    }
                    (&Transform::Rotate270,) => {
                        let mut debug_trait_builder = f.debug_tuple("Rotate270");
                        debug_trait_builder.finish()
                    }
                    (&Transform::FlipRow,) => {
                        let mut debug_trait_builder = f.debug_tuple("FlipRow");
                        debug_trait_builder.finish()
                    }
                    (&Transform::FlipCol,) => {
                        let mut debug_trait_builder = f.debug_tuple("FlipCol");
                        debug_trait_builder.finish()
                    }
                    (&Transform::FlipDiag,) => {
                        let mut debug_trait_builder = f.debug_tuple("FlipDiag");
                        debug_trait_builder.finish()
                    }
                    (&Transform::FlipAntidiag,) => {
                        let mut debug_trait_builder = f.debug_tuple("FlipAntidiag");
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[allow(unused_qualifications)]
        impl ::std::default::Default for Transform {
            fn default() -> Self {
                Transform::Id
            }
        }
        impl ::core::marker::StructuralPartialEq for Transform {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Transform {
            #[inline]
            fn eq(&self, other: &Transform) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            _ => true,
                        }
                    } else {
                        false
                    }
                }
            }
        }
        impl ::core::marker::StructuralEq for Transform {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Transform {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {}
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::hash::Hash for Transform {
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                match (&*self,) {
                    _ => ::core::hash::Hash::hash(
                        &::core::intrinsics::discriminant_value(self),
                        state,
                    ),
                }
            }
        }
        impl FromStr for Transform {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "Id" => Ok(Transform::Id),
                    "R90" => Ok(Transform::Rotate90),
                    "R180" => Ok(Transform::Rotate180),
                    "R270" => Ok(Transform::Rotate270),
                    "F-" => Ok(Transform::FlipRow),
                    "F|" => Ok(Transform::FlipCol),
                    "F\\" => Ok(Transform::FlipDiag),
                    "F/" => Ok(Transform::FlipAntidiag),
                    _ => Err(String::from("invalid Transform")),
                }
            }
        }
        impl Display for Transform {
            fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
                let s = match self {
                    Transform::Id => "Id",
                    Transform::Rotate90 => "R90",
                    Transform::Rotate180 => "R180",
                    Transform::Rotate270 => "R270",
                    Transform::FlipRow => "F-",
                    Transform::FlipCol => "F|",
                    Transform::FlipDiag => "F\\",
                    Transform::FlipAntidiag => "F/",
                };
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&s,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                ))?;
                Ok(())
            }
        }
        impl Mul for Transform {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self {
                match (self, rhs) {
                    (Transform::Id, Transform::Id)
                    | (Transform::Rotate90, Transform::Rotate270)
                    | (Transform::Rotate180, Transform::Rotate180)
                    | (Transform::Rotate270, Transform::Rotate90)
                    | (Transform::FlipRow, Transform::FlipRow)
                    | (Transform::FlipCol, Transform::FlipCol)
                    | (Transform::FlipDiag, Transform::FlipDiag)
                    | (Transform::FlipAntidiag, Transform::FlipAntidiag) => Transform::Id,
                    (Transform::Id, Transform::Rotate90)
                    | (Transform::Rotate90, Transform::Id)
                    | (Transform::Rotate180, Transform::Rotate270)
                    | (Transform::Rotate270, Transform::Rotate180)
                    | (Transform::FlipRow, Transform::FlipAntidiag)
                    | (Transform::FlipCol, Transform::FlipDiag)
                    | (Transform::FlipDiag, Transform::FlipRow)
                    | (Transform::FlipAntidiag, Transform::FlipCol) => Transform::Rotate90,
                    (Transform::Id, Transform::Rotate180)
                    | (Transform::Rotate90, Transform::Rotate90)
                    | (Transform::Rotate180, Transform::Id)
                    | (Transform::Rotate270, Transform::Rotate270)
                    | (Transform::FlipRow, Transform::FlipCol)
                    | (Transform::FlipCol, Transform::FlipRow)
                    | (Transform::FlipDiag, Transform::FlipAntidiag)
                    | (Transform::FlipAntidiag, Transform::FlipDiag) => Transform::Rotate180,
                    (Transform::Id, Transform::Rotate270)
                    | (Transform::Rotate90, Transform::Rotate180)
                    | (Transform::Rotate180, Transform::Rotate90)
                    | (Transform::Rotate270, Transform::Id)
                    | (Transform::FlipRow, Transform::FlipDiag)
                    | (Transform::FlipCol, Transform::FlipAntidiag)
                    | (Transform::FlipDiag, Transform::FlipCol)
                    | (Transform::FlipAntidiag, Transform::FlipRow) => Transform::Rotate270,
                    (Transform::Id, Transform::FlipRow)
                    | (Transform::Rotate90, Transform::FlipAntidiag)
                    | (Transform::Rotate180, Transform::FlipCol)
                    | (Transform::Rotate270, Transform::FlipDiag)
                    | (Transform::FlipRow, Transform::Id)
                    | (Transform::FlipCol, Transform::Rotate180)
                    | (Transform::FlipDiag, Transform::Rotate90)
                    | (Transform::FlipAntidiag, Transform::Rotate270) => Transform::FlipRow,
                    (Transform::Id, Transform::FlipCol)
                    | (Transform::Rotate90, Transform::FlipDiag)
                    | (Transform::Rotate180, Transform::FlipRow)
                    | (Transform::Rotate270, Transform::FlipAntidiag)
                    | (Transform::FlipRow, Transform::Rotate180)
                    | (Transform::FlipCol, Transform::Id)
                    | (Transform::FlipDiag, Transform::Rotate270)
                    | (Transform::FlipAntidiag, Transform::Rotate90) => Transform::FlipCol,
                    (Transform::Id, Transform::FlipDiag)
                    | (Transform::Rotate90, Transform::FlipRow)
                    | (Transform::Rotate180, Transform::FlipAntidiag)
                    | (Transform::Rotate270, Transform::FlipCol)
                    | (Transform::FlipRow, Transform::Rotate270)
                    | (Transform::FlipCol, Transform::Rotate90)
                    | (Transform::FlipDiag, Transform::Id)
                    | (Transform::FlipAntidiag, Transform::Rotate180) => Transform::FlipDiag,
                    (Transform::Id, Transform::FlipAntidiag)
                    | (Transform::Rotate90, Transform::FlipCol)
                    | (Transform::Rotate180, Transform::FlipDiag)
                    | (Transform::Rotate270, Transform::FlipRow)
                    | (Transform::FlipRow, Transform::Rotate90)
                    | (Transform::FlipCol, Transform::Rotate270)
                    | (Transform::FlipDiag, Transform::Rotate180)
                    | (Transform::FlipAntidiag, Transform::Id) => Transform::FlipAntidiag,
                }
            }
        }
        impl Transform {
            /// Whether this transformation requires the world to be square.
            ///
            /// Returns `true` for `R90`, `R270`, `F\` and `F/`.
            pub fn require_square_world(self) -> bool {
                !self.is_in(Symmetry::D4Ortho)
            }
            /// Whether this transformation requires the world to have no diagonal width.
            ///
            /// Returns `true` for `R90`, `R270`, `F-` and `F|`.
            pub fn require_no_diagonal_width(self) -> bool {
                !self.is_in(Symmetry::D4Diag)
            }
            /// The inverse of this transformation.
            pub fn inverse(self) -> Self {
                match self {
                    Transform::Rotate90 => Transform::Rotate270,
                    Transform::Rotate270 => Transform::Rotate90,
                    x => x,
                }
            }
            /// Whether the transformation is a member of the symmetry group,
            /// i.e., whether patterns with this symmetry are invariant under
            /// this transformation.
            pub fn is_in(self, sym: Symmetry) -> bool {
                match (self, sym) {
                    (Transform::Id, _)
                    | (_, Symmetry::D8)
                    | (Transform::Rotate90, Symmetry::C4)
                    | (Transform::Rotate180, Symmetry::C2)
                    | (Transform::Rotate180, Symmetry::C4)
                    | (Transform::Rotate180, Symmetry::D4Ortho)
                    | (Transform::Rotate180, Symmetry::D4Diag)
                    | (Transform::Rotate270, Symmetry::C4)
                    | (Transform::FlipRow, Symmetry::D2Row)
                    | (Transform::FlipRow, Symmetry::D4Ortho)
                    | (Transform::FlipCol, Symmetry::D2Col)
                    | (Transform::FlipCol, Symmetry::D4Ortho)
                    | (Transform::FlipDiag, Symmetry::D2Diag)
                    | (Transform::FlipDiag, Symmetry::D4Diag)
                    | (Transform::FlipAntidiag, Symmetry::D2Antidiag)
                    | (Transform::FlipAntidiag, Symmetry::D4Diag) => true,
                    _ => false,
                }
            }
            /// Apply the transformation on a coordinate.
            pub fn act_on(self, coord: Coord, width: i32, height: i32) -> Coord {
                let (x, y, t) = coord;
                match self {
                    Transform::Id => (x, y, t),
                    Transform::Rotate90 => (y, width - 1 - x, t),
                    Transform::Rotate180 => (width - 1 - x, height - 1 - y, t),
                    Transform::Rotate270 => (height - 1 - y, x, t),
                    Transform::FlipRow => (x, height - 1 - y, t),
                    Transform::FlipCol => (width - 1 - x, y, t),
                    Transform::FlipDiag => (y, x, t),
                    Transform::FlipAntidiag => (height - 1 - y, width - 1 - x, t),
                }
            }
        }
        /// Symmetries of the pattern.
        ///
        /// For each symmetry, its [symmetry group](https://en.wikipedia.org/wiki/Symmetry_group)
        /// is a subgroup of the dihedral group _D_<sub>8</sub>.
        /// 10 different symmetries correspond to 10 subgroups of _D_<sub>8</sub>.
        ///
        /// The notations are stolen from Oscar Cunningham's
        /// [Logic Life Search](https://github.com/OscarCunningham/logic-life-search).
        /// Please see the [Life Wiki](https://conwaylife.com/wiki/Symmetry) for details.
        ///
        /// Some of the symmetries are only valid when the world is square,
        /// and some are only valid when the world has no diagonal width.
        #[derivative(Default)]
        pub enum Symmetry {
            /// `C1`.
            ///
            /// No symmetry at all.
            #[derivative(Default)]
            C1,
            /// `C2`.
            ///
            /// Symmetry under 180° rotation.
            C2,
            /// `C4`.
            ///
            /// Symmetry under 90° rotation.
            ///
            /// Requires the world to be square and have no diagonal width.
            C4,
            /// `D2-`.
            ///
            /// Symmetry under reflection across the middle row.
            ///
            /// Requires the world to have no diagonal width.
            D2Row,
            /// `D2|`.
            ///
            /// Symmetry under reflection across the middle column.
            ///
            /// Requires the world to have no diagonal width.
            D2Col,
            /// `D2\`.
            ///
            /// Symmetry under reflection across the diagonal.
            ///
            /// Requires the world to be square.
            D2Diag,
            /// `D2/`.
            ///
            /// Symmetry under reflection across the antidiagonal.
            ///
            /// Requires the world to be square.
            D2Antidiag,
            /// `D4+`.
            ///
            /// Symmetry under reflections across the middle row
            /// and the middle column.
            ///
            /// Requires the world to have no diagonal width.
            D4Ortho,
            /// `D4X`.
            ///
            /// Symmetry under reflections across the diagonal
            /// and the antidiagonal.
            ///
            /// Requires the world to be square.
            D4Diag,
            /// `D8`.
            ///
            /// Symmetry under all 8 transformations.
            ///
            /// Requires the world to be square and have no diagonal width.
            D8,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for Symmetry {
            #[inline]
            fn clone(&self) -> Symmetry {
                {
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for Symmetry {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for Symmetry {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&Symmetry::C1,) => {
                        let mut debug_trait_builder = f.debug_tuple("C1");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::C2,) => {
                        let mut debug_trait_builder = f.debug_tuple("C2");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::C4,) => {
                        let mut debug_trait_builder = f.debug_tuple("C4");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::D2Row,) => {
                        let mut debug_trait_builder = f.debug_tuple("D2Row");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::D2Col,) => {
                        let mut debug_trait_builder = f.debug_tuple("D2Col");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::D2Diag,) => {
                        let mut debug_trait_builder = f.debug_tuple("D2Diag");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::D2Antidiag,) => {
                        let mut debug_trait_builder = f.debug_tuple("D2Antidiag");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::D4Ortho,) => {
                        let mut debug_trait_builder = f.debug_tuple("D4Ortho");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::D4Diag,) => {
                        let mut debug_trait_builder = f.debug_tuple("D4Diag");
                        debug_trait_builder.finish()
                    }
                    (&Symmetry::D8,) => {
                        let mut debug_trait_builder = f.debug_tuple("D8");
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[allow(unused_qualifications)]
        impl ::std::default::Default for Symmetry {
            fn default() -> Self {
                Symmetry::C1
            }
        }
        impl ::core::marker::StructuralPartialEq for Symmetry {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for Symmetry {
            #[inline]
            fn eq(&self, other: &Symmetry) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            _ => true,
                        }
                    } else {
                        false
                    }
                }
            }
        }
        impl ::core::marker::StructuralEq for Symmetry {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for Symmetry {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {}
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::hash::Hash for Symmetry {
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                match (&*self,) {
                    _ => ::core::hash::Hash::hash(
                        &::core::intrinsics::discriminant_value(self),
                        state,
                    ),
                }
            }
        }
        impl FromStr for Symmetry {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "C1" => Ok(Symmetry::C1),
                    "C2" => Ok(Symmetry::C2),
                    "C4" => Ok(Symmetry::C4),
                    "D2-" => Ok(Symmetry::D2Row),
                    "D2|" => Ok(Symmetry::D2Col),
                    "D2\\" => Ok(Symmetry::D2Diag),
                    "D2/" => Ok(Symmetry::D2Antidiag),
                    "D4+" => Ok(Symmetry::D4Ortho),
                    "D4X" => Ok(Symmetry::D4Diag),
                    "D8" => Ok(Symmetry::D8),
                    _ => Err(String::from("invalid symmetry")),
                }
            }
        }
        impl Display for Symmetry {
            fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
                let s = match self {
                    Symmetry::C1 => "C1",
                    Symmetry::C2 => "C2",
                    Symmetry::C4 => "C4",
                    Symmetry::D2Row => "D2-",
                    Symmetry::D2Col => "D2|",
                    Symmetry::D2Diag => "D2\\",
                    Symmetry::D2Antidiag => "D2/",
                    Symmetry::D4Ortho => "D4+",
                    Symmetry::D4Diag => "D4X",
                    Symmetry::D8 => "D8",
                };
                f.write_fmt(::core::fmt::Arguments::new_v1(
                    &[""],
                    &match (&s,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                ))?;
                Ok(())
            }
        }
        impl PartialOrd for Symmetry {
            /// We say that symmetry `a` is smaller than symmetry `b`,
            /// when the symmetry group of `a` is a subgroup of that of `b`,
            /// i.e., all patterns with symmetry `b` also have symmetry `a`.
            ///
            /// For example, `Symmetry::C1` is smaller than all other symmetries.
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                if self == other {
                    return Some(Ordering::Equal);
                }
                match (self, other) {
                    (Symmetry::C1, _)
                    | (_, Symmetry::D8)
                    | (Symmetry::C2, Symmetry::C4)
                    | (Symmetry::C2, Symmetry::D4Ortho)
                    | (Symmetry::C2, Symmetry::D4Diag)
                    | (Symmetry::D2Row, Symmetry::D4Ortho)
                    | (Symmetry::D2Col, Symmetry::D4Ortho)
                    | (Symmetry::D2Diag, Symmetry::D4Diag)
                    | (Symmetry::D2Antidiag, Symmetry::D4Diag) => Some(Ordering::Less),
                    (Symmetry::D8, _)
                    | (_, Symmetry::C1)
                    | (Symmetry::C4, Symmetry::C2)
                    | (Symmetry::D4Ortho, Symmetry::C2)
                    | (Symmetry::D4Diag, Symmetry::C2)
                    | (Symmetry::D4Ortho, Symmetry::D2Row)
                    | (Symmetry::D4Ortho, Symmetry::D2Col)
                    | (Symmetry::D4Diag, Symmetry::D2Diag)
                    | (Symmetry::D4Diag, Symmetry::D2Antidiag) => Some(Ordering::Greater),
                    _ => None,
                }
            }
        }
        impl Symmetry {
            /// Whether this symmetry requires the world to be square.
            ///
            /// Returns `true` for `C4`, `D2\`, `D2/`, `D4X` and `D8`.
            pub fn require_square_world(self) -> bool {
                match self.partial_cmp(&Symmetry::D4Ortho) {
                    Some(Ordering::Greater) | None => true,
                    _ => false,
                }
            }
            /// Whether this transformation requires the world to have no diagonal width.
            ///
            /// Returns `true` for `C4`, `D2-`, `D2|`, `D4+` and `D8`.
            pub fn require_no_diagonal_width(self) -> bool {
                match self.partial_cmp(&Symmetry::D4Diag) {
                    Some(Ordering::Greater) | None => true,
                    _ => false,
                }
            }
            /// Transformations contained in the symmetry group.
            pub fn members(self) -> Vec<Transform> {
                match self {
                    Symmetry::C1 => <[_]>::into_vec(box [Transform::Id]),
                    Symmetry::C2 => <[_]>::into_vec(box [Transform::Id, Transform::Rotate180]),
                    Symmetry::C4 => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::Rotate90,
                        Transform::Rotate180,
                        Transform::Rotate270,
                    ]),
                    Symmetry::D2Row => <[_]>::into_vec(box [Transform::Id, Transform::FlipRow]),
                    Symmetry::D2Col => <[_]>::into_vec(box [Transform::Id, Transform::FlipCol]),
                    Symmetry::D2Diag => <[_]>::into_vec(box [Transform::Id, Transform::FlipDiag]),
                    Symmetry::D2Antidiag => {
                        <[_]>::into_vec(box [Transform::Id, Transform::FlipAntidiag])
                    }
                    Symmetry::D4Ortho => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::FlipRow,
                        Transform::FlipCol,
                        Transform::Rotate180,
                    ]),
                    Symmetry::D4Diag => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::FlipDiag,
                        Transform::FlipAntidiag,
                        Transform::Rotate180,
                    ]),
                    Symmetry::D8 => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::Rotate90,
                        Transform::Rotate180,
                        Transform::Rotate270,
                        Transform::FlipRow,
                        Transform::FlipCol,
                        Transform::FlipDiag,
                        Transform::FlipAntidiag,
                    ]),
                }
            }
            /// A list of coset representatives,
            /// seeing the symmetry group as a subgroup of _D_<sub>8</sub>.
            ///
            /// The first element in the result is always [`Transform::Id`].
            pub fn cosets(self) -> Vec<Transform> {
                match self {
                    Symmetry::C1 => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::Rotate90,
                        Transform::Rotate180,
                        Transform::Rotate270,
                        Transform::FlipRow,
                        Transform::FlipCol,
                        Transform::FlipDiag,
                        Transform::FlipAntidiag,
                    ]),
                    Symmetry::C2 => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::Rotate90,
                        Transform::FlipRow,
                        Transform::FlipDiag,
                    ]),
                    Symmetry::C4 => <[_]>::into_vec(box [Transform::Id, Transform::FlipRow]),
                    Symmetry::D2Row => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::FlipCol,
                        Transform::FlipDiag,
                        Transform::FlipAntidiag,
                    ]),
                    Symmetry::D2Col => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::FlipRow,
                        Transform::FlipDiag,
                        Transform::FlipAntidiag,
                    ]),
                    Symmetry::D2Diag => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::FlipRow,
                        Transform::FlipCol,
                        Transform::FlipAntidiag,
                    ]),
                    Symmetry::D2Antidiag => <[_]>::into_vec(box [
                        Transform::Id,
                        Transform::FlipRow,
                        Transform::FlipCol,
                        Transform::FlipDiag,
                    ]),
                    Symmetry::D4Ortho => <[_]>::into_vec(box [Transform::Id, Transform::FlipDiag]),
                    Symmetry::D4Diag => <[_]>::into_vec(box [Transform::Id, Transform::FlipRow]),
                    Symmetry::D8 => <[_]>::into_vec(box [Transform::Id]),
                }
            }
        }
        impl Config {
            /// Applies the transformation and translation to a coord.
            pub(crate) fn translate(&self, coord: Coord) -> Coord {
                let mut coord = coord;
                while coord.2 < 0 {
                    coord = self
                        .transform
                        .inverse()
                        .act_on(coord, self.width, self.height);
                    coord.0 -= self.dx;
                    coord.1 -= self.dy;
                    coord.2 += self.period;
                }
                while coord.2 >= self.period {
                    coord.0 += self.dx;
                    coord.1 += self.dy;
                    coord.2 -= self.period;
                    coord = self.transform.act_on(coord, self.width, self.height);
                }
                coord
            }
        }
    }
    mod search_order {
        use super::{Config, Coord, Symmetry};
        use std::{borrow::Cow, cmp::Ordering};
        /// The order to find a new unknown cell.
        ///
        /// It will always search all generations of one cell
        /// before going to another cell.
        pub enum SearchOrder {
            /// Searches all cells of one row before going to the next row.
            ///
            /// ```plaintext
            /// 123
            /// 456
            /// 789
            /// ```
            RowFirst,
            /// Searches all cells of one column before going to the next column.
            ///
            /// ```plaintext
            /// 147
            /// 258
            /// 369
            /// ```
            ColumnFirst,
            /// Diagonal.
            ///
            /// ```plaintext
            /// 136
            /// 258
            /// 479
            /// ```
            ///
            /// This search order requires the world to be square.
            Diagonal,
            /// Specify the search order by a vector of coordinates.
            ///
            /// This vector should cover every cell in the search range,
            /// and should not have any duplication, otherwise rlifesrc
            /// would give a wrong result.
            FromVec(Vec<Coord>),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for SearchOrder {
            #[inline]
            fn clone(&self) -> SearchOrder {
                match (&*self,) {
                    (&SearchOrder::RowFirst,) => SearchOrder::RowFirst,
                    (&SearchOrder::ColumnFirst,) => SearchOrder::ColumnFirst,
                    (&SearchOrder::Diagonal,) => SearchOrder::Diagonal,
                    (&SearchOrder::FromVec(ref __self_0),) => {
                        SearchOrder::FromVec(::core::clone::Clone::clone(&(*__self_0)))
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for SearchOrder {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match (&*self,) {
                    (&SearchOrder::RowFirst,) => {
                        let mut debug_trait_builder = f.debug_tuple("RowFirst");
                        debug_trait_builder.finish()
                    }
                    (&SearchOrder::ColumnFirst,) => {
                        let mut debug_trait_builder = f.debug_tuple("ColumnFirst");
                        debug_trait_builder.finish()
                    }
                    (&SearchOrder::Diagonal,) => {
                        let mut debug_trait_builder = f.debug_tuple("Diagonal");
                        debug_trait_builder.finish()
                    }
                    (&SearchOrder::FromVec(ref __self_0),) => {
                        let mut debug_trait_builder = f.debug_tuple("FromVec");
                        let _ = debug_trait_builder.field(&&(*__self_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        impl ::core::marker::StructuralPartialEq for SearchOrder {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for SearchOrder {
            #[inline]
            fn eq(&self, other: &SearchOrder) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (
                                &SearchOrder::FromVec(ref __self_0),
                                &SearchOrder::FromVec(ref __arg_1_0),
                            ) => (*__self_0) == (*__arg_1_0),
                            _ => true,
                        }
                    } else {
                        false
                    }
                }
            }
            #[inline]
            fn ne(&self, other: &SearchOrder) -> bool {
                {
                    let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                    let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                    if true && __self_vi == __arg_1_vi {
                        match (&*self, &*other) {
                            (
                                &SearchOrder::FromVec(ref __self_0),
                                &SearchOrder::FromVec(ref __arg_1_0),
                            ) => (*__self_0) != (*__arg_1_0),
                            _ => false,
                        }
                    } else {
                        true
                    }
                }
            }
        }
        impl ::core::marker::StructuralEq for SearchOrder {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for SearchOrder {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<Vec<Coord>>;
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::hash::Hash for SearchOrder {
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                match (&*self,) {
                    (&SearchOrder::FromVec(ref __self_0),) => {
                        ::core::hash::Hash::hash(
                            &::core::intrinsics::discriminant_value(self),
                            state,
                        );
                        ::core::hash::Hash::hash(&(*__self_0), state)
                    }
                    _ => ::core::hash::Hash::hash(
                        &::core::intrinsics::discriminant_value(self),
                        state,
                    ),
                }
            }
        }
        impl Config {
            /// Automatically determines the search order if `search_order` is `None`.
            pub(crate) fn auto_search_order(&self) -> Cow<SearchOrder> {
                if let Some(search_order) = &self.search_order {
                    Cow::Borrowed(search_order)
                } else {
                    let (width, height) = match self.symmetry {
                        Symmetry::D2Row => (self.width, (self.height + 1) / 2),
                        Symmetry::D2Col => ((self.width + 1) / 2, self.height),
                        _ => (self.width, self.height),
                    };
                    let search_order = match width.cmp(&height) {
                        Ordering::Greater => SearchOrder::ColumnFirst,
                        Ordering::Less => SearchOrder::RowFirst,
                        Ordering::Equal => {
                            if self.diagonal_width.is_some()
                                && 2 * self.diagonal_width.unwrap() <= self.width
                            {
                                SearchOrder::Diagonal
                            } else if self.dx.abs() >= self.dy.abs() {
                                SearchOrder::ColumnFirst
                            } else {
                                SearchOrder::RowFirst
                            }
                        }
                    };
                    Cow::Owned(search_order)
                }
            }
            /// Generates an iterator over cells coordinates from the search order.
            pub(crate) fn search_order_iter(
                &self,
                search_order: &SearchOrder,
            ) -> Box<dyn Iterator<Item = Coord>> {
                let width = self.width;
                let height = self.height;
                let period = self.period;
                let x_start = if self.symmetry >= Symmetry::D2Col {
                    self.width / 2
                } else {
                    0
                };
                let y_start = if self.symmetry >= Symmetry::D2Row {
                    self.height / 2
                } else {
                    0
                };
                match search_order {
                    SearchOrder::ColumnFirst => Box::new((0..width).rev().flat_map(move |x| {
                        (y_start..height)
                            .rev()
                            .flat_map(move |y| (0..period).rev().map(move |t| (x, y, t)))
                    })),
                    SearchOrder::RowFirst => Box::new((0..height).rev().flat_map(move |y| {
                        (x_start..width)
                            .rev()
                            .flat_map(move |x| (0..period).rev().map(move |t| (x, y, t)))
                    })),
                    SearchOrder::Diagonal => {
                        if self.symmetry >= Symmetry::D2Diag {
                            Box::new(
                                (0..width)
                                    .rev()
                                    .flat_map(move |d| {
                                        ((width + d + 1) / 2..width).rev().flat_map(move |x| {
                                            (0..period).rev().map(move |t| (x, width + d - x, t))
                                        })
                                    })
                                    .chain((0..width).rev().flat_map(move |d| {
                                        ((d + 1) / 2..=d).rev().flat_map(move |x| {
                                            (0..period).rev().map(move |t| (x, d - x, t))
                                        })
                                    })),
                            )
                        } else {
                            Box::new(
                                (0..width)
                                    .rev()
                                    .flat_map(move |d| {
                                        (d + 1..width).rev().flat_map(move |x| {
                                            (0..period).rev().map(move |t| (x, width + d - x, t))
                                        })
                                    })
                                    .chain((0..width).rev().flat_map(move |d| {
                                        (0..=d).rev().flat_map(move |x| {
                                            (0..period).rev().map(move |t| (x, d - x, t))
                                        })
                                    })),
                            )
                        }
                    }
                    SearchOrder::FromVec(vec) => Box::new(vec.clone().into_iter().rev()),
                }
            }
            /// Generates a closure to determine whether a cell is in the front.
            ///
            /// Return `None` when we should not force the front to be nonempty,
            /// or there isn't a well-defined 'front'.
            pub(crate) fn is_front_fn(
                &self,
                rule_is_b0: bool,
                search_order: &SearchOrder,
            ) -> Option<Box<dyn Fn(Coord) -> bool>> {
                let dx = self.dx;
                let dy = self.dy;
                let width = self.width;
                let height = self.height;
                match search_order {
                    SearchOrder::RowFirst => {
                        if self.symmetry <= Symmetry::D2Col
                            && self.transform.is_in(Symmetry::D2Col)
                            && self.diagonal_width.is_none()
                        {
                            if !rule_is_b0 && dx == 0 && dy >= 0 {
                                Some(Box::new(move |(x, y, t)| {
                                    y == (dy - 1).max(0) && t == 0 && x <= width / 2
                                }))
                            } else {
                                Some(Box::new(|(_, y, _)| y == 0))
                            }
                        } else {
                            None
                        }
                    }
                    SearchOrder::ColumnFirst => {
                        if self.symmetry <= Symmetry::D2Row
                            && self.transform.is_in(Symmetry::D2Row)
                            && self.diagonal_width.is_none()
                        {
                            if !rule_is_b0 && dx >= 0 && dy == 0 {
                                Some(Box::new(move |(x, y, t)| {
                                    x == (dx - 1).max(0) && t == 0 && y <= height / 2
                                }))
                            } else {
                                Some(Box::new(|(x, _, _)| x == 0))
                            }
                        } else {
                            None
                        }
                    }
                    SearchOrder::Diagonal => {
                        if self.symmetry <= Symmetry::D2Diag
                            && self.transform.is_in(Symmetry::D2Diag)
                        {
                            if !rule_is_b0 && dx >= 0 && dx == dy {
                                Some(Box::new(move |(x, _, t)| x == (dx - 1).max(0) && t == 0))
                            } else {
                                Some(Box::new(|(x, y, _)| x == 0 || y == 0))
                            }
                        } else {
                            None
                        }
                    }
                    SearchOrder::FromVec(_) => None,
                }
            }
        }
    }
    pub use d8::{Symmetry, Transform};
    pub use search_order::SearchOrder;
    /// How to choose a state for an unknown cell.
    #[derivative(Default)]
    pub enum NewState {
        /// Chooses the background state.
        ///
        /// For rules without `B0`, it always chooses [`DEAD`].
        ///
        /// For rules with `B0`, the background changes periodically.
        /// For example, for non-Generations rules,
        /// it chooses [`DEAD`] on even generations,
        /// [`ALIVE`] on odd generations.
        ChooseDead,
        /// Chooses the opposite of the background state.
        ///
        /// For rules without `B0`, it always chooses [`ALIVE`].
        ///
        /// For rules with `B0`, the background changes periodically.
        /// For example, for non-Generations rules,
        /// it chooses [`ALIVE`] on even generations,
        /// [`DEAD`] on odd generations.
        #[derivative(Default)]
        ChooseAlive,
        /// Random.
        ///
        /// For non-Generations rules,
        /// the probability of either state is `1/2`.
        ///
        /// For Generations rules with `n` states,
        /// the probability of each state is `1/n`.
        Random,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NewState {
        #[inline]
        fn clone(&self) -> NewState {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NewState {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NewState {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&NewState::ChooseDead,) => {
                    let mut debug_trait_builder = f.debug_tuple("ChooseDead");
                    debug_trait_builder.finish()
                }
                (&NewState::ChooseAlive,) => {
                    let mut debug_trait_builder = f.debug_tuple("ChooseAlive");
                    debug_trait_builder.finish()
                }
                (&NewState::Random,) => {
                    let mut debug_trait_builder = f.debug_tuple("Random");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl ::std::default::Default for NewState {
        fn default() -> Self {
            NewState::ChooseAlive
        }
    }
    impl ::core::marker::StructuralPartialEq for NewState {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for NewState {
        #[inline]
        fn eq(&self, other: &NewState) -> bool {
            {
                let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for NewState {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for NewState {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for NewState {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                _ => ::core::hash::Hash::hash(&::core::intrinsics::discriminant_value(self), state),
            }
        }
    }
    /// A cell whose state is known before the search.
    pub struct KnownCell {
        /// The coordinates of the set cell.
        pub coord: Coord,
        /// The state.
        pub state: State,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for KnownCell {
        #[inline]
        fn clone(&self) -> KnownCell {
            {
                let _: ::core::clone::AssertParamIsClone<Coord>;
                let _: ::core::clone::AssertParamIsClone<State>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for KnownCell {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for KnownCell {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                KnownCell {
                    coord: ref __self_0_0,
                    state: ref __self_0_1,
                } => {
                    let mut debug_trait_builder = f.debug_struct("KnownCell");
                    let _ = debug_trait_builder.field("coord", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("state", &&(*__self_0_1));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for KnownCell {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for KnownCell {
        #[inline]
        fn eq(&self, other: &KnownCell) -> bool {
            match *other {
                KnownCell {
                    coord: ref __self_1_0,
                    state: ref __self_1_1,
                } => match *self {
                    KnownCell {
                        coord: ref __self_0_0,
                        state: ref __self_0_1,
                    } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
                },
            }
        }
        #[inline]
        fn ne(&self, other: &KnownCell) -> bool {
            match *other {
                KnownCell {
                    coord: ref __self_1_0,
                    state: ref __self_1_1,
                } => match *self {
                    KnownCell {
                        coord: ref __self_0_0,
                        state: ref __self_0_1,
                    } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for KnownCell {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for KnownCell {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<Coord>;
                let _: ::core::cmp::AssertParamIsEq<State>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for KnownCell {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match *self {
                KnownCell {
                    coord: ref __self_0_0,
                    state: ref __self_0_1,
                } => {
                    ::core::hash::Hash::hash(&(*__self_0_0), state);
                    ::core::hash::Hash::hash(&(*__self_0_1), state)
                }
            }
        }
    }
    /// World configuration.
    ///
    /// The world will be generated from this configuration.
    #[derivative(Default)]
    pub struct Config {
        /// Width.
        #[derivative(Default(value = "16"))]
        pub width: i32,
        /// Height.
        #[derivative(Default(value = "16"))]
        pub height: i32,
        /// Period.
        #[derivative(Default(value = "1"))]
        pub period: i32,
        /// Horizontal translation.
        pub dx: i32,
        /// Vertical translation.
        pub dy: i32,
        /// Transformations (rotations and reflections) after the last generation
        /// in a period.
        ///
        /// After the last generation in a period, the pattern will return to
        /// the first generation, applying this transformation first,
        /// and then the translation defined by `dx` and `dy`.
        pub transform: Transform,
        /// Symmetries of the pattern.
        pub symmetry: Symmetry,
        /// The order to find a new unknown cell.
        ///
        /// It will always search all generations of one cell
        /// before going to another cell.
        ///
        /// `None` means that it will automatically choose a search order
        /// according to the width and height of the world.
        pub search_order: Option<SearchOrder>,
        /// How to choose a state for an unknown cell.
        pub new_state: NewState,
        /// The number of minimum living cells in all generations must not
        /// exceed this number.
        ///
        /// `None` means that there is no limit for the cell count.
        pub max_cell_count: Option<usize>,
        /// Whether to automatically reduce the [`max_cell_count`](#structfield.max_cell_count)
        /// when a result is found.
        ///
        /// The [`max_cell_count`](#structfield.max_cell_count) will be set to the cell count of
        /// the current result minus one.
        pub reduce_max: bool,
        /// The rule string of the cellular automaton.
        #[derivative(Default(value = "String::from(\"B3/S23\")"))]
        pub rule_string: String,
        /// Diagonal width.
        ///
        /// If the diagonal width is `n`, the cells at position `(x, y)`
        /// where `abs(x - y) >= n` are assumed to be dead.
        pub diagonal_width: Option<i32>,
        /// Whether to skip patterns whose fundamental period are smaller than the given period.
        #[derivative(Default(value = "true"))]
        pub skip_subperiod: bool,
        /// Whether to skip patterns which are invariant under more transformations than
        /// required by the given symmetry.
        ///
        /// In another word, whether to skip patterns whose symmetry group properly contains
        /// the given symmetry group.
        pub skip_subsymmetry: bool,
        /// Cells whose states are known before the search.
        pub known_cells: Vec<KnownCell>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Config {
        #[inline]
        fn clone(&self) -> Config {
            match *self {
                Config {
                    width: ref __self_0_0,
                    height: ref __self_0_1,
                    period: ref __self_0_2,
                    dx: ref __self_0_3,
                    dy: ref __self_0_4,
                    transform: ref __self_0_5,
                    symmetry: ref __self_0_6,
                    search_order: ref __self_0_7,
                    new_state: ref __self_0_8,
                    max_cell_count: ref __self_0_9,
                    reduce_max: ref __self_0_10,
                    rule_string: ref __self_0_11,
                    diagonal_width: ref __self_0_12,
                    skip_subperiod: ref __self_0_13,
                    skip_subsymmetry: ref __self_0_14,
                    known_cells: ref __self_0_15,
                } => Config {
                    width: ::core::clone::Clone::clone(&(*__self_0_0)),
                    height: ::core::clone::Clone::clone(&(*__self_0_1)),
                    period: ::core::clone::Clone::clone(&(*__self_0_2)),
                    dx: ::core::clone::Clone::clone(&(*__self_0_3)),
                    dy: ::core::clone::Clone::clone(&(*__self_0_4)),
                    transform: ::core::clone::Clone::clone(&(*__self_0_5)),
                    symmetry: ::core::clone::Clone::clone(&(*__self_0_6)),
                    search_order: ::core::clone::Clone::clone(&(*__self_0_7)),
                    new_state: ::core::clone::Clone::clone(&(*__self_0_8)),
                    max_cell_count: ::core::clone::Clone::clone(&(*__self_0_9)),
                    reduce_max: ::core::clone::Clone::clone(&(*__self_0_10)),
                    rule_string: ::core::clone::Clone::clone(&(*__self_0_11)),
                    diagonal_width: ::core::clone::Clone::clone(&(*__self_0_12)),
                    skip_subperiod: ::core::clone::Clone::clone(&(*__self_0_13)),
                    skip_subsymmetry: ::core::clone::Clone::clone(&(*__self_0_14)),
                    known_cells: ::core::clone::Clone::clone(&(*__self_0_15)),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Config {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                Config {
                    width: ref __self_0_0,
                    height: ref __self_0_1,
                    period: ref __self_0_2,
                    dx: ref __self_0_3,
                    dy: ref __self_0_4,
                    transform: ref __self_0_5,
                    symmetry: ref __self_0_6,
                    search_order: ref __self_0_7,
                    new_state: ref __self_0_8,
                    max_cell_count: ref __self_0_9,
                    reduce_max: ref __self_0_10,
                    rule_string: ref __self_0_11,
                    diagonal_width: ref __self_0_12,
                    skip_subperiod: ref __self_0_13,
                    skip_subsymmetry: ref __self_0_14,
                    known_cells: ref __self_0_15,
                } => {
                    let mut debug_trait_builder = f.debug_struct("Config");
                    let _ = debug_trait_builder.field("width", &&(*__self_0_0));
                    let _ = debug_trait_builder.field("height", &&(*__self_0_1));
                    let _ = debug_trait_builder.field("period", &&(*__self_0_2));
                    let _ = debug_trait_builder.field("dx", &&(*__self_0_3));
                    let _ = debug_trait_builder.field("dy", &&(*__self_0_4));
                    let _ = debug_trait_builder.field("transform", &&(*__self_0_5));
                    let _ = debug_trait_builder.field("symmetry", &&(*__self_0_6));
                    let _ = debug_trait_builder.field("search_order", &&(*__self_0_7));
                    let _ = debug_trait_builder.field("new_state", &&(*__self_0_8));
                    let _ = debug_trait_builder.field("max_cell_count", &&(*__self_0_9));
                    let _ = debug_trait_builder.field("reduce_max", &&(*__self_0_10));
                    let _ = debug_trait_builder.field("rule_string", &&(*__self_0_11));
                    let _ = debug_trait_builder.field("diagonal_width", &&(*__self_0_12));
                    let _ = debug_trait_builder.field("skip_subperiod", &&(*__self_0_13));
                    let _ = debug_trait_builder.field("skip_subsymmetry", &&(*__self_0_14));
                    let _ = debug_trait_builder.field("known_cells", &&(*__self_0_15));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl ::std::default::Default for Config {
        fn default() -> Self {
            Config {
                width: 16,
                height: 16,
                period: 1,
                dx: ::std::default::Default::default(),
                dy: ::std::default::Default::default(),
                transform: ::std::default::Default::default(),
                symmetry: ::std::default::Default::default(),
                search_order: ::std::default::Default::default(),
                new_state: ::std::default::Default::default(),
                max_cell_count: ::std::default::Default::default(),
                reduce_max: ::std::default::Default::default(),
                rule_string: String::from("B3/S23"),
                diagonal_width: ::std::default::Default::default(),
                skip_subperiod: true,
                skip_subsymmetry: ::std::default::Default::default(),
                known_cells: ::std::default::Default::default(),
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Config {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Config {
        #[inline]
        fn eq(&self, other: &Config) -> bool {
            match *other {
                Config {
                    width: ref __self_1_0,
                    height: ref __self_1_1,
                    period: ref __self_1_2,
                    dx: ref __self_1_3,
                    dy: ref __self_1_4,
                    transform: ref __self_1_5,
                    symmetry: ref __self_1_6,
                    search_order: ref __self_1_7,
                    new_state: ref __self_1_8,
                    max_cell_count: ref __self_1_9,
                    reduce_max: ref __self_1_10,
                    rule_string: ref __self_1_11,
                    diagonal_width: ref __self_1_12,
                    skip_subperiod: ref __self_1_13,
                    skip_subsymmetry: ref __self_1_14,
                    known_cells: ref __self_1_15,
                } => match *self {
                    Config {
                        width: ref __self_0_0,
                        height: ref __self_0_1,
                        period: ref __self_0_2,
                        dx: ref __self_0_3,
                        dy: ref __self_0_4,
                        transform: ref __self_0_5,
                        symmetry: ref __self_0_6,
                        search_order: ref __self_0_7,
                        new_state: ref __self_0_8,
                        max_cell_count: ref __self_0_9,
                        reduce_max: ref __self_0_10,
                        rule_string: ref __self_0_11,
                        diagonal_width: ref __self_0_12,
                        skip_subperiod: ref __self_0_13,
                        skip_subsymmetry: ref __self_0_14,
                        known_cells: ref __self_0_15,
                    } => {
                        (*__self_0_0) == (*__self_1_0)
                            && (*__self_0_1) == (*__self_1_1)
                            && (*__self_0_2) == (*__self_1_2)
                            && (*__self_0_3) == (*__self_1_3)
                            && (*__self_0_4) == (*__self_1_4)
                            && (*__self_0_5) == (*__self_1_5)
                            && (*__self_0_6) == (*__self_1_6)
                            && (*__self_0_7) == (*__self_1_7)
                            && (*__self_0_8) == (*__self_1_8)
                            && (*__self_0_9) == (*__self_1_9)
                            && (*__self_0_10) == (*__self_1_10)
                            && (*__self_0_11) == (*__self_1_11)
                            && (*__self_0_12) == (*__self_1_12)
                            && (*__self_0_13) == (*__self_1_13)
                            && (*__self_0_14) == (*__self_1_14)
                            && (*__self_0_15) == (*__self_1_15)
                    }
                },
            }
        }
        #[inline]
        fn ne(&self, other: &Config) -> bool {
            match *other {
                Config {
                    width: ref __self_1_0,
                    height: ref __self_1_1,
                    period: ref __self_1_2,
                    dx: ref __self_1_3,
                    dy: ref __self_1_4,
                    transform: ref __self_1_5,
                    symmetry: ref __self_1_6,
                    search_order: ref __self_1_7,
                    new_state: ref __self_1_8,
                    max_cell_count: ref __self_1_9,
                    reduce_max: ref __self_1_10,
                    rule_string: ref __self_1_11,
                    diagonal_width: ref __self_1_12,
                    skip_subperiod: ref __self_1_13,
                    skip_subsymmetry: ref __self_1_14,
                    known_cells: ref __self_1_15,
                } => match *self {
                    Config {
                        width: ref __self_0_0,
                        height: ref __self_0_1,
                        period: ref __self_0_2,
                        dx: ref __self_0_3,
                        dy: ref __self_0_4,
                        transform: ref __self_0_5,
                        symmetry: ref __self_0_6,
                        search_order: ref __self_0_7,
                        new_state: ref __self_0_8,
                        max_cell_count: ref __self_0_9,
                        reduce_max: ref __self_0_10,
                        rule_string: ref __self_0_11,
                        diagonal_width: ref __self_0_12,
                        skip_subperiod: ref __self_0_13,
                        skip_subsymmetry: ref __self_0_14,
                        known_cells: ref __self_0_15,
                    } => {
                        (*__self_0_0) != (*__self_1_0)
                            || (*__self_0_1) != (*__self_1_1)
                            || (*__self_0_2) != (*__self_1_2)
                            || (*__self_0_3) != (*__self_1_3)
                            || (*__self_0_4) != (*__self_1_4)
                            || (*__self_0_5) != (*__self_1_5)
                            || (*__self_0_6) != (*__self_1_6)
                            || (*__self_0_7) != (*__self_1_7)
                            || (*__self_0_8) != (*__self_1_8)
                            || (*__self_0_9) != (*__self_1_9)
                            || (*__self_0_10) != (*__self_1_10)
                            || (*__self_0_11) != (*__self_1_11)
                            || (*__self_0_12) != (*__self_1_12)
                            || (*__self_0_13) != (*__self_1_13)
                            || (*__self_0_14) != (*__self_1_14)
                            || (*__self_0_15) != (*__self_1_15)
                    }
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for Config {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Config {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<i32>;
                let _: ::core::cmp::AssertParamIsEq<i32>;
                let _: ::core::cmp::AssertParamIsEq<i32>;
                let _: ::core::cmp::AssertParamIsEq<i32>;
                let _: ::core::cmp::AssertParamIsEq<i32>;
                let _: ::core::cmp::AssertParamIsEq<Transform>;
                let _: ::core::cmp::AssertParamIsEq<Symmetry>;
                let _: ::core::cmp::AssertParamIsEq<Option<SearchOrder>>;
                let _: ::core::cmp::AssertParamIsEq<NewState>;
                let _: ::core::cmp::AssertParamIsEq<Option<usize>>;
                let _: ::core::cmp::AssertParamIsEq<bool>;
                let _: ::core::cmp::AssertParamIsEq<String>;
                let _: ::core::cmp::AssertParamIsEq<Option<i32>>;
                let _: ::core::cmp::AssertParamIsEq<bool>;
                let _: ::core::cmp::AssertParamIsEq<bool>;
                let _: ::core::cmp::AssertParamIsEq<Vec<KnownCell>>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for Config {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match *self {
                Config {
                    width: ref __self_0_0,
                    height: ref __self_0_1,
                    period: ref __self_0_2,
                    dx: ref __self_0_3,
                    dy: ref __self_0_4,
                    transform: ref __self_0_5,
                    symmetry: ref __self_0_6,
                    search_order: ref __self_0_7,
                    new_state: ref __self_0_8,
                    max_cell_count: ref __self_0_9,
                    reduce_max: ref __self_0_10,
                    rule_string: ref __self_0_11,
                    diagonal_width: ref __self_0_12,
                    skip_subperiod: ref __self_0_13,
                    skip_subsymmetry: ref __self_0_14,
                    known_cells: ref __self_0_15,
                } => {
                    ::core::hash::Hash::hash(&(*__self_0_0), state);
                    ::core::hash::Hash::hash(&(*__self_0_1), state);
                    ::core::hash::Hash::hash(&(*__self_0_2), state);
                    ::core::hash::Hash::hash(&(*__self_0_3), state);
                    ::core::hash::Hash::hash(&(*__self_0_4), state);
                    ::core::hash::Hash::hash(&(*__self_0_5), state);
                    ::core::hash::Hash::hash(&(*__self_0_6), state);
                    ::core::hash::Hash::hash(&(*__self_0_7), state);
                    ::core::hash::Hash::hash(&(*__self_0_8), state);
                    ::core::hash::Hash::hash(&(*__self_0_9), state);
                    ::core::hash::Hash::hash(&(*__self_0_10), state);
                    ::core::hash::Hash::hash(&(*__self_0_11), state);
                    ::core::hash::Hash::hash(&(*__self_0_12), state);
                    ::core::hash::Hash::hash(&(*__self_0_13), state);
                    ::core::hash::Hash::hash(&(*__self_0_14), state);
                    ::core::hash::Hash::hash(&(*__self_0_15), state)
                }
            }
        }
    }
    impl Config {
        /// Sets up a new configuration with given size.
        pub fn new(width: i32, height: i32, period: i32) -> Self {
            Config {
                width,
                height,
                period,
                ..Config::default()
            }
        }
        /// Sets the translations `(dx, dy)`.
        pub fn set_translate(mut self, dx: i32, dy: i32) -> Self {
            self.dx = dx;
            self.dy = dy;
            self
        }
        /// Sets the transformation.
        pub fn set_transform(mut self, transform: Transform) -> Self {
            self.transform = transform;
            self
        }
        /// Sets the symmetry.
        pub fn set_symmetry(mut self, symmetry: Symmetry) -> Self {
            self.symmetry = symmetry;
            self
        }
        /// Sets the search order.
        pub fn set_search_order<T: Into<Option<SearchOrder>>>(mut self, search_order: T) -> Self {
            self.search_order = search_order.into();
            self
        }
        /// Sets how to choose a state for an unknown cell.
        pub fn set_new_state(mut self, new_state: NewState) -> Self {
            self.new_state = new_state;
            self
        }
        /// Sets the maximal number of living cells.
        pub fn set_max_cell_count<T: Into<Option<usize>>>(mut self, max_cell_count: T) -> Self {
            self.max_cell_count = max_cell_count.into();
            self
        }
        /// Sets whether to automatically reduce the `max_cell_count`
        /// when a result is found.
        pub fn set_reduce_max(mut self, reduce_max: bool) -> Self {
            self.reduce_max = reduce_max;
            self
        }
        /// Sets the rule string.
        pub fn set_rule_string<S: ToString>(mut self, rule_string: S) -> Self {
            self.rule_string = rule_string.to_string();
            self
        }
        /// Sets the diagonal width.
        pub fn set_diagonal_width<T: Into<Option<i32>>>(mut self, diagonal_width: T) -> Self {
            self.diagonal_width = diagonal_width.into();
            self
        }
        /// Sets whether to skip patterns whose fundamental period
        /// is smaller than the given period.
        pub fn set_skip_subperiod(mut self, skip_subperiod: bool) -> Self {
            self.skip_subperiod = skip_subperiod;
            self
        }
        /// Sets whether to skip patterns which are invariant under
        /// more transformations than required by the given symmetry.
        pub fn set_skip_subsymmetry(mut self, skip_subsymmetry: bool) -> Self {
            self.skip_subsymmetry = skip_subsymmetry;
            self
        }
        /// Sets cells whose states are known before the search.
        pub fn set_known_cells<T: Into<Vec<KnownCell>>>(mut self, known_cells: T) -> Self {
            self.known_cells = known_cells.into();
            self
        }
        /// Whether the configuration requires the world to be square.
        pub fn require_square_world(&self) -> bool {
            self.symmetry.require_square_world()
                || self.transform.require_square_world()
                || self.search_order == Some(SearchOrder::Diagonal)
        }
        /// Whether the configuration requires the world to have no diagonal width.
        pub fn require_no_diagonal_width(&self) -> bool {
            self.symmetry.require_no_diagonal_width() || self.transform.require_no_diagonal_width()
        }
        /// Creates a new world from the configuration.
        /// Returns an error if the rule string is invalid.
        pub fn world(&self) -> Result<Box<dyn Search>, Error> {
            if self.width <= 0 || self.height <= 0 || self.period <= 0 {
                return Err(Error::NonPositiveError);
            }
            if let Some(diagonal_width) = self.diagonal_width {
                if diagonal_width <= 0 {
                    return Err(Error::NonPositiveError);
                }
            }
            if self.require_square_world() && self.width != self.height {
                return Err(Error::SquareWorldError);
            }
            if self.require_no_diagonal_width() && self.diagonal_width.is_some() {
                return Err(Error::DiagonalWidthError);
            }
            if let Ok(rule) = self.rule_string.parse::<Life>() {
                Ok(Box::new(World::new(&self, rule)))
            } else if let Ok(rule) = self.rule_string.parse::<NtLife>() {
                Ok(Box::new(World::new(&self, rule)))
            } else if let Ok(rule) = self.rule_string.parse::<LifeGen>() {
                if rule.gen() > 2 {
                    Ok(Box::new(World::new(&self, rule)))
                } else {
                    let rule = rule.non_gen();
                    Ok(Box::new(World::new(&self, rule)))
                }
            } else {
                let rule = self.rule_string.parse::<NtLifeGen>()?;
                if rule.gen() > 2 {
                    Ok(Box::new(World::new(&self, rule)))
                } else {
                    let rule = rule.non_gen();
                    Ok(Box::new(World::new(&self, rule)))
                }
            }
        }
    }
}
mod error {
    //! All kinds of errors in this crate.
    use crate::cells::Coord;
    use ca_rules::ParseRuleError;
    use thiserror::Error;
    /// All kinds of errors in this crate.
    pub enum Error {
        /// Unable to set a cell.
        #[error("Unable to set cell at {0:?}.")]
        SetCellError(Coord),
        /// Invalid rule.
        #[error("Invalid rule: {0:?}.")]
        ParseRuleError(#[from] ParseRuleError),
        /// B0S8 rules are not supported yet. Please use the inverted rule.
        #[error("B0S8 rules are not supported yet. Please use the inverted rule.")]
        B0S8Error,
        /// Symmetry or transformation requires the world to be square.
        #[error("Symmetry or transformation requires the world to be square.")]
        SquareWorldError,
        /// Symmetry or transformation requires the world to have no diagonal width.
        #[error("Symmetry or transformation requires the world to have no diagonal width.")]
        DiagonalWidthError,
        /// Width / height / period should be positive.
        #[error("Width / height / period should be positive.")]
        NonPositiveError,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Error {
        #[inline]
        fn clone(&self) -> Error {
            match (&*self,) {
                (&Error::SetCellError(ref __self_0),) => {
                    Error::SetCellError(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&Error::ParseRuleError(ref __self_0),) => {
                    Error::ParseRuleError(::core::clone::Clone::clone(&(*__self_0)))
                }
                (&Error::B0S8Error,) => Error::B0S8Error,
                (&Error::SquareWorldError,) => Error::SquareWorldError,
                (&Error::DiagonalWidthError,) => Error::DiagonalWidthError,
                (&Error::NonPositiveError,) => Error::NonPositiveError,
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Error {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Error::SetCellError(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("SetCellError");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&Error::ParseRuleError(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("ParseRuleError");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&Error::B0S8Error,) => {
                    let mut debug_trait_builder = f.debug_tuple("B0S8Error");
                    debug_trait_builder.finish()
                }
                (&Error::SquareWorldError,) => {
                    let mut debug_trait_builder = f.debug_tuple("SquareWorldError");
                    debug_trait_builder.finish()
                }
                (&Error::DiagonalWidthError,) => {
                    let mut debug_trait_builder = f.debug_tuple("DiagonalWidthError");
                    debug_trait_builder.finish()
                }
                (&Error::NonPositiveError,) => {
                    let mut debug_trait_builder = f.debug_tuple("NonPositiveError");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Error {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Error {
        #[inline]
        fn eq(&self, other: &Error) -> bool {
            {
                let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &Error::SetCellError(ref __self_0),
                            &Error::SetCellError(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        (
                            &Error::ParseRuleError(ref __self_0),
                            &Error::ParseRuleError(ref __arg_1_0),
                        ) => (*__self_0) == (*__arg_1_0),
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &Error) -> bool {
            {
                let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (
                            &Error::SetCellError(ref __self_0),
                            &Error::SetCellError(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        (
                            &Error::ParseRuleError(ref __self_0),
                            &Error::ParseRuleError(ref __arg_1_0),
                        ) => (*__self_0) != (*__arg_1_0),
                        _ => false,
                    }
                } else {
                    true
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Error {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Error {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<Coord>;
                let _: ::core::cmp::AssertParamIsEq<ParseRuleError>;
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for Error {
        fn source(&self) -> std::option::Option<&(dyn std::error::Error + 'static)> {
            use thiserror::private::AsDynError;
            #[allow(deprecated)]
            match self {
                Error::SetCellError { .. } => std::option::Option::None,
                Error::ParseRuleError { 0: source, .. } => {
                    std::option::Option::Some(source.as_dyn_error())
                }
                Error::B0S8Error { .. } => std::option::Option::None,
                Error::SquareWorldError { .. } => std::option::Option::None,
                Error::DiagonalWidthError { .. } => std::option::Option::None,
                Error::NonPositiveError { .. } => std::option::Option::None,
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::fmt::Display for Error {
        fn fmt(&self, __formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
            match self {
                Error::SetCellError(_0) => __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Unable to set cell at ", "."],
                    &match (&_0,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                )),
                Error::ParseRuleError(_0) => __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                    &["Invalid rule: ", "."],
                    &match (&_0,) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)],
                    },
                )),
                Error::B0S8Error {} => __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                    &["B0S8 rules are not supported yet. Please use the inverted rule."],
                    &match () {
                        () => [],
                    },
                )),
                Error::SquareWorldError {} => {
                    __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Symmetry or transformation requires the world to be square."],
                        &match () {
                            () => [],
                        },
                    ))
                }
                Error::DiagonalWidthError {} => __formatter
                    .write_fmt(::core::fmt::Arguments::new_v1(
                    &["Symmetry or transformation requires the world to have no diagonal width."],
                    &match () {
                        () => [],
                    },
                )),
                Error::NonPositiveError {} => {
                    __formatter.write_fmt(::core::fmt::Arguments::new_v1(
                        &["Width / height / period should be positive."],
                        &match () {
                            () => [],
                        },
                    ))
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::convert::From<ParseRuleError> for Error {
        #[allow(deprecated)]
        fn from(source: ParseRuleError) -> Self {
            Error::ParseRuleError { 0: source }
        }
    }
}
pub mod rules {
    //! Cellular automata rules.
    //!
    //! For the notations of rule strings, please see
    //! [this article on LifeWiki](https://conwaylife.com/wiki/Rulestring).
    #[doc(hidden)]
    mod macros {
        //! A macro to generate the corresponding Generations rule of a rule.
        #![macro_use]
    }
    mod life {
        //! Totalistic Life-like rules.
        use crate::{
            cells::{CellRef, State, ALIVE, DEAD},
            error::Error,
            rules::Rule,
            search::Reason,
            world::World,
        };
        use bitflags::bitflags;
        use ca_rules::{ParseLife, ParseLifeGen};
        use std::str::FromStr;
        /// Flags to imply the state of a cell and its neighbors.
        struct ImplFlags {
            bits: u8,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for ImplFlags {
            #[inline]
            fn default() -> ImplFlags {
                ImplFlags {
                    bits: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for ImplFlags {}
        impl ::core::marker::StructuralPartialEq for ImplFlags {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for ImplFlags {
            #[inline]
            fn eq(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ::core::marker::StructuralEq for ImplFlags {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for ImplFlags {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<u8>;
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ImplFlags {
            #[inline]
            fn clone(&self) -> ImplFlags {
                {
                    let _: ::core::clone::AssertParamIsClone<u8>;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialOrd for ImplFlags {
            #[inline]
            fn partial_cmp(
                &self,
                other: &ImplFlags,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => match ::core::cmp::PartialOrd::partial_cmp(
                            &(*__self_0_0),
                            &(*__self_1_0),
                        ) {
                            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                                ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                            }
                            cmp => cmp,
                        },
                    },
                }
            }
            #[inline]
            fn lt(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Greater,
                            ) == ::core::cmp::Ordering::Less
                        }
                    },
                }
            }
            #[inline]
            fn le(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Greater,
                            ) != ::core::cmp::Ordering::Greater
                        }
                    },
                }
            }
            #[inline]
            fn gt(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Less,
                            ) == ::core::cmp::Ordering::Greater
                        }
                    },
                }
            }
            #[inline]
            fn ge(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Less,
                            ) != ::core::cmp::Ordering::Less
                        }
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Ord for ImplFlags {
            #[inline]
            fn cmp(&self, other: &ImplFlags) -> ::core::cmp::Ordering {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => match ::core::cmp::Ord::cmp(&(*__self_0_0), &(*__self_1_0)) {
                            ::core::cmp::Ordering::Equal => ::core::cmp::Ordering::Equal,
                            cmp => cmp,
                        },
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::hash::Hash for ImplFlags {
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                match *self {
                    ImplFlags {
                        bits: ref __self_0_0,
                    } => ::core::hash::Hash::hash(&(*__self_0_0), state),
                }
            }
        }
        impl ::bitflags::_core::fmt::Debug for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                #[allow(non_snake_case)]
                trait __BitFlags {
                    #[inline]
                    fn CONFLICT(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SUCC_ALIVE(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SUCC_DEAD(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SUCC(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SELF_ALIVE(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SELF_DEAD(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SELF(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn NBHD_ALIVE(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn NBHD_DEAD(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn NBHD(&self) -> bool {
                        false
                    }
                }
                impl __BitFlags for ImplFlags {
                    #[allow(deprecated)]
                    #[inline]
                    fn CONFLICT(&self) -> bool {
                        if Self::CONFLICT.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::CONFLICT.bits == Self::CONFLICT.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SUCC_ALIVE(&self) -> bool {
                        if Self::SUCC_ALIVE.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SUCC_ALIVE.bits == Self::SUCC_ALIVE.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SUCC_DEAD(&self) -> bool {
                        if Self::SUCC_DEAD.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SUCC_DEAD.bits == Self::SUCC_DEAD.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SUCC(&self) -> bool {
                        if Self::SUCC.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SUCC.bits == Self::SUCC.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SELF_ALIVE(&self) -> bool {
                        if Self::SELF_ALIVE.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SELF_ALIVE.bits == Self::SELF_ALIVE.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SELF_DEAD(&self) -> bool {
                        if Self::SELF_DEAD.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SELF_DEAD.bits == Self::SELF_DEAD.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SELF(&self) -> bool {
                        if Self::SELF.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SELF.bits == Self::SELF.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn NBHD_ALIVE(&self) -> bool {
                        if Self::NBHD_ALIVE.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::NBHD_ALIVE.bits == Self::NBHD_ALIVE.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn NBHD_DEAD(&self) -> bool {
                        if Self::NBHD_DEAD.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::NBHD_DEAD.bits == Self::NBHD_DEAD.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn NBHD(&self) -> bool {
                        if Self::NBHD.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::NBHD.bits == Self::NBHD.bits
                        }
                    }
                }
                let mut first = true;
                if <ImplFlags as __BitFlags>::CONFLICT(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("CONFLICT")?;
                }
                if <ImplFlags as __BitFlags>::SUCC_ALIVE(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SUCC_ALIVE")?;
                }
                if <ImplFlags as __BitFlags>::SUCC_DEAD(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SUCC_DEAD")?;
                }
                if <ImplFlags as __BitFlags>::SUCC(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SUCC")?;
                }
                if <ImplFlags as __BitFlags>::SELF_ALIVE(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SELF_ALIVE")?;
                }
                if <ImplFlags as __BitFlags>::SELF_DEAD(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SELF_DEAD")?;
                }
                if <ImplFlags as __BitFlags>::SELF(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SELF")?;
                }
                if <ImplFlags as __BitFlags>::NBHD_ALIVE(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("NBHD_ALIVE")?;
                }
                if <ImplFlags as __BitFlags>::NBHD_DEAD(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("NBHD_DEAD")?;
                }
                if <ImplFlags as __BitFlags>::NBHD(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("NBHD")?;
                }
                let extra_bits = self.bits & !ImplFlags::all().bits();
                if extra_bits != 0 {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("0x")?;
                    ::bitflags::_core::fmt::LowerHex::fmt(&extra_bits, f)?;
                }
                if first {
                    f.write_str("(empty)")?;
                }
                Ok(())
            }
        }
        impl ::bitflags::_core::fmt::Binary for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::Binary::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::Octal for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::Octal::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::LowerHex for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::LowerHex::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::UpperHex for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::UpperHex::fmt(&self.bits, f)
            }
        }
        #[allow(dead_code)]
        impl ImplFlags {
            /// A conflict is detected.
            pub const CONFLICT: ImplFlags = ImplFlags { bits: 0b_0000_0001 };
            /// The successor must be alive.
            pub const SUCC_ALIVE: ImplFlags = ImplFlags { bits: 0b_0000_0100 };
            /// The successor must be dead.
            pub const SUCC_DEAD: ImplFlags = ImplFlags { bits: 0b_0000_1000 };
            /// The state of the successor is implied.
            pub const SUCC: ImplFlags = ImplFlags {
                bits: Self::SUCC_ALIVE.bits | Self::SUCC_DEAD.bits,
            };
            /// The cell itself must be alive.
            pub const SELF_ALIVE: ImplFlags = ImplFlags { bits: 0b_0001_0000 };
            /// The cell itself must be dead.
            pub const SELF_DEAD: ImplFlags = ImplFlags { bits: 0b_0010_0000 };
            /// The state of the cell itself is implied.
            pub const SELF: ImplFlags = ImplFlags {
                bits: Self::SELF_ALIVE.bits | Self::SELF_DEAD.bits,
            };
            /// All unknown neighbors must be alive.
            pub const NBHD_ALIVE: ImplFlags = ImplFlags { bits: 0b_0100_0000 };
            /// All unknown neighbors must be dead.
            pub const NBHD_DEAD: ImplFlags = ImplFlags { bits: 0b_1000_0000 };
            /// The states of all unknown neighbors are implied.
            pub const NBHD: ImplFlags = ImplFlags {
                bits: Self::NBHD_ALIVE.bits | Self::NBHD_DEAD.bits,
            };
            /// Returns an empty set of flags
            #[inline]
            pub const fn empty() -> ImplFlags {
                ImplFlags { bits: 0 }
            }
            /// Returns the set containing all flags.
            #[inline]
            pub const fn all() -> ImplFlags {
                #[allow(non_snake_case)]
                trait __BitFlags {
                    const CONFLICT: u8 = 0;
                    const SUCC_ALIVE: u8 = 0;
                    const SUCC_DEAD: u8 = 0;
                    const SUCC: u8 = 0;
                    const SELF_ALIVE: u8 = 0;
                    const SELF_DEAD: u8 = 0;
                    const SELF: u8 = 0;
                    const NBHD_ALIVE: u8 = 0;
                    const NBHD_DEAD: u8 = 0;
                    const NBHD: u8 = 0;
                }
                impl __BitFlags for ImplFlags {
                    #[allow(deprecated)]
                    const CONFLICT: u8 = Self::CONFLICT.bits;
                    #[allow(deprecated)]
                    const SUCC_ALIVE: u8 = Self::SUCC_ALIVE.bits;
                    #[allow(deprecated)]
                    const SUCC_DEAD: u8 = Self::SUCC_DEAD.bits;
                    #[allow(deprecated)]
                    const SUCC: u8 = Self::SUCC.bits;
                    #[allow(deprecated)]
                    const SELF_ALIVE: u8 = Self::SELF_ALIVE.bits;
                    #[allow(deprecated)]
                    const SELF_DEAD: u8 = Self::SELF_DEAD.bits;
                    #[allow(deprecated)]
                    const SELF: u8 = Self::SELF.bits;
                    #[allow(deprecated)]
                    const NBHD_ALIVE: u8 = Self::NBHD_ALIVE.bits;
                    #[allow(deprecated)]
                    const NBHD_DEAD: u8 = Self::NBHD_DEAD.bits;
                    #[allow(deprecated)]
                    const NBHD: u8 = Self::NBHD.bits;
                }
                ImplFlags {
                    bits: <ImplFlags as __BitFlags>::CONFLICT
                        | <ImplFlags as __BitFlags>::SUCC_ALIVE
                        | <ImplFlags as __BitFlags>::SUCC_DEAD
                        | <ImplFlags as __BitFlags>::SUCC
                        | <ImplFlags as __BitFlags>::SELF_ALIVE
                        | <ImplFlags as __BitFlags>::SELF_DEAD
                        | <ImplFlags as __BitFlags>::SELF
                        | <ImplFlags as __BitFlags>::NBHD_ALIVE
                        | <ImplFlags as __BitFlags>::NBHD_DEAD
                        | <ImplFlags as __BitFlags>::NBHD,
                }
            }
            /// Returns the raw value of the flags currently stored.
            #[inline]
            pub const fn bits(&self) -> u8 {
                self.bits
            }
            /// Convert from underlying bit representation, unless that
            /// representation contains bits that do not correspond to a flag.
            #[inline]
            pub fn from_bits(bits: u8) -> ::bitflags::_core::option::Option<ImplFlags> {
                if (bits & !ImplFlags::all().bits()) == 0 {
                    ::bitflags::_core::option::Option::Some(ImplFlags { bits })
                } else {
                    ::bitflags::_core::option::Option::None
                }
            }
            /// Convert from underlying bit representation, dropping any bits
            /// that do not correspond to flags.
            #[inline]
            pub const fn from_bits_truncate(bits: u8) -> ImplFlags {
                ImplFlags {
                    bits: bits & ImplFlags::all().bits,
                }
            }
            /// Convert from underlying bit representation, preserving all
            /// bits (even those not corresponding to a defined flag).
            #[inline]
            pub const unsafe fn from_bits_unchecked(bits: u8) -> ImplFlags {
                ImplFlags { bits }
            }
            /// Returns `true` if no flags are currently stored.
            #[inline]
            pub const fn is_empty(&self) -> bool {
                self.bits() == ImplFlags::empty().bits()
            }
            /// Returns `true` if all flags are currently set.
            #[inline]
            pub const fn is_all(&self) -> bool {
                self.bits == ImplFlags::all().bits
            }
            /// Returns `true` if there are flags common to both `self` and `other`.
            #[inline]
            pub const fn intersects(&self, other: ImplFlags) -> bool {
                !ImplFlags {
                    bits: self.bits & other.bits,
                }
                .is_empty()
            }
            /// Returns `true` all of the flags in `other` are contained within `self`.
            #[inline]
            pub const fn contains(&self, other: ImplFlags) -> bool {
                (self.bits & other.bits) == other.bits
            }
            /// Inserts the specified flags in-place.
            #[inline]
            pub fn insert(&mut self, other: ImplFlags) {
                self.bits |= other.bits;
            }
            /// Removes the specified flags in-place.
            #[inline]
            pub fn remove(&mut self, other: ImplFlags) {
                self.bits &= !other.bits;
            }
            /// Toggles the specified flags in-place.
            #[inline]
            pub fn toggle(&mut self, other: ImplFlags) {
                self.bits ^= other.bits;
            }
            /// Inserts or removes the specified flags depending on the passed value.
            #[inline]
            pub fn set(&mut self, other: ImplFlags, value: bool) {
                if value {
                    self.insert(other);
                } else {
                    self.remove(other);
                }
            }
        }
        impl ::bitflags::_core::ops::BitOr for ImplFlags {
            type Output = ImplFlags;
            /// Returns the union of the two sets of flags.
            #[inline]
            fn bitor(self, other: ImplFlags) -> ImplFlags {
                ImplFlags {
                    bits: self.bits | other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitOrAssign for ImplFlags {
            /// Adds the set of flags.
            #[inline]
            fn bitor_assign(&mut self, other: ImplFlags) {
                self.bits |= other.bits;
            }
        }
        impl ::bitflags::_core::ops::BitXor for ImplFlags {
            type Output = ImplFlags;
            /// Returns the left flags, but with all the right flags toggled.
            #[inline]
            fn bitxor(self, other: ImplFlags) -> ImplFlags {
                ImplFlags {
                    bits: self.bits ^ other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitXorAssign for ImplFlags {
            /// Toggles the set of flags.
            #[inline]
            fn bitxor_assign(&mut self, other: ImplFlags) {
                self.bits ^= other.bits;
            }
        }
        impl ::bitflags::_core::ops::BitAnd for ImplFlags {
            type Output = ImplFlags;
            /// Returns the intersection between the two sets of flags.
            #[inline]
            fn bitand(self, other: ImplFlags) -> ImplFlags {
                ImplFlags {
                    bits: self.bits & other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitAndAssign for ImplFlags {
            /// Disables all flags disabled in the set.
            #[inline]
            fn bitand_assign(&mut self, other: ImplFlags) {
                self.bits &= other.bits;
            }
        }
        impl ::bitflags::_core::ops::Sub for ImplFlags {
            type Output = ImplFlags;
            /// Returns the set difference of the two sets of flags.
            #[inline]
            fn sub(self, other: ImplFlags) -> ImplFlags {
                ImplFlags {
                    bits: self.bits & !other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::SubAssign for ImplFlags {
            /// Disables all flags enabled in the set.
            #[inline]
            fn sub_assign(&mut self, other: ImplFlags) {
                self.bits &= !other.bits;
            }
        }
        impl ::bitflags::_core::ops::Not for ImplFlags {
            type Output = ImplFlags;
            /// Returns the complement of this set of flags.
            #[inline]
            fn not(self) -> ImplFlags {
                ImplFlags { bits: !self.bits } & ImplFlags::all()
            }
        }
        impl ::bitflags::_core::iter::Extend<ImplFlags> for ImplFlags {
            fn extend<T: ::bitflags::_core::iter::IntoIterator<Item = ImplFlags>>(
                &mut self,
                iterator: T,
            ) {
                for item in iterator {
                    self.insert(item)
                }
            }
        }
        impl ::bitflags::_core::iter::FromIterator<ImplFlags> for ImplFlags {
            fn from_iter<T: ::bitflags::_core::iter::IntoIterator<Item = ImplFlags>>(
                iterator: T,
            ) -> ImplFlags {
                let mut result = Self::empty();
                result.extend(iterator);
                result
            }
        }
        /// The neighborhood descriptor.
        ///
        /// It is a 12-bit integer of the form `0b_abcd_efgh_ij_kl`,
        /// where:
        ///
        /// * `0b_abcd` is the number of dead cells in the neighborhood.
        /// * `0b_efgh` is the number of living cells in the neighborhood.
        /// * `0b_ij` is the state of the successor.
        /// * `0b_kl` is the state of the cell itself.
        ///
        /// For `0b_ij` and `0b_kl`:
        /// * `0b_10` means dead,
        /// * `0b_01` means alive,
        /// * `0b_00` means unknown.
        pub struct NbhdDesc(u16);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for NbhdDesc {
            #[inline]
            fn clone(&self) -> NbhdDesc {
                {
                    let _: ::core::clone::AssertParamIsClone<u16>;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for NbhdDesc {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for NbhdDesc {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    NbhdDesc(ref __self_0_0) => {
                        let mut debug_trait_builder = f.debug_tuple("NbhdDesc");
                        let _ = debug_trait_builder.field(&&(*__self_0_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for NbhdDesc {
            #[inline]
            fn default() -> NbhdDesc {
                NbhdDesc(::core::default::Default::default())
            }
        }
        impl ::core::marker::StructuralPartialEq for NbhdDesc {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for NbhdDesc {
            #[inline]
            fn eq(&self, other: &NbhdDesc) -> bool {
                match *other {
                    NbhdDesc(ref __self_1_0) => match *self {
                        NbhdDesc(ref __self_0_0) => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &NbhdDesc) -> bool {
                match *other {
                    NbhdDesc(ref __self_1_0) => match *self {
                        NbhdDesc(ref __self_0_0) => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ::core::marker::StructuralEq for NbhdDesc {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for NbhdDesc {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<u16>;
                }
            }
        }
        /// Totalistic Life-like rules.
        pub struct Life {
            /// Whether the rule contains `B0`.
            b0: bool,
            /// Whether the rule contains `S8`.
            s8: bool,
            /// An array of actions for all neighborhood descriptors.
            impl_table: [ImplFlags; 1 << 12],
        }
        /// A parser for the rule.
        impl ParseLife for Life {
            fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
                Self::new(b, s)
            }
        }
        impl FromStr for Life {
            type Err = Error;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let rule: Life = ParseLife::parse_rule(input).map_err(Error::ParseRuleError)?;
                if rule.has_b0_s8() {
                    Err(Error::B0S8Error)
                } else {
                    Ok(rule)
                }
            }
        }
        impl Rule for Life {
            type Desc = NbhdDesc;
            const IS_GEN: bool = false;
            fn has_b0(&self) -> bool {
                self.b0
            }
            fn has_b0_s8(&self) -> bool {
                self.b0 && self.s8
            }
            fn gen(&self) -> usize {
                2
            }
            fn new_desc(state: State, succ_state: State) -> Self::Desc {
                let nbhd_state = match state {
                    ALIVE => 0x08,
                    _ => 0x80,
                };
                let succ_state = match succ_state {
                    ALIVE => 0b01,
                    _ => 0b10,
                };
                let state = match state {
                    ALIVE => 0b01,
                    _ => 0b10,
                };
                NbhdDesc(nbhd_state << 4 | succ_state << 2 | state)
            }
            fn update_desc(cell: CellRef<Self>, state: Option<State>, new: bool) {
                {
                    let state_num = match state {
                        Some(ALIVE) => 0x01,
                        Some(_) => 0x10,
                        None => 0,
                    };
                    for &neigh in cell.nbhd.iter() {
                        let neigh = neigh.unwrap();
                        let mut desc = neigh.desc.get();
                        if new {
                            desc.0 += state_num << 4;
                        } else {
                            desc.0 -= state_num << 4;
                        }
                        neigh.desc.set(desc);
                    }
                }
                let change_num = match state {
                    Some(ALIVE) => 0b01,
                    Some(_) => 0b10,
                    _ => 0,
                };
                if let Some(pred) = cell.pred {
                    let mut desc = pred.desc.get();
                    desc.0 ^= change_num << 2;
                    pred.desc.set(desc);
                }
                let mut desc = cell.desc.get();
                desc.0 ^= change_num;
                cell.desc.set(desc);
            }
            fn consistify<'a>(world: &mut World<'a, Self>, cell: CellRef<'a, Self>) -> bool {
                let flags = world.rule.impl_table[cell.desc.get().0 as usize];
                if flags.is_empty() {
                    return true;
                }
                if flags.contains(ImplFlags::CONFLICT) {
                    return false;
                }
                if flags.intersects(ImplFlags::SUCC) {
                    let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                        DEAD
                    } else {
                        ALIVE
                    };
                    let succ = cell.succ.unwrap();
                    return world.set_cell(succ, state, Reason::Deduce);
                }
                if flags.intersects(ImplFlags::SELF) {
                    let state = if flags.contains(ImplFlags::SELF_DEAD) {
                        DEAD
                    } else {
                        ALIVE
                    };
                    if !world.set_cell(cell, state, Reason::Deduce) {
                        return false;
                    }
                }
                if flags.intersects(ImplFlags::NBHD) {
                    {
                        let state = if flags.contains(ImplFlags::NBHD_DEAD) {
                            DEAD
                        } else {
                            ALIVE
                        };
                        for &neigh in cell.nbhd.iter() {
                            if let Some(neigh) = neigh {
                                if neigh.state.get().is_none()
                                    && !world.set_cell(neigh, state, Reason::Deduce)
                                {
                                    return false;
                                }
                            }
                        }
                    }
                }
                true
            }
        }
        /// The neighborhood descriptor.
        ///
        /// Including a descriptor for the corresponding non-Generations rule,
        /// and the states of the successor.
        pub struct NbhdDescGen(u16, Option<State>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for NbhdDescGen {
            #[inline]
            fn clone(&self) -> NbhdDescGen {
                {
                    let _: ::core::clone::AssertParamIsClone<u16>;
                    let _: ::core::clone::AssertParamIsClone<Option<State>>;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for NbhdDescGen {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for NbhdDescGen {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    NbhdDescGen(ref __self_0_0, ref __self_0_1) => {
                        let mut debug_trait_builder = f.debug_tuple("NbhdDescGen");
                        let _ = debug_trait_builder.field(&&(*__self_0_0));
                        let _ = debug_trait_builder.field(&&(*__self_0_1));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for NbhdDescGen {
            #[inline]
            fn default() -> NbhdDescGen {
                NbhdDescGen(
                    ::core::default::Default::default(),
                    ::core::default::Default::default(),
                )
            }
        }
        impl ::core::marker::StructuralPartialEq for NbhdDescGen {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for NbhdDescGen {
            #[inline]
            fn eq(&self, other: &NbhdDescGen) -> bool {
                match *other {
                    NbhdDescGen(ref __self_1_0, ref __self_1_1) => match *self {
                        NbhdDescGen(ref __self_0_0, ref __self_0_1) => {
                            (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &NbhdDescGen) -> bool {
                match *other {
                    NbhdDescGen(ref __self_1_0, ref __self_1_1) => match *self {
                        NbhdDescGen(ref __self_0_0, ref __self_0_1) => {
                            (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1)
                        }
                    },
                }
            }
        }
        impl ::core::marker::StructuralEq for NbhdDescGen {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for NbhdDescGen {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<u16>;
                    let _: ::core::cmp::AssertParamIsEq<Option<State>>;
                }
            }
        }
        /// Totalistic Life-like Generations rules.
        pub struct LifeGen {
            /// Whether the rule contains `B0`.
            b0: bool,
            /// Whether the rule contains `S8`.
            s8: bool,
            /// Number of states.
            gen: usize,
            /// An array of actions for all neighborhood descriptors.
            impl_table: [ImplFlags; 1 << 12],
        }
        impl LifeGen {
            /// Constructs a new rule from the `b` and `s` data
            /// and the number of states.
            pub fn new(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
                let life = Life::new(b, s);
                let impl_table = life.impl_table;
                Self {
                    b0: life.b0,
                    s8: life.s8,
                    gen,
                    impl_table,
                }
            }
            /// Converts to the corresponding non-Generations rule.
            pub fn non_gen(self) -> Life {
                Life {
                    b0: self.b0,
                    s8: self.s8,
                    impl_table: self.impl_table,
                }
            }
        }
        /// A parser for the rule.
        impl ParseLifeGen for LifeGen {
            fn from_bsg(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
                Self::new(b, s, gen)
            }
        }
        impl FromStr for LifeGen {
            type Err = Error;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let rule: LifeGen =
                    ParseLifeGen::parse_rule(input).map_err(Error::ParseRuleError)?;
                if rule.has_b0_s8() {
                    Err(Error::B0S8Error)
                } else {
                    Ok(rule)
                }
            }
        }
        /// NOTE: This implementation does work when the number of states is 2.
        impl Rule for LifeGen {
            type Desc = NbhdDescGen;
            const IS_GEN: bool = true;
            fn has_b0(&self) -> bool {
                self.b0
            }
            fn has_b0_s8(&self) -> bool {
                self.b0 && self.s8
            }
            fn gen(&self) -> usize {
                self.gen
            }
            fn new_desc(state: State, succ_state: State) -> Self::Desc {
                let desc = Life::new_desc(state, succ_state);
                NbhdDescGen(desc.0, Some(succ_state))
            }
            fn update_desc(cell: CellRef<Self>, state: Option<State>, new: bool) {
                {
                    let state_num = match state {
                        Some(ALIVE) => 0x01,
                        Some(_) => 0x10,
                        None => 0,
                    };
                    for &neigh in cell.nbhd.iter() {
                        let neigh = neigh.unwrap();
                        let mut desc = neigh.desc.get();
                        if new {
                            desc.0 += state_num << 4;
                        } else {
                            desc.0 -= state_num << 4;
                        }
                        neigh.desc.set(desc);
                    }
                }
                let change_num = match state {
                    Some(ALIVE) => 0b01,
                    Some(_) => 0b10,
                    _ => 0,
                };
                if let Some(pred) = cell.pred {
                    let mut desc = pred.desc.get();
                    desc.0 ^= change_num << 2;
                    desc.1 = if new { state } else { None };
                    pred.desc.set(desc);
                }
                let mut desc = cell.desc.get();
                desc.0 ^= change_num;
                cell.desc.set(desc);
            }
            fn consistify<'a>(world: &mut World<'a, Self>, cell: CellRef<'a, Self>) -> bool {
                let desc = cell.desc.get();
                let flags = world.rule.impl_table[desc.0 as usize];
                let gen = world.rule.gen;
                match cell.state.get() {
                    Some(DEAD) => {
                        if let Some(State(j)) = desc.1 {
                            if j >= 2 {
                                return false;
                            }
                        }
                        if flags.intersects(ImplFlags::SUCC) {
                            let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                                DEAD
                            } else {
                                ALIVE
                            };
                            let succ = cell.succ.unwrap();
                            return world.set_cell(succ, state, Reason::Deduce);
                        }
                    }
                    Some(ALIVE) => {
                        if let Some(State(j)) = desc.1 {
                            if j == 0 || j > 2 {
                                return false;
                            }
                        }
                        if flags.intersects(ImplFlags::SUCC) {
                            let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                                State(2)
                            } else {
                                ALIVE
                            };
                            let succ = cell.succ.unwrap();
                            return world.set_cell(succ, state, Reason::Deduce);
                        }
                    }
                    Some(State(i)) => {
                        if !(i >= 2) {
                            ::core::panicking::panic("assertion failed: i >= 2")
                        };
                        if let Some(State(j)) = desc.1 {
                            return j == (i + 1) % gen;
                        } else {
                            let succ = cell.succ.unwrap();
                            return world.set_cell(succ, State((i + 1) % gen), Reason::Deduce);
                        }
                    }
                    None => match desc.1 {
                        Some(DEAD) => {
                            if flags.contains(ImplFlags::SELF_ALIVE) {
                                return world.set_cell(cell, State(gen - 1), Reason::Deduce);
                            } else {
                                return true;
                            }
                        }
                        Some(ALIVE) => {
                            if flags.intersects(ImplFlags::SELF) {
                                let state = if flags.contains(ImplFlags::SELF_DEAD) {
                                    DEAD
                                } else {
                                    ALIVE
                                };
                                if !world.set_cell(cell, state, Reason::Deduce) {
                                    return false;
                                }
                            }
                        }
                        Some(State(j)) => {
                            return world.set_cell(cell, State(j - 1), Reason::Deduce);
                        }
                        None => return true,
                    },
                }
                if flags.is_empty() {
                    return true;
                }
                if flags.contains(ImplFlags::CONFLICT) {
                    return false;
                }
                {
                    if flags.intersects(ImplFlags::NBHD_ALIVE) {
                        for &neigh in cell.nbhd.iter() {
                            if let Some(neigh) = neigh {
                                if neigh.state.get().is_none()
                                    && !world.set_cell(neigh, ALIVE, Reason::Deduce)
                                {
                                    return false;
                                }
                            }
                        }
                    }
                }
                true
            }
        }
        impl Life {
            /// Constructs a new rule from the `b` and `s` data.
            pub fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
                let b0 = b.contains(&0);
                let s8 = s.contains(&8);
                let impl_table = [ImplFlags::empty(); 1 << 12];
                Life { b0, s8, impl_table }
                    .init_trans(b, s)
                    .init_conflict()
                    .init_impl()
                    .init_impl_nbhd()
            }
            /// Deduces the implication for the successor.
            fn init_trans(mut self, b: Vec<u8>, s: Vec<u8>) -> Self {
                for alives in 0..=8 {
                    let desc = ((8 - alives) << 8) | alives << 4;
                    let alives = alives as u8;
                    self.impl_table[desc | 0b10] |= if b.contains(&alives) {
                        ImplFlags::SUCC_ALIVE
                    } else {
                        ImplFlags::SUCC_DEAD
                    };
                    self.impl_table[desc | 0b01] |= if s.contains(&alives) {
                        ImplFlags::SUCC_ALIVE
                    } else {
                        ImplFlags::SUCC_DEAD
                    };
                    self.impl_table[desc] |= if b.contains(&alives) && s.contains(&alives) {
                        ImplFlags::SUCC_ALIVE
                    } else if !b.contains(&alives) && !s.contains(&alives) {
                        ImplFlags::SUCC_DEAD
                    } else {
                        ImplFlags::empty()
                    };
                }
                for unknowns in 1..=8 {
                    for alives in 0..=8 - unknowns {
                        let desc = (8 - alives - unknowns) << 8 | alives << 4;
                        let desc0 = (8 - alives - unknowns + 1) << 8 | alives << 4;
                        let desc1 = (8 - alives - unknowns) << 8 | (alives + 1) << 4;
                        for state in 0..=2 {
                            let trans0 = self.impl_table[desc0 | state];
                            if trans0 == self.impl_table[desc1 | state] {
                                self.impl_table[desc | state] |= trans0;
                            }
                        }
                    }
                }
                self
            }
            /// Deduces the conflicts.
            fn init_conflict(mut self) -> Self {
                for nbhd_state in 0..0xff {
                    for state in 0..=2 {
                        let desc = nbhd_state << 4 | state;
                        if self.impl_table[desc].contains(ImplFlags::SUCC_ALIVE) {
                            self.impl_table[desc | 0b10 << 2] = ImplFlags::CONFLICT;
                        } else if self.impl_table[desc].contains(ImplFlags::SUCC_DEAD) {
                            self.impl_table[desc | 0b01 << 2] = ImplFlags::CONFLICT;
                        }
                    }
                }
                self
            }
            /// Deduces the implication for the cell itself.
            fn init_impl(mut self) -> Self {
                for unknowns in 0..=8 {
                    for alives in 0..=8 - unknowns {
                        let desc = (8 - alives - unknowns) << 8 | alives << 4;
                        for succ_state in 1..=2 {
                            let flag = if succ_state == 0b10 {
                                ImplFlags::SUCC_ALIVE | ImplFlags::CONFLICT
                            } else {
                                ImplFlags::SUCC_DEAD | ImplFlags::CONFLICT
                            };
                            let possibly_dead = !self.impl_table[desc | 0b10].intersects(flag);
                            let possibly_alive = !self.impl_table[desc | 0b01].intersects(flag);
                            let index = desc | succ_state << 2;
                            if possibly_dead && !possibly_alive {
                                self.impl_table[index] |= ImplFlags::SELF_DEAD;
                            } else if !possibly_dead && possibly_alive {
                                self.impl_table[index] |= ImplFlags::SELF_ALIVE;
                            } else if !possibly_dead && !possibly_alive {
                                self.impl_table[index] = ImplFlags::CONFLICT;
                            }
                        }
                    }
                }
                self
            }
            ///  Deduces the implication for the neighbors.
            fn init_impl_nbhd(mut self) -> Self {
                for unknowns in 1..=8 {
                    for alives in 0..=8 - unknowns {
                        let desc = (8 - alives - unknowns) << 8 | alives << 4;
                        let desc0 = (8 - alives - unknowns + 1) << 8 | alives << 4;
                        let desc1 = (8 - alives - unknowns) << 8 | (alives + 1) << 4;
                        for succ_state in 1..=2 {
                            let flag = if succ_state == 0b10 {
                                ImplFlags::SUCC_ALIVE | ImplFlags::CONFLICT
                            } else {
                                ImplFlags::SUCC_DEAD | ImplFlags::CONFLICT
                            };
                            let index = desc | succ_state << 2;
                            for state in 0..=2 {
                                let possibly_dead =
                                    !self.impl_table[desc0 | state].intersects(flag);
                                let possibly_alive =
                                    !self.impl_table[desc1 | state].intersects(flag);
                                if possibly_dead && !possibly_alive {
                                    self.impl_table[index | state] |= ImplFlags::NBHD_DEAD;
                                } else if !possibly_dead && possibly_alive {
                                    self.impl_table[index | state] |= ImplFlags::NBHD_ALIVE;
                                } else if !possibly_dead && !possibly_alive {
                                    self.impl_table[index | state] = ImplFlags::CONFLICT;
                                }
                            }
                        }
                    }
                }
                self
            }
        }
    }
    mod ntlife {
        //! Non-totalistic Life-like rules.
        use crate::{
            cells::{CellRef, State, ALIVE, DEAD},
            error::Error,
            rules::Rule,
            search::Reason,
            world::World,
        };
        use bitflags::bitflags;
        use ca_rules::{ParseNtLife, ParseNtLifeGen};
        use std::str::FromStr;
        /// Flags to imply the state of a cell and its neighbors.
        struct ImplFlags {
            bits: u32,
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for ImplFlags {
            #[inline]
            fn default() -> ImplFlags {
                ImplFlags {
                    bits: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for ImplFlags {}
        impl ::core::marker::StructuralPartialEq for ImplFlags {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for ImplFlags {
            #[inline]
            fn eq(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ::core::marker::StructuralEq for ImplFlags {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for ImplFlags {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<u32>;
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for ImplFlags {
            #[inline]
            fn clone(&self) -> ImplFlags {
                {
                    let _: ::core::clone::AssertParamIsClone<u32>;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialOrd for ImplFlags {
            #[inline]
            fn partial_cmp(
                &self,
                other: &ImplFlags,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => match ::core::cmp::PartialOrd::partial_cmp(
                            &(*__self_0_0),
                            &(*__self_1_0),
                        ) {
                            ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                                ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                            }
                            cmp => cmp,
                        },
                    },
                }
            }
            #[inline]
            fn lt(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Greater,
                            ) == ::core::cmp::Ordering::Less
                        }
                    },
                }
            }
            #[inline]
            fn le(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Greater,
                            ) != ::core::cmp::Ordering::Greater
                        }
                    },
                }
            }
            #[inline]
            fn gt(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Less,
                            ) == ::core::cmp::Ordering::Greater
                        }
                    },
                }
            }
            #[inline]
            fn ge(&self, other: &ImplFlags) -> bool {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => {
                            ::core::option::Option::unwrap_or(
                                ::core::cmp::PartialOrd::partial_cmp(
                                    &(*__self_0_0),
                                    &(*__self_1_0),
                                ),
                                ::core::cmp::Ordering::Less,
                            ) != ::core::cmp::Ordering::Less
                        }
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Ord for ImplFlags {
            #[inline]
            fn cmp(&self, other: &ImplFlags) -> ::core::cmp::Ordering {
                match *other {
                    ImplFlags {
                        bits: ref __self_1_0,
                    } => match *self {
                        ImplFlags {
                            bits: ref __self_0_0,
                        } => match ::core::cmp::Ord::cmp(&(*__self_0_0), &(*__self_1_0)) {
                            ::core::cmp::Ordering::Equal => ::core::cmp::Ordering::Equal,
                            cmp => cmp,
                        },
                    },
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::hash::Hash for ImplFlags {
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                match *self {
                    ImplFlags {
                        bits: ref __self_0_0,
                    } => ::core::hash::Hash::hash(&(*__self_0_0), state),
                }
            }
        }
        impl ::bitflags::_core::fmt::Debug for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                #[allow(non_snake_case)]
                trait __BitFlags {
                    #[inline]
                    fn CONFLICT(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SUCC_ALIVE(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SUCC_DEAD(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SUCC(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SELF_ALIVE(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SELF_DEAD(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn SELF(&self) -> bool {
                        false
                    }
                    #[inline]
                    fn NBHD(&self) -> bool {
                        false
                    }
                }
                impl __BitFlags for ImplFlags {
                    #[allow(deprecated)]
                    #[inline]
                    fn CONFLICT(&self) -> bool {
                        if Self::CONFLICT.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::CONFLICT.bits == Self::CONFLICT.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SUCC_ALIVE(&self) -> bool {
                        if Self::SUCC_ALIVE.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SUCC_ALIVE.bits == Self::SUCC_ALIVE.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SUCC_DEAD(&self) -> bool {
                        if Self::SUCC_DEAD.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SUCC_DEAD.bits == Self::SUCC_DEAD.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SUCC(&self) -> bool {
                        if Self::SUCC.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SUCC.bits == Self::SUCC.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SELF_ALIVE(&self) -> bool {
                        if Self::SELF_ALIVE.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SELF_ALIVE.bits == Self::SELF_ALIVE.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SELF_DEAD(&self) -> bool {
                        if Self::SELF_DEAD.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SELF_DEAD.bits == Self::SELF_DEAD.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn SELF(&self) -> bool {
                        if Self::SELF.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::SELF.bits == Self::SELF.bits
                        }
                    }
                    #[allow(deprecated)]
                    #[inline]
                    fn NBHD(&self) -> bool {
                        if Self::NBHD.bits == 0 && self.bits != 0 {
                            false
                        } else {
                            self.bits & Self::NBHD.bits == Self::NBHD.bits
                        }
                    }
                }
                let mut first = true;
                if <ImplFlags as __BitFlags>::CONFLICT(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("CONFLICT")?;
                }
                if <ImplFlags as __BitFlags>::SUCC_ALIVE(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SUCC_ALIVE")?;
                }
                if <ImplFlags as __BitFlags>::SUCC_DEAD(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SUCC_DEAD")?;
                }
                if <ImplFlags as __BitFlags>::SUCC(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SUCC")?;
                }
                if <ImplFlags as __BitFlags>::SELF_ALIVE(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SELF_ALIVE")?;
                }
                if <ImplFlags as __BitFlags>::SELF_DEAD(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SELF_DEAD")?;
                }
                if <ImplFlags as __BitFlags>::SELF(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("SELF")?;
                }
                if <ImplFlags as __BitFlags>::NBHD(self) {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("NBHD")?;
                }
                let extra_bits = self.bits & !ImplFlags::all().bits();
                if extra_bits != 0 {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    first = false;
                    f.write_str("0x")?;
                    ::bitflags::_core::fmt::LowerHex::fmt(&extra_bits, f)?;
                }
                if first {
                    f.write_str("(empty)")?;
                }
                Ok(())
            }
        }
        impl ::bitflags::_core::fmt::Binary for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::Binary::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::Octal for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::Octal::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::LowerHex for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::LowerHex::fmt(&self.bits, f)
            }
        }
        impl ::bitflags::_core::fmt::UpperHex for ImplFlags {
            fn fmt(
                &self,
                f: &mut ::bitflags::_core::fmt::Formatter,
            ) -> ::bitflags::_core::fmt::Result {
                ::bitflags::_core::fmt::UpperHex::fmt(&self.bits, f)
            }
        }
        #[allow(dead_code)]
        impl ImplFlags {
            /// A conflict is detected.
            pub const CONFLICT: ImplFlags = ImplFlags { bits: 0b_0000_0001 };
            /// The successor must be alive.
            pub const SUCC_ALIVE: ImplFlags = ImplFlags { bits: 0b_0000_0100 };
            /// The successor must be dead.
            pub const SUCC_DEAD: ImplFlags = ImplFlags { bits: 0b_0000_1000 };
            /// The state of the successor is implied.
            pub const SUCC: ImplFlags = ImplFlags {
                bits: Self::SUCC_ALIVE.bits | Self::SUCC_DEAD.bits,
            };
            /// The cell itself must be alive.
            pub const SELF_ALIVE: ImplFlags = ImplFlags { bits: 0b_0001_0000 };
            /// The cell itself must be dead.
            pub const SELF_DEAD: ImplFlags = ImplFlags { bits: 0b_0010_0000 };
            /// The state of the cell itself is implied.
            pub const SELF: ImplFlags = ImplFlags {
                bits: Self::SELF_ALIVE.bits | Self::SELF_DEAD.bits,
            };
            /// The state of at least one unknown neighbor is implied.
            pub const NBHD: ImplFlags = ImplFlags { bits: 0xffff << 6 };
            /// Returns an empty set of flags
            #[inline]
            pub const fn empty() -> ImplFlags {
                ImplFlags { bits: 0 }
            }
            /// Returns the set containing all flags.
            #[inline]
            pub const fn all() -> ImplFlags {
                #[allow(non_snake_case)]
                trait __BitFlags {
                    const CONFLICT: u32 = 0;
                    const SUCC_ALIVE: u32 = 0;
                    const SUCC_DEAD: u32 = 0;
                    const SUCC: u32 = 0;
                    const SELF_ALIVE: u32 = 0;
                    const SELF_DEAD: u32 = 0;
                    const SELF: u32 = 0;
                    const NBHD: u32 = 0;
                }
                impl __BitFlags for ImplFlags {
                    #[allow(deprecated)]
                    const CONFLICT: u32 = Self::CONFLICT.bits;
                    #[allow(deprecated)]
                    const SUCC_ALIVE: u32 = Self::SUCC_ALIVE.bits;
                    #[allow(deprecated)]
                    const SUCC_DEAD: u32 = Self::SUCC_DEAD.bits;
                    #[allow(deprecated)]
                    const SUCC: u32 = Self::SUCC.bits;
                    #[allow(deprecated)]
                    const SELF_ALIVE: u32 = Self::SELF_ALIVE.bits;
                    #[allow(deprecated)]
                    const SELF_DEAD: u32 = Self::SELF_DEAD.bits;
                    #[allow(deprecated)]
                    const SELF: u32 = Self::SELF.bits;
                    #[allow(deprecated)]
                    const NBHD: u32 = Self::NBHD.bits;
                }
                ImplFlags {
                    bits: <ImplFlags as __BitFlags>::CONFLICT
                        | <ImplFlags as __BitFlags>::SUCC_ALIVE
                        | <ImplFlags as __BitFlags>::SUCC_DEAD
                        | <ImplFlags as __BitFlags>::SUCC
                        | <ImplFlags as __BitFlags>::SELF_ALIVE
                        | <ImplFlags as __BitFlags>::SELF_DEAD
                        | <ImplFlags as __BitFlags>::SELF
                        | <ImplFlags as __BitFlags>::NBHD,
                }
            }
            /// Returns the raw value of the flags currently stored.
            #[inline]
            pub const fn bits(&self) -> u32 {
                self.bits
            }
            /// Convert from underlying bit representation, unless that
            /// representation contains bits that do not correspond to a flag.
            #[inline]
            pub fn from_bits(bits: u32) -> ::bitflags::_core::option::Option<ImplFlags> {
                if (bits & !ImplFlags::all().bits()) == 0 {
                    ::bitflags::_core::option::Option::Some(ImplFlags { bits })
                } else {
                    ::bitflags::_core::option::Option::None
                }
            }
            /// Convert from underlying bit representation, dropping any bits
            /// that do not correspond to flags.
            #[inline]
            pub const fn from_bits_truncate(bits: u32) -> ImplFlags {
                ImplFlags {
                    bits: bits & ImplFlags::all().bits,
                }
            }
            /// Convert from underlying bit representation, preserving all
            /// bits (even those not corresponding to a defined flag).
            #[inline]
            pub const unsafe fn from_bits_unchecked(bits: u32) -> ImplFlags {
                ImplFlags { bits }
            }
            /// Returns `true` if no flags are currently stored.
            #[inline]
            pub const fn is_empty(&self) -> bool {
                self.bits() == ImplFlags::empty().bits()
            }
            /// Returns `true` if all flags are currently set.
            #[inline]
            pub const fn is_all(&self) -> bool {
                self.bits == ImplFlags::all().bits
            }
            /// Returns `true` if there are flags common to both `self` and `other`.
            #[inline]
            pub const fn intersects(&self, other: ImplFlags) -> bool {
                !ImplFlags {
                    bits: self.bits & other.bits,
                }
                .is_empty()
            }
            /// Returns `true` all of the flags in `other` are contained within `self`.
            #[inline]
            pub const fn contains(&self, other: ImplFlags) -> bool {
                (self.bits & other.bits) == other.bits
            }
            /// Inserts the specified flags in-place.
            #[inline]
            pub fn insert(&mut self, other: ImplFlags) {
                self.bits |= other.bits;
            }
            /// Removes the specified flags in-place.
            #[inline]
            pub fn remove(&mut self, other: ImplFlags) {
                self.bits &= !other.bits;
            }
            /// Toggles the specified flags in-place.
            #[inline]
            pub fn toggle(&mut self, other: ImplFlags) {
                self.bits ^= other.bits;
            }
            /// Inserts or removes the specified flags depending on the passed value.
            #[inline]
            pub fn set(&mut self, other: ImplFlags, value: bool) {
                if value {
                    self.insert(other);
                } else {
                    self.remove(other);
                }
            }
        }
        impl ::bitflags::_core::ops::BitOr for ImplFlags {
            type Output = ImplFlags;
            /// Returns the union of the two sets of flags.
            #[inline]
            fn bitor(self, other: ImplFlags) -> ImplFlags {
                ImplFlags {
                    bits: self.bits | other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitOrAssign for ImplFlags {
            /// Adds the set of flags.
            #[inline]
            fn bitor_assign(&mut self, other: ImplFlags) {
                self.bits |= other.bits;
            }
        }
        impl ::bitflags::_core::ops::BitXor for ImplFlags {
            type Output = ImplFlags;
            /// Returns the left flags, but with all the right flags toggled.
            #[inline]
            fn bitxor(self, other: ImplFlags) -> ImplFlags {
                ImplFlags {
                    bits: self.bits ^ other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitXorAssign for ImplFlags {
            /// Toggles the set of flags.
            #[inline]
            fn bitxor_assign(&mut self, other: ImplFlags) {
                self.bits ^= other.bits;
            }
        }
        impl ::bitflags::_core::ops::BitAnd for ImplFlags {
            type Output = ImplFlags;
            /// Returns the intersection between the two sets of flags.
            #[inline]
            fn bitand(self, other: ImplFlags) -> ImplFlags {
                ImplFlags {
                    bits: self.bits & other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::BitAndAssign for ImplFlags {
            /// Disables all flags disabled in the set.
            #[inline]
            fn bitand_assign(&mut self, other: ImplFlags) {
                self.bits &= other.bits;
            }
        }
        impl ::bitflags::_core::ops::Sub for ImplFlags {
            type Output = ImplFlags;
            /// Returns the set difference of the two sets of flags.
            #[inline]
            fn sub(self, other: ImplFlags) -> ImplFlags {
                ImplFlags {
                    bits: self.bits & !other.bits,
                }
            }
        }
        impl ::bitflags::_core::ops::SubAssign for ImplFlags {
            /// Disables all flags enabled in the set.
            #[inline]
            fn sub_assign(&mut self, other: ImplFlags) {
                self.bits &= !other.bits;
            }
        }
        impl ::bitflags::_core::ops::Not for ImplFlags {
            type Output = ImplFlags;
            /// Returns the complement of this set of flags.
            #[inline]
            fn not(self) -> ImplFlags {
                ImplFlags { bits: !self.bits } & ImplFlags::all()
            }
        }
        impl ::bitflags::_core::iter::Extend<ImplFlags> for ImplFlags {
            fn extend<T: ::bitflags::_core::iter::IntoIterator<Item = ImplFlags>>(
                &mut self,
                iterator: T,
            ) {
                for item in iterator {
                    self.insert(item)
                }
            }
        }
        impl ::bitflags::_core::iter::FromIterator<ImplFlags> for ImplFlags {
            fn from_iter<T: ::bitflags::_core::iter::IntoIterator<Item = ImplFlags>>(
                iterator: T,
            ) -> ImplFlags {
                let mut result = Self::empty();
                result.extend(iterator);
                result
            }
        }
        /// The neighborhood descriptor.
        ///
        /// It is a 20-bit integer of the form `0b_abcdefgh_ijklmnop_qr_st`,
        /// where:
        ///
        /// * `0b_ai`, `0b_bj`, ..., `0b_hp` are the states of the eight neighbors,
        /// * `0b_qr` is the state of the successor.
        /// * `0b_st` is the state of the cell itself.
        /// * `0b_10` means dead,
        /// * `0b_01` means alive,
        /// * `0b_00` means unknown.
        pub struct NbhdDesc(u32);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for NbhdDesc {
            #[inline]
            fn clone(&self) -> NbhdDesc {
                {
                    let _: ::core::clone::AssertParamIsClone<u32>;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for NbhdDesc {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for NbhdDesc {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    NbhdDesc(ref __self_0_0) => {
                        let mut debug_trait_builder = f.debug_tuple("NbhdDesc");
                        let _ = debug_trait_builder.field(&&(*__self_0_0));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for NbhdDesc {
            #[inline]
            fn default() -> NbhdDesc {
                NbhdDesc(::core::default::Default::default())
            }
        }
        impl ::core::marker::StructuralPartialEq for NbhdDesc {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for NbhdDesc {
            #[inline]
            fn eq(&self, other: &NbhdDesc) -> bool {
                match *other {
                    NbhdDesc(ref __self_1_0) => match *self {
                        NbhdDesc(ref __self_0_0) => (*__self_0_0) == (*__self_1_0),
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &NbhdDesc) -> bool {
                match *other {
                    NbhdDesc(ref __self_1_0) => match *self {
                        NbhdDesc(ref __self_0_0) => (*__self_0_0) != (*__self_1_0),
                    },
                }
            }
        }
        impl ::core::marker::StructuralEq for NbhdDesc {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for NbhdDesc {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<u32>;
                }
            }
        }
        /// Non-totalistic Life-like rules.
        ///
        /// This includes any rule that can be converted to a non-totalistic
        /// Life-like rule: isotropic non-totalistic rules,
        /// non-isotropic rules, hexagonal rules, rules with von Neumann
        /// neighborhoods, etc.
        pub struct NtLife {
            /// Whether the rule contains `B0`.
            b0: bool,
            /// Whether the rule contains `S8`.
            s8: bool,
            /// An array of actions for all neighborhood descriptors.
            impl_table: Vec<ImplFlags>,
        }
        /// A parser for the rule.
        impl ParseNtLife for NtLife {
            fn from_bs(b: Vec<u8>, s: Vec<u8>) -> Self {
                Self::new(b, s)
            }
        }
        impl FromStr for NtLife {
            type Err = Error;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let rule: NtLife = ParseNtLife::parse_rule(input).map_err(Error::ParseRuleError)?;
                if rule.has_b0_s8() {
                    Err(Error::B0S8Error)
                } else {
                    Ok(rule)
                }
            }
        }
        impl Rule for NtLife {
            type Desc = NbhdDesc;
            const IS_GEN: bool = false;
            fn has_b0(&self) -> bool {
                self.b0
            }
            fn has_b0_s8(&self) -> bool {
                self.b0 && self.s8
            }
            fn gen(&self) -> usize {
                2
            }
            fn new_desc(state: State, succ_state: State) -> Self::Desc {
                let nbhd_state = match state {
                    ALIVE => 0x00ff,
                    _ => 0xff00,
                };
                let succ_state = match succ_state {
                    ALIVE => 0b01,
                    _ => 0b10,
                };
                let state = match state {
                    ALIVE => 0b01,
                    _ => 0b10,
                };
                NbhdDesc(nbhd_state << 4 | succ_state << 2 | state)
            }
            fn update_desc(cell: CellRef<Self>, state: Option<State>, _new: bool) {
                {
                    let nbhd_change_num = match state {
                        Some(ALIVE) => 0x0001,
                        Some(_) => 0x0100,
                        _ => 0x0000,
                    };
                    for (i, &neigh) in cell.nbhd.iter().rev().enumerate() {
                        let neigh = neigh.unwrap();
                        let mut desc = neigh.desc.get();
                        desc.0 ^= nbhd_change_num << i << 4;
                        neigh.desc.set(desc);
                    }
                }
                let change_num = match state {
                    Some(ALIVE) => 0b01,
                    Some(_) => 0b10,
                    _ => 0,
                };
                if let Some(pred) = cell.pred {
                    let mut desc = pred.desc.get();
                    desc.0 ^= change_num << 2;
                    pred.desc.set(desc);
                }
                let mut desc = cell.desc.get();
                desc.0 ^= change_num;
                cell.desc.set(desc);
            }
            fn consistify<'a>(world: &mut World<'a, Self>, cell: CellRef<'a, Self>) -> bool {
                let flags = world.rule.impl_table[cell.desc.get().0 as usize];
                if flags.is_empty() {
                    return true;
                }
                if flags.contains(ImplFlags::CONFLICT) {
                    return false;
                }
                if flags.intersects(ImplFlags::SUCC) {
                    let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                        DEAD
                    } else {
                        ALIVE
                    };
                    let succ = cell.succ.unwrap();
                    return world.set_cell(succ, state, Reason::Deduce);
                }
                if flags.intersects(ImplFlags::SELF) {
                    let state = if flags.contains(ImplFlags::SELF_DEAD) {
                        DEAD
                    } else {
                        ALIVE
                    };
                    if !world.set_cell(cell, state, Reason::Deduce) {
                        return false;
                    }
                }
                if flags.intersects(ImplFlags::NBHD) {
                    {
                        for (i, &neigh) in cell.nbhd.iter().enumerate() {
                            if flags.intersects(ImplFlags::from_bits(3 << (2 * i + 6)).unwrap()) {
                                if let Some(neigh) = neigh {
                                    let state = if flags
                                        .contains(ImplFlags::from_bits(1 << (2 * i + 7)).unwrap())
                                    {
                                        DEAD
                                    } else {
                                        ALIVE
                                    };
                                    if !world.set_cell(neigh, state, Reason::Deduce) {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
                true
            }
        }
        /// The neighborhood descriptor.
        ///
        /// Including a descriptor for the corresponding non-Generations rule,
        /// and the states of the successor.
        pub struct NbhdDescGen(u32, Option<State>);
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::clone::Clone for NbhdDescGen {
            #[inline]
            fn clone(&self) -> NbhdDescGen {
                {
                    let _: ::core::clone::AssertParamIsClone<u32>;
                    let _: ::core::clone::AssertParamIsClone<Option<State>>;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::marker::Copy for NbhdDescGen {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::fmt::Debug for NbhdDescGen {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    NbhdDescGen(ref __self_0_0, ref __self_0_1) => {
                        let mut debug_trait_builder = f.debug_tuple("NbhdDescGen");
                        let _ = debug_trait_builder.field(&&(*__self_0_0));
                        let _ = debug_trait_builder.field(&&(*__self_0_1));
                        debug_trait_builder.finish()
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::default::Default for NbhdDescGen {
            #[inline]
            fn default() -> NbhdDescGen {
                NbhdDescGen(
                    ::core::default::Default::default(),
                    ::core::default::Default::default(),
                )
            }
        }
        impl ::core::marker::StructuralPartialEq for NbhdDescGen {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::PartialEq for NbhdDescGen {
            #[inline]
            fn eq(&self, other: &NbhdDescGen) -> bool {
                match *other {
                    NbhdDescGen(ref __self_1_0, ref __self_1_1) => match *self {
                        NbhdDescGen(ref __self_0_0, ref __self_0_1) => {
                            (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1)
                        }
                    },
                }
            }
            #[inline]
            fn ne(&self, other: &NbhdDescGen) -> bool {
                match *other {
                    NbhdDescGen(ref __self_1_0, ref __self_1_1) => match *self {
                        NbhdDescGen(ref __self_0_0, ref __self_0_1) => {
                            (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1)
                        }
                    },
                }
            }
        }
        impl ::core::marker::StructuralEq for NbhdDescGen {}
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl ::core::cmp::Eq for NbhdDescGen {
            #[inline]
            #[doc(hidden)]
            fn assert_receiver_is_total_eq(&self) -> () {
                {
                    let _: ::core::cmp::AssertParamIsEq<u32>;
                    let _: ::core::cmp::AssertParamIsEq<Option<State>>;
                }
            }
        }
        /// Non-totalistic Life-like Generations rules.
        ///
        /// This includes any rule that can be converted to a non-totalistic
        /// Life-like Generations rule.
        pub struct NtLifeGen {
            /// Whether the rule contains `B0`.
            b0: bool,
            /// Whether the rule contains `S8`.
            s8: bool,
            /// Number of states.
            gen: usize,
            /// An array of actions for all neighborhood descriptors.
            impl_table: Vec<ImplFlags>,
        }
        impl NtLifeGen {
            /// Constructs a new rule from the `b` and `s` data
            /// and the number of states.
            pub fn new(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
                let life = NtLife::new(b, s);
                let impl_table = life.impl_table;
                Self {
                    b0: life.b0,
                    s8: life.s8,
                    gen,
                    impl_table,
                }
            }
            /// Converts to the corresponding non-Generations rule.
            pub fn non_gen(self) -> NtLife {
                NtLife {
                    b0: self.b0,
                    s8: self.s8,
                    impl_table: self.impl_table,
                }
            }
        }
        /// A parser for the rule.
        impl ParseNtLifeGen for NtLifeGen {
            fn from_bsg(b: Vec<u8>, s: Vec<u8>, gen: usize) -> Self {
                Self::new(b, s, gen)
            }
        }
        impl FromStr for NtLifeGen {
            type Err = Error;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let rule: NtLifeGen =
                    ParseNtLifeGen::parse_rule(input).map_err(Error::ParseRuleError)?;
                if rule.has_b0_s8() {
                    Err(Error::B0S8Error)
                } else {
                    Ok(rule)
                }
            }
        }
        /// NOTE: This implementation does work when the number of states is 2.
        impl Rule for NtLifeGen {
            type Desc = NbhdDescGen;
            const IS_GEN: bool = true;
            fn has_b0(&self) -> bool {
                self.b0
            }
            fn has_b0_s8(&self) -> bool {
                self.b0 && self.s8
            }
            fn gen(&self) -> usize {
                self.gen
            }
            fn new_desc(state: State, succ_state: State) -> Self::Desc {
                let desc = NtLife::new_desc(state, succ_state);
                NbhdDescGen(desc.0, Some(succ_state))
            }
            fn update_desc(cell: CellRef<Self>, state: Option<State>, _new: bool) {
                {
                    let nbhd_change_num = match state {
                        Some(ALIVE) => 0x0001,
                        Some(_) => 0x0100,
                        _ => 0x0000,
                    };
                    for (i, &neigh) in cell.nbhd.iter().rev().enumerate() {
                        let neigh = neigh.unwrap();
                        let mut desc = neigh.desc.get();
                        desc.0 ^= nbhd_change_num << i << 4;
                        neigh.desc.set(desc);
                    }
                }
                let change_num = match state {
                    Some(ALIVE) => 0b01,
                    Some(_) => 0b10,
                    _ => 0,
                };
                if let Some(pred) = cell.pred {
                    let mut desc = pred.desc.get();
                    desc.0 ^= change_num << 2;
                    desc.1 = if _new { state } else { None };
                    pred.desc.set(desc);
                }
                let mut desc = cell.desc.get();
                desc.0 ^= change_num;
                cell.desc.set(desc);
            }
            fn consistify<'a>(world: &mut World<'a, Self>, cell: CellRef<'a, Self>) -> bool {
                let desc = cell.desc.get();
                let flags = world.rule.impl_table[desc.0 as usize];
                let gen = world.rule.gen;
                match cell.state.get() {
                    Some(DEAD) => {
                        if let Some(State(j)) = desc.1 {
                            if j >= 2 {
                                return false;
                            }
                        }
                        if flags.intersects(ImplFlags::SUCC) {
                            let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                                DEAD
                            } else {
                                ALIVE
                            };
                            let succ = cell.succ.unwrap();
                            return world.set_cell(succ, state, Reason::Deduce);
                        }
                    }
                    Some(ALIVE) => {
                        if let Some(State(j)) = desc.1 {
                            if j == 0 || j > 2 {
                                return false;
                            }
                        }
                        if flags.intersects(ImplFlags::SUCC) {
                            let state = if flags.contains(ImplFlags::SUCC_DEAD) {
                                State(2)
                            } else {
                                ALIVE
                            };
                            let succ = cell.succ.unwrap();
                            return world.set_cell(succ, state, Reason::Deduce);
                        }
                    }
                    Some(State(i)) => {
                        if !(i >= 2) {
                            ::core::panicking::panic("assertion failed: i >= 2")
                        };
                        if let Some(State(j)) = desc.1 {
                            return j == (i + 1) % gen;
                        } else {
                            let succ = cell.succ.unwrap();
                            return world.set_cell(succ, State((i + 1) % gen), Reason::Deduce);
                        }
                    }
                    None => match desc.1 {
                        Some(DEAD) => {
                            if flags.contains(ImplFlags::SELF_ALIVE) {
                                return world.set_cell(cell, State(gen - 1), Reason::Deduce);
                            } else {
                                return true;
                            }
                        }
                        Some(ALIVE) => {
                            if flags.intersects(ImplFlags::SELF) {
                                let state = if flags.contains(ImplFlags::SELF_DEAD) {
                                    DEAD
                                } else {
                                    ALIVE
                                };
                                if !world.set_cell(cell, state, Reason::Deduce) {
                                    return false;
                                }
                            }
                        }
                        Some(State(j)) => {
                            return world.set_cell(cell, State(j - 1), Reason::Deduce);
                        }
                        None => return true,
                    },
                }
                if flags.is_empty() {
                    return true;
                }
                if flags.contains(ImplFlags::CONFLICT) {
                    return false;
                }
                {
                    if flags.intersects(ImplFlags::NBHD) {
                        for (i, &neigh) in cell.nbhd.iter().enumerate() {
                            if flags.intersects(ImplFlags::from_bits(1 << (2 * i + 6)).unwrap()) {
                                if let Some(neigh) = neigh {
                                    if !world.set_cell(neigh, ALIVE, Reason::Deduce) {
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
                true
            }
        }
        impl NtLife {
            /// Constructs a new rule from the `b` and `s` data.
            pub fn new(b: Vec<u8>, s: Vec<u8>) -> Self {
                let b0 = b.contains(&0x00);
                let s8 = s.contains(&0xff);
                let impl_table = ::alloc::vec::from_elem(ImplFlags::empty(), 1 << 20);
                NtLife { b0, s8, impl_table }
                    .init_trans(b, s)
                    .init_conflict()
                    .init_impl()
                    .init_impl_nbhd()
            }
            /// Deduces the implication for the successor.
            fn init_trans(mut self, b: Vec<u8>, s: Vec<u8>) -> Self {
                for alives in 0..=0xff {
                    let desc = (0xff & !alives) << 12 | alives << 4;
                    let alives = alives as u8;
                    self.impl_table[desc | 0b10] |= if b.contains(&alives) {
                        ImplFlags::SUCC_ALIVE
                    } else {
                        ImplFlags::SUCC_DEAD
                    };
                    self.impl_table[desc | 0b01] |= if s.contains(&alives) {
                        ImplFlags::SUCC_ALIVE
                    } else {
                        ImplFlags::SUCC_DEAD
                    };
                    self.impl_table[desc] |= if b.contains(&alives) && s.contains(&alives) {
                        ImplFlags::SUCC_ALIVE
                    } else if !b.contains(&alives) && !s.contains(&alives) {
                        ImplFlags::SUCC_DEAD
                    } else {
                        ImplFlags::empty()
                    };
                }
                for unknowns in 1usize..=0xff {
                    let n =
                        unknowns.next_power_of_two() >> usize::from(!unknowns.is_power_of_two());
                    for alives in (0..=0xff).filter(|a| a & unknowns == 0) {
                        let desc = (0xff & !alives & !unknowns) << 12 | alives << 4;
                        let desc0 = (0xff & !alives & !unknowns | n) << 12 | alives << 4;
                        let desc1 = (0xff & !alives & !unknowns) << 12 | (alives | n) << 4;
                        for state in 0..=2 {
                            let trans0 = self.impl_table[desc0 | state];
                            if trans0 == self.impl_table[desc1 | state] {
                                self.impl_table[desc | state] |= trans0;
                            }
                        }
                    }
                }
                self
            }
            /// Deduces the conflicts.
            fn init_conflict(mut self) -> Self {
                for nbhd_state in 0..0xffff {
                    for state in 0..=2 {
                        let desc = nbhd_state << 4 | state;
                        if self.impl_table[desc].contains(ImplFlags::SUCC_ALIVE) {
                            self.impl_table[desc | 0b10 << 2] = ImplFlags::CONFLICT;
                        } else if self.impl_table[desc].contains(ImplFlags::SUCC_DEAD) {
                            self.impl_table[desc | 0b01 << 2] = ImplFlags::CONFLICT;
                        }
                    }
                }
                self
            }
            /// Deduces the implication for the cell itself.
            fn init_impl(mut self) -> Self {
                for unknowns in 0..=0xff {
                    for alives in (0..=0xff).filter(|a| a & unknowns == 0) {
                        let desc = (0xff & !alives & !unknowns) << 12 | alives << 4;
                        for succ_state in 1..=2 {
                            let flag = if succ_state == 0b10 {
                                ImplFlags::SUCC_ALIVE | ImplFlags::CONFLICT
                            } else {
                                ImplFlags::SUCC_DEAD | ImplFlags::CONFLICT
                            };
                            let possibly_dead = !self.impl_table[desc | 0b10].intersects(flag);
                            let possibly_alive = !self.impl_table[desc | 0b01].intersects(flag);
                            let index = desc | succ_state << 2;
                            if possibly_dead && !possibly_alive {
                                self.impl_table[index] |= ImplFlags::SELF_DEAD;
                            } else if !possibly_dead && possibly_alive {
                                self.impl_table[index] |= ImplFlags::SELF_ALIVE;
                            } else if !possibly_dead && !possibly_alive {
                                self.impl_table[index] = ImplFlags::CONFLICT;
                            }
                        }
                    }
                }
                self
            }
            ///  Deduces the implication for the neighbors.
            fn init_impl_nbhd(mut self) -> Self {
                for unknowns in 1usize..=0xff {
                    for n in (0..8).map(|i| 1 << i).filter(|n| unknowns & n != 0) {
                        for alives in 0..=0xff {
                            let desc = (0xff & !alives & !unknowns) << 12 | alives << 4;
                            let desc0 = (0xff & !alives & !unknowns | n) << 12 | alives << 4;
                            let desc1 = (0xff & !alives & !unknowns) << 12 | (alives | n) << 4;
                            for succ_state in 1..=2 {
                                let flag = if succ_state == 0b10 {
                                    ImplFlags::SUCC_ALIVE | ImplFlags::CONFLICT
                                } else {
                                    ImplFlags::SUCC_DEAD | ImplFlags::CONFLICT
                                };
                                let index = desc | succ_state << 2;
                                for state in 0..=2 {
                                    let possibly_dead =
                                        !self.impl_table[desc0 | state].intersects(flag);
                                    let possibly_alive =
                                        !self.impl_table[desc1 | state].intersects(flag);
                                    if possibly_dead && !possibly_alive {
                                        self.impl_table[index | state] |=
                                            ImplFlags::from_bits((n.pow(2) << 7) as u32).unwrap();
                                    } else if !possibly_dead && possibly_alive {
                                        self.impl_table[index | state] |=
                                            ImplFlags::from_bits((n.pow(2) << 6) as u32).unwrap();
                                    } else if !possibly_dead && !possibly_alive {
                                        self.impl_table[index | state] = ImplFlags::CONFLICT;
                                    }
                                }
                            }
                        }
                    }
                }
                self
            }
        }
    }
    use crate::{
        cells::{CellRef, State},
        world::World,
    };
    pub use life::{Life, LifeGen};
    pub use ntlife::{NtLife, NtLifeGen};
    /// A cellular automaton rule.
    ///
    /// Some details of this trait is hidden in the doc.
    /// Please use the following structs instead of implementing by yourself:
    /// - [`Life`]
    /// - [`LifeGen`]
    /// - [`NtLife`]
    /// - [`NtLifeGen`]
    pub trait Rule: Sized {
        /// The type of neighborhood descriptor of the rule.
        ///
        /// It describes the states of the successor and neighbors of a cell,
        /// and is used to determine the state of the cell in the next generation.
        #[doc(hidden)]
        type Desc: Copy;
        /// Whether the rule is a Generations rule.
        const IS_GEN: bool;
        /// Whether the rule contains `B0`.
        ///
        /// In other words, whether a dead cell would become [`ALIVE`] in the next
        /// generation, if all its neighbors in this generation are dead.
        fn has_b0(&self) -> bool;
        /// Whether the rule contains both `B0` and `S8`.
        ///
        /// In a rule that contains `B0`, a dead cell would become [`ALIVE`] in the next
        /// generation, if all its neighbors in this generation are dead.
        ///
        /// In a rule that contains `S8`, a living cell would stay [`ALIVE`] in the next
        /// generation, if all its neighbors in this generation are alive.
        fn has_b0_s8(&self) -> bool;
        /// The number of states.
        fn gen(&self) -> usize;
        /// Generates a neighborhood descriptor which says that all neighboring
        /// cells have states `state`, and the successor has state `succ_state`.
        #[doc(hidden)]
        fn new_desc(state: State, succ_state: State) -> Self::Desc;
        /// Updates the neighborhood descriptors of all neighbors and the predecessor
        /// when the state of one cell is changed.
        ///
        /// The `state` is the new state of the cell when `new` is true,
        /// the old state when `new` is false.
        #[doc(hidden)]
        fn update_desc(cell: CellRef<Self>, state: Option<State>, new: bool);
        /// Consistifies a cell.
        ///
        /// Examines the state and the neighborhood descriptor of the cell,
        /// and makes sure that it can validly produce the cell in the next
        /// generation. If possible, determines the states of some of the
        /// cells involved.
        ///
        /// Returns `false` if there is a conflict,
        /// `true` if the cells are consistent.
        #[doc(hidden)]
        fn consistify<'a>(world: &mut World<'a, Self>, cell: CellRef<'a, Self>) -> bool;
    }
}
mod search {
    //! The search process.
    use crate::{
        cells::{CellRef, State},
        config::NewState,
        rules::Rule,
        world::World,
    };
    use rand::{thread_rng, Rng};
    /// Search status.
    pub enum Status {
        /// Initial status. Waiting to start.
        Initial,
        /// A result is found.
        Found,
        /// Such pattern does not exist.
        None,
        /// Still searching.
        Searching,
        /// Paused.
        Paused,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Status {
        #[inline]
        fn clone(&self) -> Status {
            {
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Status {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Status {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Status::Initial,) => {
                    let mut debug_trait_builder = f.debug_tuple("Initial");
                    debug_trait_builder.finish()
                }
                (&Status::Found,) => {
                    let mut debug_trait_builder = f.debug_tuple("Found");
                    debug_trait_builder.finish()
                }
                (&Status::None,) => {
                    let mut debug_trait_builder = f.debug_tuple("None");
                    debug_trait_builder.finish()
                }
                (&Status::Searching,) => {
                    let mut debug_trait_builder = f.debug_tuple("Searching");
                    debug_trait_builder.finish()
                }
                (&Status::Paused,) => {
                    let mut debug_trait_builder = f.debug_tuple("Paused");
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Status {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Status {
        #[inline]
        fn eq(&self, other: &Status) -> bool {
            {
                let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Status {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Status {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {}
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for Status {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                _ => ::core::hash::Hash::hash(&::core::intrinsics::discriminant_value(self), state),
            }
        }
    }
    /// Reasons for setting a cell.
    pub(crate) enum Reason {
        /// Known before the search starts,
        Known,
        /// Decides the state of a cell by choice.
        Decide,
        /// Determines the state of a cell by other cells.
        Deduce,
        /// Tries another state of a cell when the original state
        /// leads to a conflict.
        ///
        /// Remembers the number of remaining states to try.
        ///
        /// Only used in Generations rules.
        TryAnother(usize),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for Reason {
        #[inline]
        fn clone(&self) -> Reason {
            {
                let _: ::core::clone::AssertParamIsClone<usize>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for Reason {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Reason {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Reason::Known,) => {
                    let mut debug_trait_builder = f.debug_tuple("Known");
                    debug_trait_builder.finish()
                }
                (&Reason::Decide,) => {
                    let mut debug_trait_builder = f.debug_tuple("Decide");
                    debug_trait_builder.finish()
                }
                (&Reason::Deduce,) => {
                    let mut debug_trait_builder = f.debug_tuple("Deduce");
                    debug_trait_builder.finish()
                }
                (&Reason::TryAnother(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("TryAnother");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl ::core::marker::StructuralPartialEq for Reason {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for Reason {
        #[inline]
        fn eq(&self, other: &Reason) -> bool {
            {
                let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (&Reason::TryAnother(ref __self_0), &Reason::TryAnother(ref __arg_1_0)) => {
                            (*__self_0) == (*__arg_1_0)
                        }
                        _ => true,
                    }
                } else {
                    false
                }
            }
        }
        #[inline]
        fn ne(&self, other: &Reason) -> bool {
            {
                let __self_vi = ::core::intrinsics::discriminant_value(&*self);
                let __arg_1_vi = ::core::intrinsics::discriminant_value(&*other);
                if true && __self_vi == __arg_1_vi {
                    match (&*self, &*other) {
                        (&Reason::TryAnother(ref __self_0), &Reason::TryAnother(ref __arg_1_0)) => {
                            (*__self_0) != (*__arg_1_0)
                        }
                        _ => false,
                    }
                } else {
                    true
                }
            }
        }
    }
    impl ::core::marker::StructuralEq for Reason {}
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for Reason {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<usize>;
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::hash::Hash for Reason {
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            match (&*self,) {
                (&Reason::TryAnother(ref __self_0),) => {
                    ::core::hash::Hash::hash(&::core::intrinsics::discriminant_value(self), state);
                    ::core::hash::Hash::hash(&(*__self_0), state)
                }
                _ => ::core::hash::Hash::hash(&::core::intrinsics::discriminant_value(self), state),
            }
        }
    }
    /// Records the cells whose values are set and their reasons.
    pub(crate) struct SetCell<'a, R: Rule> {
        /// The set cell.
        pub(crate) cell: CellRef<'a, R>,
        /// The reason for setting a cell.
        pub(crate) reason: Reason,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<'a, R: ::core::clone::Clone + Rule> ::core::clone::Clone for SetCell<'a, R> {
        #[inline]
        fn clone(&self) -> SetCell<'a, R> {
            match *self {
                SetCell {
                    cell: ref __self_0_0,
                    reason: ref __self_0_1,
                } => SetCell {
                    cell: ::core::clone::Clone::clone(&(*__self_0_0)),
                    reason: ::core::clone::Clone::clone(&(*__self_0_1)),
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<'a, R: ::core::marker::Copy + Rule> ::core::marker::Copy for SetCell<'a, R> {}
    impl<'a, R: Rule> SetCell<'a, R> {
        /// Get a reference to the set cell.
        pub(crate) fn new(cell: CellRef<'a, R>, reason: Reason) -> Self {
            SetCell { cell, reason }
        }
    }
    impl<'a, R: Rule> World<'a, R> {
        /// Consistifies a cell.
        ///
        /// Examines the state and the neighborhood descriptor of the cell,
        /// and makes sure that it can validly produce the cell in the next
        /// generation. If possible, determines the states of some of the
        /// cells involved.
        ///
        /// Returns `false` if there is a conflict,
        /// `true` if the cells are consistent.
        fn consistify(&mut self, cell: CellRef<'a, R>) -> bool {
            Rule::consistify(self, cell)
        }
        /// Consistifies a cell, its neighbors, and its predecessor.
        ///
        /// Returns `false` if there is a conflict,
        /// `true` if the cells are consistent.
        fn consistify10(&mut self, cell: CellRef<'a, R>) -> bool {
            self.consistify(cell)
                && {
                    if let Some(pred) = cell.pred {
                        self.consistify(pred)
                    } else {
                        true
                    }
                }
                && cell
                    .nbhd
                    .iter()
                    .all(|&neigh| self.consistify(neigh.unwrap()))
        }
        /// Deduces all the consequences by [`consistify`](Self::consistify) and symmetry.
        ///
        /// Returns `false` if there is a conflict,
        /// `true` if the cells are consistent.
        fn proceed(&mut self) -> bool {
            while self.check_index < self.set_stack.len() {
                let cell = self.set_stack[self.check_index].cell;
                let state = cell.state.get().unwrap();
                for &sym in cell.sym.iter() {
                    if let Some(old_state) = sym.state.get() {
                        if state != old_state {
                            return false;
                        }
                    } else if !self.set_cell(sym, state, Reason::Deduce) {
                        return false;
                    }
                }
                if !self.consistify10(cell) {
                    return false;
                }
                self.check_index += 1;
            }
            true
        }
        /// Backtracks to the last time when a unknown cell is decided by choice,
        /// and switch that cell to the other state.
        ///
        /// Returns `true` if it backtracks successfully,
        /// `false` if it goes back to the time before the first cell is set.
        fn backup(&mut self) -> bool {
            while let Some(set_cell) = self.set_stack.pop() {
                let cell = set_cell.cell;
                match set_cell.reason {
                    Reason::Decide => {
                        self.check_index = self.set_stack.len();
                        self.next_unknown = cell.next;
                        if R::IS_GEN {
                            let State(j) = cell.state.get().unwrap();
                            let state = State((j + 1) % self.rule.gen());
                            self.clear_cell(cell);
                            if self.set_cell(cell, state, Reason::TryAnother(self.rule.gen() - 2)) {
                                return true;
                            }
                        } else {
                            let state = !cell.state.get().unwrap();
                            self.clear_cell(cell);
                            if self.set_cell(cell, state, Reason::Deduce) {
                                return true;
                            }
                        }
                    }
                    Reason::TryAnother(n) => {
                        self.check_index = self.set_stack.len();
                        self.next_unknown = cell.next;
                        let State(j) = cell.state.get().unwrap();
                        let state = State((j + 1) % self.rule.gen());
                        self.clear_cell(cell);
                        let reason = if n == 1 {
                            Reason::Deduce
                        } else {
                            Reason::TryAnother(n - 1)
                        };
                        if self.set_cell(cell, state, reason) {
                            return true;
                        }
                    }
                    Reason::Deduce => {
                        self.clear_cell(cell);
                    }
                    Reason::Known => {
                        break;
                    }
                }
            }
            self.set_stack.clear();
            self.check_index = 0;
            self.next_unknown = None;
            false
        }
        /// Keeps proceeding and backtracking,
        /// until there are no more cells to examine (and returns `true`),
        /// or the backtracking goes back to the time before the first cell is set
        /// (and returns `false`).
        ///
        /// It also records the number of steps it has walked in the parameter
        /// `step`. A step consists of a [`proceed`](Self::proceed) and a [`backup`](Self::backup).
        fn go(&mut self, step: &mut u64) -> bool {
            loop {
                *step += 1;
                if self.proceed() {
                    return true;
                } else {
                    self.conflicts += 1;
                    if !self.backup() {
                        return false;
                    }
                }
            }
        }
        /// Deduce all cells that could be deduced before the first decision.
        pub(crate) fn presearch(mut self) -> Self {
            loop {
                if self.proceed() {
                    self.set_stack.clear();
                    self.check_index = 0;
                    return self;
                } else {
                    self.conflicts += 1;
                    if !self.backup() {
                        return self;
                    }
                }
            }
        }
        /// Makes a decision.
        ///
        /// Chooses an unknown cell, assigns a state for it,
        /// and push a reference to it to the [`set_stack`](#structfield.set_stack).
        ///
        /// Returns `None` is there is no unknown cell,
        /// `Some(false)` if the new state leads to an immediate conflict.
        fn decide(&mut self) -> Option<bool> {
            if let Some(cell) = self.get_unknown() {
                self.next_unknown = cell.next;
                let state = match self.config.new_state {
                    NewState::ChooseDead => cell.background,
                    NewState::ChooseAlive => !cell.background,
                    NewState::Random => State(thread_rng().gen_range(0..self.rule.gen())),
                };
                Some(self.set_cell(cell, state, Reason::Decide))
            } else {
                None
            }
        }
        /// The search function.
        ///
        /// Returns [`Status::Found`] if a result is found,
        /// [`Status::None`] if such pattern does not exist,
        /// [`Status::Searching`] if the number of steps exceeds `max_step`
        /// and no results are found.
        pub fn search(&mut self, max_step: Option<u64>) -> Status {
            let mut step_count = 0;
            if self.next_unknown.is_none() && !self.backup() {
                return Status::None;
            }
            while self.go(&mut step_count) {
                if let Some(result) = self.decide() {
                    if !result && !self.backup() {
                        return Status::None;
                    }
                } else if !self.is_boring() {
                    if self.config.reduce_max {
                        self.config.max_cell_count = Some(self.cell_count() - 1);
                    }
                    return Status::Found;
                } else if !self.backup() {
                    return Status::None;
                }
                if let Some(max) = max_step {
                    if step_count > max {
                        return Status::Searching;
                    }
                }
            }
            Status::None
        }
        /// Set the max cell counts.
        pub(crate) fn set_max_cell_count(&mut self, max_cell_count: Option<usize>) {
            self.config.max_cell_count = max_cell_count;
            if let Some(max) = self.config.max_cell_count {
                while self.cell_count() > max {
                    if !self.backup() {
                        break;
                    }
                }
            }
        }
    }
}
mod traits {
    //! A trait for [`World`].
    use crate::{
        cells::{Coord, State, ALIVE, DEAD},
        config::Config,
        rules::Rule,
        search::Status,
        world::World,
    };
    use std::fmt::Write;
    /// A trait for [`World`].
    ///
    /// So that we can switch between different rule types using trait objects.
    pub trait Search {
        /// The search function.
        ///
        /// Returns [`Status::Found`] if a result is found,
        /// [`Status::None`] if such pattern does not exist,
        /// [`Status::Searching`] if the number of steps exceeds `max_step`
        /// and no results are found.
        fn search(&mut self, max_step: Option<u64>) -> Status;
        /// Gets the state of a cell. Returns `Err(())` if there is no such cell.
        fn get_cell_state(&self, coord: Coord) -> Option<State>;
        /// World configuration.
        fn config(&self) -> &Config;
        /// Whether the rule is a Generations rule.
        fn is_gen_rule(&self) -> bool;
        /// Whether the rule contains `B0`.
        ///
        /// In other words, whether a cell would become [`ALIVE`] in the next
        /// generation, if all its neighbors in this generation are dead.
        fn is_b0_rule(&self) -> bool;
        /// Number of known living cells in some generation.
        ///
        /// For Generations rules, dying cells are not counted.
        fn cell_count_gen(&self, t: i32) -> usize;
        /// Minumum number of known living cells in all generation.
        ///
        /// For Generations rules, dying cells are not counted.
        fn cell_count(&self) -> usize;
        /// Number of conflicts during the search.
        fn conflicts(&self) -> u64;
        /// Set the max cell counts.
        ///
        /// Currently this is the only parameter that you can change
        /// during the search.
        fn set_max_cell_count(&mut self, max_cell_count: Option<usize>);
        /// Displays the whole world in some generation,
        /// in a mix of [Plaintext](https://conwaylife.com/wiki/Plaintext) and
        /// [RLE](https://conwaylife.com/wiki/Rle) format.
        ///
        /// * **Dead** cells are represented by `.`;
        /// * **Living** cells are represented by `o` for rules with 2 states,
        ///   `A` for rules with more states;
        /// * **Dying** cells are represented by uppercase letters starting from `B`;
        /// * **Unknown** cells are represented by `?`;
        /// * Each line is ended with `$`;
        /// * The whole pattern is ended with `!`.
        fn rle_gen(&self, t: i32) -> String {
            let mut str = String::new();
            str.write_fmt(::core::fmt::Arguments::new_v1(
                &["x = ", ", y = ", ", rule = ", "\n"],
                &match (
                    &self.config().width,
                    &self.config().height,
                    &self.config().rule_string,
                ) {
                    (arg0, arg1, arg2) => [
                        ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                        ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ::core::fmt::ArgumentV1::new(arg2, ::core::fmt::Display::fmt),
                    ],
                },
            ))
            .unwrap();
            for y in 0..self.config().height {
                for x in 0..self.config().width {
                    let state = self.get_cell_state((x, y, t));
                    match state {
                        Some(DEAD) => str.push('.'),
                        Some(ALIVE) => {
                            if self.is_gen_rule() {
                                str.push('A')
                            } else {
                                str.push('o')
                            }
                        }
                        Some(State(i)) => str.push((b'A' + i as u8 - 1) as char),
                        _ => str.push('?'),
                    };
                }
                if y == self.config().height - 1 {
                    str.push('!')
                } else {
                    str.push('$')
                };
                str.push('\n');
            }
            str
        }
        /// Displays the whole world in some generation in
        /// [Plaintext](https://conwaylife.com/wiki/Plaintext) format.
        ///
        /// * **Dead** cells are represented by `.`;
        /// * **Living** and **Dying** cells are represented by `o`;
        /// * **Unknown** cells are represented by `?`.
        fn plaintext_gen(&self, t: i32) -> String {
            let mut str = String::new();
            for y in 0..self.config().height {
                for x in 0..self.config().width {
                    let state = self.get_cell_state((x, y, t));
                    match state {
                        Some(DEAD) => str.push('.'),
                        Some(_) => str.push('o'),
                        None => str.push('?'),
                    };
                }
                str.push('\n');
            }
            str
        }
    }
    /// The [`Search`] trait is implemented for every [`World`].
    impl<'a, R: Rule> Search for World<'a, R> {
        fn search(&mut self, max_step: Option<u64>) -> Status {
            self.search(max_step)
        }
        fn get_cell_state(&self, coord: Coord) -> Option<State> {
            self.get_cell_state(coord)
        }
        fn config(&self) -> &Config {
            &self.config
        }
        fn is_gen_rule(&self) -> bool {
            R::IS_GEN
        }
        fn is_b0_rule(&self) -> bool {
            self.rule.has_b0()
        }
        fn cell_count_gen(&self, t: i32) -> usize {
            self.cell_count[t as usize]
        }
        fn cell_count(&self) -> usize {
            self.cell_count()
        }
        fn conflicts(&self) -> u64 {
            self.conflicts
        }
        fn set_max_cell_count(&mut self, max_cell_count: Option<usize>) {
            self.set_max_cell_count(max_cell_count)
        }
    }
}
mod world {
    //! The world.
    use crate::{
        cells::{CellRef, Coord, LifeCell, State, DEAD},
        config::{Config, KnownCell, SearchOrder},
        rules::Rule,
        search::{Reason, SetCell},
    };
    use std::mem;
    /// The world.
    pub struct World<'a, R: Rule> {
        /// World configuration.
        pub(crate) config: Config,
        /// The rule of the cellular automaton.
        pub(crate) rule: R,
        /// A vector that stores all the cells in the search range.
        ///
        /// This vector will not be moved after its creation.
        /// All the cells will live throughout the lifetime of the world.
        cells: Vec<LifeCell<'a, R>>,
        /// Number of known living cells in each generation.
        ///
        /// For Generations rules, dying cells are not counted.
        pub(crate) cell_count: Vec<usize>,
        /// Number of unknown or living cells on the first row or column.
        pub(crate) front_cell_count: usize,
        /// Number of conflicts during the search.
        pub(crate) conflicts: u64,
        /// A stack to record the cells whose values are set during the search.
        ///
        /// The cells in this stack always have known states.
        ///
        /// It is used in the backtracking.
        pub(crate) set_stack: Vec<SetCell<'a, R>>,
        /// The position of the next cell to be examined in the [`set_stack`](#structfield.set_stack).
        ///
        /// See [`proceed`](Self::proceed) for details.
        pub(crate) check_index: usize,
        /// The starting point to look for an unknown cell.
        ///
        /// There must be no unknown cell before this cell.
        pub(crate) next_unknown: Option<CellRef<'a, R>>,
        /// Whether to force the first row/column to be nonempty.
        ///
        /// Depending on the search order, the 'front' means:
        /// * the first row, when the search order is row first;
        /// * the first column, when the search order is column first;
        /// * the first row plus the first column, when the search order is diagonal.
        pub(crate) non_empty_front: bool,
    }
    impl<'a, R: Rule> World<'a, R> {
        /// Creates a new world from the configuration and the rule.
        pub fn new(config: &Config, rule: R) -> Self {
            let search_order = config.auto_search_order();
            let size = ((config.width + 2) * (config.height + 2) * config.period) as usize;
            let mut cells = Vec::with_capacity(size);
            let is_front = config.is_front_fn(rule.has_b0(), &search_order);
            for x in -1..=config.width {
                for y in -1..=config.height {
                    for t in 0..config.period {
                        let state = if rule.has_b0() {
                            State(t as usize % rule.gen())
                        } else {
                            DEAD
                        };
                        let succ_state = if rule.has_b0() {
                            State((t as usize + 1) % rule.gen())
                        } else {
                            DEAD
                        };
                        let mut cell = LifeCell::new((x, y, t), state, succ_state);
                        if let Some(is_front) = &is_front {
                            if is_front((x, y, t)) {
                                cell.is_front = true;
                            }
                        }
                        cells.push(cell);
                    }
                }
            }
            World {
                config: config.clone(),
                rule,
                cells,
                cell_count: ::alloc::vec::from_elem(0, config.period as usize),
                front_cell_count: 0,
                conflicts: 0,
                set_stack: Vec::with_capacity(size),
                check_index: 0,
                next_unknown: None,
                non_empty_front: is_front.is_some(),
            }
            .init_nbhd()
            .init_pred_succ()
            .init_sym()
            .init_state()
            .init_known_cells(&config.known_cells)
            .init_search_order(search_order.as_ref())
            .presearch()
        }
        /// Links the cells to their neighbors.
        ///
        /// Note that for cells on the edges of the search range,
        /// some neighbors might point to `None`.
        fn init_nbhd(mut self) -> Self {
            const NBHD: [(i32, i32); 8] = [
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ];
            for x in -1..=self.config.width {
                for y in -1..=self.config.height {
                    if let Some(d) = self.config.diagonal_width {
                        if (x - y).abs() > d + 1 {
                            continue;
                        }
                    }
                    for t in 0..self.config.period {
                        let cell_ptr = self.find_cell_mut((x, y, t)).unwrap();
                        for (i, (nx, ny)) in NBHD.iter().enumerate() {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.nbhd[i] = self.find_cell((x + nx, y + ny, t));
                            }
                        }
                    }
                }
            }
            self
        }
        /// Links a cell to its predecessor and successor.
        ///
        /// If the predecessor is out of the search range,
        /// then marks the current cell as known.
        ///
        /// If the successor is out of the search range,
        /// then sets it to `None`.
        fn init_pred_succ(mut self) -> Self {
            for x in -1..=self.config.width {
                for y in -1..=self.config.height {
                    if let Some(d) = self.config.diagonal_width {
                        if (x - y).abs() > d + 1 {
                            continue;
                        }
                    }
                    for t in 0..self.config.period {
                        let cell_ptr = self.find_cell_mut((x, y, t)).unwrap();
                        let cell = self.find_cell((x, y, t)).unwrap();
                        if t != 0 {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.pred = self.find_cell((x, y, t - 1));
                            }
                        } else {
                            let pred = self.find_cell(self.config.translate((x, y, t - 1)));
                            if pred.is_some() {
                                unsafe {
                                    let cell = cell_ptr.as_mut().unwrap();
                                    cell.pred = pred;
                                }
                            } else if 0 <= x
                                && x < self.config.width
                                && 0 <= y
                                && y < self.config.height
                                && (self.config.diagonal_width.is_none()
                                    || (x - y).abs() < self.config.diagonal_width.unwrap())
                                && !self.set_stack.iter().any(|s| s.cell == cell)
                            {
                                self.set_stack.push(SetCell::new(cell, Reason::Known));
                            }
                        }
                        if t != self.config.period - 1 {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.succ = self.find_cell((x, y, t + 1));
                            }
                        } else {
                            unsafe {
                                let cell = cell_ptr.as_mut().unwrap();
                                cell.succ = self.find_cell(self.config.translate((x, y, t + 1)));
                            }
                        }
                    }
                }
            }
            self
        }
        /// Links a cell to the symmetric cells.
        ///
        /// If some symmetric cell is out of the search range,
        /// then  marks the current cell as known.
        fn init_sym(mut self) -> Self {
            for x in -1..=self.config.width {
                for y in -1..=self.config.height {
                    if let Some(d) = self.config.diagonal_width {
                        if (x - y).abs() > d + 1 {
                            continue;
                        }
                    }
                    for t in 0..self.config.period {
                        let cell_ptr = self.find_cell_mut((x, y, t)).unwrap();
                        let cell = self.find_cell((x, y, t)).unwrap();
                        for transform in self.config.symmetry.members() {
                            let coord =
                                transform.act_on((x, y, t), self.config.width, self.config.height);
                            if 0 <= coord.0
                                && coord.0 < self.config.width
                                && 0 <= coord.1
                                && coord.1 < self.config.height
                                && (self.config.diagonal_width.is_none()
                                    || (coord.0 - coord.1).abs()
                                        < self.config.diagonal_width.unwrap())
                            {
                                unsafe {
                                    let cell = cell_ptr.as_mut().unwrap();
                                    cell.sym.push(self.find_cell(coord).unwrap());
                                }
                            } else if 0 <= x
                                && x < self.config.width
                                && 0 <= y
                                && y < self.config.height
                                && (self.config.diagonal_width.is_none()
                                    || (x - y).abs() < self.config.diagonal_width.unwrap())
                                && !self.set_stack.iter().any(|s| s.cell == cell)
                            {
                                self.set_stack.push(SetCell::new(cell, Reason::Known));
                            }
                        }
                    }
                }
            }
            self
        }
        /// Sets states for the cells.
        ///
        /// All cells are set to unknown unless they are on the boundary,
        /// or are marked as known in [`init_pred_succ`](Self::init_pred_succ)
        /// or [`init_sym`](Self::init_sym).
        fn init_state(mut self) -> Self {
            for x in 0..self.config.width {
                for y in 0..self.config.height {
                    if let Some(d) = self.config.diagonal_width {
                        if (x - y).abs() >= d {
                            continue;
                        }
                    }
                    for t in 0..self.config.period {
                        let cell = self.find_cell((x, y, t)).unwrap();
                        if !self.set_stack.iter().any(|s| s.cell == cell) {
                            self.clear_cell(cell);
                        }
                    }
                }
            }
            self
        }
        /// Sets the known cells.
        fn init_known_cells(mut self, known_cells: &[KnownCell]) -> Self {
            for &KnownCell { coord, state } in known_cells.iter() {
                if let Some(cell) = self.find_cell(coord) {
                    if cell.state.get().is_none() && state.0 < self.rule.gen() {
                        self.set_cell(cell, state, Reason::Known);
                    }
                }
            }
            self
        }
        /// Set the [`next`](LifeCell#structfield.next) of a cell to be
        /// [`next_unknown`](#structfield.next_unknown) and set
        /// [`next_unknown`](#structfield.next_unknown) to be this cell.
        fn set_next(&mut self, coord: Coord) {
            if let Some(cell) = self.find_cell(coord) {
                if cell.state.get().is_none() && cell.next.is_none() {
                    let next = mem::replace(&mut self.next_unknown, Some(cell));
                    let cell_ptr = self.find_cell_mut(coord).unwrap();
                    unsafe {
                        let cell = cell_ptr.as_mut().unwrap();
                        cell.next = next;
                    }
                }
            }
        }
        /// Sets the search order.
        fn init_search_order(mut self, search_order: &SearchOrder) -> Self {
            for coord in self.config.search_order_iter(search_order) {
                self.set_next(coord);
            }
            self
        }
        /// Finds a cell by its coordinates. Returns a [`CellRef`].
        pub(crate) fn find_cell(&self, coord: Coord) -> Option<CellRef<'a, R>> {
            let (x, y, t) = coord;
            if x >= -1
                && x <= self.config.width
                && y >= -1
                && y <= self.config.height
                && t >= 0
                && t < self.config.period
            {
                let index = ((x + 1) * (self.config.height + 2) + y + 1) * self.config.period + t;
                let cell = &self.cells[index as usize];
                Some(cell.borrow())
            } else {
                None
            }
        }
        /// Finds a cell by its coordinates. Returns a mutable pointer.
        fn find_cell_mut(&mut self, coord: Coord) -> Option<*mut LifeCell<'a, R>> {
            let (x, y, t) = coord;
            if x >= -1
                && x <= self.config.width
                && y >= -1
                && y <= self.config.height
                && t >= 0
                && t < self.config.period
                && (self.config.diagonal_width.is_none()
                    || (x - y).abs() <= self.config.diagonal_width.unwrap() + 1)
            {
                let index = ((x + 1) * (self.config.height + 2) + y + 1) * self.config.period + t;
                Some(&mut self.cells[index as usize])
            } else {
                None
            }
        }
        /// Sets the [`state`](LifeCell#structfield.state) of a cell,
        /// push it to the [`set_stack`](#structfield.set_stack),
        /// and update the neighborhood descriptor of its neighbors.
        ///
        /// The original state of the cell must be unknown.
        ///
        /// Return `false` if the number of living cells exceeds the
        /// [`max_cell_count`](#structfield.max_cell_count) or the front becomes empty.
        pub(crate) fn set_cell(
            &mut self,
            cell: CellRef<'a, R>,
            state: State,
            reason: Reason,
        ) -> bool {
            cell.state.set(Some(state));
            let mut result = true;
            cell.update_desc(Some(state), true);
            if state == !cell.background {
                self.cell_count[cell.coord.2 as usize] += 1;
                if let Some(max) = self.config.max_cell_count {
                    if self.cell_count() > max {
                        result = false;
                    }
                }
            }
            if cell.is_front && state == cell.background {
                self.front_cell_count -= 1;
                if self.non_empty_front && self.front_cell_count == 0 {
                    result = false;
                }
            }
            self.set_stack.push(SetCell::new(cell, reason));
            result
        }
        /// Clears the [`state`](LifeCell#structfield.state) of a cell,
        /// and update the neighborhood descriptor of its neighbors.
        pub(crate) fn clear_cell(&mut self, cell: CellRef<'a, R>) {
            let old_state = cell.state.take();
            if old_state != None {
                cell.update_desc(old_state, false);
                if old_state == Some(!cell.background) {
                    self.cell_count[cell.coord.2 as usize] -= 1;
                }
                if cell.is_front && old_state == Some(cell.background) {
                    self.front_cell_count += 1;
                }
            }
        }
        /// Gets a references to the first unknown cell since [`next_unknown`](#structfield.next_unknown).
        pub(crate) fn get_unknown(&mut self) -> Option<CellRef<'a, R>> {
            while let Some(cell) = self.next_unknown {
                if cell.state.get().is_none() {
                    return Some(cell);
                } else {
                    self.next_unknown = cell.next;
                }
            }
            None
        }
        /// Tests if the result is borling.
        pub(crate) fn is_boring(&self) -> bool {
            self.is_trivial()
                || self.is_stable()
                || (self.config.skip_subperiod && self.is_subperiodic())
                || (self.config.skip_subsymmetry && self.is_subsymmetric())
        }
        /// Tests if the result is trivial.
        fn is_trivial(&self) -> bool {
            self.cell_count[0] == 0
        }
        /// Tests if the result is stable.
        fn is_stable(&self) -> bool {
            self.config.period > 1
                && self
                    .cells
                    .chunks(self.config.period as usize)
                    .all(|c| c[0].state.get() == c[1].state.get())
        }
        /// Tests if the fundamental period of the result is smaller than the given period.
        fn is_subperiodic(&self) -> bool {
            (2..=self.config.period).any(|f| {
                self.config.period % f == 0
                    && self.config.dx % f == 0
                    && self.config.dy % f == 0
                    && {
                        let t = self.config.period / f;
                        let dx = self.config.dx / f;
                        let dy = self.config.dy / f;
                        self.cells
                            .iter()
                            .step_by(self.config.period as usize)
                            .all(|c| {
                                let (x, y, _) = c.coord;
                                c.state.get() == self.get_cell_state((x - dx, y - dy, t))
                            })
                    }
            })
        }
        /// Tests if the result is invariant under more transformations than
        /// required by the given symmetry.
        fn is_subsymmetric(&self) -> bool {
            let cosets = self.config.symmetry.cosets();
            self.cells
                .iter()
                .step_by(self.config.period as usize)
                .all(|c| {
                    cosets.iter().skip(1).any(|t| {
                        let coord = t.act_on(c.coord, self.config.width, self.config.height);
                        c.state.get() == self.get_cell_state(coord)
                    })
                })
        }
        /// Gets the state of a cell. Returns `Err(())` if there is no such cell.
        pub fn get_cell_state(&self, coord: Coord) -> Option<State> {
            let (x, y, t) = self.config.translate(coord);
            self.find_cell((x, y, t)).map_or_else(
                || self.find_cell((0, 0, t)).map(|c1| c1.background),
                |c1| c1.state.get(),
            )
        }
        /// Minimum number of known living cells in all generation.
        ///
        /// For Generations rules, dying cells are not counted.
        pub(crate) fn cell_count(&self) -> usize {
            *self.cell_count.iter().min().unwrap()
        }
    }
}
pub use cells::{Coord, State, ALIVE, DEAD};
pub use config::{Config, KnownCell, NewState, SearchOrder, Symmetry, Transform};
pub use error::Error;
pub use search::Status;
pub use traits::Search;
pub use world::World;
