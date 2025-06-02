use macroquad::prelude::*;
use ::rand::prelude::*;
use std::collections::VecDeque;

const DEFAULT_BOARD_WIDTH: u16 = 30;
const DEFAULT_BOARD_HEIGHT: u16 = 30;
const CELL_SIZE: f32 = 20.0;
const METRICS_HEIGHT: f32 = 60.0;
const EXTRA_PADDING: f32 = 20.0;
const BASE_INTERVAL: f32 = 0.15; // seconds

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone)]
enum FoodType {
    Red,    // +1
    Blue,   // +2
    Yellow, // +3
    Purple, // +4
    Orange, // +5
}

impl FoodType {
    fn color(self) -> Color {
        match self {
            FoodType::Red => RED,
            FoodType::Blue => BLUE,
            FoodType::Yellow => YELLOW,
            FoodType::Purple => PURPLE,
            FoodType::Orange => ORANGE,
        }
    }
    fn length(self) -> usize {
        match self {
            FoodType::Red => 1,
            FoodType::Blue => 2,
            FoodType::Yellow => 3,
            FoodType::Purple => 4,
            FoodType::Orange => 5,
        }
    }
    fn random(rng: &mut ThreadRng) -> Self {
        match rng.gen_range(0..5) {
            0 => FoodType::Red,
            1 => FoodType::Blue,
            2 => FoodType::Yellow,
            3 => FoodType::Purple,
            _ => FoodType::Orange,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum GameState {
    Splash,
    Options,
    Playing,
}

struct Game {
    eel: VecDeque<(u16, u16)>,
    dir: Direction,
    food: (u16, u16, FoodType),
    over: bool,
    width: u16,
    height: u16,
    grow: usize,
    eel_gradient: (Color, Color), // (head, tail)
}

impl Game {
    fn new_with_size(width: u16, height: u16, rng: &mut ThreadRng) -> Self {
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
        let food = Self::random_food(&eel, width, height, rng);
        let (head, tail) = random_eel_gradient(rng);
        Self {
            eel,
            dir,
            food,
            over: false,
            width,
            height,
            grow: 0,
            eel_gradient: (head, tail),
        }
    }
    fn new(rng: &mut ThreadRng, width: u16, height: u16) -> Self {
        Self::new_with_size(width, height, rng)
    }
    fn random_food(eel: &VecDeque<(u16, u16)>, width: u16, height: u16, rng: &mut ThreadRng) -> (u16, u16, FoodType) {
        loop {
            let p = (rng.gen_range(0..width), rng.gen_range(0..height));
            if !eel.contains(&p) {
                return (p.0, p.1, FoodType::random(rng));
            }
        }
    }
    fn update(&mut self, rng: &mut ThreadRng) {
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
        if (x, y) == (self.food.0, self.food.1) {
            self.grow += self.food.2.length();
            let new_head = self.food.2.color();
            let prev_tail = self.eel_gradient.1;
            self.eel_gradient = (new_head, prev_tail);
            self.food = Self::random_food(&self.eel, self.width, self.height, rng);
        } else if self.grow > 0 {
            self.grow -= 1;
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

fn color_fade(color: Color, fade: f32) -> Color {
    Color::new(
        color.r * (1.0 - fade) + fade,
        color.g * (1.0 - fade) + fade,
        color.b * (1.0 - fade) + fade,
        color.a,
    )
}

fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn random_eel_gradient(rng: &mut ThreadRng) -> (Color, Color) {
    let base = match rng.gen_range(0..5) {
        0 => GREEN,
        1 => BLUE,
        2 => BROWN,
        3 => DARKGREEN,
        _ => DARKBLUE,
    };
    let tail = color_fade(base, 0.3);
    (base, tail)
}

const BROWN: Color = Color::new(0.5, 0.3, 0.1, 1.0);
const DARKGREEN: Color = Color::new(0.1, 0.3, 0.1, 1.0);
const DARKBLUE: Color = Color::new(0.1, 0.1, 0.3, 1.0);

#[macroquad::main("Eelgame")]
async fn main() {
    let mut rng = ::rand::thread_rng();
    let mut state = GameState::Splash;
    let mut board_width = DEFAULT_BOARD_WIDTH;
    let mut board_height = DEFAULT_BOARD_HEIGHT;
    let mut game = Game::new(&mut rng, board_width, board_height);
    let mut paused = false;
    let mut speed: u8 = 1;
    let mut last_update = get_time();
    loop {
        let win_width = board_width as f32 * CELL_SIZE;
        let win_height = board_height as f32 * CELL_SIZE + METRICS_HEIGHT + EXTRA_PADDING;
        request_new_screen_size(win_width, win_height);
        clear_background(BLACK);
        match state {
            GameState::Splash => {
                // Draw eel (arc) and sushi (circle + bands)
                let cx = win_width / 2.0;
                let cy = win_height / 2.0 - 40.0;
                // Eel body
                for i in 0..8 {
                    let t = i as f32 / 7.0;
                    let angle = std::f32::consts::PI * (0.7 + 0.6 * t);
                    let x = cx + angle.cos() * 80.0 * (1.0 - t * 0.2);
                    let y = cy + angle.sin() * 80.0 * (1.0 - t * 0.2);
                    let color = color_fade(GREEN, 1.0 - t * 0.7);
                    draw_circle(x, y, 18.0 - t * 4.0, color);
                }
                // Eel head
                draw_circle(cx + 80.0 * 0.7, cy, 20.0, YELLOW);
                // Sushi
                let sushi_x = cx + 120.0;
                let sushi_y = cy + 20.0;
                draw_rectangle(sushi_x - 18.0, sushi_y - 12.0, 36.0, 24.0, WHITE);
                draw_rectangle(sushi_x - 18.0, sushi_y - 4.0, 36.0, 8.0, RED);
                draw_rectangle(sushi_x - 18.0, sushi_y + 4.0, 36.0, 4.0, ORANGE);
                draw_rectangle(sushi_x - 18.0, sushi_y - 12.0, 36.0, 4.0, GREEN);
                draw_text("EELGAME", cx - 120.0, cy - 80.0, 64.0, WHITE);
                draw_text("Press SPACE to start", cx - 140.0, cy + 100.0, 32.0, WHITE);
                if is_key_pressed(KeyCode::Space) {
                    state = GameState::Options;
                }
            }
            GameState::Options => {
                draw_text("OPTIONS", 40.0, 60.0, 48.0, WHITE);
                draw_text(&format!("Board Width:  {}", board_width), 60.0, 120.0, 32.0, WHITE);
                draw_text(&format!("Board Height: {}", board_height), 60.0, 160.0, 32.0, WHITE);
                draw_text("Use arrow keys to change. Press SPACE to start!", 60.0, 220.0, 28.0, GRAY);
                if is_key_pressed(KeyCode::Left) && board_width > 10 {
                    board_width -= 1;
                }
                if is_key_pressed(KeyCode::Right) && board_width < 60 {
                    board_width += 1;
                }
                if is_key_pressed(KeyCode::Up) && board_height < 60 {
                    board_height += 1;
                }
                if is_key_pressed(KeyCode::Down) && board_height > 10 {
                    board_height -= 1;
                }
                if is_key_pressed(KeyCode::Space) {
                    game = Game::new(&mut rng, board_width, board_height);
                    paused = false;
                    speed = 1;
                    last_update = get_time();
                    state = GameState::Playing;
                }
            }
            GameState::Playing => {
                // Draw thin border
                draw_rectangle_lines(
                    0.0,
                    0.0,
                    board_width as f32 * CELL_SIZE,
                    board_height as f32 * CELL_SIZE,
                    2.0,
                    GREEN,
                );
                // Draw eel as a solid tube with rounded turns
                let len = game.eel.len().max(1) as f32;
                let seg_radius = CELL_SIZE * 0.48;
                let seg_width = seg_radius * 2.0;
                for i in 0..game.eel.len() {
                    let t = i as f32 / (len - 1.0).max(1.0);
                    let color = color_lerp(game.eel_gradient.0, game.eel_gradient.1, t);
                    let (x, y) = game.eel[i];
                    let cx = x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                    let cy = y as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                    // Draw line to next segment (if any)
                    if i + 1 < game.eel.len() {
                        let (nx, ny) = game.eel[i + 1];
                        let ncx = nx as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                        let ncy = ny as f32 * CELL_SIZE + CELL_SIZE / 2.0;
                        draw_line(cx, cy, ncx, ncy, seg_width, color);
                    }
                    // Draw circle at center for rounded ends
                    draw_circle(cx, cy, seg_radius, color);
                }
                // Draw food
                let (fx, fy, food_type) = game.food;
                draw_rectangle(
                    fx as f32 * CELL_SIZE,
                    fy as f32 * CELL_SIZE,
                    CELL_SIZE,
                    CELL_SIZE,
                    food_type.color(),
                );
                // Draw metrics below the field
                draw_rectangle(
                    0.0,
                    board_height as f32 * CELL_SIZE,
                    win_width,
                    METRICS_HEIGHT + EXTRA_PADDING,
                    BLACK,
                );
                draw_text(
                    &format!("Speed: {}", speed),
                    10.0,
                    board_height as f32 * CELL_SIZE + 28.0,
                    28.0,
                    WHITE,
                );
                draw_text(
                    &format!("Length: {}", game.eel.len()),
                    160.0,
                    board_height as f32 * CELL_SIZE + 28.0,
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
                        (board_width as f32 * CELL_SIZE - text_dim.width) / 2.0,
                        (board_height as f32 * CELL_SIZE) / 2.0,
                        80.0,
                        WHITE,
                    );
                    draw_text(
                        "Press SPACE to restart",
                        60.0,
                        (board_height as f32 * CELL_SIZE) / 2.0 + 60.0,
                        32.0,
                        WHITE,
                    );
                }
                // Controls
                if is_key_pressed(KeyCode::Space) {
                    if game.over {
                        game = Game::new(&mut rng, board_width, board_height);
                        paused = false;
                        speed = 1;
                        last_update = get_time();
                        state = GameState::Playing;
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
                    game.update(&mut rng);
                    if fast {
                        game.update(&mut rng);
                    }
                    last_update = get_time();
                }
            }
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
            break;
        }
        next_frame().await;
    }
}

