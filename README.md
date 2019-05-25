# rlifesrc

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），写得非常糟糕。和原版的 lifesrc 相比速度不相上下，但缺乏很多功能。

用法：`cargo run --release X Y P DX DY -s SYMMETRY -r RULE`，其中 `X`，`Y` 表示图样的大小，`P` 表示周期（默认为 `1`），`DX`、`DY` 表示每周期的平移（默认为 `0`）。`SYMMETRY` 表示对称性，支持的对称性的写法和 [Logic Life Search](https://github.com/OscarCunningham/logic-life-search) 一样。`RULE` 表示规则，目前只支持没有 `B0` 的 Life-like 的规则。

比如说，要想找生命游戏中的飞船 [25P3H1V0.1](http://conwaylife.com/wiki/25P3H1V0.1)，可以用`cargo run --release 16 5 3 0 1`。

不加 `--release` 的话会特别慢。

搜索没有随机性，每次搜索的结果都是一样的。