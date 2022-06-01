use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use sys::Focus;
use tui::backend::CrosstermBackend;

mod crypto;
mod game;
mod sys;
mod ui;

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
    let map = ui::map_size(screen);

    let game = game::Game::new(map);
    let mut state = sys::State::new(game);

    let tick_rate = Duration::from_secs(3);

    let mut last_tick = Instant::now();
    let timeout = tick_rate
        .checked_sub(last_tick.elapsed())
        .unwrap_or_else(|| Duration::from_secs(0));

    loop {
        terminal.draw(|frame| ui::draw_state(frame, &mut state))?;

        if state.game().is_finished() {
            loop {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('q') = key.code {
                        return Ok(());
                    }
                }
            }
        }

        if event::poll(timeout)? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
                state.handle_key(key);
            }
        }
        if matches!(state.focus(), Focus::None) && last_tick.elapsed() >= tick_rate {
            let packet = state.game.tick_enemy()?;
            state.push_log(format!("EAVESDROPPED: {}", hex::encode(&packet)));
            state.send(&packet);
            last_tick = Instant::now();
        }
    }
}
