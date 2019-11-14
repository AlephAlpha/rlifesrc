# [rlifesrc-web](https://alephalpha.github.io/rlifesrc/)

试玩 Rust。尝试写一个生命游戏搜索工具。具体来说就是照抄 David Bell 写的 [lifesrc](https://github.com/DavidKinder/Xlife/tree/master/Xlife35/source/lifesearch) 和 Jason Summers 写的 [WinLifeSearch](https://github.com/jsummers/winlifesearch/)。其具体的算法可见 [Dean Hickerson 的说明](https://github.com/DavidKinder/Xlife/blob/master/Xlife35/source/lifesearch/ORIGIN)。

由于是从一种没学过的语言（C）抄到一种没用过的语言（Rust），而且在不懂 javascript 的情况下弄成一个网页，写得非常糟糕，和 WinLifeSearch 相比缺少很多功能，而且速度要慢很多，但支持更多规则。

支持 [Life-like](http://conwaylife.com/wiki/Totalistic_Life-like_cellular_automaton) 和 [non-totalistic](https://www.conwaylife.com/wiki/Non-isotropic_Life-like_cellular_automaton) 的规则，但后者比前者要略慢一些。也支持[六边形](https://www.conwaylife.com/wiki/Hexagonal_neighbourhood)以及[von Neumann 邻域](https://www.conwaylife.com/wiki/Von_Neumann_neighbourhood)的规则，但目前是通过转化成 non-totalistic 规则来实现的，速度较慢。

[点此试用。](https://alephalpha.github.io/rlifesrc/)

这里是 rlifesrc 的网页版。文本界面的说明见[`tui/`](../tui/README.md) 目录中的 `README.md`。

* [编译](#编译)
* [用法](#用法)

## 编译

这是用 Rust 写的。没有 Rust 的话，先安装 [Rust](https://www.rust-lang.org/)。

网页版无法编译成机器码，只能编译成 WebAssembly，因此需要还要安装 [cargo-web](https://github.com/koute/cargo-web)。

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

进入页面后按照说明调整图样的宽度、高度、周期、平移等参数，然后点击 “Set World” 来确定这些参数。然后点 “Start” 开始搜索。如果没有反应，可能是 wasm 未加载完成，可以等一下再按一次 “Start”。

搜到结果后再点 “Start” 会在当前结果的基础上搜下一个结果。如果要从头开始搜索，可以点击 “Set World” 来重置世界。

搜索的过程与结果以 [Plaintext](http://conwaylife.com/wiki/Plaintext) 格式显示，用 `.` 表示死细胞，`O` 表示活细胞，`?` 表示未知的细胞。

点击左上角的 “Generation” 右边的加减号可以切换显示的代数。注意 “Cell count” 指的是第 __0__ 代的活细胞个数。

以下是各种参数的具体说明：

<dl>
  <dt>Rule</dt>
  <dd>
  元胞自动机的规则

  支持 Life-like 和 Isotropic non-totalistic 的规则
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

  8 种不同的变换，对应二面体群 D8 的 8 个元素。`Id` 表示恒等变换。`Rotate` 表示旋转， 后面的数字表示逆时针旋转的角度。`Flip` 表示翻转， 后面的符号表示翻转的轴线。比如说，如果想要搜索竖直方向的 [glide symmetric](http://www.conwaylife.com/wiki/Types_of_spaceships#Glide_symmetric_spaceship) 的飞船，变换可以设成 `Flip |`。

  注意有些变换要求世界是正方形。
  </dd>

  <dt>Symmetry</dt>
  <dd>
  图样的对称性。

  10 种不同的对称性，对应二面体群 D8 的 10 个子群。这些对称性的用法和 Oscar Cunningham 的 Logic Life Search 一样。详见 [Life Wiki 上的相应说明](http://conwaylife.com/wiki/Symmetry)。

  注意有些对称性要求世界是正方形。
  </dd>

  <dt>Max cell count</dt>
  <dd>
  第 0 代活细胞个数的极大值。

  如果这个值设为 0，则不限制活细胞的个数。
  </dd>

  <dt>Max cell count</dt>
  <dd>
  搜索顺序。

  先搜行还是先搜列，或者根据图样的宽度和高度自动选取。
  </dd>

  <dt>Choice of state for unknown cells</dt>
  <dd>
  如何为未知的细胞选取状态。

  有先选活、先选死、随机选取三种选项。

  <dt>Non empty front</dt>
  <dd>
  强制要求第一行/第一列非空。具体是行还是列由搜索顺序决定。

  在搜索宽扁或者瘦高的不对称或沿长边 `C2` 对称的图样时，勾选此选项可去掉大量的重复搜索。
  </dd>
</dl>