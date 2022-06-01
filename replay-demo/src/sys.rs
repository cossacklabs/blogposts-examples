use crossterm::event::{KeyCode, KeyEvent};
use rand::{distributions::Uniform, Rng};
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
        let mut rng = rand::thread_rng();
        let x = Uniform::new(0, map.width - 1);
        let y = Uniform::new(0, map.height - 1);
        let robot = Coords {
            x: rng.sample(&x),
            y: rng.sample(&y),
        };
        let enemy_base = Coords {
            x: rng.sample(&x),
            y: rng.sample(&y),
        };
        let friend_base = Coords {
            x: rng.sample(&x),
            y: rng.sample(&y),
        };
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

    pub fn input_encrypted(&mut self, packet: &[u8]) -> anyhow::Result<()> {
        let ignoring_packet = "ignoring the packet";
        let decrypted = match crypto::open(&self.key, packet) {
            Ok(ok) => ok,
            Err(_) => anyhow::bail!(ignoring_packet),
        };

        match decrypted.as_slice() {
            b"u" => self.robot.y = self.robot.y.saturating_sub(1),
            b"d" => self.robot.y = u16::min(self.robot.y + 1, self.map.height),
            b"l" => self.robot.x = self.robot.x.saturating_sub(1),
            b"r" => self.robot.x = u16::min(self.robot.x + 1, self.map.width),
            _ => anyhow::bail!(ignoring_packet),
        }

        Ok(())
    }

    pub fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        crypto::seal(&self.key, data)
    }
}

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
