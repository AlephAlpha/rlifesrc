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
        .version("0.1.0")
        .arg(Arg::with_name("X")
            .help("Width of the pattern")
            .required(true)
            .index(1))
        .arg(Arg::with_name("Y")
            .help("Height of the pattern")
            .required(true)
            .index(2))
        .arg(Arg::with_name("P")
            .help("Period of the pattern")
            .default_value("1")
            .index(3))
        .arg(Arg::with_name("DX")
            .help("Horizontal translation")
            .default_value("0")
            .index(4))
        .arg(Arg::with_name("DY")
            .help("Vertical translation")
            .default_value("0")
            .index(5))
        .arg(Arg::with_name("SYMMETRY")
            .help("Symmetry of the pattern")
            .short("s")
            .long("symmetry")
            .possible_values(&["C1","C2","C4","D2|","D2-","D2\\","D2/","D4+","D4X","D8"])
            .default_value("C1")
            .takes_value(true))
        .arg(Arg::with_name("RULE")
            .help("Rule of the cellular automaton")
            .short("r")
            .long("rule")
            .default_value("B3/S23")
            .takes_value(true))
        .arg(Arg::with_name("ALL")
            .help("Searches for all possible patterns")
            .short("a")
            .long("all"))
        .arg(Arg::with_name("RANDOM")
            .help("Searches for a random pattern")
            .conflicts_with("ALL")
            .long("random"))
        .arg(Arg::with_name("TIME")
            .help("Shows the time used in milliseconds")
            .short("t")
            .long("time"))
        .get_matches();

    let width = matches.value_of("X").unwrap().parse().unwrap();
    let height = matches.value_of("Y").unwrap().parse().unwrap();
    let period = matches.value_of("P").unwrap().parse().unwrap();
    let dx = matches.value_of("DX").unwrap().parse().unwrap();
    let dy = matches.value_of("DY").unwrap().parse().unwrap();

    let symmetry = matches.value_of("SYMMETRY").unwrap().parse().unwrap();
    let all = matches.is_present("ALL");
    let random = matches.is_present("RANDOM");

    let rule = matches.value_of("RULE").unwrap().parse().unwrap();

    let time = matches.is_present("TIME");
    let mut stopwatch = Stopwatch::new();
    if time {
        stopwatch.start();
    }

    let mut search = Search::new(Life::new(width, height, period, dx, dy, symmetry, rule),
        random);
    if all {
        while search.search().is_ok() {
            search.world.display();
            println!("");
        }
    } else if search.search().is_ok() {
        search.world.display();
    }
    if time {
        println!("Time taken: {:?}.", stopwatch.elapsed());
    }
}
