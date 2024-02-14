mod board;

use sdl2::{
    image::LoadTexture,
    event::Event,
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::Color,
    rect::{Rect},
};
use std::net::UdpSocket;
use crate::board::{
    TileState,
    TileValue,
    Board,
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


fn main() -> Result<(), String> {
    // let (mut pressed_i, mut pressed_j) = (None, None);

    // open socket
    let socket = UdpSocket::bind("192.168.178.25:2024").map_err(|e| e.to_string())?;

    // receive board parameters from client
    let mut buf = [0; 50];
    let (amt, src) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;
    let mut tmp: Vec<u8> = Vec::from(buf);
    tmp.resize(amt, 0);
    let serialized = String::from_utf8(Vec::from(tmp)).unwrap();

    // creating board with given parameters
    let args: (u32,u32,u32,u32) = serde_json::from_str(&serialized).unwrap();
    let mut board = Board::new(args.1, args.2, args.3);

    // sending board back
    socket.connect(src).map_err(|e| e.to_string())?;
    let message = serde_json::to_string(&board).expect("could'nt serialize the board");
    let amt = socket.send(message.as_bytes()).map_err(|e| e.to_string())?;

    return Ok(());
    // let mut game_state = GameState::Menu;
    // let mut event_pump = sdl_context.event_pump()?;

    // 'game_loop: loop {
    //     match game_state {
    //         GameState::Menu => {
    //             for event in event_pump.poll_iter() {
    //                 match event {
    //                     Event::Quit { .. }
    //                     | Event::KeyDown {
    //                             keycode: Some(Keycode::Escape),
    //                             ..
    //                     } => break 'game_loop,
    //                     Event::KeyDown {
    //                             keycode: Some(Keycode::Return),
    //                             ..
    //                     } => game_state = GameState::InGame,
    //                     _ => {},
    //                 };
    //             }
    //             canvas.set_draw_color(Color::RGB(50, 50, 50));
    //             canvas.clear();
        
    //             let center = Rect::new(0,0,TILE_SIZE * TILE_COLUMNS, TILE_SIZE * TILE_ROWS).center();
    //             canvas.copy(
    //                 &menu_texture,
    //                 None,
    //                 Rect::from_center(center, menu_rect.width(), menu_rect.height()),
    //             )?;
        
    //             canvas.present();
    //         },

    //         GameState::InGame => {
    //             for event in event_pump.poll_iter() {
    //                 match event {
    //                     Event::Quit { .. }
    //                     | Event::KeyDown {
    //                         keycode: Some(Keycode::Escape),
    //                         ..
    //                     } => break 'game_loop,
    //                     Event::MouseButtonDown {
    //                         mouse_btn: MouseButton::Left,
    //                         x,
    //                         y,
    //                         ..
    //                     } => {
    //                         pressed_i = Some((y / TILE_SIZE as i32) as usize);
    //                         pressed_j = Some((x / TILE_SIZE as i32) as usize);
    //                     },
    //                     Event::MouseButtonUp {
    //                         mouse_btn: MouseButton::Left,
    //                         x,
    //                         y,
    //                         ..
    //                     } => {
    //                         let i = (y / TILE_SIZE as i32) as usize;
    //                         let j = (x / TILE_SIZE as i32) as usize;
    //                         match (pressed_i, pressed_j) {
    //                             (Some(i1), Some(j1)) => {
    //                                 if i1 == i && j1 == j {
    //                                     board.resolve_click(&mut game_state, i, j);
    //                                 }
    //                             }
    //                             _ => continue,
    //                         };
    //                     },
    //                     Event::MouseButtonDown {
    //                         mouse_btn: MouseButton::Right,
    //                         x,
    //                         y,
    //                         ..
    //                     } => {
    //                         let i = (y / TILE_SIZE as i32) as usize;
    //                         let j = (x / TILE_SIZE as i32) as usize;
    //                         board.resolve_flag(i, j);
    //                     },
    //                     _ => {},
    //                 }
    //             }

    //             if let GameState::GameOver = game_state {
    //                 continue;
    //             }
        
    //             game_state = GameState::Won;
    //             'check_won: for row in board.iter_field() {
    //                 for tile in row.iter() {
    //                     match (tile.value(), tile.state()) {
    //                         (TileValue::Adjacent(_), TileState::Hidden)
    //                         | (TileValue::Adjacent(_), TileState::Flagged) 
    //                         => {
    //                             game_state = GameState::InGame;
    //                             break 'check_won;
    //                         },
    //                         _ => {},
    //                     };
    //                 }
    //             }

    //             canvas.set_draw_color(Color::RGB(0, 0, 0));
    //             canvas.clear();
    //             for row in board.iter_field() {
    //                 for tile in row.iter() {
    //                     match tile.state() {
    //                         TileState::Revealed => {
    //                             canvas.copy(
    //                                 &revealed_texture,
    //                                 None,
    //                                 tile.rect(),
    //                             )?;
    //                             if let TileValue::Adjacent(x) = tile.value() {
    //                                 canvas.copy(
    //                                     number_textures.get(x as usize).expect(format!("texture for index {x} doesnt exist").as_str()),
    //                                     None,
    //                                     Rect::from_center(tile.center(), surface_rect.width(), surface_rect.height())
    //                                 )?;
    //                             } 
    //                         },
    //                         TileState::Flagged => {
    //                             canvas.copy(
    //                                 &flag_texture,
    //                                 None,
    //                                 tile.rect(),
    //                             )?;
    //                         },
    //                         TileState::Hidden => {
    //                             canvas.copy(
    //                                 &hidden_texture,
    //                                 None,
    //                                 tile.rect(),
    //                             )?;
    //                         },
    //                     }           
    //                 }
    //             }
        
    //             canvas.present();
    //         },

    //         GameState::GameOver => {
    //             for event in event_pump.poll_iter() {
    //                 match event {
    //                     Event::Quit { .. }
    //                     | Event::KeyDown {
    //                             keycode: Some(Keycode::Escape),
    //                             ..
    //                     } => break 'game_loop,
    //                     _ => {},
    //                 };
    //             }
    //             canvas.set_draw_color(Color::RGB(50, 50, 50));
    //             canvas.clear();
        
    //             let center = Rect::new(0,0,TILE_SIZE * TILE_COLUMNS, TILE_SIZE * TILE_ROWS).center();
    //             canvas.copy(
    //                 &end_texture,
    //                 None,
    //                 Rect::from_center(center, end_rect.width(), end_rect.height()),
    //             )?;
        
    //             canvas.present();
    //         },

    //         GameState::Won => {
    //             println!("you've beaten the game :)");
    //             break 'game_loop;
    //         },
    //     }
    // }

    // Ok(())
}