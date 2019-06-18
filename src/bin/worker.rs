// 非网页版完全不需要这个文件，但不知道怎样让 cargo 忽略它
// 即使加了 #! 开头这一行，

#![cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]

use rlifesrc::worker::Worker;
use yew::agent::Threaded;

fn main() {
    yew::initialize();
    Worker::register();
    yew::run_loop();
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
fn main() {}
