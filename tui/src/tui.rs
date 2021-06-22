use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, Show},
    event::{Event, EventStream, KeyCode, KeyEvent},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
    ExecutableCommand, QueueableCommand, Result as CrosstermResult,
};
use futures_util::{future, select_biased, FutureExt, TryStreamExt};
use rlifesrc_lib::{PolyWorld, State, Status, ALIVE, DEAD};
use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

#[cfg(debug_assertions)]
const VIEW_FREQ: u64 = 5000;
#[cfg(not(debug_assertions))]
const VIEW_FREQ: u64 = 100000;

/// Different modes to handle events.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Searching or paused.
    Main,
    /// Asking for quit.
    AskingQuit,
}

struct App<'a, W: Write> {
    gen: i32,
    period: i32,
    world: PolyWorld,
    status: Status,
    paused: bool,
    start_time: Option<Instant>,
    timing: Duration,
    reset: bool,
    output: &'a mut W,
    term_size: (u16, u16),
    world_size: (i32, i32),
    mode: Mode,
}

impl<'a, W: Write> App<'a, W> {
    fn new(world: PolyWorld, reset: bool, output: &'a mut W) -> CrosstermResult<Self> {
        let period = world.config().period;
        let mut app = App {
            gen: 0,
            period,
            world,
            status: Status::Initial,
            paused: true,
            start_time: None,
            timing: Duration::default(),
            reset,
            output,
            term_size: (80, 24),
            world_size: (80, 24),
            mode: Mode::Main,
        };
        app.init()?;
        Ok(app)
    }

    /// Initializes the screen.
    fn init(&mut self) -> CrosstermResult<()> {
        self.output
            .execute(EnterAlternateScreen)?
            .execute(Hide)?
            .execute(SetTitle("rlifesrc"))?;
        terminal::enable_raw_mode()?;
        self.term_size = terminal::size()?;
        self.world_size.0 = self.world.config().width.min(self.term_size.0 as i32 - 1);
        self.world_size.1 = self.world.config().height.min(self.term_size.1 as i32 - 3);
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
                    self.world.cell_count_gen(self.gen),
                    self.world.conflicts(),
                    if !self.paused {
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
                self.world.config().width,
                self.world.config().height,
                self.world.config().rule_string
            )))?
            .queue(MoveToNextLine(1))?;
        for y in 0..self.world_size.1 {
            let mut line = String::new();
            for x in 0..self.world_size.0 {
                let state = self.world.get_cell_state((x, y, self.gen));
                match state {
                    Some(DEAD) => line.push('.'),
                    Some(ALIVE) => {
                        if self.world.is_gen_rule() {
                            line.push('A')
                        } else {
                            line.push('o')
                        }
                    }
                    Some(State(i)) => line.push((b'A' + i as u8 - 1) as char),
                    _ => line.push('?'),
                };
            }
            if y == self.world.config().height - 1 {
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
                    Status::Searching =>
                        if self.paused {
                            PAUSED
                        } else {
                            SEARCHING
                        },
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
        self.paused = true;
        if let Some(instant) = self.start_time.take() {
            self.timing += instant.elapsed();
        }
    }

    /// Starts or resumes.
    fn start(&mut self) {
        self.paused = false;
        self.start_time = Some(Instant::now());
    }

    /// Searches for one step.
    fn step(&mut self) {
        match self.world.search(Some(VIEW_FREQ)) {
            Status::Searching => (),
            s => {
                self.status = s;
                self.paused = true;
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

        self.mode = Mode::AskingQuit;
        Ok(())
    }

    /// Handles a key event. Returns `true` to quit the program.
    fn handle(&mut self, event: Option<Event>) -> CrosstermResult<bool> {
        /// A macro to generate key event patterns.
        macro_rules! key_event {
            ($( $code:pat )|+) => {
                $(
                    Some(Event::Key(KeyEvent { code: $code , .. }))
                )|+
            };
        }

        match self.mode {
            Mode::Main => match event {
                key_event!(KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc) => {
                    if !self.paused {
                        self.pause();
                    }
                    self.update()?;
                    if self.status == Status::Searching {
                        self.ask_quit()?;
                    } else {
                        return Ok(true);
                    }
                }
                key_event!(KeyCode::PageDown) => {
                    self.gen = (self.gen + 1) % self.period;
                    self.update()?;
                }
                key_event!(KeyCode::PageUp) => {
                    self.gen = (self.gen + self.period - 1) % self.period;
                    self.update()?;
                }
                key_event!(KeyCode::Char(' ') | KeyCode::Enter) => {
                    if !self.paused {
                        self.pause();
                    } else {
                        self.start();
                    }
                    self.update()?;
                }
                Some(Event::Resize(width, height)) => {
                    self.term_size = (width, height);
                    self.world_size.0 = self.world.config().width.min(self.term_size.0 as i32 - 1);
                    self.world_size.1 = self.world.config().height.min(self.term_size.1 as i32 - 3);
                    self.output
                        .queue(ResetColor)?
                        .queue(Clear(ClearType::All))?;
                    self.update()?;
                }
                Some(_) => (),
                None => {
                    if !self.paused {
                        self.pause();
                    }
                    return Ok(true);
                }
            },
            Mode::AskingQuit => match event {
                key_event!(KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter) => {
                    return Ok(true)
                }
                Some(Event::Resize(width, height)) => {
                    self.term_size = (width, height);
                    self.world_size.0 = self.world_size.0.min(self.term_size.0 as i32 - 1);
                    self.world_size.1 = self.world_size.1.min(self.term_size.1 as i32 - 3);
                    self.output
                        .queue(ResetColor)?
                        .queue(Clear(ClearType::All))?;
                    self.update()?;
                    self.ask_quit()?;
                }
                Some(_) => {
                    self.mode = Mode::Main;
                    self.update()?;
                }
                None => {
                    return Ok(true);
                }
            },
        }

        Ok(false)
    }

    /// The main loop.
    async fn main_loop(&mut self, reader: &mut EventStream) -> CrosstermResult<()> {
        loop {
            if !self.paused {
                select_biased! {
                    event = reader.try_next().fuse() => {
                        if self.handle(event?)? {
                            break;
                        }
                    },
                    _ = future::lazy(|_| self.step()) => {
                        self.update()?;
                    },
                };
            } else if self.handle(reader.try_next().await?)? {
                break;
            }
        }
        Ok(())
    }
}

impl<'a, W: Write> Drop for App<'a, W> {
    fn drop(&mut self) {
        self.quit().unwrap()
    }
}

/// Runs the search with a TUI.
///
/// If `reset` is true, the time will be reset when starting a new search.
pub fn tui(world: PolyWorld, reset: bool) -> CrosstermResult<()> {
    let mut stdout = stdout();
    let mut reader = EventStream::new();
    let result;
    {
        let mut app = App::new(world, reset, &mut stdout)?;
        futures_executor::block_on(app.main_loop(&mut reader))?;
        result = app.world.rle_gen(app.gen);
    }
    println!("{}", result);
    Ok(())
}
