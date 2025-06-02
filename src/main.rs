use minifb::{Key, Window, WindowOptions};
use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const DEFAULT_WIDTH: u16 = 30;
const DEFAULT_HEIGHT: u16 = 30;
const CELL_SIZE: usize = 10;

#[derive(Copy, Clone)]
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
        let mut rng = thread_rng();
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
                (center_x as i16 + i * dx).clamp(1, (width - 2) as i16) as u16,
                (center_y as i16 + i * dy).clamp(1, (height - 2) as i16) as u16
            ))
            .collect();
        let fish = Self::random_pos(&eel, width, height);
        Self { eel, dir, fish, over: false, width, height }
    }

    fn new() -> Self {
        Self::new_with_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }

    fn random_pos(eel: &VecDeque<(u16, u16)>, width: u16, height: u16) -> (u16, u16) {
        let mut rng = thread_rng();
        loop {
            let p = (rng.gen_range(1..width-1), rng.gen_range(1..height-1));
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
            // ignore reverse direction
        } else {
            self.dir = new_dir;
        }
    }
}

#[derive(Copy, Clone)]
enum Color {
    Black,
    Green,
    Red,
    Yellow,
}

fn color_to_u32(color: Color) -> u32 {
    match color {
        Color::Black => 0x000000,
        Color::Green => 0x00FF00,
        Color::Red => 0xFF0000,
        Color::Yellow => 0xFFFF00,
    }
}

fn draw_window(game: &Game, buffer: &mut [u32], width: usize, _height: usize) {
    let field_w = game.width as usize;
    let field_h = game.height as usize;
    for y in 0..field_h {
        for x in 0..field_w {
            let color = if (y == 0 || y == field_h - 1) && (x == 0 || x == field_w - 1) {
                Color::Green
            } else if y == 0 || y == field_h - 1 {
                Color::Green
            } else if x == 0 || x == field_w - 1 {
                Color::Green
            } else if game.eel.front() == Some(&(x as u16, y as u16)) {
                Color::Yellow
            } else if game.eel.contains(&(x as u16, y as u16)) {
                Color::Green
            } else if game.fish == (x as u16, y as u16) {
                Color::Red
            } else {
                Color::Black
            };
            let value = color_to_u32(color);
            let px = x * CELL_SIZE;
            let py = y * CELL_SIZE;
            for dy in 0..CELL_SIZE {
                for dx in 0..CELL_SIZE {
                    let idx = (py + dy) * width + (px + dx);
                    if idx < buffer.len() {
                        buffer[idx] = value;
                    }
                }
            }
        }
    }
    // Draw GAME OVER message if needed
    if game.over {
        let msg = "GAME OVER";
        let msg_len = msg.len() as usize;
        let char_w = 8; // rough width in pixels per char
        let char_h = 16; // rough height in pixels per char
        let x0 = (width / 2).saturating_sub((msg_len * char_w) / 2);
        let y0 = (field_h * CELL_SIZE) / 2 - char_h / 2;
        // Draw a simple blocky message (white rectangle for each char)
        for (i, _c) in msg.chars().enumerate() {
            let cx = x0 + i * char_w;
            for dy in 0..char_h {
                for dx in 0..char_w {
                    let idx = (y0 + dy) * width + (cx + dx);
                    if idx < buffer.len() {
                        buffer[idx] = 0xFFFFFF;
                    }
                }
            }
        }
    }
}

fn main() {
    let field_width = DEFAULT_WIDTH as usize;
    let field_height = DEFAULT_HEIGHT as usize;
    let win_width = field_width * CELL_SIZE;
    let win_height = field_height * CELL_SIZE;
    let mut window = Window::new(
        "Eelgame",
        win_width,
        win_height,
        WindowOptions::default(),
    ).unwrap();
    let mut buffer = vec![0; win_width * win_height];
    let mut game = Game::new();
    let mut last_tick = Instant::now();
    let mut paused = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Pause toggle
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            paused = !paused;
        }
        // Input
        let mut fast = false;
        let mut direction = None;
        if !game.over {
            if window.is_key_down(Key::Up) {
                direction = Some(Direction::Up);
                if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
                    fast = true;
                }
            } else if window.is_key_down(Key::Down) {
                direction = Some(Direction::Down);
                if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
                    fast = true;
                }
            } else if window.is_key_down(Key::Left) {
                direction = Some(Direction::Left);
                if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
                    fast = true;
                }
            } else if window.is_key_down(Key::Right) {
                direction = Some(Direction::Right);
                if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
                    fast = true;
                }
            }
            if let Some(dir) = direction {
                game.change_dir(dir);
            }
        }
        // Update
        if !paused && !game.over && last_tick.elapsed() >= Duration::from_millis(150) {
            game.update();
            if fast {
                game.update(); // move twice if shift is held
            }
            last_tick = Instant::now();
        }
        // Draw
        draw_window(&game, &mut buffer, win_width, win_height);
        window.update_with_buffer(&buffer, win_width, win_height).unwrap();
    }
}

