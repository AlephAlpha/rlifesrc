#![allow(clippy::borrowed_box)]

use async_std::task;
use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, Show},
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand, Result as CrosstermResult,
};
use futures::{poll, task::Poll, TryStreamExt};
use rlifesrc_lib::{Search, State, Status, ALIVE, DEAD};
use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

#[cfg(debug_assertions)]
const VIEW_FREQ: u64 = 5000;
#[cfg(not(debug_assertions))]
const VIEW_FREQ: u64 = 100000;

struct App<'a, W: Write> {
    gen: isize,
    period: isize,
    search: Box<dyn Search>,
    status: Status,
    start_time: Option<Instant>,
    timing: Duration,
    reset: bool,
    output: &'a mut W,
    term_size: (u16, u16),
    world_size: (isize, isize),
}

impl<'a, W: Write> App<'a, W> {
    fn new(search: Box<dyn Search>, reset: bool, output: &'a mut W) -> Self {
        let period = search.config().period;
        let status = Status::Paused;
        let start_time: Option<Instant> = None;
        let timing = Duration::default();
        let world_size = (search.config().width, search.config().height);
        App {
            gen: 0,
            period,
            search,
            status,
            start_time,
            timing,
            reset,
            output,
            term_size: (80, 24),
            world_size,
        }
    }

    fn init(&mut self) -> CrosstermResult<()> {
        self.output.execute(EnterAlternateScreen)?.execute(Hide)?;
        terminal::enable_raw_mode()?;
        self.term_size = terminal::size()?;
        self.world_size.0 = self.world_size.0.min(self.term_size.0 as isize - 1);
        self.world_size.1 = self.world_size.1.min(self.term_size.1 as isize - 3);
        self.update()
    }

    fn quit(&mut self) -> CrosstermResult<()> {
        terminal::disable_raw_mode()?;
        self.output.execute(Show)?.execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn update_header(&mut self) -> CrosstermResult<()> {
        self.output
            .queue(MoveTo(0, 0))?
            .queue(SetBackgroundColor(Color::White))?
            .queue(SetForegroundColor(Color::Black))?
            .queue(Print(format!(
                "{:1$}",
                format!(
                    "Gen: {}  Cells: {}  Confl: {}{}",
                    self.gen,
                    self.search.cell_count_gen(self.gen),
                    self.search.conflicts(),
                    if self.status == Status::Searching {
                        String::new()
                    } else {
                        format!("  Time: {:.2?}", self.timing)
                    }
                ),
                self.term_size.0 as usize
            )))?;
        Ok(())
    }

    fn update_main(&mut self) -> CrosstermResult<()> {
        self.output
            .queue(MoveTo(0, 1))?
            .queue(ResetColor)?
            .queue(Print(format!(
                "x = {}, y = {}, rule = {}",
                self.search.config().width,
                self.search.config().height,
                self.search.config().rule_string
            )))?
            .queue(MoveToNextLine(1))?;
        for y in 0..self.world_size.1 {
            let mut line = String::new();
            for x in 0..self.world_size.0 {
                let state = self.search.get_cell_state((x, y, self.gen)).unwrap();
                match state {
                    Some(DEAD) => line.push('.'),
                    Some(ALIVE) => {
                        if self.search.is_gen_rule() {
                            line.push('A')
                        } else {
                            line.push('o')
                        }
                    }
                    Some(State(i)) => line.push((b'A' + i as u8 - 1) as char),
                    _ => line.push('?'),
                };
            }
            if y == self.search.config().height - 1 {
                line.push('!')
            } else {
                line.push('$')
            };
            self.output.queue(Print(line))?.queue(MoveToNextLine(1))?;
        }
        Ok(())
    }

    fn update_footer(&mut self) -> CrosstermResult<()> {
        const FOUND: &str = "Found a result. Press [q] to quit or [space] to search for the next.";
        const NONE: &str = "No more result. Press [q] to quit.";
        const SEARCHING: &str = "Searching... Press [space] to pause.";
        const PAUSED: &str = "Paused. Press [space] to resume.";

        self.output
            .queue(MoveTo(0, self.term_size.1 - 1))?
            .queue(SetBackgroundColor(Color::White))?
            .queue(SetForegroundColor(Color::Black))?
            .queue(Print(format!(
                "{:1$}",
                match self.status {
                    Status::Found => FOUND,
                    Status::None => NONE,
                    Status::Searching => SEARCHING,
                    Status::Paused => PAUSED,
                },
                self.term_size.0 as usize
            )))?;
        Ok(())
    }

    fn update(&mut self) -> CrosstermResult<()> {
        self.update_header()?;
        self.update_main()?;
        self.update_footer()?;
        self.output.flush()?;
        Ok(())
    }

    fn step(&mut self) {
        match self.search.search(Some(VIEW_FREQ)) {
            Status::Searching => (),
            s => {
                self.status = s;
                self.pause();
                if self.reset {
                    self.start_time = None;
                    self.timing = Duration::default();
                }
            }
        }
    }

    fn pause(&mut self) {
        if let Some(instant) = self.start_time.take() {
            self.timing += instant.elapsed();
        }
    }

    fn start(&mut self) {
        self.status = Status::Searching;
        self.start_time = Some(Instant::now());
    }

    async fn main_loop(&mut self) -> CrosstermResult<()> {
        macro_rules! const_key {
            ($($name:ident => $key:expr),* $(,)?) => {
                $(
                    const $name: Event = Event::Key(KeyEvent {
                        code: $key,
                        modifiers: KeyModifiers::empty(),
                    });
                )*
            };
        }
        const_key! {
            KEY_Q => KeyCode::Char('q'),
            KEY_ESC => KeyCode::Esc,
            KEY_PAGEUP => KeyCode::PageUp,
            KEY_PAGEDOWN => KeyCode::PageDown,
            KEY_SPACE => KeyCode::Char(' '),
            KEY_ENTER => KeyCode::Enter,
            KEY_Y => KeyCode::Char('y'),
            KEY_UPPER_Y => KeyCode::Char('Y'),
        };

        let mut reader = EventStream::new();
        loop {
            if let Status::Searching = self.status {
                let poll_event = poll!(reader.try_next())?;
                if let Poll::Ready(maybe_event) = poll_event {
                    match maybe_event {
                        Some(KEY_Q) | Some(KEY_ESC) => {
                            self.status = Status::Paused;
                            self.pause();
                            self.update()?;
                            self.output
                                .queue(MoveTo(0, self.term_size.1 - 1))?
                                .queue(SetBackgroundColor(Color::White))?
                                .queue(SetForegroundColor(Color::Black))?
                                .queue(Print(format!(
                                    "{:1$}",
                                    "Are you sure to quit? [Y/n]", self.term_size.0 as usize
                                )))?
                                .flush()?;
                            if let Some(KEY_Y) | Some(KEY_UPPER_Y) | Some(KEY_ENTER) =
                                reader.try_next().await?
                            {
                                break;
                            } else {
                                self.update()?;
                            }
                        }
                        Some(KEY_PAGEDOWN) => {
                            self.gen = (self.gen + 1) % self.period;
                            self.update()?;
                        }
                        Some(KEY_PAGEUP) => {
                            self.gen = (self.gen + self.period - 1) % self.period;
                            self.update()?;
                        }
                        Some(KEY_SPACE) | Some(KEY_ENTER) => {
                            self.status = Status::Paused;
                            self.pause();
                            self.update()?;
                        }
                        Some(Event::Resize(width, height)) => {
                            self.term_size = (width, height);
                            self.world_size.0 =
                                self.world_size.0.min(self.term_size.0 as isize - 1);
                            self.world_size.1 =
                                self.world_size.1.min(self.term_size.1 as isize - 3);
                            self.output
                                .queue(ResetColor)?
                                .queue(Clear(ClearType::All))?;
                            self.update()?;
                        }
                        Some(_) => (),
                        None => {
                            self.status = Status::Paused;
                            self.pause();
                            self.update()?;
                            break;
                        }
                    }
                }
                self.step();
                self.update()?;
            } else {
                let maybe_event = reader.try_next().await?;
                match maybe_event {
                    Some(KEY_Q) | Some(KEY_ESC) => {
                        self.pause();
                        self.update()?;
                        if let Status::Paused = self.status {
                            self.output
                                .queue(MoveTo(0, self.term_size.1 - 1))?
                                .queue(SetBackgroundColor(Color::White))?
                                .queue(SetForegroundColor(Color::Black))?
                                .queue(Print(format!(
                                    "{:1$}",
                                    "Are you sure to quit? [Y/n]", self.term_size.0 as usize
                                )))?
                                .flush()?;
                            if let Some(KEY_Y) | Some(KEY_UPPER_Y) | Some(KEY_ENTER) =
                                reader.try_next().await?
                            {
                                break;
                            } else {
                                self.update()?;
                            }
                        } else {
                            break;
                        }
                    }
                    Some(KEY_PAGEDOWN) => {
                        self.gen = (self.gen + 1) % self.period;
                        self.update()?;
                    }
                    Some(KEY_PAGEUP) => {
                        self.gen = (self.gen + self.period - 1) % self.period;
                        self.update()?;
                    }
                    Some(KEY_SPACE) | Some(KEY_ENTER) => {
                        self.start();
                        self.update()?;
                    }
                    Some(Event::Resize(width, height)) => {
                        self.term_size = (width, height);
                        self.world_size.0 = self.world_size.0.min(self.term_size.0 as isize - 1);
                        self.world_size.1 = self.world_size.1.min(self.term_size.1 as isize - 3);
                        self.output
                            .queue(ResetColor)?
                            .queue(Clear(ClearType::All))?;
                        self.update()?;
                    }
                    Some(_) => (),
                    None => {
                        self.pause();
                        self.update()?;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn tui(search: Box<dyn Search>, reset: bool) -> CrosstermResult<()> {
    let mut stdout = stdout();
    let mut app = App::new(search, reset, &mut stdout);
    app.init()?;
    task::block_on(app.main_loop())?;
    app.quit()?;
    println!("{}", app.search.rle_gen(app.gen));
    Ok(())
}
