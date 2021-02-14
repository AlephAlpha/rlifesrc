# [rlifesrc](https://github.com/AlephAlpha/rlifesrc)

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/AlephAlpha/rlifesrc/test)](https://github.com/AlephAlpha/rlifesrc/actions) [![Crates.io](https://img.shields.io/crates/v/rlifesrc)](https://crates.io/crates/rlifesrc) [![Docs.rs](https://docs.rs/rlifesrc-lib/badge.svg)](https://docs.rs/rlifesrc-lib/) [![中文](https://img.shields.io/badge/readme-%E4%B8%AD%E6%96%87-brightgreen)](README.md)

__Rust Life Search__, or __rlifesrc__, is a Game of Life pattern searcher written in Rust.

The program is based on David Bell's [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch) and Jason Summers's [WinLifeSearch](https://github.com/jsummers/winlifesearch/), using [an algorithm invented by Dean Hickerson](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN).

Compared to WinLifeSearch, rlifesrc is still slower, and lacks many important features. But it supports non-totalistic Life-like and Generations rules.

There is a [command-line tool with a TUI](tui/) and a [web app using WebAssembly](web/). The implementation of the algorithm itself is in the [`lib/`](lib/) folder.

You can try the web app [here](https://alephalpha.github.io/rlifesrc/).
