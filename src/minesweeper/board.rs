use sdl2::rect::{Point, Rect};
use serde::{Serialize, Deserialize};
use rand::Rng;
use crate::{
    TILE_SIZE,
    GameState,
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    Reveal(u32, u32),
    ToggleFlag(u32, u32),
    Won,
    Quit,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum TileState {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum TileValue {
    Bomb,
    Adjacent(u32),
}

#[derive(Serialize, Deserialize)]
pub struct Tile {
    state: TileState,
    x: i32,
    y: i32,
    value: TileValue,
}
impl Tile {
    pub fn new_blank(x: i32, y: i32) -> Self {
        Tile {
            state: TileState::Hidden,
            x: x,
            y: y,
            value: TileValue::Adjacent(0),
        }
    }
    pub fn set_bomb(&mut self) {
        self.value = TileValue::Bomb;
    }
    pub fn is_bomb(&self) -> bool {
        match self.value {
            TileValue::Bomb => true,
            TileValue::Adjacent(_) => false,
        }
    }
    pub fn value(&self) -> TileValue {
        self.value
    }
    pub fn set_value(&mut self, val: TileValue) {
        self.value = val;
    }
    pub fn state(&self) -> TileState {
        self.state
    }
    pub fn set_state(&mut self, state: TileState) {
        self.state = state;
    }
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, TILE_SIZE, TILE_SIZE)
    }
    pub fn center(&self) -> Point {
        self.rect().center()
    }
}

pub fn build_minefield(row_count: u32, col_count: u32, mut bomb_count: u32) -> Vec<Vec<Tile>> {
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
            minefield[i][j].set_value(TileValue::Adjacent(count));
        }
    }
    return minefield;
}

#[derive(Serialize, Deserialize)]
pub struct Board {
    minefield: Vec<Vec<Tile>>,
    pub tile_rows: u32,
    pub tile_columns: u32,

    bomb_count: u32,
}

impl Board {
    pub fn new(rows: u32, columns: u32, bombs: u32) -> Self {
        Self {
            minefield: build_minefield(rows, columns, bombs),
            tile_rows: rows,
            tile_columns: columns,
            bomb_count: bombs,
        }
    }

    fn reveal(&mut self, first_i: usize, first_j: usize, first_chain_reveal: bool) -> Result<(), String> {
        let mut to_reveal = vec![(first_i, first_j, first_chain_reveal)];
        let mut checked = vec![vec![false; self.tile_columns as usize]; self.tile_rows as usize];
        
        while !to_reveal.is_empty() {
            let (i,j, mut chain_reveal) = to_reveal.pop().unwrap();
        
            if checked[i][j] {
                continue;
            }
            checked[i][j] = true;
            match self.minefield[i][j].state() {
                TileState::Flagged => continue,
                TileState::Revealed
                | TileState::Hidden => {},
            };
            let flag_count = self.surrounding_flags(i, j);
            match self.minefield[i][j].value() {
                TileValue::Adjacent(x) => {
                    self.minefield[i][j].set_state(TileState::Revealed);
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
                    if j < self.tile_columns as usize - 1 {
                        to_reveal.push((i, j+1, false));
                    }
                    if i > 0 {
                        to_reveal.push((i-1, j, false));
                        if j > 0 {
                            to_reveal.push((i-1, j-1, false));
                        }
                        if j < self.tile_columns as usize - 1 {
                            to_reveal.push((i-1, j+1, false));
                        }
                    }
                    if i < self.tile_rows as usize - 1 {
                        to_reveal.push((i+1, j, false));
                        if j > 0 {
                            to_reveal.push((i+1, j-1, false));
                        }
                        if j < self.tile_columns as usize - 1 {
                            to_reveal.push((i+1, j+1, false));
                        }
                    }
                },
                TileValue::Bomb => return Err("Bomb was triggered while revealing".to_string()),
            }
        }
        Ok(())
    }

    fn surrounding_flags(&self, i: usize, j: usize) -> u32 {
        let mut count = 0;
        if j > 0 {
            if let TileState::Flagged = self.minefield[i][j-1].state() {
                count += 1;
            }
        }
        if j < self.tile_columns as usize - 1 {
            if let TileState::Flagged = self.minefield[i][j+1].state() {
                count += 1;
            }
        }
        if i > 0 {
            if let TileState::Flagged = self.minefield[i-1][j].state() {
                count += 1;
            }
            if j > 0 {
                if let TileState::Flagged = self.minefield[i-1][j-1].state() {
                    count += 1;
                }
            }
            if j < self.tile_columns as usize - 1 {
                if let TileState::Flagged = self.minefield[i-1][j+1].state() {
                    count += 1;
                }
            }
        }
        if i < self.tile_rows as usize - 1 {
            if let TileState::Flagged = self.minefield[i+1][j].state() {
                count += 1;
            }
            if j > 0 {
                if let TileState::Flagged = self.minefield[i+1][j-1].state() {
                    count += 1;
            }
            }
            if j < self.tile_columns as usize - 1 {
                if let TileState::Flagged = self.minefield[i+1][j+1].state() {
                    count += 1;
                }
            }
        }
        return count;
    }

    pub fn resolve_click(&mut self, game_state: &mut GameState, i: usize, j: usize) {
        match self.minefield[i][j].state {
            TileState::Hidden => {
                match self.reveal(i, j, false) {
                    Err(_) => {
                        *game_state = GameState::GameOver;
                    },
                    Ok(_) => {},
                };
            },
            TileState::Revealed => {
                match self.reveal(i, j, true) {
                    Err(_) => {
                        *game_state = GameState::GameOver;
                    },
                    Ok(_) => {},
                };
            },
            TileState::Flagged => {}, 
        };
    }
    
    pub fn resolve_flag(&mut self, i: usize, j: usize) {
        self.minefield[i][j].state = match self.minefield[i][j].state() {
                TileState::Hidden => TileState::Flagged,
                TileState::Revealed => TileState::Revealed,
                TileState::Flagged => TileState::Hidden,
        };
    }

    pub fn iter_field(&self) -> std::slice::Iter<Vec<Tile>> {
        return self.minefield.iter()
    }

    pub fn check_game_state(&self) -> GameState {
        for row in self.iter_field() {
            for tile in row.iter() {
                match (tile.value(), tile.state()) {
                    (TileValue::Adjacent(_), TileState::Hidden)
                    | (TileValue::Adjacent(_), TileState::Flagged) 
                    => {
                        return GameState::InGame
                    },
                    _ => {},
                };
            }
        }
        GameState::Won
    }
}

pub fn clean_input(input: &str) -> String {
    let mut res = String::new();
    for c in input.chars() {
        if c.is_digit(10) {
            res.push(c);
        }
    }
    return res;
}

pub fn input_to_number(input: &str) -> u32 {
    let mut res = 0;
    for c in input.chars() {
        if c.is_digit(10) {
            res *= 10;
            res += c.to_digit(10).unwrap();
        }
    }
    return res;
}