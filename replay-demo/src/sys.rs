use crossterm::event::{self, Event, KeyCode};
use tui::{layout::Rect, widgets::ListState};

use crate::crypto::{self, Key};

#[derive(Clone, Copy)]
pub struct Coords {
    pub x: u16,
    pub y: u16,
}

pub struct Game {
    map: Rect,
    robot: Coords,
    enemy_base: Coords,
    friend_base: Coords,

    key: Key,
}

impl Game {
    pub fn new(map: Rect) -> Self {
        let robot = Coords { x: 0, y: 0 };
        let enemy_base = Coords { x: 0, y: 1 };
        let friend_base = Coords { x: 1, y: 0 };
        let key = crypto::random_key();
        Self {
            map,
            robot,
            enemy_base,
            friend_base,
            key,
        }
    }

    pub fn map(&self) -> Rect {
        self.map
    }

    pub fn robot(&self) -> Coords {
        self.robot
    }

    pub fn enemy_base(&self) -> Coords {
        self.enemy_base
    }

    pub fn friend_base(&self) -> Coords {
        self.friend_base
    }
}

pub struct State {
    game: Game,
    screen: Rect,
    logs: Vec<String>,
    log_selected: Option<usize>,
}

impl State {
    pub fn new(game: Game, screen: Rect) -> Self {
        Self {
            game,
            screen,
            logs: vec![],
            log_selected: None,
        }
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn screen(&self) -> Rect {
        self.screen
    }

    pub fn logs(&self) -> &[String] {
        self.logs.as_ref()
    }

    pub fn push_log(&mut self, log: impl Into<String>) {
        self.logs.push(log.into());
    }

    pub fn log_selected(&self) -> Option<usize> {
        self.log_selected
    }

    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => match self.log_selected {
                    Some(index) => self.log_selected = Some(index.saturating_sub(1)),
                    None => self.log_selected = Some(self.logs.len() - 1),
                },
                KeyCode::Down => match self.log_selected {
                    Some(index) => self.log_selected = Some(index.saturating_add(1)),
                    None => self.log_selected = Some(0),
                },
                KeyCode::Esc => self.log_selected = None,
                _ => {}
            }
        }
    }
}
