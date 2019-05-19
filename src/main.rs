extern crate clap;
use clap::{Arg, App};
use crate::search::Search;
use crate::rule::Life;
use crate::rule::Symmetry;
mod search;
mod rule;
mod world;

fn main() {
    let matches = App::new("rlifesrc")
        .arg(Arg::with_name("WIDTH")
             .help("Number of columns")
             .required(true)
             .index(1))
        .arg(Arg::with_name("HEIGHT")
             .help("Number of rows")
             .required(true)
             .index(2))
        .arg(Arg::with_name("PERIOD")
             .help("Number of generations")
             .index(3))
        .arg(Arg::with_name("DX")
             .help("Row translation")
             .index(4))
        .arg(Arg::with_name("DY")
             .help("Column translation")
             .index(5))
        .arg(Arg::with_name("SYMMETRY")
             .help("Symmetry of the pattern")
             .short("s")
             .long("symmetry")
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
    let width = matches.value_of("WIDTH").unwrap().parse().unwrap();
    let height = matches.value_of("HEIGHT").unwrap().parse().unwrap();
    let period = matches.value_of("PERIOD").unwrap_or("1").parse().unwrap();
    let dx = matches.value_of("DX").unwrap_or("0").parse().unwrap();
    let dy = matches.value_of("DY").unwrap_or("0").parse().unwrap();
    let symmetry = match matches.value_of("SYMMETRY").unwrap_or("C1") {
        "C1" => Symmetry::C1,
        "C2" => Symmetry::C2,
        "C4" => Symmetry::C4,
        "D2|" => Symmetry::D2Row,
        "D2-" => Symmetry::D2Column,
        "D2\\" => Symmetry::D2Diag,
        "D2/" => Symmetry::D2Antidiag,
        "D4+" => Symmetry::D4Ortho,
        "D4X" => Symmetry::D4Diag,
        "D8" => Symmetry::D8,
        _ => Symmetry::C1,
    };
    let all = matches.is_present("ALL");
    let time = matches.is_present("TIME");
    let mut search = Search::new(Life::new(width, height, period, dx, dy, symmetry), time);
    if all {
        while search.search() {
            search.display();
            println!("");
        }
    } else if search.search() {
        search.display();
    }
}
