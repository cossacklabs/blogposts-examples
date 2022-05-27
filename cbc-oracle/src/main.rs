pub mod machine;
pub mod ui;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use machine::{DecryptingMachine, Oracle};

use std::{error::Error, io, time::Duration};

use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::machine::State;

// Encrypted by the `hex` gang (⌐■_■)
const SECRET: &str = "546865206d6f737420616d617a696e672073797374656d7320627265616b20696e20746865206d6f737420616d617a696e67207761792e";

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res.map_err(Into::into)
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> anyhow::Result<()> {
    let oracle = Oracle::new();

    let ciphertext = oracle.encrypt(&hex::decode(SECRET).expect("should be ok"));

    let mut machine = DecryptingMachine::new(oracle, ciphertext);

    // Timeout between redraws
    let mut timeout = Duration::from_millis(200);

    const TIMEOUT_STEP: Duration = Duration::from_millis(10);

    let mut advance = false;
    let mut run_without_pause = false;
    loop {
        terminal.draw(|frame| ui::ui(frame, &machine, timeout, advance))?;

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('c') => advance = true,
                    KeyCode::Char('r') => run_without_pause = !run_without_pause,
                    KeyCode::Left => timeout = timeout.saturating_sub(TIMEOUT_STEP),
                    KeyCode::Right => timeout = timeout.saturating_add(TIMEOUT_STEP),
                    _ => {}
                }
            }
        }

        if advance {
            machine.advance();
        }

        if machine.state == State::Finished {
            // Turn off so it will not redraw endlessly
            run_without_pause = false;
        }

        advance = run_without_pause || machine.state == State::IteratingByte;
    }
}
