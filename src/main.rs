use macroquad::prelude::*;
use crate::app_stage::AppStage;

mod app_stage;
mod errors;
mod screen_utils;
mod main_menu_stage;

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
    let render_target = render_target(
        screen_utils::TARGET_WIDTH as _,
        screen_utils::TARGET_HEIGHT as _
    );
    render_target.texture.set_filter(FilterMode::Nearest);
    let mut main_menu_stage = main_menu_stage::MainMenuStage::new();

    loop {
        set_camera(&Camera2D {
            zoom: vec2(2f32 / screen_utils::TARGET_WIDTH, 2f32 / screen_utils::TARGET_HEIGHT),
            target: vec2(screen_utils::TARGET_WIDTH / 2f32, screen_utils::TARGET_HEIGHT / 2f32),
            render_target: Some(render_target.clone()),
            ..Default::default()
        });

        main_menu_stage.render();

        set_default_camera();
        clear_background(BLACK);

        let scaling_factor = screen_utils::screen_scaling_factor();
        let expected_width = screen_utils::TARGET_WIDTH * scaling_factor;
        let expected_height = screen_utils::TARGET_HEIGHT * scaling_factor;
        let (origin_pos_x, origin_pos_y) = screen_utils::screen_origin_pos();

        draw_texture_ex(
            &render_target.texture,
            origin_pos_x,
            origin_pos_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(expected_width, expected_height)),
                ..Default::default()
            }
        );

        let (mouse_x, mouse_y) = screen_utils::scaled_mouse_position();
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