# Changelog

## Unreleased

### Lib

For the documentation of the unreleased lib, please visit https://alephalpha.github.io/rlifesrc-doc/rlifesrc_lib/.

### Web

The [web version on GitHub.io](https://alephalpha.github.io/rlifesrc/) is always the newest unreleased version on the master branch.

### Others

- Fix some bugs.
- Update dependencies.

## 0.5.2 - 2022/01/01

### Lib

- Add an `extra` field in the save file for saving any extra information, e.g., maximal partial result.

### TUI

- Update Clap to 3.0.0.

### Web

- Save maximal partial result in the save file.
- Update Yew to 0.19.3 and introduce some new dependencies.

### Others

- Fix some bugs.
- Update dependencies.

## 0.5.1 - 2021/07/17

### Lib

- Disable `non empty front` when there are known cells.

### Others

- Fix some bugs.
- Update dependencies.

## 0.5.0 - 2021/07/03

### Lib

- Disable backjumping when `max_cell_count` is specified.
- Remove `Status::Paused`. Now pausing is handled by `tui` and `web` instead of the `lib`.
- The `Rule` trait is now [sealed](https://rust-lang.github.io/api-guidelines/future-proofing.html).
- Add a `timing` field in the save file.
- Add an `Algorithm` trait, but no new algorithm is added. The `Reason` trait is no longer public.
- Use an enum (`PolyWorld`) instead of a trait object (`Box<dyn Search>`), and remove the `Search` trait.
- Remove the lifetime parameter for many types.

### TUI

- Use `futures-executor` instead of `async-std`.

### Web

- Show timing.
- Now you can find all results without pause.
- Now you can specify a list of known cells. [See the doc for input format.](web/src/help.md#known-cells).

### Others

- Fix some bugs.
- Update dependencies.

## 0.4.1 - 2021/02/17

- Fix a bug related to doc generation.

## 0.4.0 - 2021/02/16

- TUI version now supports reading configs from a file.
- Now you can choose to skip patterns with subperiod and/or subsymmetry.
- Now you can enable [Backjumping](https://en.wikipedia.org/wiki/Backjumping) but it is not very useful.
- When the world is square and the diagonal width is not larger than width of the world, `Automatic` search order would choose `Diagonal`.
- Use a linked list instead of a Vec to search for unknown cells. This is the original design of `lifesrc`.
- Now `rlifesrc-lib` supports specifying the search order by a vector of coordinates, and a list of cells known before starting to search. These are not yet supported in the Web version, but you can hack the save file. In the TUI version, you need to submit a config file.
- The `non empty front` option is removed. Now `rlifesrc` would automatically force the front to be non-empty when it can prove that no solution would be missed.
- `serde` feature for `rlifesrc-lib` is enabled by default.
- Fix some bugs.
- Update dependencies.

## 0.3.5 - 2020/12/22

- Now you can specify the [diagonal width](web/src/help.md#diagonal-width).
- Show maximal partial result in Web version.
- Add this Changelog.
- Fix some bugs.
- Update dependencies.

## 0.3.4 - 2020/10/13

- Support [diagonal search order](web/src/help.md#search-order).
- Disallow `B0S8` rules.
- Use [`wasm-bindgen`](https://crates.io/crates/wasm-bindgen)/[`web-sys`](https://crates.io/crates/web-sys) instead of [`cargo-web`](https://crates.io/crates/cargo-web)/[`stdweb`](https://crates.io/crates/stdweb) in Web version.
- Fix some bugs.
- Update dependencies.

## 0.3.3 - 2020/06/26
- Add a Help page in Web version.
- Use [`thiserror`](https://crates.io/crates/thiserror) to define errors.
- Fix some bugs.
- Update dependencies.

## 0.3.2 - 2020/04/05
- Add an [initial](https://docs.rs/rlifesrc-lib/0.3.2/rlifesrc_lib/enum.Status.html#variant.Initial) search status.
- Update dependencies.

## 0.3.1 - 2020/01/05
- 懒得写了。