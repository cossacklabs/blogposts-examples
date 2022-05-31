use std::io::{self, Stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::Rect,
};

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
    let state = sys::State::new(game, screen);

    terminal.draw(|frame| ui::draw_state(frame, &state))?;

    loop {
        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }
        }
    }
}
