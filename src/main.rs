use std::env;
use crate::search::Search;
use crate::rule::Life;
mod search;
mod rule;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("Not enough arguments!");
    }
    let width = (&args[1]).parse().expect("Not a number!");
    let height = (&args[2]).parse().expect("Not a number!");
    let period = if args.len() < 4 {
        1
    } else {
        (&args[3]).parse().expect("Not a number!")
    };
    let dx = if args.len() < 5 {
        0
    } else {
        (&args[4]).parse().expect("Not a number!")
    };
    let dy = if args.len() < 6 {
        0
    } else {
        (&args[5]).parse().expect("Not a number!")
    };
    let mut search = Search::new(Life::new(width, height, period, dx, dy));
    while search.search() {
        search.display();
        println!("");
    }
    println!("No more result.");
}
