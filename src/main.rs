use macroquad::prelude::*;
use crate::app_stage::*;
use crate::main_menu_stage::{MainMenuCommand, MainMenuStage};

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
    let rt = render_target(
        screen_utils::TARGET_WIDTH as u32,
        screen_utils::TARGET_HEIGHT as u32
    );
    rt.texture.set_filter(FilterMode::Nearest);

    let mut main_menu_stage = main_menu_stage::MainMenuStage::new();
    let mut app_stage = AppStage::MainMenu;

    loop {
        { // all stage logic are occur here
            match app_stage {
                AppStage::MainMenu => {
                    if !process_main_menu(&mut main_menu_stage, &mut app_stage) { break; }
                }
                _ => {}
            }
        }

        { // all normal stage renders are occur here
            set_camera(&Camera2D {
                zoom: vec2(2f32 / screen_utils::TARGET_WIDTH, 2f32 / screen_utils::TARGET_HEIGHT),
                target: vec2(screen_utils::TARGET_WIDTH / 2f32, screen_utils::TARGET_HEIGHT / 2f32),
                render_target: Some(rt.clone()),
                ..Default::default()
            });

            match app_stage {
                AppStage::MainMenu => {
                    main_menu_stage.render();
                },
                AppStage::Game => todo!(),
                AppStage::OldGame => todo!(),
                AppStage::Editor => todo!()
            }
        }

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

        let (mouse_x, mouse_y) = screen_utils::scaled_mouse_position();
        draw_text(&format!("{}, {}", mouse_x, mouse_y), 16., 16., 16., WHITE);

        next_frame().await;

        if should_quit() {
            break;
        }
    }
    Ok(())
}

fn process_main_menu(main_menu_stage: &mut MainMenuStage, app_stage: &mut AppStage) -> bool {
    match main_menu_stage.process() {
        AppStageStatus::Continue => {},
        AppStageStatus::Complete(command) => {
            match command {
                MainMenuCommand::OpenOldGame => {
                    *app_stage = AppStage::OldGame;
                }
                MainMenuCommand::StartNewGame => {
                    *app_stage = AppStage::Game;
                }
                MainMenuCommand::OpenEditor => {
                    *app_stage = AppStage::Editor;
                }
                MainMenuCommand::Exit => {
                    return false;
                }
                MainMenuCommand::VisitGithub => {
                    webbrowser::open("https://github.com/madwareru/unholy-force")
                        .unwrap();
                }
                MainMenuCommand::VisitGamedev => {
                    webbrowser::open("https://gamedev.ru/users/?id=41788")
                        .unwrap();
                }
                MainMenuCommand::VisitTelegram => {
                    webbrowser::open("https://t.me/obscure_computer_science")
                        .unwrap();
                }
                MainMenuCommand::VisitVK => {
                    webbrowser::open("https://vk.com/madware")
                        .unwrap();
                }
                MainMenuCommand::VisitMastodon => {
                    webbrowser::open("https://mastodon.gamedev.place/@madware")
                        .unwrap();
                }
                MainMenuCommand::LeaveFeedback => {
                    webbrowser::open("https://github.com/madwareru/unholy-force/issues/new/choose")
                        .unwrap();
                }
                MainMenuCommand::Donate => {
                    // todo
                }
            }
        }
    }
    true
}

fn should_quit() -> bool {
    is_key_released(KeyCode::Escape)
}