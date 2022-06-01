use crossterm::event::{KeyCode, KeyEvent};

use crate::game::Game;

#[derive(Default)]
pub struct InputField {
    buff: Vec<char>,
    cursor: usize,
    history: Vec<Vec<char>>,
    history_index: Option<usize>,
}

impl InputField {
    fn take_line(&mut self) -> String {
        let line = self.line();
        let line_vec = std::mem::take(&mut self.buff);
        let line = line.trim().to_string();
        if !line.is_empty() {
            self.history.push(line_vec);
        }
        line
    }

    fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        match key.code {
            KeyCode::Enter => {
                self.cursor = 0;
                self.history_index = None;
                let line = self.take_line();
                if line.is_empty() {
                    return None;
                } else {
                    return Some(line);
                }
            }
            KeyCode::Char(c) => {
                self.buff.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => {
                let last_char = self.buff.len();
                self.cursor = usize::min(self.cursor + 1, last_char);
            }
            KeyCode::Backspace => {
                self.cursor = self.cursor.saturating_sub(1);
                if self.cursor < self.buff.len() {
                    self.buff.remove(self.cursor);
                }
            }
            KeyCode::Up => {
                let index = if let Some(index) = self.history_index {
                    index.saturating_sub(1)
                } else if self.history.is_empty() {
                    return None;
                } else {
                    let index = self.history.len() - 1;
                    self.take_line();
                    index
                };
                self.history_index = Some(index);
                self.buff = self.history[index].clone();
                self.cursor = self.buff.len();
            }

            KeyCode::Down => {
                if let Some(index) = self.history_index {
                    if index == self.history.len() - 1 {
                        return None;
                    }
                    let index = index + 1;

                    self.buff = self.history[index].clone();
                    self.cursor = self.buff.len();
                    self.history_index = Some(index);
                }
            }

            _ => {}
        }
        None
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn line(&self) -> String {
        self.buff.iter().collect()
    }
}

#[derive(Clone, Copy)]
pub enum Focus {
    None,
    Input,
}

pub struct State {
    pub game: Game,
    logs: Vec<String>,
    log_selected: Option<usize>,
    focus: Focus,
    pub input: InputField,
}

impl State {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            logs: vec![],
            log_selected: None,
            focus: Focus::None,
            input: InputField::default(),
        }
    }

    pub fn game(&self) -> &Game {
        &self.game
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

    pub fn handle_command(&mut self, line: &str) {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        if parts.len() != 2 || parts[0] != "send" {
            self.push_log("ERROR: supported syntax is `send <hex>`");
            return;
        }
        let packet = match hex::decode(parts[1]) {
            Ok(ok) => ok,
            Err(err) => {
                self.push_log(format!("ERROR: {}", err));
                return;
            }
        };

        self.send(&packet);
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
            KeyCode::Home => self.log_selected = None,
            _ => {}
        }

        match self.focus {
            Focus::None => {
                if let KeyCode::Char('i') = key.code {
                    self.focus = Focus::Input
                }
            }
            Focus::Input => {
                if let KeyCode::Esc = key.code {
                    self.focus = Focus::None
                } else if let Some(cmd) = self.input.handle_key(key) {
                    self.handle_command(&cmd);
                }
            }
        }
    }

    pub fn log_selected(&self) -> Option<usize> {
        self.log_selected
    }

    pub fn send(&mut self, packet: &[u8]) {
        self.push_log(format!("INTERCEPTED: {}", hex::encode(packet)));
        if let Err(err) = self.game.input_encrypted(packet) {
            self.push_log(format!("ERROR: {}", err))
        }
    }
}
