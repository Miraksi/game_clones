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
use crate::TILE_SIZE;

pub fn tile_textures<'a> (canvas: &mut Canvas<Window>, texture_creator: &'a mut TextureCreator<WindowContext>) -> Result<(Texture<'a>, Texture<'a>), String> {
    enum TextureKind {
        Hidden,
        Revealed
    }
    let mut hidden_texture = texture_creator
        .create_texture_target(None, TILE_SIZE, TILE_SIZE)
        .map_err(|e| e.to_string())?;
    let mut revealed_texture = texture_creator
        .create_texture_target(None, TILE_SIZE, TILE_SIZE)
        .map_err(|e| e.to_string())?;

    let textures = vec![
        (&mut hidden_texture, TextureKind::Hidden),
        (&mut revealed_texture, TextureKind::Revealed)
    ];
    canvas
        .with_multiple_texture_canvas(textures.iter(), |texture_canvas, user_context| {
            texture_canvas.set_draw_color(Color::RGB(0, 0, 0));
            texture_canvas.clear();
            match user_context {
                TextureKind::Hidden => {
                    texture_canvas.set_draw_color(Color::RGB(220, 220, 220));
                    texture_canvas
                        .fill_rect(Rect::new(0, 0, TILE_SIZE, TILE_SIZE))
                        .expect("could not draw point");
                    texture_canvas.set_draw_color(Color::RGB(170, 170, 170));
                    texture_canvas
                        .fill_rect(Rect::new(1, 1, TILE_SIZE -2, TILE_SIZE -2))
                        .expect("could not draw point");
                },
                TextureKind::Revealed => {
                    // texture_canvas.set_draw_color(Color::RGB(80, 80, 80));
                    // texture_canvas
                    //     .fill_rect(Rect::new(0, 0, TILE_SIZE, TILE_SIZE))
                    //     .expect("could not draw point");
                    texture_canvas.set_draw_color(Color::RGB(220, 220, 220));
                    texture_canvas
                        .fill_rect(Rect::new(0, 0, TILE_SIZE, TILE_SIZE))
                        .expect("could not draw point");
                },
            };
        })
        .map_err(|e| e.to_string())?;

    Ok((hidden_texture, revealed_texture))
}

pub fn text_texture<'a> (texture_creator: &'a mut TextureCreator<WindowContext>, text: &str) -> Result<(Texture<'a>, Rect), String> {
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font = ttf_context.load_font("assets/Monaco.ttf", 24)?;

    let text_surface = font.render(text)
        .solid(Color::BLACK)
        .map_err(|e| e.to_string())?;

    let menu_rect = text_surface.rect();
    let menu_texture = texture_creator
        .create_texture_from_surface(text_surface)
        .map_err(|e| e.to_string())?;
    return Ok((menu_texture, menu_rect));
}

pub fn number_textures<'a> (texture_creator: &'a mut TextureCreator<WindowContext>) -> Result<(Vec<Texture<'a>>, Rect), String> {
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font = ttf_context.load_font("assets/Monaco.ttf", 16)?;
    
    let mut number_textures = Vec::new();

    for i in 0..=9 {
        let text_surface = font.render(format!("{i}").as_str())
            .solid(Color::BLACK)
            .map_err(|e| e.to_string())?;
        
        number_textures.push(texture_creator
            .create_texture_from_surface(text_surface)
            .map_err(|e| e.to_string())?
        );
    }
    let text_surface = font.render("0")
        .solid(Color::BLACK)
        .map_err(|e| e.to_string())?;
    let surface_rect = text_surface.rect();

    Ok((number_textures, surface_rect))
}