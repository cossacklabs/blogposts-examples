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

pub struct State {
    game: Game,
    screen: Rect,
}

impl State {
    pub fn new(game: Game, screen: Rect) -> Self {
        Self { game, screen }
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn screen(&self) -> Rect {
        self.screen
    }
}
