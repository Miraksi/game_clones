mod my_textures;
mod board;

use sdl2::{
    image::LoadTexture,
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::Color,
    rect::{Rect, Point},
};
use crate::my_textures::*;
use crate::board::{
    clean_input,
    input_to_number,
    TileState,
    TileValue,
    Board,
};

const MENU_HEIGHT: u32 = 320;
const MENU_WIDTH: u32 = 600;
const TILE_SIZE: u32 = 20;

enum GameState {
    Menu,
    InGame,
    Won,
    GameOver,
}


fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let text_subsystem = video_subsystem.text_input();
    
    let mut window = video_subsystem
        .window(
            "Minesweeper",
            MENU_WIDTH,
            MENU_HEIGHT,
        )
        .position_centered()
        .resizable()
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
    let (number_textures, surface_rect) = number_textures(&mut texture_creator1)?;

    let texture_creator2 = canvas.texture_creator();
    let flag_texture = texture_creator2
        .load_texture("assets/flag_tile.png")
        .map_err(|e| e.to_string())?;

    let texture_creator3 = canvas.texture_creator();
    let hidden_texture = texture_creator3
        .load_texture("assets/hidden_tile.png")
        .map_err(|e| e.to_string())?;

    let texture_creator4 = canvas.texture_creator();
    let revealed_texture = texture_creator4
        .load_texture("assets/revealed_tile.png")
        .map_err(|e| e.to_string())?;

    let mut texture_creator5 = canvas.texture_creator();
    let (menu_texture, menu_rect) = text_texture(&mut texture_creator5, "> Start <", 24)?;

    let mut texture_creator6 = canvas.texture_creator();
    let (game_over_texture, game_over_rect) = text_texture(&mut texture_creator6, "Game Over!", 24)?;

    let mut texture_creator7 = canvas.texture_creator();
    let (won_texture, won_rect) = text_texture(&mut texture_creator7, "You have won :)", 24)?;
    
    let (mut pressed_i, mut pressed_j) = (None, None);
    let mut board = Board::new(5, 5, 1);
    let (mut end_texture, mut end_rect) = (&game_over_texture, &game_over_rect);

    // initialize textbox
    let mut boxes = vec![
        (Rect::new(5, 5, 100, 20), "Width: 30".to_string()),
        (Rect::new(5, 25, 100, 20), "Height: 16".to_string()),
        (Rect::new(5, 45, 100, 20), "Bombs: 99".to_string()),
    ];
    let mut to_edit: Option<usize> = None;

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
                        } => {
                            game_state = GameState::InGame;
                            text_subsystem.stop();
                            let settings: Vec<u32> = boxes.iter().map(|(_,text)| input_to_number(text)).collect();
                            board = Board::new(settings[1], settings[0], settings[2]);
                            canvas
                                .window_mut()
                                .set_size(settings[0] * TILE_SIZE, settings[1] * TILE_SIZE)
                                .map_err(|e| e.to_string())?;
                        },
                        Event::MouseButtonDown {
                            mouse_btn: MouseButton::Left,
                            x,
                            y,
                            ..
                        } => {
                            to_edit = None;
                            for (i, (rect, _text)) in boxes.iter().enumerate() {
                                if rect.contains_point(Point::new(x,y)) {
                                    text_subsystem.start();
                                    text_subsystem.set_rect(*rect);
                                    to_edit = Some(i);
                                }
                            }
                            if let None = to_edit {
                                text_subsystem.stop();
                            }
                        },
                        Event::TextInput {
                            text,
                            ..
                        } => {
                            let cleaned = clean_input(&text);
                            let (_rect, input) = boxes.get_mut(to_edit.unwrap()).unwrap();
                            input.push_str(&cleaned);
                        },
                        Event::KeyDown {
                            keycode: Some(Keycode::Backspace),
                            ..
                        } => { 
                            if let Some(i) = to_edit {
                                let last = boxes.get_mut(i).unwrap().1.pop().unwrap();
                                if !last.is_digit(10) {
                                    boxes.get_mut(i).unwrap().1.push(last);
                                }
                            }
                        },
                        _ => {},
                    };
                }
                canvas.set_draw_color(Color::RGB(50, 50, 50));
                canvas.clear();
        
                let center = Rect::new(0, 0, MENU_WIDTH, MENU_HEIGHT).center();
                canvas.copy(
                    &menu_texture,
                    None,
                    Rect::from_center(center, menu_rect.width(), menu_rect.height()),
                )?;

                // render user text
                for (rect, text) in boxes.iter() {
                    render_text(&mut canvas, rect.x(), rect.y(), text.as_str())?;
                } 
                

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
                                        board.resolve_click(&mut game_state, i, j);
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
                            board.resolve_flag(i, j);
                        },
                        _ => {},
                    }
                }

                if let GameState::GameOver = game_state {
                    continue;
                }
        
                game_state = board.check_game_state();
                if let GameState::Won = game_state {
                    end_rect = &won_rect;
                    end_texture = &won_texture;
                }

                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.clear();
                for row in board.iter_field() {
                    for tile in row.iter() {
                        match tile.state() {
                            TileState::Revealed => {
                                canvas.copy(
                                    &revealed_texture,
                                    None,
                                    tile.rect(),
                                )?;
                                if let TileValue::Adjacent(x) = tile.value() {
                                    canvas.copy(
                                        number_textures.get(x as usize).expect(format!("texture for index {x} doesnt exist").as_str()),
                                        None,
                                        Rect::from_center(tile.center(), surface_rect.width(), surface_rect.height())
                                    )?;
                                } 
                            },
                            TileState::Flagged => {
                                canvas.copy(
                                    &flag_texture,
                                    None,
                                    tile.rect(),
                                )?;
                            },
                            TileState::Hidden => {
                                canvas.copy(
                                    &hidden_texture,
                                    None,
                                    tile.rect(),
                                )?;
                            },
                        }           
                    }
                }
        
                canvas.present();
            },

            GameState::GameOver
            | GameState::Won => {
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
                        } => {
                            end_rect = &game_over_rect;
                            end_texture = &game_over_texture;
                            game_state = GameState::Menu;
                            canvas
                                .window_mut()
                                .set_size(MENU_WIDTH, MENU_HEIGHT)
                                .map_err(|e| e.to_string())?;
                        },
                        _ => {},
                    };
                }
                canvas.set_draw_color(Color::RGB(50, 50, 50));
                canvas.clear();
        
                let center = Rect::new(0,0,TILE_SIZE * board.tile_columns, TILE_SIZE * board.tile_rows).center();
                canvas.copy(
                    end_texture,
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