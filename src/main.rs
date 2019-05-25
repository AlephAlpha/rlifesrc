extern crate clap;
extern crate stopwatch;
use clap::{Arg, App};
use stopwatch::Stopwatch;
use crate::search::Search;
use crate::rule::Life;
mod search;
mod rule;
mod world;

fn main() {
    let matches = App::new("rlifesrc")
        .arg(Arg::with_name("X")
            .help("Number of columns")
            .required(true)
            .index(1))
        .arg(Arg::with_name("Y")
            .help("Number of rows")
            .required(true)
            .index(2))
        .arg(Arg::with_name("P")
            .help("Number of generations")
            .index(3))
        .arg(Arg::with_name("DX")
            .help("Column translation")
            .index(4))
        .arg(Arg::with_name("DY")
            .help("Row translation")
            .index(5))
        .arg(Arg::with_name("SYMMETRY")
            .help("Symmetry of the pattern")
            .short("s")
            .long("symmetry")
            .takes_value(true))
        .arg(Arg::with_name("RULE")
            .help("Rule of the cellular automaton")
            .short("r")
            .long("rule")
            .takes_value(true))
        .arg(Arg::with_name("ALL")
            .help("Search for all possible patterns")
            .short("a")
            .long("all"))
        .arg(Arg::with_name("TIME")
            .help("Show how long the search takes")
            .short("t")
            .long("time"))
        .get_matches();

    let width = matches.value_of("X").unwrap().parse().unwrap();
    let height = matches.value_of("Y").unwrap().parse().unwrap();
    let period = matches.value_of("P").unwrap_or("1").parse().unwrap();
    let dx = matches.value_of("DX").unwrap_or("0").parse().unwrap();
    let dy = matches.value_of("DY").unwrap_or("0").parse().unwrap();

    let symmetry = matches.value_of("SYMMETRY").unwrap_or("C1").parse().unwrap();
    let all = matches.is_present("ALL");

    let rule = matches.value_of("RULE").unwrap_or("B3/S23").parse().unwrap();

    let time = matches.is_present("TIME");
    let mut stopwatch = Stopwatch::new();
    if time {
        stopwatch.start();
    }

    let mut search = Search::new(Life::new(width, height, period, dx, dy, symmetry, rule));
    if all {
        while search.search().is_ok() {
            search.display();
            println!("");
        }
    } else if search.search().is_ok() {
        search.display();
    }
    if time {
        println!("Time taken: {}ms.", stopwatch.elapsed_ms());
    }
}
