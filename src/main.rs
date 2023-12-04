extern crate termion;

use rand::seq::SliceRandom;
use rand::Rng;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

type Grid = [[u32; 4]; 4];

#[derive(Default)]
pub struct App {
    grid: Grid,
    win_count: u32,
    loss_count: u32,
}

fn draw_grid(stdout: &mut RawTerminal<io::Stdout>, app: &App) -> io::Result<()> {
    let header = "\x1b[35m";
    let border = "\x1b[33;2m";
    let reset = "\x1b[0m";

    write!(stdout, "{header}")?;
    write!(stdout, " ██████  ██████  ██   ██  ██████ \r\n")?;
    write!(stdout, "     ██  ██  ██  ██   ██  ██  ██ \r\n")?;
    write!(stdout, " ██████  ██  ██  ███████  ██████ \r\n")?;
    write!(stdout, " ██      ██  ██       ██  ██  ██ \r\n")?;
    write!(stdout, " ██████  ██████       ██  ██████ \r\n")?;
    write!(stdout, "{reset}")?;
    write!(stdout, "\r\n")?;

    write!(stdout, "{border}")?;
    write!(stdout, "┌─")?;
    for x in 0..4 {
        if x > 0 {
            write!(stdout, "─┬─")?;
        }
        write!(stdout, "─────")?;
    }
    write!(stdout, "─┐")?;
    write!(stdout, "{reset}")?;
    write!(stdout, "\r\n")?;

    macro_rules! empty_space {
        () => {{
            write!(stdout, "{border}")?;
            write!(stdout, "│ ")?;
            for x in 0..4 {
                if x > 0 {
                    write!(stdout, " │ ")?;
                }
                write!(stdout, "     ")?;
            }
            write!(stdout, " │")?;
            write!(stdout, "{reset}")?;
            write!(stdout, "\r\n")?;
        }};
    }

    for (y, row) in app.grid.into_iter().enumerate() {
        if y > 0 {
            write!(stdout, "{border}")?;
            write!(stdout, "├─")?;
            for x in 0..4 {
                if x > 0 {
                    write!(stdout, "─┼─")?;
                }
                write!(stdout, "─────")?;
            }
            write!(stdout, "─┤")?;
            write!(stdout, "{reset}")?;
            write!(stdout, "\r\n")?;
        }

        empty_space!();

        write!(stdout, "{border}")?;
        write!(stdout, "│ ")?;
        for (x, value) in row.into_iter().enumerate() {
            if x > 0 {
                write!(stdout, " │ ")?;
            }
            let color = match value {
                0 => {
                    write!(stdout, "     ")?;
                    continue;
                }
                2 => "\x1b[39;2;238;228;218m",
                4 => "\x1b[38;2;237;224;200m",
                8 => "\x1b[38;2;242;177;121m",
                16 => "\x1b[38;2;245;149;99m",
                32 => "\x1b[38;2;246;124;95m",
                64 => "\x1b[38;2;246;94;59m",
                128 => "\x1b[38;2;237;207;114m",
                256 => "\x1b[38;2;237;204;97m",
                512 => "\x1b[38;2;237;200;80m",
                1024 => "\x1b[38;2;237;197;63m",
                2048 => "\x1b[38;2;237;194;46m",
                _ => "\x1b[31m",
            };
            write!(stdout, "{reset}{color}")?;
            write!(stdout, "{:^5}", value)?;
            write!(stdout, "{border}")?;
        }
        write!(stdout, " │")?;
        write!(stdout, "{reset}")?;
        write!(stdout, "\r\n")?;

        empty_space!();
    }

    write!(stdout, "{border}")?;
    write!(stdout, "└─")?;
    for x in 0..4 {
        if x > 0 {
            write!(stdout, "─┴─")?;
        }
        write!(stdout, "─────")?;
    }
    write!(stdout, "─┘")?;
    write!(stdout, "{reset}")?;
    write!(stdout, "\r\n")?;

    write!(stdout, "\x1b[32;2m")?;
    write!(stdout, "         ")?;
    write!(stdout, "{:^7}", app.win_count)?;
    write!(stdout, "{reset}")?;
    write!(stdout, "\x1b[0;2m")?;
    write!(stdout, "|")?;
    write!(stdout, "\x1b[31;2m")?;
    write!(stdout, "{:^7}", app.loss_count)?;
    write!(stdout, "{reset}")?;
    write!(stdout, "\r\n")?;

    Ok(())
}

#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl App {
    pub fn move_tiles(&mut self, direction: Direction) {
        let old_grid = self.grid.clone();

        self.compress(direction);
        self.merge(direction);
        self.compress(direction);

        if !self.has_tile_value(0) {
            self.lose_game();
        }
        if self.has_tile_value(2048) {
            self.win_game();
        }

        if self.grid != old_grid {
            self.spawn_tile();
        }
    }

    fn compress(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                for row in &mut self.grid {
                    for x in 0..3 {
                        if row[x] > 0 {
                            continue;
                        }
                        for x2 in x..4 {
                            if row[x2] == 0 {
                                continue;
                            }
                            row[x] = row[x2];
                            row[x2] = 0;
                            break;
                        }
                    }
                }
            }
            Direction::Right => {
                for row in &mut self.grid {
                    for x in (1..4).rev() {
                        if row[x] > 0 {
                            continue;
                        }
                        for x2 in (0..=x).rev() {
                            if row[x2] == 0 {
                                continue;
                            }
                            row[x] = row[x2];
                            row[x2] = 0;
                            break;
                        }
                    }
                }
            }
            Direction::Up => {
                let grid = &mut self.grid;
                for x in 0..4 {
                    for y in 0..3 {
                        if grid[y][x] > 0 {
                            continue;
                        }
                        for y2 in y..4 {
                            if grid[y2][x] == 0 {
                                continue;
                            }
                            grid[y][x] = grid[y2][x];
                            grid[y2][x] = 0;
                            break;
                        }
                    }
                }
            }
            Direction::Down => {
                let grid = &mut self.grid;
                for x in 0..4 {
                    for y in (1..4).rev() {
                        if grid[y][x] > 0 {
                            continue;
                        }
                        for y2 in (0..=y).rev() {
                            if grid[y2][x] == 0 {
                                continue;
                            }
                            grid[y][x] = grid[y2][x];
                            grid[y2][x] = 0;
                            break;
                        }
                    }
                }
            }
        }
    }

    fn merge(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                for row in &mut self.grid {
                    for x in 0..3 {
                        if row[x] == 0 {
                            continue;
                        }
                        let x2 = x + 1;
                        if row[x] == row[x2] {
                            row[x] *= 2;
                            row[x2] = 0;
                        }
                    }
                }
            }
            Direction::Right => {
                for row in &mut self.grid {
                    for x in (1..4).rev() {
                        if row[x] == 0 {
                            continue;
                        }
                        let x2 = x - 1;
                        if row[x] == row[x2] {
                            row[x] *= 2;
                            row[x2] = 0;
                        }
                    }
                }
            }
            Direction::Up => {
                let grid = &mut self.grid;
                for x in 0..4 {
                    for y in 0..3 {
                        if grid[y][x] == 0 {
                            continue;
                        }
                        let y2 = y + 1;
                        if grid[y][x] == grid[y2][x] {
                            grid[y][x] *= 2;
                            grid[y2][x] = 0;
                        }
                    }
                }
            }
            Direction::Down => {
                let grid = &mut self.grid;
                for x in 0..4 {
                    for y in (1..4).rev() {
                        if grid[y][x] == 0 {
                            continue;
                        }
                        let y2 = y - 1;
                        if grid[y][x] == grid[y2][x] {
                            grid[y][x] *= 2;
                            grid[y2][x] = 0;
                        }
                    }
                }
            }
        }
    }

    fn has_tile_value(&self, value: u32) -> bool {
        for row in &self.grid {
            for tile in row {
                if *tile == value {
                    return true;
                }
            }
        }
        false
    }

    fn spawn_tile(&mut self) {
        let mut empty_tiles = Vec::new();
        for row in &mut self.grid {
            for tile in row {
                if *tile == 0 {
                    empty_tiles.push(tile);
                }
            }
        }
        if empty_tiles.is_empty() {
            return;
        }

        let mut rng = rand::thread_rng();
        let tile = empty_tiles.choose_mut(&mut rng).unwrap();
        let value = if rng.gen_bool(0.1) { 4 } else { 2 };

        **tile = value;
    }

    fn reset_grid(&mut self) {
        self.grid = Default::default();
    }

    fn lose_game(&mut self) {
        thread::sleep(Duration::from_millis(400));
        self.reset_grid();
        self.loss_count += 1;
    }
    fn win_game(&mut self) {
        thread::sleep(Duration::from_millis(400));
        self.reset_grid();
        self.win_count += 1;
    }
}

fn main() {
    let mut app = App::default();
    app.grid[0][0] = 2;
    app.grid[0][1] = 2;
    app.grid[0][3] = 16;
    app.grid[1][0] = 32;
    app.grid[1][2] = 128;
    app.grid[1][3] = 16;
    app.grid[2][0] = 1024;
    app.grid[2][1] = 1024;

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();

    write!(stdout, "{}", termion::clear::All).unwrap();
    write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
    draw_grid(&mut stdout, &app).unwrap();
    stdout.flush().unwrap();

    // Iterate over key events in the input stream
    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('q') | Key::Ctrl('c') => break,

            Key::Char('h') => app.move_tiles(Direction::Left),
            Key::Char('j') => app.move_tiles(Direction::Down),
            Key::Char('k') => app.move_tiles(Direction::Up),
            Key::Char('l') => app.move_tiles(Direction::Right),

            _ => {}
        }

        write!(stdout, "{}", termion::clear::All).unwrap();
        write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
        draw_grid(&mut stdout, &app).unwrap();
        stdout.flush().unwrap();
    }

    // Disable raw mode and show the cursor on program exit
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
