use clap::{Arg, App};
use pancurses::{curs_set, endwin, initscr, noecho, Input};
use crate::search::{Search, Status};
use crate::rule::Life;
mod search;
mod rule;
mod world;

fn main() {
    // 处理命令行参数
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
        .arg(Arg::with_name("RANDOM")
            .help("Searches for a random pattern")
            .conflicts_with("ALL")
            .long("random"))
        .get_matches();

    let width = matches.value_of("X").unwrap().parse().unwrap();
    let height = matches.value_of("Y").unwrap().parse().unwrap();
    let period = matches.value_of("P").unwrap().parse().unwrap();
    let dx = matches.value_of("DX").unwrap().parse().unwrap();
    let dy = matches.value_of("DY").unwrap().parse().unwrap();

    let symmetry = matches.value_of("SYMMETRY").unwrap().parse().unwrap();
    let random = matches.is_present("RANDOM");

    let rule = matches.value_of("RULE").unwrap().parse().unwrap();

    let life = Life::new(width, height, period, dx, dy, symmetry, rule);
    let mut search = Search::new(life, random, Some(10000));

    // 进入 TUI
    let window = initscr();
    curs_set(0);
    noecho();
    window.nodelay(false);
    window.printw(&format!("{}", search.world));
    window.refresh();
    loop {
        match window.getch() {
            Some(Input::Character('q')) => break,
            Some(Input::Character('p')) => window.nodelay(false),
            Some(_) => window.nodelay(true),
            _ => {
                match search.search() {
                    Status::Found => {
                        window.mv(0, 0);
                        window.printw(&format!("{}", search.world));
                        window.refresh();
                        window.nodelay(false)
                    },
                    Status::None => {
                        window.mv(0, 0);
                        window.printw(&format!("{}", search.world));
                        window.refresh();
                        window.nodelay(false)
                    },
                    Status::Searching => {
                        window.mv(0, 0);
                        window.printw(&format!("{}", search.world));
                        window.refresh()
                    },
                }
            },
        };
    }
    endwin();
}
