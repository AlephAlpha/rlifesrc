# rlifesrc

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），而且在不懂 javascript 的情况下弄成一个网页，写得非常糟糕，和原版的 lifesrc 相比缺少很多功能。

支持 [Life-like](http://conwaylife.com/wiki/Totalistic_Life-like_cellular_automaton) 和 [Isotropic non-totalistic](http://conwaylife.com/wiki/Isotropic_non-totalistic_Life-like_cellular_automaton) 的规则，但后者比前者要略慢一些。

这是网页版，速度比原版要慢不少。要用原版，请见 master 分支。

## 用法

按照说明调整图样的宽度、高度、周期、平移等信息，然后点击 “Set World” 来确定这些信息。然后点 “Start” 开始搜索。搜索时不要切换到别的标签页。

对称性的用法和 Oscar Cunningham 的 Logic Life Search 一样，详见 [Life Wiki 上的相应说明](http://conwaylife.com/wiki/Symmetry)。

“Search Order” 里的 “Automatic” 指的是先搜窄的一边。也就是说，行比列少先搜列，列比行少先搜行。

“New state for unknown cells” 指的是如何为未知的细胞选取状态。其中 “Smart” 指的是：如果这个未知细胞在第一行/第一列，则随机选取；如果不在第一行/第一列，则就先设为死；具体是第一行还是第一列由 “Search Order” 决定。

## 编译

为了把 Rust 编译成 WebAssembly，需要安装 [cargo-web](https://github.com/DenisKolodin/yew)。

用 `cargo web build --release` 来编译即可。然后用 `cargo web build --release`来运行。使用时按其说明在浏览器打开网页（一般是 127.0.0.1:8000）即可。

也可以用 `cargo web deploy --release` 来编译成静态网页，无需服务器即可使用。

不加 `--release` 的话搜索会特别慢（相差近百倍）。
