# [rlifesrc](https://alephalpha.github.io/rlifesrc/)

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），而且在不懂 javascript 的情况下弄成一个网页，写得非常糟糕，和原版的 lifesrc 相比缺少很多功能。

支持 [Life-like](http://conwaylife.com/wiki/Totalistic_Life-like_cellular_automaton) 和 [Isotropic non-totalistic](http://conwaylife.com/wiki/Isotropic_non-totalistic_Life-like_cellular_automaton) 的规则，但后者比前者要略慢一些。

这是网页版，直接从 Rust 编译成 WebAssembly，速度较慢。另有一个 Linux 命令行的版本，速度比原版的 lifesrc 还略快一些，请见这个 repo 的 [master 分支](https://github.com/AlephAlpha/rlifesrc/tree/master)。

## 用法

[点此试用](https://alephalpha.github.io/rlifesrc/)。

这个算法适合搜索小周期的宽扁或者瘦高的图样。支持 Life-like 和 Isotropic non-totalistic 的规则。

按照说明调整图样的宽度、高度、周期、平移等信息，然后点击 “Set World” 来确定这些信息。然后点 “Start” 开始搜索。

搜索时不要切换到别的标签页。这点可能以后会修正。

`.`、`O`、`?`分别代表死细胞、活细胞、未知的细胞。搜索结果可以直接复制粘贴到 Golly 中。

搜到结果后再点 “Start” 会在当前结果的基础上搜下一个结果。如果要从头开始搜索，可以点击 “Set World” 来重置世界。

对称性的用法和 Oscar Cunningham 的 Logic Life Search 一样，详见 [Life Wiki 上的相应说明](http://conwaylife.com/wiki/Symmetry)。

“Order”指的是搜索的顺序。其中 “Automatic” 指的是先搜窄的一边。也就是说，行比列少先搜列，列比行少先搜行。

“New state” 指的是如何为未知的细胞选取状态。其中 “Smart” 指的是：如果这个未知细胞在第一行/第一列，则随机选取；如果不在第一行/第一列，则就先设为死；具体是第一行还是第一列由 “Search Order” 决定。其实一点也不智能，不过我想不出别的名字了。这个模式适合搜索比较窄但可能很长的图样，但未必比 “Random” 快。

## 编译

为了把 Rust 编译成 WebAssembly，需要安装 [cargo-web](https://github.com/DenisKolodin/yew)。

用 `cargo web build --release` 来编译即可。然后用 `cargo web build --release`来运行。使用时按其说明在浏览器打开网页（一般是 127.0.0.1:8000）即可。

也可以用 `cargo web deploy --release` 来编译成静态网页，无需服务器即可使用。

不加 `--release` 的话搜索会特别慢（相差近百倍）。
