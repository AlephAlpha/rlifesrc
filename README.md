# [rlifesrc](https://alephalpha.github.io/rlifesrc/)

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch)。其具体的算法可见 [Dean Hickerson 的说明](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），而且在不懂 javascript 的情况下弄成一个网页，写得非常糟糕，和原版的 lifesrc 相比缺少很多功能，但速度要稍快一些（网页版除外）。

支持 [Life-like](http://conwaylife.com/wiki/Totalistic_Life-like_cellular_automaton) 和 [Isotropic non-totalistic](http://conwaylife.com/wiki/Isotropic_non-totalistic_Life-like_cellular_automaton) 的规则，但后者比前者要略慢一些。

* [编译](#编译)
  * [编译网页版](#编译网页版)
* [用法](#用法)
  * [命令行界面](#命令行)
  * [文本界面（TUI）](#文本界面)
  * [网页版](#网页版)

## 编译

这是用 Rust 写的。没有 Rust 的话，先安装 [Rust](https://www.rust-lang.org/)。

无论是编译，还是用 `cargo` 来运行，一定要记得加上 `--release`，不然会特别慢，相差大约一百倍。

文本界面是用 [pancurses](https://github.com/ihalila/pancurses) 写的，在编译之前请参照 [ncurses-rc](https://github.com/jeaye/ncurses-rs)（Unix-like）或 [pdcurses-sys](https://github.com/ihalila/pdcurses-sys)（Windows） 的说明来安装相应的依赖。如果只需要网页版，不必安装这些依赖。

准备好了之后，就可以下载和安装：

```bash
git clone https://github.com/AlephAlpha/rlifesrc.git
cd rlifesrc
cargo build --release
```

编译需要一定时间，请耐心等待

编译好的文件是 `./target/release/rlifesrc`。

### 编译网页版

为了把 Rust 编译成 WebAssembly，需要安装 [cargo-web](https://github.com/DenisKolodin/yew)。

用

```bash
cargo web build --release
```

来编译即可。然后用

```bash
cargo web build --release
```

来运行。使用时按其说明在浏览器打开网页（一般是 127.0.0.1:8000）即可。

也可以用

```bash
cargo web deploy --release
```

来编译成静态网页，无需服务器即可使用。

## 用法

这个算法适合搜索小周期的宽扁或者瘦高的图样，但理论上也能搜别的图样。支持 Life-like 和 Isotropic non-totalistic 的规则。

[网页版可以直接在此使用。](https://alephalpha.github.io/rlifesrc/)

本地运行的版本需要[下载和编译](#编译)，它提供[命令行](#命令行)和[文本（TUI）](#文本界面)两种界面。编译好的文件是 `./target/release/rlifesrc`。也可以用 `cargo run --release` 来运行（不加 `--release` 的话会特别慢）。其用法如下：

```plaintext
USAGE:
    rlifesrc [FLAGS] [OPTIONS] <X> <Y> [ARGS]

FLAGS:
    -a, --all
            搜索所有的满足条件的图样
            仅适用于命令行界面

    -n, --no-tui
            不进入文本界面，直接开始搜索
            此即命令行界面

        --reset-time
            开始新的搜索时重置计时
            仅适用于文本界面

    -h, --help
            显示此帮助信息的英文版

    -V, --version
            显示版本信息（永远是 0.1.0）


OPTIONS:
    -c, --choose <CHOOSE>
            如何为未知的细胞选取状态
            其中 'smart' 表示第一行/第一列的细胞随机选取一个状态，其余的细胞先设为死。
             [默认: dead]  [可能的值: dead, alive, random, smart, d, a, r, s]

    -o, --order <ORDER>
            搜索顺序
            先搜行还是先搜列。
             [默认: automatic]  [可能的值: row, column, automatic, r, c, a]

    -r, --rule <RULE>
            元胞自动机的规则
            支持 Life-like 和 Isotropic non-totalistic 的规则
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

比如说，要想找到 [25P3H1V0.1](http://conwaylife.com/wiki/25P3H1V0.1)，可以用：

```bash
./target/release/rlifesrc 16 5 3 0 1
```

网页版涉及到的参数和选项与此类似，不过是直接在网页中输入，并点击 “Set World” 确认。

对称性的用法和 Oscar Cunningham 的 Logic Life Search 一样，详见 [Life Wiki 上的相应说明](http://conwaylife.com/wiki/Symmetry)。

搜索顺序中的 “Automatic” 指的是先搜窄的一边。也就是说，行比列少先搜列，列比行少先搜行。

`CHOOSE`（网页版叫 “New state”）指的是如何为未知的细胞选取状态。其中 “smart” 指的是：如果这个未知细胞在第一行/第一列，则随机选取；如果不在第一行/第一列，则就先设为死；具体是第一行还是第一列由搜索顺序决定。其实一点也不智能，不过我想不出别的名字了。这个模式适合搜索比较窄但可能很长的图样，但未必比 “random” 快。

### 命令行

命令行界面是最简单的界面，功能只有输入命令，输出结果，不会显示搜索过程。

结果以 [Plaintext](http://conwaylife.com/wiki/Plaintext) 格式输出，用 `.` 表示死细胞，`O` 表示活细胞。

比如说，输入

```bash
./target/release/rlifesrc 7 7 3
```

会显示以下结果：

```plaintext
....O..
..OOO..
.O...OO
..O..O.
..OO.O.
OO..O..
OO.....
```

加上命令行选项 `--all` 会一个一个地输出所有的结果。

### 文本界面

文本界面也十分简陋，但可以显示搜索过程和搜索所用的时间。

刚进入文本界面的时候，大概是这个样子：

```plaintext
???????
???????
???????
???????
???????
???????
???????


Showing generation 0. Time taken: 0ns.
Paused. Press [space] to resume.
```

其中 `?` 表示未知的细胞。

然后按空格键或回车键开始/暂停搜索，按 q 键退出，按左右方向键显示图样的上一代/下一代。注意此用法和原版的 lifesrc 并不一样。

搜索到的结果同样以 Plaintext 格式显示。此时再按空格键或回车键的话会在当前结果的基础上搜下一个结果。除非加上命令行选项 `--reset-time`，否则不会重置计时。

在退出之前记得把结果复制到别的地方。退出之后不会显示任何结果。

如果搜索的图样比终端的窗口大小还要大，将无法完整显示。这是文本界面最大的缺陷。

### 网页版

由于网页版是编译成 WebAssembly 而不是机器码，速度要慢很多，但随时随地只要有浏览器就能运行。

进入页面后按照说明调整图样的宽度、高度、周期、平移等信息，然后点击 “Set World” 来确定这些信息。然后点 “Start” 开始搜索。

搜索时不要切换到别的标签页。这点可能以后会修正。

搜到结果后再点 “Start” 会在当前结果的基础上搜下一个结果。如果要从头开始搜索，可以点击 “Set World” 来重置世界。

其余的用法和文本界面差不多。
