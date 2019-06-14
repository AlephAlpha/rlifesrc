use crate::search::{Status, TraitSearch};
use pancurses::{curs_set, endwin, initscr, noecho, resize_term, Input, Window};
use stopwatch::Stopwatch;

pub fn search_with_tui(mut search: Box<dyn TraitSearch>, reset: bool) {
    let period = search.period();
    #[cfg(debug_assertions)]
    let view_freq = 500;
    #[cfg(not(debug_assertions))]
    let view_freq = 25000;
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
    print_world(&world_win, &search, gen);
    print_status(&status_bar, status, gen, &stopwatch);

    loop {
        match window.getch() {
            Some(Input::Character('q')) => match status {
                Status::Searching | Status::Paused => {
                    status = Status::Paused;
                    stopwatch.stop();
                    window.nodelay(false);
                    print_world(&world_win, &search, gen);
                    status_bar.mvprintw(1, 0, "Are you sure to quit? [Y/n]");
                    status_bar.clrtoeol();
                    status_bar.refresh();
                    match window.getch() {
                        Some(Input::Character('Y')) => break,
                        Some(Input::Character('y')) => break,
                        Some(Input::Character('\n')) => break,
                        _ => print_status(&status_bar, status, gen, &stopwatch),
                    }
                }
                _ => break,
            },
            Some(Input::KeyRight) | Some(Input::KeyNPage) => {
                gen = (gen + 1) % period;
                print_world(&world_win, &search, gen);
                print_status(&status_bar, status, gen, &stopwatch)
            }
            Some(Input::KeyLeft) | Some(Input::KeyPPage) => {
                gen = (gen + period - 1) % period;
                print_world(&world_win, &search, gen);
                print_status(&status_bar, status, gen, &stopwatch)
            }
            Some(Input::Character(' ')) | Some(Input::Character('\n')) => match status {
                Status::Searching => {
                    status = Status::Paused;
                    stopwatch.stop();
                    window.nodelay(false);
                    print_world(&world_win, &search, gen);
                    print_status(&status_bar, status, gen, &stopwatch)
                }
                _ => {
                    status = Status::Searching;
                    print_status(&status_bar, status, gen, &stopwatch);
                    stopwatch.start();
                    window.nodelay(true)
                }
            },
            Some(Input::KeyResize) => {
                resize_term(0, 0);
                let (win_y, win_x) = window.get_max_yx();
                world_win = window.subwin(win_y - 2, win_x, 0, 0).unwrap();
                status_bar = window.subwin(2, win_x, win_y - 2, 0).unwrap();
                world_win.erase();
                print_world(&world_win, &search, gen);
                print_status(&status_bar, status, gen, &stopwatch)
            }
            None => match search.search(Some(view_freq)) {
                Status::Searching => print_world(&world_win, &search, gen),
                s => {
                    status = s;
                    stopwatch.stop();
                    window.nodelay(false);
                    print_status(&status_bar, status, gen, &stopwatch);
                    if reset {
                        stopwatch.reset();
                    }
                    print_world(&world_win, &search, gen)
                }
            },
            _ => 1,
        };
    }
    endwin();
}

#[allow(clippy::borrowed_box)]
fn print_world(window: &Window, search: &Box<dyn TraitSearch>, gen: isize) -> i32 {
    window.mvprintw(0, 0, search.display_gen(gen));
    window.refresh()
}

fn print_status(window: &Window, status: Status, gen: isize, stopwatch: &Stopwatch) -> i32 {
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
        Status::Paused => "Paused. Press [space] to resume.",
    };
    window.mvprintw(1, 0, status);
    window.refresh()
}
