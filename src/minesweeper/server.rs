use rand::Rng;
use std::net::UdpSocket;
use sdl2::{
    image::LoadTexture,
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::Color,
    rect::{Rect},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};

const TILE_ROWS: u32 = 16;
const TILE_COLUMNS: u32 = 30;
const TILE_SIZE: u32 = 20;

const BOMB_COUNT: u32 = 99;

enum GameState {
    Menu,
    InGame,
    Won,
    GameOver,
}

#[derive(Clone, Copy)]
enum TileState {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy)]
enum TileValue {
    Bomb,
    Adjacent(u32),
}

struct Tile {
    state: TileState,
    rect: Rect,
    value: TileValue,
}
impl Tile {
    fn new_blank(x: i32, y: i32) -> Self {
        Tile {
            state: TileState::Hidden,
            rect: Rect::new(x, y, TILE_SIZE, TILE_SIZE),
            value: TileValue::Adjacent(0),
        }
    }
    fn set_bomb(&mut self) {
        self.value = TileValue::Bomb;
    }
    fn is_bomb(&self) -> bool {
        match self.value {
            TileValue::Bomb => true,
            TileValue::Adjacent(_) => false,
        }
    }
}

fn build_minefield(row_count: u32, col_count: u32, mut bomb_count: u32) -> Vec<Vec<Tile>> {
    let mut minefield = Vec::new();
    for i in 0..row_count {
        let mut new_row = Vec::new();
        for j in 0..col_count {
            new_row.push(Tile::new_blank((j * TILE_SIZE) as i32, (i * TILE_SIZE) as i32));
        }
        minefield.push(new_row);
    }
    let mut rng = rand::thread_rng();
    while bomb_count > 0 {
        let i = rng.gen_range(0..row_count) as usize;
        let j = rng.gen_range(0..col_count) as usize;
        if minefield[i][j].is_bomb() {
            continue;
        }
        minefield[i][j].set_bomb();
        bomb_count -= 1;
    }

    for i in 0..(row_count as usize) {
        for j in 0..(col_count as usize) {
            let mut count = 0;
            if minefield[i][j].is_bomb() {
                continue;
            }
            if i > 0 && minefield[i-1][j].is_bomb() {
                count += 1;
            }
            if i < row_count as usize - 1  && minefield[i+1][j].is_bomb() {
                count += 1;
            }
            if j > 0 && minefield[i][j-1].is_bomb() {
                count += 1;
            }
            if j < col_count as usize - 1  && minefield[i][j+1].is_bomb() {
                count += 1;
            }
            if i > 0 && j > 0 && minefield[i-1][j-1].is_bomb() {
                count += 1;
            }
            if i > 0 && j < col_count as usize - 1 && minefield[i-1][j+1].is_bomb() {
                count += 1;
            }
            if i < row_count as usize - 1 && j > 0 && minefield[i+1][j-1].is_bomb() {
                count += 1;
            }
            if i < row_count as usize - 1 && j < col_count as usize - 1 && minefield[i+1][j+1].is_bomb() {
                count += 1;
            }
            minefield[i][j].value = TileValue::Adjacent(count);
        }
    }
    return minefield;
}

fn surrounding_flags(minefield: &Vec<Vec<Tile>>, i: usize, j: usize) -> u32 {
    let mut count = 0;
    if j > 0 {
        if let TileState::Flagged = minefield[i][j-1].state {
            count += 1;
        }
    }
    if j < TILE_COLUMNS as usize - 1 {
        if let TileState::Flagged = minefield[i][j+1].state {
            count += 1;
        }
    }
    if i > 0 {
        if let TileState::Flagged = minefield[i-1][j].state {
            count += 1;
        }
        if j > 0 {
            if let TileState::Flagged = minefield[i-1][j-1].state {
                count += 1;
            }
        }
        if j < TILE_COLUMNS as usize - 1 {
            if let TileState::Flagged = minefield[i-1][j+1].state {
                count += 1;
            }
        }
    }
    if i < TILE_ROWS as usize - 1 {
        if let TileState::Flagged = minefield[i+1][j].state {
            count += 1;
        }
        if j > 0 {
            if let TileState::Flagged = minefield[i+1][j-1].state {
                count += 1;
        }
        }
        if j < TILE_COLUMNS as usize - 1 {
            if let TileState::Flagged = minefield[i+1][j+1].state {
                count += 1;
            }
        }
    }
    return count;
}

fn reveal(minefield: &mut Vec<Vec<Tile>>, first_i: usize, first_j: usize, first_chain_reveal: bool) -> Result<(), String> {
    let mut to_reveal = vec![(first_i, first_j, first_chain_reveal)];
    let mut checked = vec![vec![false; TILE_COLUMNS as usize]; TILE_ROWS as usize];
    
    while !to_reveal.is_empty() {
        let (i,j, mut chain_reveal) = to_reveal.pop().unwrap();
    
        if checked[i][j] {
            continue;
        }
        checked[i][j] = true;
        match minefield[i][j].state {
            TileState::Flagged => continue,
            TileState::Revealed
            | TileState::Hidden => {},
        };
        let flag_count = surrounding_flags(minefield, i, j);
        match minefield[i][j].value {
            TileValue::Adjacent(x) => {
                minefield[i][j].state = TileState::Revealed;
                if x == 0 {
                    chain_reveal = true;
                }
                if flag_count != x {
                    continue;
                }
                if !chain_reveal{
                    continue;
                }
                if j > 0 {
                    to_reveal.push((i, j-1, false));
                }
                if j < TILE_COLUMNS as usize - 1 {
                    to_reveal.push((i, j+1, false));
                }
                if i > 0 {
                    to_reveal.push((i-1, j, false));
                    if j > 0 {
                        to_reveal.push((i-1, j-1, false));
                    }
                    if j < TILE_COLUMNS as usize - 1 {
                        to_reveal.push((i-1, j+1, false));
                    }
                }
                if i < TILE_ROWS as usize - 1 {
                    to_reveal.push((i+1, j, false));
                    if j > 0 {
                        to_reveal.push((i+1, j-1, false));
                    }
                    if j < TILE_COLUMNS as usize - 1 {
                        to_reveal.push((i+1, j+1, false));
                    }
                }
            },
            TileValue::Bomb => return Err("Bomb was triggered while revealing".to_string()),
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {

    let mut minefield = build_minefield(TILE_ROWS, TILE_COLUMNS, BOMB_COUNT);

    let mut game_state = GameState::Menu;

    let socket = UdpSocket::bind("192.168.178.25:2024").map_err(|e| e.to_string())?; // home pc 192.168.178.25:2024
    let mut buf = [0; 100];
    let (_, src) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;
    socket.connect(src).map_err(|e| e.to_string())?;
    println!("connected");

    socket.send("connected".as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
    // 'game_loop: loop {
    //     match game_state {
    //         GameState::Menu => {},

    //         GameState::InGame => {},

    //         GameState::GameOver => {},

    //         GameState::Won => {
    //             println!("you've beaten the game :)");
    //             break 'game_loop;
    //         },
    //     }
    // }

    // Ok(())
}