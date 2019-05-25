# rlifesrc

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），写得非常糟糕。和原版的 lifesrc 相比速度要慢一些，而且缺乏很多功能。

## 用法

```text
USAGE:
    cargo run --release [FLAGS] [OPTIONS] <X> <Y> [ARGS]

FLAGS:
    -a, --all        Searches for all possible patterns
        --random     Searches for a random pattern
    -t, --time       Shows how long the search takes
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -r, --rule <RULE>            Rule of the cellular automaton [default: B3/S23]
    -s, --symmetry <SYMMETRY>    Symmetry of the pattern [default: C1]  [possible values: C1, C2, C4, D2|, D2-, D2\,
                                 D2/, D4+, D4X, D8]

ARGS:
    <X>     Number of columns
    <Y>     Number of rows
    <P>     Number of generations [default: 1]
    <DX>    Column translation [default: 0]
    <DY>    Row translation [default: 0]
```

比如说，用 `cargo run --release 16 5 3 0 1` 可以找到 [25P3H1V0.1](http://conwaylife.com/wiki/25P3H1V0.1)。

不加 `--release` 的话会特别慢。