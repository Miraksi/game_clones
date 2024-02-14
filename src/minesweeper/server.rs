mod board;

use std::net::UdpSocket;
use crate::board::{
    TileState,
    TileValue,
    Board,
    Action,
};

const TILE_SIZE: u32 = 20;

enum GameState {
    Menu,
    InGame,
    Won,
    GameOver,
}

fn await_client_action(socket: &UdpSocket) -> Result<Action, String> {
    let mut buf = [0; 50];
    let amt = socket.recv(&mut buf).map_err(|e| e.to_string())?;
    let mut tmp: Vec<u8> = Vec::from(buf);
    tmp.resize(amt, 0);
    let serialized = String::from_utf8(Vec::from(tmp)).map_err(|e| e.to_string())?;
    let action: Action = serde_json::from_str(&serialized).map_err(|e| e.to_string())?;
    return Ok(action);
}

fn send_valid(socket: &UdpSocket, valid: bool) -> Result<(), String> {
    let message = serde_json::to_string(&valid).map_err(|e| e.to_string())?;
    socket.send(message.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
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
    socket.send(message.as_bytes()).map_err(|e| e.to_string())?;

    let mut game_state = GameState::InGame;

    'game_loop: loop {
        // wait for client move
        let action = await_client_action(&socket)?;

        match game_state {
            GameState::InGame => {
                match action {
                    Action::Reveal(i,j) => {
                        board.resolve_click(&mut game_state, i as usize, j as usize);
                        send_valid(&socket, true)?;
                    },
                    Action::ToggleFlag(i,j) => {
                        board.resolve_flag(i as usize, j as usize);
                        send_valid(&socket, true)?;
                    },
                    Action::Won => {
                        send_valid(&socket, false)?;
                        break 'game_loop;
                    },
                    Action::Quit => {
                        send_valid(&socket, true)?;
                        break 'game_loop;
                    },
                }

                if let GameState::GameOver = game_state {
                    continue;
                }
        
                game_state = GameState::Won;
                'check_won: for row in board.iter_field() {
                    for tile in row.iter() {
                        match (tile.value(), tile.state()) {
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
            },

            GameState::GameOver => {
                match action {
                    Action::Quit => {
                        send_valid(&socket, true)?;
                        break 'game_loop;
                    },
                    _ => {
                        send_valid(&socket, false)?;
                        break 'game_loop;
                    },
                }
            },

            GameState::Won => match action {
                Action::Won => {
                    send_valid(&socket, true)?;
                    break 'game_loop;
                },
                _ => {
                    send_valid(&socket, false)?;
                    break 'game_loop;
                },
            },

            GameState::Menu => panic!{"something strange happened"},
        };
        
    }
    Ok(())
}