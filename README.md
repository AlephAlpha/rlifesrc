# rlifesrc

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），写得非常糟糕。和原版的 lifesrc 相比速度要慢一些，而且缺少很多功能。

## 用法

```text
USAGE:
    cargo run --release [FLAGS] [OPTIONS] <X> <Y> [ARGS]
    不加 --release 的话会特别慢。

FLAGS:
        --random     搜索一个随机的图样
    -h, --help       显示此帮助信息的英文版

OPTIONS:
    -r, --rule <RULE>            元胞自动机的规则（仅支持 Life-like 的规则） [默认: B3/S23]
    -s, --symmetry <SYMMETRY>    对称性 [默认: C1]  [可能的值: C1, C2, C4, D2|, D2-, D2\, D2/, D4+, D4X, D8]
                                 其中一些对称性可能需要加引号。这些对称性的用法和 Logic Life Search 一样。

ARGS:
    <X>     图样的宽度
    <Y>     图样的高度
    <P>     图样的周期 [默认: 1]
    <DX>    水平方向的平移 [默认: 0]
    <DY>    竖直方向的平移 [默认: 0]
```

输入命令后会进入一个 TUI，按任意键开始/继续搜索，按 q 键退出，按 p 键暂停。

比如说，用 `cargo run --release 16 5 3 0 1` 可以找到 [25P3H1V0.1](http://conwaylife.com/wiki/25P3H1V0.1)。
