# [rlifesrc](https://alephalpha.github.io/rlifesrc/)

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch) 和 Jason Summers 写的 [WinLifeSearch](https://github.com/jsummers/winlifesearch/)。其具体的算法可见 [Dean Hickerson 的说明](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），而且在不懂 javascript 的情况下弄成一个网页，写得非常糟糕，和 WinLifeSearch 相比缺少很多功能，而且速度要慢很多，但支持更多规则。

支持 [Life-like](https://conwaylife.com/wiki/Totalistic_Life-like_cellular_automaton) 和 [non-totalistic](https://www.conwaylife.com/wiki/Non-isotropic_Life-like_cellular_automaton) 的规则，但后者比前者要略慢一些。也支持[六边形](https://www.conwaylife.com/wiki/Hexagonal_neighbourhood)以及[von Neumann 邻域](https://www.conwaylife.com/wiki/Von_Neumann_neighbourhood)的规则，但目前是通过转化成 non-totalistic 规则来实现的，速度较慢。还支持 [Generations](https://www.conwaylife.com/wiki/Generations) 规则，此功能是实验性的，可能有 bug。

提供一个文本界面的命令行工具，和一个编译成 wasm 的网页版，请分别见 [`tui/`](tui/README.md) 和  [`web/`](web/README.md) 两个目录中的 `README.md`。

[点此试用网页版。](https://alephalpha.github.io/rlifesrc/)
