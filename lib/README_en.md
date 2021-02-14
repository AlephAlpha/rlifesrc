# [rlifesrc-lib](https://github.com/AlephAlpha/rlifesrc)

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/AlephAlpha/rlifesrc/test)](https://github.com/AlephAlpha/rlifesrc/actions) [![Crates.io](https://img.shields.io/crates/v/rlifesrc-lib)](https://crates.io/crates/rlifesrc-lib) [![Docs.rs](https://docs.rs/rlifesrc-lib/badge.svg)](https://docs.rs/rlifesrc-lib/) [![中文](https://img.shields.io/badge/readme-%E4%B8%AD%E6%96%87-brightgreen)](README.md)

__Rust Life Search__, or __rlifesrc__, is a Game of Life pattern searcher written in Rust.

The program is based on David Bell's [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch) and Jason Summers's [WinLifeSearch](https://github.com/jsummers/winlifesearch/), using [an algorithm invented by Dean Hickerson](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN).

Compared to WinLifeSearch, rlifesrc is still slower, and lacks many important features. But it supports non-totalistic Life-like and Generations rules.

This is the library for rlifesrc. There is also a [command-line tool with a TUI](https://github.com/AlephAlpha/rlifesrc/tree/master/tui) and a [web app using WebAssembly](https://github.com/AlephAlpha/rlifesrc/tree/master/web).

You can try the web app [here](https://alephalpha.github.io/rlifesrc/).

For the documentation of released version of this crate, please visit [docs.rs](https://docs.rs/rlifesrc-lib/). There is also [a documentation for the unreleased version on GitHub](https://alephalpha.github.io/rlifesrc-doc/rlifesrc_lib/), which includes private functions and methods.

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