use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    Result as CrosstermResult,
};
use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

const WIDTH: u16 = 40;
const HEIGHT: u16 = 20;

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
}

impl Game {
    fn new() -> Self {
        let mut eel = VecDeque::new();
        eel.push_back((WIDTH / 2, HEIGHT / 2));
        let fish = Self::random_pos(&eel);
        Self {
            eel,
            dir: Direction::Right,
            fish,
            over: false,
        }
    }

    fn random_pos(eel: &VecDeque<(u16, u16)>) -> (u16, u16) {
        let mut rng = thread_rng();
        loop {
            let p = (rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT));
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
        x = (x + WIDTH) % WIDTH;
        y = (y + HEIGHT) % HEIGHT;
        if self.eel.contains(&(x, y)) {
            self.over = true;
            return;
        }
        self.eel.push_front((x, y));
        if (x, y) == self.fish {
            self.fish = Self::random_pos(&self.eel);
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

fn draw(game: &Game, stdout: &mut std::io::Stdout) -> CrosstermResult<()> {
    execute!(stdout, cursor::MoveTo(0, 0))?;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if game.eel.front() == Some(&(x, y)) {
                print!(">");
            } else if game.eel.contains(&(x, y)) {
                print!("o");
            } else if game.fish == (x, y) {
                print!("*");
            } else {
                print!(" ");
            }
        }
        println!();
    }
    stdout.flush()
}

fn main() -> CrosstermResult<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        Clear(ClearType::All),
        SetForegroundColor(Color::Green)
    )?;
    let mut game = Game::new();
    let mut last_tick = Instant::now();
    loop {
        let timeout = Duration::from_millis(100);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('i') | KeyCode::Char('I') => game.change_dir(Direction::Up),
                    KeyCode::Char('k') | KeyCode::Char('K') => game.change_dir(Direction::Down),
                    KeyCode::Char('j') | KeyCode::Char('J') => game.change_dir(Direction::Left),
                    KeyCode::Char('l') | KeyCode::Char('L') => game.change_dir(Direction::Right),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= Duration::from_millis(150) {
            game.update();
            draw(&game, &mut stdout)?;
            if game.over {
                println!("Game Over! press q to exit.");
            }
            last_tick = Instant::now();
        }
    }
    execute!(stdout, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()
}
