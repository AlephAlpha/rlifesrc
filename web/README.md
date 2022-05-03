# [rlifesrc-web](https://github.com/AlephAlpha/rlifesrc)

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/AlephAlpha/rlifesrc/test)](https://github.com/AlephAlpha/rlifesrc/actions) [![English](https://img.shields.io/badge/readme-English-brightgreen)](src/help.md)

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch) 和 Jason Summers 写的 [WinLifeSearch](https://github.com/jsummers/winlifesearch/)。其具体的算法可见 [Dean Hickerson 的说明](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN)。

写得非常糟糕，和 WinLifeSearch 相比缺少很多功能，而且速度要慢很多，但支持更多规则。

支持 [Life-like](https://conwaylife.com/wiki/Totalistic_Life-like_cellular_automaton) 和 [non-totalistic](https://conwaylife.com/wiki/Non-isotropic_Life-like_cellular_automaton) 的规则，但后者比前者要略慢一些。也支持[六边形](https://conwaylife.com/wiki/Hexagonal_neighbourhood)以及[von Neumann 邻域](https://conwaylife.com/wiki/Von_Neumann_neighbourhood)的规则，但目前是通过转化成 non-totalistic 规则来实现的，速度较慢。还支持 [Generations](https://conwaylife.com/wiki/Generations) 规则。

[点此试用。](https://alephalpha.github.io/rlifesrc/)（[国内镜像](https://alephalpha.gitee.io/rlifesrc/)）

这里是 rlifesrc 的网页版。文本界面的说明见[`tui/`](../tui/README.md) 目录中的 `README.md`。

- [rlifesrc-web](#rlifesrc-web)
  - [编译](#编译)
  - [用法](#用法)

## 编译

网页版的部署用 Github Actions 自动完成，参见[`build-web.yml`](./../.github/workflows/build-web.yml)。

如果要手动编译的话：

1. 编译前要安装**最新版**的 [`trunk`](https://github.com/thedodd/trunk)：

    ```bash
    cargo install --locked trunk
    ```

2. 由于是编译成 WebAssembly，还要给 Rust 添加相应的 target：

    ```bash
    rustup target add wasm32-unknown-unknown
    ```

3. 然后 `cd` 到 `web` 目录，并运行：

    ```bash
    trunk build --release
    ```

4. 编译输出的文件在 `web/dist` 目录。

如果只是想在本地使用网页版，完全可以不用编译，只需要把编译好的版本 `git clone` 下来，用 Python 自带的服务器功能：

```bash
git clone --single-branch --branch=gh-pages --depth 1 https://github.com/AlephAlpha/rlifesrc.git
python3 -m http.server
```

然后在浏览器打开 `http://0.0.0.0:8000/rlifesrc/` 即可。由于前面第3步说的问题，不要先 `cd` 到 `rlifesrc` 再运行服务器。

注意由于浏览器同源策略的问题， `git clone` 下来后无法直接在浏览器中打开运行。

## 用法

这个算法适合搜索小周期的宽扁或者瘦高的图样，但理论上也能搜别的图样。支持 Life-like 和 Isotropic non-totalistic 的规则。

进入页面后在 “Settings” 标签下按照说明调整图样的宽度、高度、周期、平移等参数，然后点击 “Apply settings” 来确定这些参数。然后点 “Start” 开始搜索。如果没有反应，可能是 wasm 未加载完成，可以等一下再按一次 “Start”。

搜到结果后再点 “Start” 会在当前结果的基础上搜下一个结果。如果要从头开始搜索，可以点击 “Reset” 来重置世界。

搜索所需的时间可能很长。点击 “Save” 可以把当前的搜索状态保存在一个 JSON 文件中，点 “Load” 可以上传保存的搜索状态。

输出的结果用 Golly 的 [Extended RLE](http://golly.sourceforge.net/Help/formats.html#rle) 格式显示；但不会合并相邻的相同符号，而是采用类似于 [Plaintext](https://conwaylife.com/wiki/Plaintext) 格式的排版。

具体来说：

* `.` 表示死细胞；
* 对于两种状态的规则，`o` 表示活细胞；对于超过两种状态的 Generations 规则，`A` 表示活细胞，`B` 及以后的字母表示正在死亡的细胞；
* `?` 表示搜索过程中未知的细胞；
* 每行以 `$` 结尾；
* 整个图样以 `!` 结尾。

目前无法正常显示大于 25 种状态的 Generations 规则。

点击左上角的 “Generation” 右边的加减号，或者在数字上滚动鼠标滚轮，可以切换显示的代数。“Cell count” 指的是当前代的活细胞个数，不包括 Generations 规则中正在死亡的细胞。

以下是各种参数的具体说明：

<dl>
  <dt>Rule</dt>
  <dd>
  元胞自动机的规则

  支持 Life-like, isotropic non-totalistic, hexagonal, MAP 等规则，以及相应的 Generations 规则
  </dd>

  <dt>Width</dt>
  <dd>
  图样的宽度
  </dd>

  <dt>Height</dt>
  <dd>
  图样的高度
  </dd>

  <dt>Period</dt>
  <dd>
  图样的周期
  </dd>

  <dt>dx</dt>
  <dd>
  水平方向的平移
  </dd>

  <dt>dy</dt>
  <dd>
  水平方向的平移
  </dd>

  <dt>Diagonal width</dt>
  <dd>
  对角宽度。

  如果对角宽度为 `n > 0`，对于坐标为 `(x, y)` 的细胞，若 `abs(x - y) >= n`，则设细胞为死。

  如果这个值设为 `0`，则忽略此项。
  </dd>

  <dt>Transformation</dt>
  <dd>
  图样的变换。

  图样在一个周期中的变化相当于先进行此变换，再进行平移。

  8 种不同的变换，对应二面体群 _D_<sub>8</sub> 的 8 个元素。其中：

  * `Id` 表示恒等变换。
  * `R` 表示旋转（Rotate）， 后面的数字表示逆时针旋转的角度。
  * `F` 表示翻转（Flip）， 后面的符号表示翻转的轴线。

  比如说，如果想要搜索竖直方向的 [glide symmetric](https://conwaylife.com/wiki/Types_of_spaceships#Glide_symmetric_spaceship) 的飞船，变换可以设成 `Flip |`。

  注意有些变换要求世界是正方形，有些则要求没有对角宽度。
  </dd>

  <dt>Symmetry</dt>
  <dd>
  图样的对称性。

  10 种不同的对称性，对应二面体群 _D_<sub>8</sub> 的 10 个子群。这些对称性的写法来自 Oscar Cunningham 的 [Logic Life Search](https://github.com/OscarCunningham/logic-life-search)。详见 [Life Wiki 上的相应说明](https://conwaylife.com/wiki/Static_symmetry#Reflectional)。

  注意有些对称性要求世界是正方形，有些则要求没有对角宽度。
  </dd>

  <dt>Max cell count</dt>
  <dd>
  活细胞个数的上界（只考虑活细胞最少的一代）。

  如果这个值设为 0，则不限制活细胞的个数。
  </dd>

  <dt>Search order</dt>
  <dd>
  搜索顺序。

  无论哪种搜索顺序，总是先搜完一个细胞的每一代，再搜下一个细胞。

  `Automatic` 指的是先搜窄的一边。也就是说，行比列少先搜列，列比行少先搜行。当世界为正方形，且对角宽度不大于世界宽度的一半时，会选择对角顺序。

  对角搜索顺序要求世界是正方形。
  </dd>

  <dt>Known cells</dt>
  <dd>
  搜索前状态已知的细胞。

  支持两种输入格式

  - **JSON**:

    输入可以是一个包含已知细胞的坐标和状态信息的 [JSON](https://en.wikipedia.org/wiki/JSON) 字符串，比如说：

    ```json
    [{"coord":[0,0,0],"state":0},{"coord":[1,1,0],"state":1}]
    ```

    其中：

    * `coord` 表示细胞的坐标，`[x,y,t]` 表示位于 `(x,y)` 处的细胞的第 `t` 代；
    * `state` 表示细胞的状态。对于非 Generations 的规则，`0` 表示死，`1` 表示生。Generations 规则的其它状态也用数字来表示。

    您可以直接复制粘贴存档文件的 `"set_stack"` 字段。该字段中还包含了一个 `reason` 字段，在读取已知细胞时会被忽略。

  - **RLE**:

    输入也可以用 RLE 格式，比如说：

    ```
    x = 16, y = 16, rule = B3/S23
    ?o$2bo$2?o!
    ```

    与平常的 RLE 不同，这种 RLE 多了一个符号 `?`，用来表示未知的细胞。此时图样的背景是未知的细胞，每行末尾的死细胞不可省略。

    您可以直接复制粘贴输出的搜索结果。

    如果已知的细胞并非都在第一代，您可以输入多个 RLE 字符串，用换行隔开，每个 RLE 代表一代。每个 RLE 都必须已 `!` 结尾。

    RLE 格式的输入会被自动转化为 JSON 格式。
  </dd>

  <dt>Choice of state for unknown cells</dt>
  <dd>
  如何为未知的细胞选取状态。

  有先选活、先选死、随机选取三种选项。搜索振荡子时随机选取可能效果更佳。
  </dd>

  <dt>Reduce max cell count when a result is found</dt>
  <dd>
  搜到结果时自动缩小活细胞个数的上界。

  新的上界会被设置为当前的活细胞个数减一（只考虑活细胞最少的一代）。

  <dt>Skip patterns with subperiod</dt>
  <dd>
  跳过基本周期小于指定周期的图样。
  </dd>

  <dt>Skip patterns invariant under more transformations than the given symmetry</dt>
  <dd>
  跳过在比指定的对称性更多的变换下不变对称图样。

  也就是说，跳过对称群真包含指定的对称性的对称群的图样。
  </dd>

  <dt> (Experimental) Enable backjumping</dt>
  <dd>
  [Backjumping](https://en.wikipedia.org/wiki/Backjumping) 可以减少搜索所需的步数，但每一步所需的时间会变得更长。当前的实现对大部分搜索来说都会变得更慢，仅在搜索大静物（比如说 64x64）时有用。

  当前仅支持非 Generations 的规则，且 Max cell count 必须为 0，否则会忽略此选项。
  </dd>
</dl>
