/*!
__Rust Life Search__, or __rlifesrc__,
is a Game of Life pattern searcher written in Rust.

The program is based on David Bell's
[lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)
and Jason Summers's [WinLifeSearch](https://github.com/jsummers/winlifesearch/),
using [an algorithm invented by Dean Hickerson](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN).

Compared to WinLifeSearch, rlifesrc is still slower, and lacks many important features.
But it supports non-totalistic Life-like and Generations rules.

This is the library for rlifesrc. There is also a
[command-line tool with a TUI](https://github.com/AlephAlpha/rlifesrc/tree/master/tui)
and a [web app using WebAssembly](https://github.com/AlephAlpha/rlifesrc/tree/master/web).

You can try the web app [here](https://alephalpha.github.io/rlifesrc/).

# Example

Finds the [25P3H1V0.1](https://conwaylife.com/wiki/25P3H1V0.1) spaceship.

```rust
use rlifesrc_lib::{Config, Status};

// Configures the world.
let config = Config::new(16, 5, 3).set_translate(0, 1);

// Creates the world.
let mut search = config.world().unwrap();

// Searches and displays the generation 0 of the result.
if let Status::Found = search.search(None) {
    println!("{}", search.rle_gen(0))
}
```

Search result:

```plaintext
x = 16, y = 5, rule = B3/S23
........o.......$
.oo.ooo.ooo.....$
.oo....o..oo.oo.$
o..o.oo...o..oo.$
............o..o!
```
*/

#![cfg_attr(any(docs_rs, github_io), feature(doc_cfg))]

mod cells;
mod config;
mod error;
mod poly_world;
pub mod rules;
pub mod search;
mod world;

#[cfg(feature = "serde")]
#[cfg_attr(any(docs_rs, github_io), doc(cfg(feature = "serde")))]
pub mod save;

pub use cells::{Coord, State, ALIVE, DEAD};
pub use config::{Config, KnownCell, NewState, SearchOrder, Symmetry, Transform};
pub use error::Error;
pub use poly_world::PolyWorld;
pub use search::Status;
pub use world::World;
