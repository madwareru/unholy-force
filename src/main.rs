use macroquad::miniquad::window::dpi_scale;
use macroquad::prelude::*;

mod errors;

fn window_conf() -> Conf {
    Conf {
        window_title: "Нечистая сила".to_owned(),
        high_dpi: false,
        window_width: 1280,
        window_height: 720,
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), errors::GameError> {
    let main_menu_texture = Texture2D::from_file_with_format(
        include_bytes!("../assets/main_menu_atlas.png"),
        None
    );
    main_menu_texture.set_filter(FilterMode::Nearest);

    loop {
        clear_background(BLACK);

        draw_texture_ex(
            &main_menu_texture,
            (screen_width() - 1280.0) / 2.0,
            (screen_height() - 720.0) / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(1280.0, 720.0)),
                source: Some(Rect {
                    x: 0.0,
                    y: 360.0,
                    w: 640.0,
                    h: 360.0,
                }),
                ..Default::default()
            }
        );

        next_frame().await;

        if should_quit() {
            break;
        }
    }
    Ok(())
}

fn should_quit() -> bool {
    is_key_released(KeyCode::Escape)
}