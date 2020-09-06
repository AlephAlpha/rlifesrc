#![allow(clippy::borrowed_box)]

use async_std::task;
use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, Show},
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand, Result as CrosstermResult,
};
use futures::{select, FutureExt, TryStreamExt};
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
    asking_quit: bool,
}

impl<'a, W: Write> App<'a, W> {
    fn new(search: Box<dyn Search>, reset: bool, output: &'a mut W) -> Self {
        let period = search.config().period;
        let world_size = (search.config().width, search.config().height);
        App {
            gen: 0,
            period,
            search,
            status: Status::Paused,
            start_time: None,
            timing: Duration::default(),
            reset,
            output,
            term_size: (80, 24),
            world_size,
            asking_quit: false,
        }
    }

    /// Initializes the screen.
    fn init(&mut self) -> CrosstermResult<()> {
        self.output.execute(EnterAlternateScreen)?.execute(Hide)?;
        terminal::enable_raw_mode()?;
        self.term_size = terminal::size()?;
        self.world_size.0 = self.world_size.0.min(self.term_size.0 as isize - 1);
        self.world_size.1 = self.world_size.1.min(self.term_size.1 as isize - 3);
        self.update()
    }

    /// Quits the program.
    fn quit(&mut self) -> CrosstermResult<()> {
        terminal::disable_raw_mode()?;
        self.output
            .execute(Show)?
            .execute(ResetColor)?
            .execute(LeaveAlternateScreen)?;
        Ok(())
    }

    /// Updates the header.
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

    /// Updates the main part of the screen.
    /// Prints the pattern in a mix of
    /// [Plaintext](https://conwaylife.com/wiki/Plaintext) and
    /// [RLE](https://conwaylife.com/wiki/Rle) format.
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

    /// Updates the footer.
    fn update_footer(&mut self) -> CrosstermResult<()> {
        const INITIAL: &str = "Press [space] to start.";
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
                    Status::Initial => INITIAL,
                    Status::Found => FOUND,
                    Status::None => NONE,
                    Status::Searching => SEARCHING,
                    Status::Paused => PAUSED,
                },
                self.term_size.0 as usize
            )))?;
        Ok(())
    }

    /// Updates the screen.
    fn update(&mut self) -> CrosstermResult<()> {
        self.update_header()?;
        self.update_main()?;
        self.update_footer()?;
        self.output.flush()?;
        Ok(())
    }

    /// Pauses.
    fn pause(&mut self) {
        self.status = Status::Paused;
        if let Some(instant) = self.start_time.take() {
            self.timing += instant.elapsed();
        }
    }

    /// Starts or resumes.
    fn start(&mut self) {
        self.status = Status::Searching;
        self.start_time = Some(Instant::now());
    }

    /// Searches for one step.
    async fn step(&mut self) {
        match self.search.search(Some(VIEW_FREQ)) {
            Status::Searching => (),
            s => {
                self.status = s;
                if let Some(instant) = self.start_time.take() {
                    self.timing += instant.elapsed();
                }
                if self.reset {
                    self.start_time = None;
                    self.timing = Duration::default();
                }
            }
        }
    }

    /// Asks whether to quit.
    fn ask_quit(&mut self) -> CrosstermResult<()> {
        const ASK_QUIT: &str = "Are you sure to quit? [Y/n]";

        self.output
            .queue(MoveTo(0, self.term_size.1 - 1))?
            .queue(SetBackgroundColor(Color::White))?
            .queue(SetForegroundColor(Color::Black))?
            .queue(Print(format!("{:1$}", ASK_QUIT, self.term_size.0 as usize)))?
            .flush()?;

        self.asking_quit = true;
        Ok(())
    }

    /// Handles a key event. Return `true` to quit the program.
    async fn handle(
        &mut self,
        event: Option<Event>,
        // reader: &mut EventStream,
        is_searching: bool,
    ) -> CrosstermResult<bool> {
        /// A macro to generate constant key events.
        macro_rules! const_key {
            ($name:ident => $key:expr) => {
                const $name: Event = Event::Key(KeyEvent {
                    code: $key,
                    modifiers: KeyModifiers::empty(),
                });
            };
            ($name:ident => upper $key:expr) => {
                const $name: Event = Event::Key(KeyEvent {
                    code: $key,
                    modifiers: KeyModifiers::SHIFT,
                });
            };
        }

        const_key!(KEY_Y => KeyCode::Char('y'));
        const_key!(KEY_UPPER_Y => upper KeyCode::Char('Y'));
        const_key!(KEY_Q => KeyCode::Char('q'));
        const_key!(KEY_UPPER_Q => upper KeyCode::Char('Q'));
        const_key!(KEY_SPACE => KeyCode::Char(' '));
        const_key!(KEY_ESC => KeyCode::Esc);
        const_key!(KEY_ENTER => KeyCode::Enter);
        const_key!(KEY_PAGEUP => KeyCode::PageUp);
        const_key!(KEY_PAGEDOWN => KeyCode::PageDown);

        if self.asking_quit {
            match event {
                Some(KEY_Y) | Some(KEY_UPPER_Y) | Some(KEY_ENTER) => return Ok(true),
                Some(Event::Resize(width, height)) => {
                    self.term_size = (width, height);
                    self.world_size.0 = self.world_size.0.min(self.term_size.0 as isize - 1);
                    self.world_size.1 = self.world_size.1.min(self.term_size.1 as isize - 3);
                    self.output
                        .queue(ResetColor)?
                        .queue(Clear(ClearType::All))?;
                    self.update()?;
                    self.ask_quit()?;
                }
                Some(_) => {
                    self.asking_quit = false;
                    self.update()?;
                }
                None => {
                    return Ok(true);
                }
            }
        } else {
            match event {
                Some(KEY_Q) | Some(KEY_UPPER_Q) | Some(KEY_ESC) => {
                    if is_searching {
                        self.pause();
                    }
                    self.update()?;
                    if let Status::Paused = self.status {
                        self.ask_quit()?;
                    } else {
                        return Ok(true);
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
                    if is_searching {
                        self.pause();
                    } else {
                        self.start();
                    }
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
                    if is_searching {
                        self.pause();
                    }
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// The main loop.
    async fn main_loop(&mut self, reader: &mut EventStream) -> CrosstermResult<()> {
        loop {
            if let Status::Searching = self.status {
                select! {
                    event = reader.try_next().fuse() => {
                        if self.handle(event?, true).await? {
                            break;
                        }
                    },
                    _ = self.step().fuse() => {
                        self.update()?;
                    },
                };
            } else if self.handle(reader.try_next().await?, false).await? {
                break;
            }
        }
        Ok(())
    }
}

/// Runs the search with a TUI.
///
/// If `reset` is true, the time will be reset when starting a new search.
pub(crate) fn tui(search: Box<dyn Search>, reset: bool) -> CrosstermResult<()> {
    let mut stdout = stdout();
    let mut reader = EventStream::new();
    let mut app = App::new(search, reset, &mut stdout);
    app.init()?;
    task::block_on(app.main_loop(&mut reader))?;
    app.quit()?;
    println!("{}", app.search.rle_gen(app.gen));
    Ok(())
}
