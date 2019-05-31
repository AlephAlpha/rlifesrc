# rlifesrc

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），写得非常糟糕，和原版的 lifesrc 相比缺少很多功能，不过速度可能会稍微快一些。

## 编译

Rlifesrc 的 TUI （文本界面）是用 [pancurses](https://github.com/ihalila/pancurses) 写的，在编译之前请参照 [ncurses-rc](https://github.com/jeaye/ncurses-rs)（Unix-like）或 [pdcurses-sys](https://github.com/ihalila/pdcurses-sys)（Windows） 的说明来安装相应的依赖。

用 `cargo build` 或者 `cargo build --release` 来编译即可。由于我把默认的优化等级设成了3，编译会比较慢；不优化的话程序会因为速度太慢而毫无意义。

如果完全不需要 TUI，而且懒得安装以上的依赖，或者是想节省编译时间，可以在编译和运行的时候加上 `--no-default-features`。

## 用法

```text
USAGE:
    cargo run [FLAGS] [OPTIONS] <X> <Y> [ARGS]

FLAGS:
    -a, --all
            搜索所有的满足条件的图样
            仅适用于不进入 TUI 的情况

    -n, --no-tui
            不进入 TUI，直接开始搜索

        --reset-time
            开始新的搜索时重置计时
            仅适用于有 TUI 的情况

OPTIONS:
    -c, --choose <CHOOSE>
            如何为未知的细胞选取状态 [默认: dead]  [可能的值: dead, alive, random, d, a, r]

    -o, --order <ORDER>
            搜索顺序
            先搜行还是先搜列。 [默认: automatic]  [可能的值: row, column, automatic, r, c, a]

    -r, --rule <RULE>
            元胞自动机的规则
            当前仅支持 Life-like 的规则
             [默认: B3/S23]

    -s, --symmetry <SYMMETRY>
            图样的对称性
            其中一些对称性可能需要加上引号。
            这些对称性的用法和 Oscar Cunningham 的 Logic Life Search 一样。
            详见 http://conwaylife.com/wiki/Symmetry
             [默认: C1]  [可能的值: C1, C2, C4, D2|, D2-, D2\, D2/, D4+, D4X, D8]

ARGS:
    <X>
            图样的宽度

    <Y>
            图样的高度

    <P>
            图样的周期 [默认: 1]

    <DX>
            水平方向的平移 [默认: 0]

    <DY>
            竖直方向的平移 [默认: 0]
```

比如说，用 `cargo run 16 5 3 0 1` 可以找到 [25P3H1V0.1](http://conwaylife.com/wiki/25P3H1V0.1)。

## TUI

若没有 `--no-tui`，输入命令后会进入一个简陋的 TUI，大概是这个样子：

```text
????????????????
????????????????
????????????????
????????????????
????????????????


Showing generation 0. Time taken: 0ns.
Paused. Press [space] to resume.

```

按空格键开始/暂停搜索，按 q 键退出，按左右方向键显示图样的上一代/下一代。注意此用法和 lifesrc 不同。

`.`、`O`、`?`分别代表死细胞、活细胞、未知的细胞。搜索结果可以直接复制粘贴到 [Golly](http://golly.sourceforge.net/) 中。如果在搜索过程中，复制前请先暂停。
