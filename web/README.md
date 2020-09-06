# [rlifesrc-web](https://github.com/AlephAlpha/rlifesrc)

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch) 和 Jason Summers 写的 [WinLifeSearch](https://github.com/jsummers/winlifesearch/)。其具体的算法可见 [Dean Hickerson 的说明](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），而且在不懂 JavaScript 的情况下弄成一个网页，写得非常糟糕，和 WinLifeSearch 相比缺少很多功能，而且速度要慢很多，但支持更多规则。

支持 [Life-like](https://conwaylife.com/wiki/Totalistic_Life-like_cellular_automaton) 和 [non-totalistic](https://conwaylife.com/wiki/Non-isotropic_Life-like_cellular_automaton) 的规则，但后者比前者要略慢一些。也支持[六边形](https://conwaylife.com/wiki/Hexagonal_neighbourhood)以及[von Neumann 邻域](https://conwaylife.com/wiki/Von_Neumann_neighbourhood)的规则，但目前是通过转化成 non-totalistic 规则来实现的，速度较慢。还支持 [Generations](https://conwaylife.com/wiki/Generations) 规则，此功能是实验性的，可能有 bug。

[点此试用。](https://alephalpha.github.io/rlifesrc/)

这里是 rlifesrc 的网页版。文本界面的说明见[`tui/`](../tui/README.md) 目录中的 `README.md`。

- [rlifesrc-web](#rlifesrc-web)
  - [编译](#编译)
  - [用法](#用法)

## 编译

这是用 Rust 写的。没有 Rust 的话，先安装 [Rust](https://www.rust-lang.org/)。

编译网页版还要先安装 [cargo-web](https://github.com/koute/cargo-web)：


```bash
cargo install cargo-web
```

准备好了之后，就可以用 `git clone` 下载：

```bash
git clone https://github.com/AlephAlpha/rlifesrc.git
cd rlifesrc/
```

安装了 cargo-web 之后，用

```bash
cd web/
cargo web build --release
```

来编译即可。记得一定要 `cd` 到 `web` 目录。

由于用了 Web Worker，需要编译两个文件，无法直接使用 `cargo web start` 来运行，或者用 `cargo web deploy` 编译成静态网页。只能在编译之后手动把 `target/wasm32-unknown-unknown/release/` 文件夹里的以 `*.js` 和 `*.wasm` 结尾的四个文件，以及 `static` 中的两个文件，复制到同一个文件夹：

```bash
mkdir -p some_folder/
cp ../target/wasm32-unknown-unknown/release/*.{js,wasm} some_folder/
cp static/* some_folder/
```

然后就可以把这个文件夹中的内容部署到自己的网站，比如说 GitHub Pages。注意由于[此问题](https://developer.mozilla.org/zh-CN/docs/Web/HTTP/CORS/Errors/CORSRequestNotHttp)，无法直接在浏览器打开 `index.html` 来运行；至少火狐浏览器如此。

## 用法

这个算法适合搜索小周期的宽扁或者瘦高的图样，但理论上也能搜别的图样。支持 Life-like 和 Isotropic non-totalistic 的规则。

进入页面后在 “Settings” 标签下按照说明调整图样的宽度、高度、周期、平移等参数，然后点击 “Apply settings” 来确定这些参数。然后点 “Start” 开始搜索。如果没有反应，可能是 wasm 未加载完成，可以等一下再按一次 “Start”。

搜到结果后再点 “Start” 会在当前结果的基础上搜下一个结果。如果要从头开始搜索，可以点击 “Reset” 来重置世界。

搜索所需的时间可能很长。点击 “Save” 可以通过 [Web Storage API](https://developer.mozilla.org/zh-CN/docs/Web/API/Web_Storage_API) 把当前的搜索状态保存在浏览器中，点 “Load” 可以读取。关闭浏览器，保存的搜索状态不会消失。目前尚不支持自动保存/读取。

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

  <dt>Transformation</dt>
  <dd>
  图样的变换。

  图样在一个周期中的变化相当于先进行此变换，再进行平移。

  8 种不同的变换，对应二面体群 _D_<sub>8</sub> 的 8 个元素。`Id` 表示恒等变换。`Rotate` 表示旋转， 后面的数字表示逆时针旋转的角度。`Flip` 表示翻转， 后面的符号表示翻转的轴线。比如说，如果想要搜索竖直方向的 [glide symmetric](https://conwaylife.com/wiki/Types_of_spaceships#Glide_symmetric_spaceship) 的飞船，变换可以设成 `Flip |`。

  注意有些变换要求世界是正方形。
  </dd>

  <dt>Symmetry</dt>
  <dd>
  图样的对称性。

  10 种不同的对称性，对应二面体群 _D_<sub>8</sub> 的 10 个子群。这些对称性的写法来自 Oscar Cunningham 的 [Logic Life Search](https://github.com/OscarCunningham/logic-life-search)。详见 [Life Wiki 上的相应说明](https://conwaylife.com/wiki/Symmetry)。

  注意有些对称性要求世界是正方形。
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

  `Automatic` 指的是先搜窄的一边。也就是说，行比列少先搜列，列比行少先搜行。不会自动选择对角方向。

  对角搜索顺序要求世界是正方形。
  </dd>

  <dt>Choice of state for unknown cells</dt>
  <dd>
  如何为未知的细胞选取状态。

  有先选活、先选死、随机选取三种选项。搜索振荡子时随机选取可能效果更佳。

  <dt>Non empty front</dt>
  <dd>
  强制要求第一行/第一列非空。具体是行还是列由搜索顺序决定。

  在搜索宽扁或者瘦高的不对称或沿长边 `C2` 对称的图样时，勾选此选项可去掉大量的重复搜索。
  </dd>

  <dt>Reduce max cell count</dt>
  <dd>
  搜到结果时自动缩小活细胞个数的上界。

  新的上界会被设置为当前的活细胞个数减一（只考虑活细胞最少的一代）。
  </dd>
</dl>
