use clap::{Arg, App};
use termion::{async_stdin, cursor};
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use std::io::{Write, stdout};
use crate::search::{Search, Status};
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

    let mut async_stdin = async_stdin().keys();
    let mut screen = AlternateScreen::from(stdout()).into_raw_mode().unwrap();
    let mut pause = true;
    write!(screen, "{}{}{}", Goto(1, 1), search.world, cursor::Hide).unwrap();
    screen.flush().unwrap();
    loop {
        match async_stdin.next() {
            Some(Ok(Key::Char('q'))) => break,
            Some(Ok(Key::Ctrl('c'))) => break,
            Some(Ok(Key::Char('p'))) => pause = true,
            Some(Ok(_)) => pause = false,
            _ => if !pause {
                match search.search() {
                    Status::Found => {
                        write!(screen, "{}{}", Goto(1, 1), search.world).unwrap();
                        screen.flush().unwrap();
                        pause = true;
                    },
                    Status::None => {
                        write!(screen, "{}{}", Goto(1, 1), search.world).unwrap();
                        screen.flush().unwrap();
                        pause = true;
                    },
                    Status::Searching => {
                        write!(screen, "{}{}", Goto(1, 1), search.world).unwrap();
                        screen.flush().unwrap();
                    },
                }
            },
        }
    }
    write!(screen, "{}", cursor::Show).unwrap();
}
