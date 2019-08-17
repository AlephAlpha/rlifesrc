#![allow(clippy::borrowed_box)]

#[cfg(debug_assertions)]
const VIEW_FREQ: u32 = 1000;
#[cfg(not(debug_assertions))]
const VIEW_FREQ: u32 = 50000;

use pancurses::{ColorPair, Input, Window};
use rlifesrc_lib::{Status, TraitSearch};
use std::time::{Duration, Instant};

struct SearchWindow {
    gen: isize,
    status: Status,
    start_time: Option<Instant>,
    timing: Duration,

    window: Window,
    top_bar: Window,
    bottom_bar: Window,
    world_win: Window,
}

impl SearchWindow {
    fn new() -> Self {
        let gen = 0;
        let status = Status::Paused;
        let start_time: Option<Instant> = None;
        let timing = Default::default();
        let window = pancurses::initscr();
        let (win_y, win_x) = window.get_max_yx();
        let top_bar = window.subwin(1, win_x, 0, 0).unwrap();
        let bottom_bar = window.subwin(1, win_x, win_y - 1, 0).unwrap();
        let world_win = window.subwin(win_y - 2, win_x, 1, 0).unwrap();

        pancurses::start_color();
        pancurses::init_pair(1, pancurses::COLOR_BLACK, pancurses::COLOR_WHITE);
        top_bar.bkgdset(ColorPair(1));
        bottom_bar.bkgdset(ColorPair(1));
        pancurses::curs_set(0);
        pancurses::noecho();
        window.keypad(true);
        window.nodelay(false);

        SearchWindow {
            gen,
            status,
            start_time,
            timing,
            window,
            top_bar,
            bottom_bar,
            world_win,
        }
    }

    fn update(&self, search: &Box<dyn TraitSearch>) {
        self.world_win.erase();
        self.world_win.mvprintw(0, 0, search.display_gen(self.gen));
        self.world_win.refresh();
        self.top_bar.erase();
        self.top_bar.mvprintw(0, 0, format!("Gen: {}", self.gen));
        self.top_bar
            .printw(format!("  Cells: {}", search.cell_count()));
        match self.status {
            Status::Searching => 1,
            _ => self.top_bar.printw(format!("  Time: {:.2?}", self.timing)),
        };
        self.top_bar.refresh();
        let status_str = match self.status {
            Status::Found => "Found a result. Press [q] to quit or [space] to search for the next.",
            Status::None => "No more result. Press [q] to quit.",
            Status::Searching => "Searching... Press [space] to pause.",
            Status::Paused => "Paused. Press [space] to resume.",
        };
        self.bottom_bar.erase();
        self.bottom_bar.mvprintw(0, 0, status_str);
        self.bottom_bar.refresh();
    }

    fn resize(&mut self) {
        pancurses::resize_term(0, 0);
        let (win_y, win_x) = self.window.get_max_yx();
        self.top_bar = self.window.subwin(1, win_x, 0, 0).unwrap();
        self.bottom_bar = self.window.subwin(1, win_x, win_y - 1, 0).unwrap();
        self.world_win = self.window.subwin(win_y - 2, win_x, 1, 0).unwrap();
    }

    fn quit(&self) -> bool {
        self.window.nodelay(false);
        self.bottom_bar.erase();
        self.bottom_bar
            .mvprintw(0, 0, "Are you sure to quit? [Y/n]");
        self.bottom_bar.refresh();
        match self.window.getch() {
            Some(Input::Character('Y'))
            | Some(Input::Character('y'))
            | Some(Input::Character('\n')) => true,
            _ => false,
        }
    }

    fn pause(&mut self) {
        self.status = Status::Paused;
        if let Some(instant) = self.start_time.take() {
            self.timing += instant.elapsed();
        }
        self.window.nodelay(false);
    }

    fn start(&mut self) {
        self.status = Status::Searching;
        self.start_time = Some(Instant::now());
        self.window.nodelay(true);
    }
}

pub fn search_with_tui(mut search: Box<dyn TraitSearch>, reset: bool) {
    let period = search.period();
    let mut search_win = SearchWindow::new();
    search_win.update(&search);

    loop {
        match search_win.window.getch() {
            Some(Input::Character('q')) => match search_win.status {
                Status::Searching | Status::Paused => {
                    search_win.pause();
                    search_win.update(&search);
                    if search_win.quit() {
                        break;
                    } else {
                        search_win.update(&search);
                    }
                }
                _ => break,
            },
            Some(Input::KeyRight) | Some(Input::KeyNPage) => {
                search_win.gen = (search_win.gen + 1) % period;
                search_win.update(&search);
            }
            Some(Input::KeyLeft) | Some(Input::KeyPPage) => {
                search_win.gen = (search_win.gen + period - 1) % period;
                search_win.update(&search);
            }
            Some(Input::Character(' ')) | Some(Input::Character('\n')) | Some(Input::KeyEnter) => {
                match search_win.status {
                    Status::Searching => {
                        search_win.pause();
                        search_win.update(&search);
                    }
                    _ => {
                        search_win.start();
                        search_win.update(&search);
                    }
                }
            }
            Some(Input::KeyResize) => {
                search_win.resize();
                search_win.update(&search);
            }
            None => match search.search(Some(VIEW_FREQ)) {
                Status::Searching => search_win.update(&search),
                s => {
                    search_win.pause();
                    search_win.status = s;
                    search_win.update(&search);
                    if reset {
                        search_win.start_time = None;
                        search_win.timing = Default::default();
                    }
                }
            },
            _ => (),
        };
    }
    pancurses::endwin();
    println!("{}", search.display_gen(search_win.gen));
}
