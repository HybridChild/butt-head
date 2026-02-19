use std::io::{self, Write};
use std::time::{Duration, Instant};

use butt_head::{ButtHead, Config, ServiceTiming, TimeDuration, TimeInstant};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal;

// --- Time wrappers ---

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct StdDuration(Duration);

impl TimeDuration for StdDuration {
    const ZERO: Self = StdDuration(Duration::ZERO);

    fn as_millis(&self) -> u64 {
        self.0.as_millis() as u64
    }

    fn from_millis(millis: u64) -> Self {
        StdDuration(Duration::from_millis(millis))
    }

    fn saturating_sub(self, other: Self) -> Self {
        StdDuration(self.0.saturating_sub(other.0))
    }
}

#[derive(Copy, Clone)]
struct StdInstant(Instant);

impl TimeInstant for StdInstant {
    type Duration = StdDuration;

    fn duration_since(&self, earlier: Self) -> StdDuration {
        StdDuration(self.0.duration_since(earlier.0))
    }

    fn checked_add(self, duration: StdDuration) -> Option<Self> {
        self.0.checked_add(duration.0).map(StdInstant)
    }

    fn checked_sub(self, duration: StdDuration) -> Option<Self> {
        self.0.checked_sub(duration.0).map(StdInstant)
    }
}

// --- Config ---

static CONFIG: Config<StdDuration> = Config {
    active_low: false,
    click_timeout: StdDuration(Duration::from_millis(120)),
    hold_delay: StdDuration(Duration::from_millis(500)),
    hold_interval: StdDuration(Duration::from_millis(300)),
};

// --- Main ---

fn run() -> io::Result<()> {
    let mut stdout = io::stdout();

    execute!(stdout, EnableMouseCapture)?;

    println!("butt-head native example\r");
    println!("─────────────────────────────────────────\r");
    println!("  Left mouse button  →  button input\r");
    println!("  Q or ESC           →  quit\r");
    println!("─────────────────────────────────────────\r\n\r");

    let mut button = ButtHead::<StdInstant>::new(&CONFIG);
    let mut is_pressed = false;
    let mut result = button.update(false, StdInstant(Instant::now()));

    loop {
        if let Some(ev) = result.event {
            println!("{ev:?}\r");
            stdout.flush()?;
        }

        let timeout = match result.next_service {
            ServiceTiming::Immediate => Some(Duration::ZERO),
            ServiceTiming::Delay(d) => Some(d.0),
            ServiceTiming::Idle => None,
        };

        let input = if let Some(d) = timeout {
            if event::poll(d)? {
                Some(event::read()?)
            } else {
                None
            }
        } else {
            Some(event::read()?)
        };

        match input {
            Some(Event::Key(key)) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => break,
                _ => {}
            },
            Some(Event::Mouse(mouse)) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => is_pressed = true,
                MouseEventKind::Up(MouseButton::Left) => is_pressed = false,
                _ => {}
            },
            _ => {}
        }

        result = button.update(is_pressed, StdInstant(Instant::now()));
    }

    execute!(stdout, DisableMouseCapture)?;
    println!("\r\nBye!\r");

    Ok(())
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let result = run();
    terminal::disable_raw_mode().ok();
    result
}
