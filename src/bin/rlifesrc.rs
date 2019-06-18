#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
use rlifesrc::web;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
use rlifesrc::cli;

#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
fn main() {
    yew::start_app::<web::Model>();
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
fn main() {
    let args = cli::parse_args().unwrap();
    cli::search(args);
}
