mod search;

#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
mod web;

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
mod cli;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
mod tui;

#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
fn main() {
    yew::start_app::<web::Model>();
}

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
fn main() {
    let args = cli::parse_args().unwrap();
    cli::search(args);
}
