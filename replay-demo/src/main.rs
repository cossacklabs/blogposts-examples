use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use tui::backend::CrosstermBackend;

mod crypto;
mod sys;
pub mod ui;

pub type Terminal = tui::Terminal<CrosstermBackend<Stdout>>;
pub type Frame<'a> = tui::Frame<'a, CrosstermBackend<Stdout>>;

fn main() -> anyhow::Result<()> {
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

fn run_app(terminal: &mut Terminal) -> anyhow::Result<()> {
    let screen = terminal.size()?;
    let (map, _, _) = ui::split_screen(screen);

    let game = sys::Game::new(map);
    let mut state = sys::State::new(game, screen);

    loop {
        terminal.draw(|frame| ui::draw_state(frame, &mut state))?;

        if event::poll(Duration::from_secs(1))? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
                state.handle_event(event);
            }
        }

        state.push_log(format!(
            "Intercepted {:x}",
            rand::thread_rng().gen::<u128>()
        ));
    }
}
