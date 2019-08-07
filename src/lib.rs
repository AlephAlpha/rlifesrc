#![recursion_limit = "256"]

mod rules;
mod search;
mod world;

#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
pub mod web;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
pub mod worker;

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
pub mod cli;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
pub mod tui;
