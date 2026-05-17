use macroquad::prelude::*;
use crate::app::app_stage::AppStageStatus;

mod errors;
mod screen_utils;
mod app;

fn window_conf() -> Conf {
    let (w, h) = if std::env::args().find(|it| it.starts_with("unsized")).is_some() {
        (640, 360)
    } else {
        (1280, 720)
    };
    let conf = Conf {
        window_title: "Нечистая сила".to_owned(),
        high_dpi: false,
        window_width: w,
        window_height: h,
        ..Default::default()
    };

    if std::env::args().find(|it| it.starts_with("windowed")).is_some() {
        Conf { fullscreen: false, window_resizable: false, ..conf }
    } else {
        Conf { fullscreen: true, ..conf }
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), errors::GameError> {
    let rt = render_target(
        screen_utils::TARGET_WIDTH as u32,
        screen_utils::TARGET_HEIGHT as u32
    );
    rt.texture.set_filter(FilterMode::Nearest);

    let mut app = app::App::new();

    loop {
        match app.process() {
            AppStageStatus::Continue => {}
            AppStageStatus::Complete(()) => {
                break;
            }
        }

        set_camera(&Camera2D {
            zoom: vec2(2f32 / screen_utils::TARGET_WIDTH, 2f32 / screen_utils::TARGET_HEIGHT),
            target: vec2(screen_utils::TARGET_WIDTH / 2f32, screen_utils::TARGET_HEIGHT / 2f32),
            render_target: Some(rt.clone()),
            ..Default::default()
        });
        clear_background(BLACK);
        app.render();

        set_default_camera();
        clear_background(BLACK);

        let scaling_factor = screen_utils::screen_scaling_factor();
        let expected_width = screen_utils::TARGET_WIDTH * scaling_factor;
        let expected_height = screen_utils::TARGET_HEIGHT * scaling_factor;
        let (origin_pos_x, origin_pos_y) = screen_utils::screen_origin_pos();

        draw_texture_ex(
            &rt.texture,
            origin_pos_x,
            origin_pos_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(expected_width, expected_height)),
                ..Default::default()
            }
        );
        next_frame().await;
    }
    Ok(())
}