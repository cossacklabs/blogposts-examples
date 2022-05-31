use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use tui::layout::Rect;

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

#[derive(Default)]
pub struct InputField {
    pub buff: Vec<char>,
    pub index: usize,
}

#[derive(Clone, Copy)]
pub enum Focus {
    None,
    Input,
}

pub struct State {
    game: Game,
    screen: Rect,
    logs: Vec<String>,
    log_selected: Option<usize>,
    focus: Focus,
    pub input: InputField,
}

impl State {
    pub fn new(game: Game, screen: Rect) -> Self {
        Self {
            game,
            screen,
            logs: vec![],
            log_selected: None,
            focus: Focus::None,
            input: InputField::default(),
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

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let last_log = self.logs.len().saturating_sub(1);
        match key.code {
            KeyCode::PageUp => {
                self.log_selected = Some(if let Some(index) = self.log_selected {
                    index.saturating_sub(1)
                } else {
                    last_log
                })
            }
            KeyCode::PageDown => {
                self.log_selected = Some(if let Some(index) = self.log_selected {
                    usize::min(index + 1, last_log)
                } else {
                    0
                })
            }
            _ => {}
        }

        match self.focus {
            Focus::None => {
                if let KeyCode::Char('i') = key.code {
                    self.focus = Focus::Input
                }
            }
            Focus::Input => match key.code {
                KeyCode::Enter => {
                    let line: String = std::mem::take(&mut self.input.buff).into_iter().collect();
                    self.input.index = 0;
                    self.push_log(format!("received {:?}", line))
                }
                KeyCode::Char(c) => {
                    self.input.buff.insert(self.input.index, c);
                    self.input.index += 1;
                }
                KeyCode::Left => self.input.index = self.input.index.saturating_sub(1),
                KeyCode::Right => {
                    let last_char = self.input.buff.len();
                    self.input.index = usize::min(self.input.index + 1, last_char);
                }
                KeyCode::Backspace => {
                    self.input.index = self.input.index.saturating_sub(1);
                    if self.input.index < self.input.buff.len() {
                        self.input.buff.remove(self.input.index);
                    }
                }
                KeyCode::Esc => self.focus = Focus::None,

                _ => {}
            },
        }
    }

    pub fn log_selected(&self) -> Option<usize> {
        self.log_selected
    }
}
