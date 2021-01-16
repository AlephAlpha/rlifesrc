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

mod cells;
mod config;
mod error;
pub mod rules;
mod search;
mod traits;
mod world;

#[cfg(any(feature = "serde", doc))]
pub mod save;

pub use cells::{Coord, State, ALIVE, DEAD};
pub use config::{Config, NewState, SearchOrder, Symmetry, Transform};
pub use error::Error;
pub use search::Status;
pub use traits::Search;
pub use world::World;
