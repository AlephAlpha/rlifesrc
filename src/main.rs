use clap::{Arg, App};
use pancurses::{curs_set, endwin, initscr, noecho, Input, Window};
use stopwatch::Stopwatch;
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
    let mut search = Search::new(life, random);

    // 进入 TUI
    let window = initscr();
    let (win_y, win_x) = window.get_max_yx();
    let world_win = window.subwin(win_y - 2, win_x, 0, 0).unwrap();
    let status_win = window.subwin(2, win_x, win_y - 2, 0).unwrap();
    let mut gen = 0;
    let mut status = Status::Paused;
    let mut stopwatch = Stopwatch::new();
    curs_set(0);
    noecho();
    window.keypad(true);
    window.nodelay(false);
    print_world(&world_win, &search.world, gen);
    print_status(&status_win, &status, gen, &stopwatch);
    loop {
        match window.getch() {
            Some(Input::Character('q')) => break,
            Some(Input::KeyRight) | Some(Input::KeyNPage) => {
                gen = (gen + 1) % period;
                print_world(&world_win, &search.world, gen);
                print_status(&status_win, &status, gen, &stopwatch)
            },
            Some(Input::KeyLeft) | Some(Input::KeyPPage) => {
                gen = (gen + period - 1) % period;
                print_world(&world_win, &search.world, gen);
                print_status(&status_win, &status, gen, &stopwatch)
            },
            Some(Input::Character(' ')) | Some(Input::KeyEnter) => {
                match status {
                    Status::Searching => {
                        status = Status::Paused;
                        stopwatch.stop();
                        print_status(&status_win, &status, gen, &stopwatch);
                        window.nodelay(false)
                    },
                    _ => {
                        status = Status::Searching;
                        print_status(&status_win, &status, gen, &stopwatch);
                        stopwatch.start();
                        window.nodelay(true)
                    },
                }
            },
            Some(_) => 1,
            _ => {
                match search.search(Some(10000)) {
                    Status::Searching => {
                        print_world(&world_win, &search.world, gen)
                    },
                    s => {
                        status = s;
                        stopwatch.stop();
                        print_status(&status_win, &status, gen, &stopwatch);
                        stopwatch.reset();
                        print_world(&world_win, &search.world, gen);
                        window.nodelay(false)
                    },
                }
            },
        };
    }
    endwin();
}

fn print_world(window: &Window, world: &Life, gen: isize) -> i32 {
    window.mvprintw(0, 0, world.display_gen(gen));
    window.refresh()
}

fn print_status(window: &Window, status: &Status, gen: isize, stopwatch: &Stopwatch) -> i32 {
    window.erase();
    window.mvprintw(0, 0, format!("Showing generation {}. ", gen));
    match status {
        Status::Searching => 1,
        _ => window.printw(format!("Time taken: {:?}. ", stopwatch.elapsed())),
    };
    let status = match status {
        Status::Found => "Found a result. Press [space] to continue.",
        Status::None => "No more result. Press [q] to quit.",
        Status::Searching => "Searching... Press [space] to pause.",
        Status::Paused => "Paused. Press [space] to continue."
    };
    window.mvprintw(1, 0, status);
    window.refresh()
}