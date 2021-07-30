mod args;

#[cfg(feature = "tui")]
mod tui;

use args::Args;
use rlifesrc_lib::{PolyWorld, Status};
use std::process::exit;

/// Runs the search without TUI.
///
/// If `all` is true, it will print all possible results
/// instead of only the first one.
fn run_search(world: &mut PolyWorld, all: bool) {
    if all {
        let mut found = false;
        loop {
            match world.search(None) {
                Status::Found => {
                    found = true;
                    println!("{}", world.rle_gen(0));
                }
                Status::None => break,
                _ => (),
            }
        }
        if !found {
            eprintln!("Not found.");
            exit(1);
        }
    } else if let Status::Found = world.search(None) {
        println!("{}", world.rle_gen(0));
    } else {
        eprintln!("Not found.");
        exit(1);
    }
}

#[cfg(feature = "tui")]
fn main() {
    let args = Args::parse().unwrap_or_else(|e| e.exit());
    let mut world = args.world;
    if args.no_tui {
        run_search(&mut world, args.all);
    } else {
        tui::tui(world, args.reset).unwrap();
    }
}

#[cfg(not(feature = "tui"))]
fn main() {
    let mut args = Args::parse().unwrap_or_else(|e| e.exit());
    run_search(&mut args.world, args.all);
}
