mod my_textures;

use rand::Rng;
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
use crate::my_textures::*;

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
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    
    let mut window = video_subsystem
        .window(
            "SpaceInvaders",
            TILE_COLUMNS * TILE_SIZE,
            TILE_ROWS * TILE_SIZE,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    window.set_bordered(false);

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    
    let mut texture_creator1 = canvas.texture_creator();
    let (hidden_texture, revealed_texture) = tile_textures(&mut canvas, &mut texture_creator1)?;

    let mut texture_creator2 = canvas.texture_creator();
    let (number_textures, surface_rect) = number_textures(&mut texture_creator2)?;

    let texture_creator3 = canvas.texture_creator();
    let flag_texture = texture_creator3
        .load_texture("assets/flag.svg")
        .map_err(|e| e.to_string())?;

    let mut texture_creator4 = canvas.texture_creator();
    let (menu_texture, menu_rect) = text_texture(&mut texture_creator4, "> Start <")?;

    let mut texture_creator5 = canvas.texture_creator();
    let (end_texture, end_rect) = text_texture(&mut texture_creator5, "Game Over!")?;
    
    let (mut pressed_i, mut pressed_j) = (None, None);
    let mut minefield = build_minefield(TILE_ROWS, TILE_COLUMNS, BOMB_COUNT);

    let mut game_state = GameState::Menu;
    let mut event_pump = sdl_context.event_pump()?;

    'game_loop: loop {
        match game_state {
            GameState::Menu => {
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                                keycode: Some(Keycode::Escape),
                                ..
                        } => break 'game_loop,
                        Event::KeyDown {
                                keycode: Some(Keycode::Return),
                                ..
                        } => game_state = GameState::InGame,
                        _ => {},
                    };
                }
                canvas.set_draw_color(Color::RGB(50, 50, 50));
                canvas.clear();
        
                let center = Rect::new(0,0,TILE_SIZE * TILE_COLUMNS, TILE_SIZE * TILE_ROWS).center();
                canvas.copy(
                    &menu_texture,
                    None,
                    Rect::from_center(center, menu_rect.width(), menu_rect.height()),
                )?;
        
                canvas.present();
            },

            GameState::InGame => {
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'game_loop,
                        Event::MouseButtonDown {
                            mouse_btn: MouseButton::Left,
                            x,
                            y,
                            ..
                        } => {
                            pressed_i = Some((y / TILE_SIZE as i32) as usize);
                            pressed_j = Some((x / TILE_SIZE as i32) as usize);
                        },
                        Event::MouseButtonUp {
                            mouse_btn: MouseButton::Left,
                            x,
                            y,
                            ..
                        } => {
                            let i = (y / TILE_SIZE as i32) as usize;
                            let j = (x / TILE_SIZE as i32) as usize;
                            match (pressed_i, pressed_j) {
                                (Some(i1), Some(j1)) => {
                                    if i1 == i && j1 == j {
                                        match minefield[i][j].state {
                                            TileState::Hidden => {
                                                match reveal(&mut minefield, i1, j1, false) {
                                                    Err(_) => {
                                                        game_state = GameState::GameOver;
                                                        continue 'game_loop;
                                                    },
                                                    Ok(_) => {},
                                                };
                                            },
                                            TileState::Revealed => {
                                                match reveal(&mut minefield, i1, j1, true) {
                                                    Err(_) => {
                                                        game_state = GameState::GameOver;
                                                        continue 'game_loop;
                                                    },
                                                    Ok(_) => {},
                                                };
                                            },
                                            TileState::Flagged => {}, 
                                        };
                                    }
                                }
                                _ => continue,
                            };
                        },
                        Event::MouseButtonDown {
                            mouse_btn: MouseButton::Right,
                            x,
                            y,
                            ..
                        } => {
                            let i = (y / TILE_SIZE as i32) as usize;
                            let j = (x / TILE_SIZE as i32) as usize;
                            minefield[i][j].state = match minefield[i][j].state {
                                TileState::Hidden => TileState::Flagged,
                                TileState::Revealed => continue,
                                TileState::Flagged => TileState::Hidden,
                            };
                        },
                        _ => {},
                    }
                }
        
                game_state = GameState::Won;
                'check_won: for row in minefield.iter() {
                    for tile in row.iter() {
                        match (tile.value, tile.state) {
                            (TileValue::Adjacent(_), TileState::Hidden)
                            | (TileValue::Adjacent(_), TileState::Flagged) 
                            => {
                                game_state = GameState::InGame;
                                break 'check_won;
                            },
                            _ => {},
                        };
                    }
                }

                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.clear();
                for row in minefield.iter() {
                    for tile in row.iter() {
                        match tile.state {
                            TileState::Revealed => {
                                canvas.copy(
                                    &revealed_texture,
                                    None,
                                    tile.rect,
                                )?;
                                if let TileValue::Adjacent(x) = tile.value {
                                    canvas.copy(
                                        number_textures.get(x as usize).expect(format!("texture for index {x} doesnt exist").as_str()),
                                        None,
                                        Rect::from_center(tile.rect.center(), surface_rect.width(), surface_rect.height())
                                    )?;
                                } 
                            },
                            TileState::Flagged => {
                                canvas.copy(
                                    &flag_texture,
                                    None,
                                    tile.rect,
                                )?;
                            },
                            TileState::Hidden => {
                                canvas.copy(
                                    &hidden_texture,
                                    None,
                                    tile.rect,
                                )?;
                            },
                        }           
                    }
                }
        
                canvas.present();
            },

            GameState::GameOver => {
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. }
                        | Event::KeyDown {
                                keycode: Some(Keycode::Escape),
                                ..
                        } => break 'game_loop,
                        _ => {},
                    };
                }
                canvas.set_draw_color(Color::RGB(50, 50, 50));
                canvas.clear();
        
                let center = Rect::new(0,0,TILE_SIZE * TILE_COLUMNS, TILE_SIZE * TILE_ROWS).center();
                canvas.copy(
                    &end_texture,
                    None,
                    Rect::from_center(center, end_rect.width(), end_rect.height()),
                )?;
        
                canvas.present();
            },

            GameState::Won => {
                println!("you've beaten the game :)");
                break 'game_loop;
            },
        }
    }

    Ok(())
}