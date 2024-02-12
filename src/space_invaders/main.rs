use core::slice::Iter;
use std::time::Instant;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};
const WINDOW_WIDTH: u32 = 500;
const WINDOW_HEIGHT: u32 = 500;

const PLAYER_WIDTH: u32 = 50;
const PLAYER_HEIGHT: u32 = 20;
const PLAYER_MOVE_SPEED: i32 = 1;

const BULLET_WIDTH: u32 = 3;
const BULLET_HEIGHT: u32 = 10;
const BULLET_SPEED: i32 = 3;
const COOLDOWN: u64 = 1;

const ENEMY_HEALTH: u8 = 2;
const ENEMY_WIDTH: u32 = 15;
const ENEMY_HEIGHT: u32 = 10;
const ENEMY_MOVE_SPEED: i32 = 1;
const ENEMY_DISTANT: i32 = 10;


enum GameState {
    Running,
    GameOver,
}

struct Enemy {
    rect: Rect,
    health: u8,
    dir: i32,
}
impl Enemy {
    fn new() -> Enemy {
        Enemy {
            rect: Rect::new(0, 0, ENEMY_WIDTH, ENEMY_HEIGHT),
            health: ENEMY_HEALTH,
            dir: 1,
        }
    }
    fn impact(&mut self, bullet: &Rect) -> bool {
        if !self.rect.has_intersection(*bullet) {
            return false;
        }
        self.health -= 1;
        return true;
    }
    fn is_dead(&self) -> bool {
        self.health == 0
    }
}

struct SpaceInvaders {
    player_rect: Rect,
    bullets: Vec<Rect>,
    enemies: Vec<Enemy>,
    enemies_to_spawn: u32,
    state: GameState,
}
impl SpaceInvaders {
    fn new(enemy_count: u32) -> Self {
        Self {
            player_rect: Rect::new(0, 0, PLAYER_WIDTH, PLAYER_HEIGHT),
            bullets: Vec::new(),
            enemies: vec![Enemy::new()],
            enemies_to_spawn: enemy_count - 1,
            state: GameState::Running,
        }
    }
    fn enemies(&self) -> Iter<Enemy> {
        self.enemies.iter()
    }
    fn update_player(&mut self, left: i32, right: i32) {
        self.player_rect.set_x(self.player_rect.x() + (right - left) * PLAYER_MOVE_SPEED);
        if self.player_rect.x() + PLAYER_WIDTH as i32 > WINDOW_WIDTH as i32{
            self.player_rect.set_x((WINDOW_WIDTH - PLAYER_WIDTH) as i32);
        }
        if self.player_rect.x() < 0 {
            self.player_rect.set_x(0);
        }
    }
    fn update(&mut self) {
        self.update_enemies();
        self.update_bullets();
        self.check_hits();
        self.maybe_spawn_enemy();
    }
    fn update_enemies(&mut self) {
        for enemy in self.enemies.iter_mut() {
            enemy.rect.set_x(enemy.rect.x + enemy.dir * ENEMY_MOVE_SPEED);
            if enemy.rect.x + ENEMY_WIDTH as i32 > WINDOW_WIDTH as i32 {
                enemy.rect.set_x((WINDOW_WIDTH - ENEMY_WIDTH) as i32);
                enemy.rect.set_y(enemy.rect.y + 5 + ENEMY_HEIGHT as i32);
                enemy.dir = -1;
            }
            if enemy.rect.x < 0 {
                enemy.rect.set_x(0);
                enemy.rect.set_y(enemy.rect.y + 5 + ENEMY_HEIGHT as i32);
                enemy.dir = 1;
            }
        }
    }
    fn update_bullets(&mut self) {
        let mut i = 0;
        while i < self.bullets.len() {
            let bullet = self.bullets.get_mut(i).unwrap();
            bullet.set_y(bullet.y - BULLET_SPEED);
            if bullet.y + (BULLET_HEIGHT as i32) < 0 {
                self.bullets.remove(i);
                continue;
            }
            i += 1;
        }
    }
    fn shoot(&mut self) {
        self.bullets.push(Rect::new(
            self.player_rect.x + PLAYER_WIDTH as i32 / 2 - BULLET_WIDTH as i32,
            self.player_rect.y,
            BULLET_WIDTH,
            BULLET_HEIGHT
        ));
    }
    fn check_hits(&mut self) {
        let mut i = 0;
        while i < self.bullets.len() {
            let bullet = self.bullets.get(i).unwrap();
            for j in 0..self.enemies.len() {
                let enemy = self.enemies.get_mut(j).unwrap();
                if enemy.impact(bullet) {
                    if enemy.is_dead() {
                        self.enemies.remove(j);
                    }
                    self.bullets.remove(i);
                    break;
                }
            }
            i += 1;
        }
    }
    fn maybe_spawn_enemy(&mut self) {
        if self.enemies_to_spawn <= 0 {
            return;
        }
        match self.enemies.last() {
            Some(enemy) => {
                if enemy.rect.x > (ENEMY_WIDTH as i32) + ENEMY_DISTANT || enemy.rect.y > ENEMY_HEIGHT as i32 {
                    self.enemies.push(Enemy::new());
                    self.enemies_to_spawn -= 1;
                }
            },
            None => {
                self.enemies.push(Enemy::new());
                self.enemies_to_spawn -= 1;
            },
        }
    }
}

fn enemy_textures<'a> (canvas: &mut Canvas<Window>, texture_creator: &'a mut TextureCreator<WindowContext>, game: &SpaceInvaders) -> Result<Vec<Texture<'a>>, String> {
    let mut textures = Vec::new();
    for _ in 0..ENEMY_HEALTH {
        textures.push(
            texture_creator
                .create_texture_target(None, ENEMY_WIDTH, ENEMY_HEIGHT)
                .map_err(|e| e.to_string())?
        );
    }
    let mut texture_refs = Vec::new();
    for (i, texture_ref) in textures.iter_mut().enumerate() {
        texture_refs.push((texture_ref, i as u8));
    }
    let distance = 200 / (ENEMY_HEALTH - 1);

    canvas
        .with_multiple_texture_canvas(texture_refs.iter(), |texture_canvas, user_context| {
            texture_canvas.set_draw_color(Color::RGB(0, 0, 0));
            texture_canvas.clear();
            texture_canvas.set_draw_color(Color::RGB(200 - distance * user_context, distance * user_context, 0));
            texture_canvas
                .fill_rect(Rect::new(0, 0, ENEMY_WIDTH, ENEMY_HEIGHT))
                .expect("could not draw point");
        })
        .map_err(|e| e.to_string())?;

    Ok(textures)
}

fn dummy_texture<'a> (canvas: &mut Canvas<Window>, texture_creator: &'a mut TextureCreator<WindowContext>, game: &SpaceInvaders) -> Result<(Texture<'a>, Texture<'a>, Texture<'a>), String> {
    enum TextureKind {
        Enemy,
        Player,
        Bullet,
    }
    let mut enemy_texture = texture_creator
        .create_texture_target(None, ENEMY_WIDTH, ENEMY_HEIGHT)
        .map_err(|e| e.to_string())?;
    let mut player_texture = texture_creator
        .create_texture_target(None, PLAYER_WIDTH, PLAYER_HEIGHT)
        .map_err(|e| e.to_string())?;
    let mut bullet_texture = texture_creator
        .create_texture_target(None, BULLET_WIDTH, BULLET_HEIGHT)
        .map_err(|e| e.to_string())?;

    let textures = vec![
        (&mut enemy_texture, TextureKind::Enemy),
        (&mut player_texture, TextureKind::Player),
        (&mut bullet_texture, TextureKind::Bullet),
    ];
    canvas
        .with_multiple_texture_canvas(textures.iter(), |texture_canvas, user_context| {
            texture_canvas.set_draw_color(Color::RGB(0, 0, 0));
            texture_canvas.clear();
            match user_context {
                TextureKind::Enemy => {
                    texture_canvas.set_draw_color(Color::RGB(200, 0, 100));
                    texture_canvas
                    .fill_rect(Rect::new(0, 0, ENEMY_WIDTH, ENEMY_HEIGHT))
                    .expect("could not draw point");
                },
                TextureKind::Player => {
                    let mut ship = game.player_rect.clone();
                    ship.set_height(ship.height() - 5);
                    ship.set_y(ship.y() + 5);
                    let mut barrel = Rect::from_center(game.player_rect.center(), 5, PLAYER_HEIGHT);
                    texture_canvas.set_draw_color(Color::RGB(100, 0, 200));
                    texture_canvas
                    .fill_rect(ship)
                    .expect("could not draw point");
                    texture_canvas
                    .fill_rect(barrel)
                    .expect("could not draw point");
                },
                TextureKind::Bullet => {
                    texture_canvas.set_draw_color(Color::RGB(200, 200, 200));
                    texture_canvas
                    .fill_rect(Rect::new(0, 0, BULLET_WIDTH, BULLET_HEIGHT))
                    .expect("could not draw point");
                },
            };
        })
        .map_err(|e| e.to_string())?;

    Ok((enemy_texture, player_texture, bullet_texture))
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut window = video_subsystem
        .window(
            "SpaceInvaders",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
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
    // clears the canvas with the color we set in `set_draw_color`.
    canvas.clear();
    canvas.present();

    let mut game = SpaceInvaders::new(10);
    let mut last_shot = Instant::now();
    

    let mut texture_creator1 = canvas.texture_creator();
    let (enemy_texture, player_texture, bullet_texture) = dummy_texture(&mut canvas, &mut texture_creator1, &game)?;
    let mut texture_creator2 = canvas.texture_creator();
    let enemy_textures = enemy_textures(&mut canvas, &mut texture_creator2, &game)?;

    let mut event_pump = sdl_context.event_pump()?;
    let mut left = 0;
    let mut right = 0;
    game.player_rect.set_x((WINDOW_WIDTH / 2 - PLAYER_WIDTH / 2) as i32);
    game.player_rect.set_y((WINDOW_HEIGHT - PLAYER_HEIGHT) as i32);

    'running: loop {
        // get the inputs here
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => left = 1,
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => left = 0,
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => right = 1,
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => right = 0,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    if last_shot.elapsed().as_secs() >= COOLDOWN {
                        game.shoot();
                        last_shot = Instant::now();
                    }
                },
                _ => {},
            }
        }
        game.update_player(left, right);
        game.update();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for enemy in game.enemies() {
            canvas.copy(
                enemy_textures.get((enemy.health - 1) as usize).unwrap(),
                None,
                enemy.rect,
            )?;
        }
        for bullet in game.bullets.iter() {
            canvas.copy(
                &bullet_texture,
                None,
                *bullet,
            )?;
        }
        canvas.copy(
            &player_texture,
            None,
            game.player_rect,
        )?;

        canvas.present();
    }
    Ok(())
}
