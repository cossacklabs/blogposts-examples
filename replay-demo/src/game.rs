use rand::Rng;
use tui::layout::{Margin, Rect};

use crate::{
    crypto::{self, Key},
    ui,
};

const NEXT_TARGET_RADIUS: u16 = 10;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Coords {
    pub x: u16,
    pub y: u16,
}

pub struct Game {
    map: Rect,
    inner_map: Rect,
    robot: Coords,
    robot_target: Coords,
    base: Coords,

    key: Key,
}

impl Game {
    pub fn new(map: Rect) -> Self {
        let inner_map = ui::inner(map);

        let mut rng = rand::thread_rng();
        let robot = Coords {
            x: rng.gen_range(0..inner_map.width),
            y: rng.gen_range(0..inner_map.height),
        };

        let friend_base = loop {
            let coords = Coords {
                x: rng.gen_range(0..inner_map.width),
                y: rng.gen_range(0..inner_map.height),
            };
            if coords != robot {
                break coords;
            }
        };
        let key = crypto::random_key();
        Self {
            map,
            inner_map,
            robot,
            base: friend_base,
            key,
            robot_target: robot,
        }
    }

    pub fn robot(&self) -> Coords {
        self.robot
    }

    pub fn base(&self) -> Coords {
        self.base
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

    fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        crypto::seal(&self.key, data)
    }

    pub fn tick_enemy(&mut self) -> anyhow::Result<Vec<u8>> {
        if self.robot == self.robot_target {
            self.robot_target = random_in_radius(self.robot, NEXT_TARGET_RADIUS, self.inner_map);
        }

        let (robot, target) = (self.robot, self.robot_target);
        let x_diff = u16::abs_diff(robot.x, target.x);
        let y_diff = u16::abs_diff(robot.y, target.y);

        let direction = if x_diff > y_diff {
            if robot.x < target.x {
                "r"
            } else {
                "l"
            }
        } else if robot.y < target.y {
            "d"
        } else {
            "u"
        };

        self.encrypt(direction.as_bytes())
    }

    pub fn is_finished(&self) -> bool {
        self.robot == self.base
    }

    pub fn map(&self) -> Rect {
        self.map
    }
}

fn random_in_radius(center: Coords, radius: u16, map: Rect) -> Coords {
    let mut rng = rand::thread_rng();
    let left = center.x.saturating_sub(radius);
    let right = u16::min(center.x + radius, map.width);
    let up = center.y.saturating_sub(radius);
    let bottom = u16::min(center.y + radius, map.height);

    loop {
        let x: u16 = rng.gen_range(left..right);
        let y: u16 = rng.gen_range(up..bottom);
        let x_diff = x as i32 - center.x as i32;
        let y_diff = y as i32 - center.y as i32;
        if x_diff * x_diff + y_diff * y_diff < radius as i32 * radius as i32 {
            return Coords { x, y };
        }
    }
}
