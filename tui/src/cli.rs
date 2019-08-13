use crate::tui::search_with_tui;
use ca_rules::ParseBSRules;
use clap::AppSettings::AllowNegativeNumbers;
use clap::{App, Arg};
use rlifesrc_lib::rules::{isotropic, life};
use rlifesrc_lib::NewState::{Choose, FirstRandomThenDead, Random};
use rlifesrc_lib::State::{Alive, Dead};
use rlifesrc_lib::{Search, Status, TraitSearch, World};

fn is_positive(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit()) && s != "0" && !s.starts_with('-')
}

pub struct Args {
    search: Box<dyn TraitSearch>,
    all: bool,
    reset: bool,
    no_tui: bool,
}

pub fn parse_args() -> Option<Args> {
    let app = App::new("rlifesrc")
        .version("0.1.0")
        .setting(AllowNegativeNumbers)
        .arg(
            Arg::with_name("X")
                .help("Width of the pattern")
                .required(true)
                .index(1)
                .validator(|x| {
                    if is_positive(&x) {
                        Ok(())
                    } else {
                        Err(String::from("width must be a positive integer"))
                    }
                }),
        )
        .arg(
            Arg::with_name("Y")
                .help("Height of the pattern")
                .required(true)
                .index(2)
                .validator(|y| {
                    if is_positive(&y) {
                        Ok(())
                    } else {
                        Err(String::from("height must be a positive integer"))
                    }
                }),
        )
        .arg(
            Arg::with_name("P")
                .help("Period of the pattern")
                .default_value("1")
                .index(3)
                .validator(|p| {
                    if is_positive(&p) {
                        Ok(())
                    } else {
                        Err(String::from("period must be a positive integer"))
                    }
                }),
        )
        .arg(
            Arg::with_name("DX")
                .help("Horizontal translation")
                .default_value("0")
                .index(4)
                .validator(|d| d.parse::<isize>().map(|_| ()).map_err(|e| e.to_string())),
        )
        .arg(
            Arg::with_name("DY")
                .help("Vertical translation")
                .default_value("0")
                .index(5)
                .validator(|d| d.parse::<isize>().map(|_| ()).map_err(|e| e.to_string())),
        )
        .arg(
            Arg::with_name("SYMMETRY")
                .help("Symmetry of the pattern")
                .long_help(
                    "Symmetry of the pattern\n\
                     You may need to add quotation marks for some of the symmetries.\n\
                     The usages of these symmetries are the same as Oscar Cunningham's \
                     Logic Life Search.\nSee http://conwaylife.com/wiki/Symmetry.\n",
                )
                .short("s")
                .long("symmetry")
                .possible_values(&[
                    "C1", "C2", "C4", "D2|", "D2-", "D2\\", "D2/", "D4+", "D4X", "D8",
                ])
                .default_value("C1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("RULE")
                .help("Rule of the cellular automaton")
                .long_help(
                    "Rule of the cellular automaton\n\
                     Supports Life-like and isotropic non-totalistic rules.\n",
                )
                .short("r")
                .long("rule")
                .default_value("B3/S23")
                .takes_value(true)
                .validator(|d| {
                    isotropic::Life::parse_rule(&d)
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }),
        )
        .arg(
            Arg::with_name("ORDER")
                .help("Search order")
                .long_help(
                    "Search order\n\
                     Row first or column first.\n",
                )
                .short("o")
                .long("order")
                .possible_values(&["row", "column", "automatic", "r", "c", "a"])
                .default_value("automatic")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("CHOOSE")
                .help("How to choose a state for unknown cells")
                .long_help(
                    "How to choose a state for unknown cells\n\
                     \"Smart\" means choosing a random state for cells in the first\
                     row/column,\nand dead for other cells.\n",
                )
                .short("c")
                .long("choose")
                .possible_values(&["dead", "alive", "random", "smart", "d", "a", "r", "s"])
                .default_value("dead")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("MAX")
                .help("Maximal number of living cells in the first generation")
                .long_help(
                    "Maximal number of living cells in the first generation\n\
                     If this value is set to 0, it means there is no limitation.\n",
                )
                .short("m")
                .long("max")
                .default_value("0")
                .takes_value(true)
                .validator(|d| d.parse::<u32>().map(|_| ()).map_err(|e| e.to_string())),
        )
        .arg(
            Arg::with_name("ALL")
                .help("Searches for all possible pattern")
                .long_help(
                    "Searches for all possible pattern\n\
                     Only useful when --no-tui is set.\n",
                )
                .short("a")
                .long("all"),
        )
        .arg(
            Arg::with_name("RESET")
                .help("Resets the time when starting a new search")
                .long("reset-time")
                .conflicts_with("NOTUI"),
        )
        .arg(
            Arg::with_name("NOTUI")
                .help("Starts searching immediately, without entering the TUI")
                .short("n")
                .long("no-tui"),
        );

    let matches = app.get_matches();

    let width = matches.value_of("X").unwrap().parse().unwrap();
    let height = matches.value_of("Y").unwrap().parse().unwrap();
    let period = matches.value_of("P").unwrap().parse().unwrap();
    let dimensions = (width, height, period);

    let dx = matches.value_of("DX").unwrap().parse().unwrap();
    let dy = matches.value_of("DY").unwrap().parse().unwrap();

    let symmetry = matches.value_of("SYMMETRY").unwrap().parse().unwrap();
    let all = matches.is_present("ALL");
    let reset = matches.is_present("RESET");
    let no_tui = matches.is_present("NOTUI");
    let column_first = match matches.value_of("ORDER").unwrap() {
        "row" | "r" => Some(false),
        "column" | "c" => Some(true),
        _ => None,
    };
    let new_state = match matches.value_of("CHOOSE").unwrap() {
        "dead" | "d" => Choose(Dead),
        "alive" | "a" => Choose(Alive),
        "random" | "r" => Random,
        _ => FirstRandomThenDead(0),
    };
    let max_cell_count = matches.value_of("MAX").unwrap().parse().unwrap();
    let max_cell_count = match max_cell_count {
        0 => None,
        i => Some(i),
    };

    let rule_string = &matches.value_of("RULE").unwrap();

    if let Ok(rule) = life::Life::parse_rule(rule_string) {
        let world = World::new(dimensions, dx, dy, symmetry, rule, column_first);
        let search = Box::new(Search::new(world, new_state, max_cell_count));
        Some(Args {
            search,
            all,
            no_tui,
            reset,
        })
    } else if let Ok(rule) = isotropic::Life::parse_rule(rule_string) {
        let world = World::new(dimensions, dx, dy, symmetry, rule, column_first);
        let search = Box::new(Search::new(world, new_state, max_cell_count));
        Some(Args {
            search,
            all,
            no_tui,
            reset,
        })
    } else {
        None
    }
}

pub fn search(args: Args) {
    let mut search = args.search;
    if args.no_tui {
        if args.all {
            loop {
                match search.search(None) {
                    Status::Found => println!("{}", search.display_gen(0)),
                    Status::None => break,
                    _ => (),
                }
            }
        } else if let Status::Found = search.search(None) {
            println!("{}", search.display_gen(0))
        }
    } else {
        search_with_tui(search, args.reset)
    }
}
