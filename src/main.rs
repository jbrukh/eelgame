use macroquad::prelude::*;
use ::rand::prelude::*;
use std::collections::VecDeque;

const BOARD_WIDTH: u16 = 30;
const BOARD_HEIGHT: u16 = 30;
const CELL_SIZE: f32 = 20.0;
const METRICS_HEIGHT: f32 = 40.0;
const BASE_INTERVAL: f32 = 0.15; // seconds

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Game {
    eel: VecDeque<(u16, u16)>,
    dir: Direction,
    fish: (u16, u16),
    over: bool,
    width: u16,
    height: u16,
}

impl Game {
    fn new_with_size(width: u16, height: u16) -> Self {
        let mut rng = ::rand::thread_rng();
        let dir = match rng.gen_range(0..4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        };
        let (dx, dy) = match dir {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (1, 0),
            Direction::Right => (-1, 0),
        };
        let center_x = width / 2;
        let center_y = height / 2;
        let eel: VecDeque<_> = (0..3)
            .map(|i| (
                (center_x as i16 + i * dx).clamp(0, (width - 1) as i16) as u16,
                (center_y as i16 + i * dy).clamp(0, (height - 1) as i16) as u16
            ))
            .collect();
        let fish = Self::random_pos(&eel, width, height);
        Self { eel, dir, fish, over: false, width, height }
    }

    fn new() -> Self {
        Self::new_with_size(BOARD_WIDTH, BOARD_HEIGHT)
    }

    fn random_pos(eel: &VecDeque<(u16, u16)>, width: u16, height: u16) -> (u16, u16) {
        let mut rng = ::rand::thread_rng();
        loop {
            let p = (rng.gen_range(0..width), rng.gen_range(0..height));
            if !eel.contains(&p) {
                return p;
            }
        }
    }

    fn update(&mut self) {
        if self.over {
            return;
        }
        let (mut x, mut y) = *self.eel.front().unwrap();
        match self.dir {
            Direction::Up => y = y.wrapping_sub(1),
            Direction::Down => y = y.wrapping_add(1),
            Direction::Left => x = x.wrapping_sub(1),
            Direction::Right => x = x.wrapping_add(1),
        }
        // Wrap around the field
        x = (x + self.width) % self.width;
        y = (y + self.height) % self.height;
        if self.eel.contains(&(x, y)) {
            self.over = true;
            return;
        }
        self.eel.push_front((x, y));
        if (x, y) == self.fish {
            self.fish = Self::random_pos(&self.eel, self.width, self.height);
        } else {
            self.eel.pop_back();
        }
    }

    fn change_dir(&mut self, new_dir: Direction) {
        use Direction::*;
        if matches!((self.dir, new_dir), (Up, Down) | (Down, Up) | (Left, Right) | (Right, Left)) {
            // Ignore opposite direction
        } else {
            self.dir = new_dir;
        }
    }
}

#[macroquad::main("Eelgame")]
async fn main() {
    let win_width = BOARD_WIDTH as f32 * CELL_SIZE;
    let win_height = BOARD_HEIGHT as f32 * CELL_SIZE + METRICS_HEIGHT;
    request_new_screen_size(win_width, win_height);
    let mut game = Game::new();
    let mut paused = false;
    let mut speed: u8 = 1;
    let mut last_update = get_time();
    loop {
        clear_background(BLACK);
        // Draw thin border
        draw_rectangle_lines(
            0.0,
            0.0,
            BOARD_WIDTH as f32 * CELL_SIZE,
            BOARD_HEIGHT as f32 * CELL_SIZE,
            2.0,
            GREEN,
        );
        // Draw eel
        for (i, &(x, y)) in game.eel.iter().enumerate() {
            let color = if i == 0 { YELLOW } else { GREEN };
            draw_rectangle(
                x as f32 * CELL_SIZE,
                y as f32 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
                color,
            );
        }
        // Draw fish
        let (fx, fy) = game.fish;
        draw_rectangle(
            fx as f32 * CELL_SIZE,
            fy as f32 * CELL_SIZE,
            CELL_SIZE,
            CELL_SIZE,
            RED,
        );
        // Draw metrics below the field (always on top)
        draw_rectangle(
            0.0,
            BOARD_HEIGHT as f32 * CELL_SIZE,
            win_width,
            METRICS_HEIGHT,
            BLACK,
        );
        draw_text(
            &format!("Speed: {}", speed),
            10.0,
            BOARD_HEIGHT as f32 * CELL_SIZE + 28.0,
            28.0,
            WHITE,
        );
        if paused && !game.over {
            draw_text("PAUSED", 120.0, 60.0, 48.0, WHITE);
        }
        if game.over {
            let msg = "GAME OVER";
            let text_dim = measure_text(msg, None, 80, 1.0);
            draw_text(
                msg,
                (game.width as f32 * CELL_SIZE - text_dim.width) / 2.0,
                (game.height as f32 * CELL_SIZE) / 2.0,
                80.0,
                WHITE,
            );
            draw_text(
                "Press SPACE to restart",
                60.0,
                (game.height as f32 * CELL_SIZE) / 2.0 + 60.0,
                32.0,
                WHITE,
            );
        }
        // Controls
        if is_key_pressed(KeyCode::Space) {
            if game.over {
                game = Game::new();
                paused = false;
                speed = 1;
            } else {
                paused = !paused;
            }
        }
        for n in 1..=9 {
            let key = match n {
                1 => KeyCode::Key1,
                2 => KeyCode::Key2,
                3 => KeyCode::Key3,
                4 => KeyCode::Key4,
                5 => KeyCode::Key5,
                6 => KeyCode::Key6,
                7 => KeyCode::Key7,
                8 => KeyCode::Key8,
                9 => KeyCode::Key9,
                _ => continue,
            };
            if is_key_pressed(key) {
                speed = n;
            }
        }
        let mut new_dir = None;
        let mut fast = false;
        if !game.over {
            if is_key_down(KeyCode::Up) {
                new_dir = Some(Direction::Up);
                if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    fast = true;
                }
            } else if is_key_down(KeyCode::Down) {
                new_dir = Some(Direction::Down);
                if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    fast = true;
                }
            } else if is_key_down(KeyCode::Left) {
                new_dir = Some(Direction::Left);
                if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    fast = true;
                }
            } else if is_key_down(KeyCode::Right) {
                new_dir = Some(Direction::Right);
                if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    fast = true;
                }
            }
            if let Some(dir) = new_dir {
                game.change_dir(dir);
            }
        }
        // Speed logic: 1 = 150ms, 9 = 75ms
        let interval = BASE_INTERVAL / (1.0 + (speed as f32 - 1.0) / 8.0);
        if !paused && !game.over && get_time() - last_update >= interval as f64 {
            game.update();
            if fast {
                game.update();
            }
            last_update = get_time();
        }
        next_frame().await;
    }
}

