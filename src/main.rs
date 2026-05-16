use macroquad::prelude::*;

mod errors;
mod screen_utils;

fn window_conf() -> Conf {
    let (w, h) = if std::env::args().find(|it| it.starts_with("unsized")).is_some() {
        (640, 360)
    } else {
        (1280, 720)
    };
    if std::env::args().find(|it| it.starts_with("windowed")).is_some() {
        Conf {
            window_title: "Нечистая сила".to_owned(),
            high_dpi: false,
            window_width: w,
            window_height: h,
            fullscreen: false,
            window_resizable: false,
            ..Default::default()
        }
    } else {
        Conf {
            window_title: "Нечистая сила".to_owned(),
            high_dpi: false,
            window_width: w,
            window_height: h,
            fullscreen: true,
            ..Default::default()
        }
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

        let scaling_factor = screen_utils::screen_scaling_factor();
        let expected_width = screen_utils::TARGET_WIDTH * scaling_factor;
        let expected_height = screen_utils::TARGET_HEIGHT * scaling_factor;

        let (origin_pos_x, origin_pos_y) = screen_utils::screen_origin_pos();

        let (mouse_x, mouse_y) = screen_utils::scaled_mouse_position();

        draw_texture_ex(
            &main_menu_texture,
            origin_pos_x,
            origin_pos_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(expected_width, expected_height)),
                source: Some(Rect {
                    x: 0.0,
                    y: 360.0,
                    w: 640.0,
                    h: 360.0,
                }),
                ..Default::default()
            }
        );

        draw_text(&format!("{}, {}", mouse_x, mouse_y), 16., 16., 16., WHITE);

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