mod args;

#[cfg(feature = "tui")]
mod tui;

use args::Args;
use rlifesrc_lib::{Search, Status};
use std::process::exit;

/// Runs the search without TUI.
///
/// If `all` is true, it will print all possible results
/// instead of only the first one.
fn run_search(mut search: Box<dyn Search>, all: bool) {
    if all {
        let mut found = false;
        loop {
            match search.search(None) {
                Status::Found => {
                    found = true;
                    println!("{}", search.rle_gen(0))
                }
                Status::None => break,
                _ => (),
            }
        }
        if !found {
            eprintln!("Not found.");
            exit(1);
        }
    } else if let Status::Found = search.search(None) {
        println!("{}", search.rle_gen(0));
    } else {
        eprintln!("Not found.");
        exit(1);
    }
}

#[cfg(feature = "tui")]
fn main() {
    let args = Args::parse().unwrap_or_else(|e| e.exit());
    let search = args.search;
    if args.no_tui {
        run_search(search, args.all);
    } else {
        tui::tui(search, args.reset).unwrap();
    }
}

#[cfg(not(feature = "tui"))]
fn main() {
    let args = Args::parse().unwrap_or_else(|e| e.exit());
    run_search(args.search, args.all);
}
