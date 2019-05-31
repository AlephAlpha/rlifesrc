use clap::{Arg, App};
use clap::AppSettings::AllowNegativeNumbers;
#[cfg(feature = "tui")]
use pancurses::{curs_set, endwin, initscr, noecho, resize_term, Input, Window};
#[cfg(feature = "tui")]
use stopwatch::Stopwatch;
use crate::search::{Search, Status};
use crate::rule::{Life, NbhdDesc, Rule};
use crate::world::State::{Dead, Alive};
mod search;
mod rule;
mod world;

fn is_positive(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit()) && s != "0" && !s.starts_with("-")
}

fn main() {
    let mut app = App::new("rlifesrc")
        .version("0.1.0")
        .setting(AllowNegativeNumbers)
        .arg(Arg::with_name("X")
            .help("Width of the pattern")
            .required(true)
            .index(1)
            .validator(|x| if is_positive(&x) {
                Ok(())
            } else {
                Err(String::from("width must be a positive integer"))
            }))
        .arg(Arg::with_name("Y")
            .help("Height of the pattern")
            .required(true)
            .index(2)
            .validator(|y| if is_positive(&y) {
                Ok(())
            } else {
                Err(String::from("height must be a positive integer"))
            }))
        .arg(Arg::with_name("P")
            .help("Period of the pattern")
            .default_value("1")
            .index(3)
            .validator(|p| if is_positive(&p) {
                Ok(())
            } else {
                Err(String::from("period must be a positive integer"))
            }))
        .arg(Arg::with_name("DX")
            .help("Horizontal translation")
            .default_value("0")
            .index(4)
            .validator(|d| d.parse().map(|_ : isize| ()).map_err(|e| e.to_string())))
        .arg(Arg::with_name("DY")
            .help("Vertical translation")
            .default_value("0")
            .index(5)
            .validator(|d| d.parse().map(|_ : isize| ()).map_err(|e| e.to_string())))
        .arg(Arg::with_name("SYMMETRY")
            .help("Symmetry of the pattern")
            .long_help("Symmetry of the pattern\n\
                You may need to add quotation marks for some of the symmetries.\n\
                The usages of these symmetries are the same as Oscar Cunningham's \
                Logic Life Search.\nSee http://conwaylife.com/wiki/Symmetry.\n")
            .short("s")
            .long("symmetry")
            .possible_values(&["C1","C2","C4","D2|","D2-","D2\\","D2/","D4+","D4X","D8"])
            .default_value("C1")
            .takes_value(true))
        .arg(Arg::with_name("RULE")
            .help("Rule of the cellular automaton")
            .long_help("Rule of the cellular automaton\n\
                Currently, only Life-like rules are supported.\n")
            .short("r")
            .long("rule")
            .default_value("B3/S23")
            .takes_value(true)
            .validator(|d| d.parse().map(|_ : Rule| ())))
        .arg(Arg::with_name("CHOOSE")
            .help("How to choose a state for unknown cells")
            .short("c")
            .long("choose")
            .possible_values(&["dead", "alive", "random", "d", "a", "r"])
            .default_value("dead")
            .takes_value(true))
        .arg(Arg::with_name("ORDER")
            .help("Search order")
            .long_help("Search order\n\
                Row first or column first.")
            .short("o")
            .long("order")
            .possible_values(&["row", "column", "automatic", "r", "c", "a"])
            .default_value("automatic")
            .takes_value(true));

    #[cfg(not(feature = "tui"))]
    {
        app = app.arg(Arg::with_name("ALL")
            .help("Searches for all possible pattern")
            .short("a")
            .long("all"));
    }

    #[cfg(feature = "tui")]
    {
        app = app.arg(Arg::with_name("ALL")
            .help("Searches for all possible pattern")
            .long_help("Searches for all possible pattern\n\
                Only useful when --no-tui is set.")
            .short("a")
            .long("all"))
        .arg(Arg::with_name("RESET")
            .help("Resets the time when starting a new search")
            .long("reset-time")
            .conflicts_with("NOTUI"))
        .arg(Arg::with_name("NOTUI")
            .help("Starts searching immediately, without entering the TUI")
            .short("n")
            .long("no-tui"));
    }


    let matches = app.get_matches();

    let width = matches.value_of("X").unwrap().parse().unwrap();
    let height = matches.value_of("Y").unwrap().parse().unwrap();
    let period = matches.value_of("P").unwrap().parse().unwrap();
    let dx = matches.value_of("DX").unwrap().parse().unwrap();
    let dy = matches.value_of("DY").unwrap().parse().unwrap();

    let symmetry = matches.value_of("SYMMETRY").unwrap().parse().unwrap();
    let rule = matches.value_of("RULE").unwrap().parse().unwrap();
    let all = matches.is_present("ALL");
    let column_first = match matches.value_of("ORDER").unwrap() {
        "row" | "r" => Some(false),
        "column" | "c" => Some(true),
        _ => None,
    };

    let life = Life::new(width, height, period, dx, dy, symmetry, rule, column_first);
    let new_state = match matches.value_of("CHOOSE").unwrap() {
        "dead" | "d" => Some(Dead),
        "alive" | "a" => Some(Alive),
        _ => None,
    };
    let mut search = Search::new(life, new_state);

    #[cfg(not(feature = "tui"))]
    {
        search_without_tui(&mut search, all)
    }

    #[cfg(feature = "tui")]
    {
        let reset = matches.is_present("RESET");
        let notui = matches.is_present("NOTUI");

        if notui {
            search_without_tui(&mut search, all)
        } else {
            search_with_tui(&mut search, reset)
        }
    }
}

fn search_without_tui(search: &mut Search<Life, NbhdDesc>, all: bool) {
    if all {
        loop {
            match search.search(None) {
                Status::Found => println!("{}", search.world.display_gen(0)),
                Status::None => break,
                _ => (),
            }
        }
    } else {
        match search.search(None) {
            Status::Found => println!("{}", search.world.display_gen(0)),
            _ => (),
        }
    }
}

#[cfg(feature = "tui")]
fn search_with_tui(search: &mut Search<Life, NbhdDesc>, reset: bool) {
    let period = search.world.period;
    let window = initscr();
    let (win_y, win_x) = window.get_max_yx();
    let mut world_win = window.subwin(win_y - 2, win_x, 0, 0).unwrap();
    let mut status_bar = window.subwin(2, win_x, win_y - 2, 0).unwrap();
    let mut gen = 0;
    let mut status = Status::Paused;
    let mut stopwatch = Stopwatch::new();
    curs_set(0);
    noecho();
    window.keypad(true);
    window.nodelay(false);
    print_world(&world_win, &search.world, gen);
    print_status(&status_bar, &status, gen, &stopwatch);

    loop {
        match window.getch() {
            Some(Input::Character('q')) => {
                match status {
                    Status::Searching | Status::Paused => {
                        status = Status::Paused;
                        stopwatch.stop();
                        window.nodelay(false);
                        print_world(&world_win, &search.world, gen);
                        status_bar.mvprintw(1, 0, "Are you sure to quit? [Y/n]");
                        status_bar.clrtoeol();
                        status_bar.refresh();
                        match window.getch() {
                            Some(Input::Character('Y')) => break,
                            Some(Input::Character('y')) => break,
                            Some(Input::Character('\n')) => break,
                            _ => print_status(&status_bar, &status, gen, &stopwatch),
                        }
                    },
                    _ => break,
                }
            },
            Some(Input::KeyRight) | Some(Input::KeyNPage) => {
                gen = (gen + 1) % period;
                print_world(&world_win, &search.world, gen);
                print_status(&status_bar, &status, gen, &stopwatch)
            },
            Some(Input::KeyLeft) | Some(Input::KeyPPage) => {
                gen = (gen + period - 1) % period;
                print_world(&world_win, &search.world, gen);
                print_status(&status_bar, &status, gen, &stopwatch)
            },
            Some(Input::Character(' ')) | Some(Input::Character('\n')) => {
                match status {
                    Status::Searching => {
                        status = Status::Paused;
                        stopwatch.stop();
                        window.nodelay(false);
                        print_world(&world_win, &search.world, gen);
                        print_status(&status_bar, &status, gen, &stopwatch)
                    },
                    _ => {
                        status = Status::Searching;
                        print_status(&status_bar, &status, gen, &stopwatch);
                        stopwatch.start();
                        window.nodelay(true)
                    },
                }
            },
            Some(Input::KeyResize) => {
                resize_term(0, 0);
                let (win_y, win_x) = window.get_max_yx();
                world_win = window.subwin(win_y - 2, win_x, 0, 0).unwrap();
                status_bar = window.subwin(2, win_x, win_y - 2, 0).unwrap();
                world_win.erase();
                print_world(&world_win, &search.world, gen);
                print_status(&status_bar, &status, gen, &stopwatch)
            },
            None => {
                match search.search(Some(25000)) {
                    Status::Searching => {
                        print_world(&world_win, &search.world, gen)
                    },
                    s => {
                        status = s;
                        stopwatch.stop();
                        window.nodelay(false);
                        print_status(&status_bar, &status, gen, &stopwatch);
                        if reset {
                            stopwatch.reset();
                        }
                        print_world(&world_win, &search.world, gen)
                    },
                }
            },
            _ => 1,
        };
    }
    endwin();
}

#[cfg(feature = "tui")]
fn print_world(window: &Window, world: &Life, gen: isize) -> i32 {
    window.mvprintw(0, 0, world.display_gen(gen));
    window.refresh()
}

#[cfg(feature = "tui")]
fn print_status(window: &Window, status: &Status, gen: isize, stopwatch: &Stopwatch) -> i32 {
    window.erase();
    window.mvprintw(0, 0, format!("Showing generation {}. ", gen));
    match status {
        Status::Searching => 1,
        _ => window.printw(format!("Time taken: {:?}. ", stopwatch.elapsed())),
    };
    let status = match status {
        Status::Found => "Found a result. Press [space] to search for the next.",
        Status::None => "No more result. Press [q] to quit.",
        Status::Searching => "Searching... Press [space] to pause.",
        Status::Paused => "Paused. Press [space] to resume."
    };
    window.mvprintw(1, 0, status);
    window.refresh()
}
