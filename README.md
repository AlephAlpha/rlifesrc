# [rlifesrc](https://github.com/AlephAlpha/rlifesrc)

[![Travis (.org)](https://img.shields.io/travis/AlephAlpha/rlifesrc)](https://travis-ci.org/AlephAlpha/rlifesrc) [![Crates.io](https://img.shields.io/crates/v/rlifesrc)](https://crates.io/crates/rlifesrc) [![Docs.rs](https://docs.rs/rlifesrc-lib/badge.svg)](https://docs.rs/rlifesrc-lib/) [![English](https://img.shields.io/badge/readme-English-brightgreen)](README_en.md)

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch) 和 Jason Summers 写的 [WinLifeSearch](https://github.com/jsummers/winlifesearch/)。其具体的算法可见 [Dean Hickerson 的说明](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），而且在不懂 javascript 的情况下弄成一个网页，写得非常糟糕，和 WinLifeSearch 相比缺少很多功能，而且速度要慢很多，但支持更多规则。

支持 [Life-like](https://conwaylife.com/wiki/Totalistic_Life-like_cellular_automaton) 和 [non-totalistic](https://conwaylife.com/wiki/Non-isotropic_Life-like_cellular_automaton) 的规则，但后者比前者要略慢一些。也支持[六边形](https://conwaylife.com/wiki/Hexagonal_neighbourhood)以及[von Neumann 邻域](https://conwaylife.com/wiki/Von_Neumann_neighbourhood)的规则，但目前是通过转化成 non-totalistic 规则来实现的，速度较慢。还支持 [Generations](https://conwaylife.com/wiki/Generations) 规则，此功能是实验性的，可能有 bug。

提供一个文本界面的命令行工具，和一个编译成 wasm 的网页版，请分别见 [`tui/`](tui/) 和  [`web/`](web/) 两个目录。

[点此试用网页版。](https://alephalpha.github.io/rlifesrc/)
