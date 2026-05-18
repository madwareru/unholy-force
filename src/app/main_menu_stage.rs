use std::{
    collections::HashMap,
    sync::Arc
};
use lazy_static::lazy_static;
use macroquad::prelude::*;
use serde::Deserialize;
use super::app_stage::*;
use crate::screen_utils;

#[derive(Copy, Clone, Deserialize)]
pub enum MainMenuCommand {
    OpenOldGame,
    StartNewGame,
    OpenEditor,
    Exit,
    VisitGithub,
    VisitGamedev,
    VisitTelegram,
    VisitVK,
    VisitMastodon,
    LeaveFeedback,
    Donate
}

#[derive(Deserialize)]
struct AtlasLabelDef {
    size: [u8; 2],
    coords: [u8; 2],
}

#[derive(Deserialize)]
struct MainMenuBindingDef {
    command: MainMenuCommand,
    #[serde(default)]
    condition: Option<String>,
}

#[derive(Deserialize)]
struct AtlasButtonDef {
    size: [u8; 2],
    states: HashMap<String, [u8; 2]>
}

#[derive(Deserialize)]
struct AtlasDef {
    tile_size: [u8; 2],
    labels: HashMap<String, AtlasLabelDef>,
    buttons: HashMap<String, AtlasButtonDef>,
}

#[derive(Deserialize)]
struct MenuLayoutDef {
    labels: HashMap<String, [u8; 2]>,
    buttons: HashMap<String, [u8; 2]>,
}

lazy_static!(
    static ref ATLAS_TEXTURE: Texture2D = {
        let main_menu_texture = Texture2D::from_file_with_format(
            include_bytes!("../../assets/main_menu_atlas.png"),
            None
        );
        main_menu_texture.set_filter(FilterMode::Nearest);
        main_menu_texture
    };

    static ref ATLAS_DEF: Arc<AtlasDef> = {
        const CONFIG_TEXT: &str = include_str!("../../assets/main_menu_atlas.json5");
        let atlas_def = json5::from_str(CONFIG_TEXT);
        Arc::new(atlas_def.expect("Failed to load atlas json"))
    };

    static ref MENU_LAYOUT: Arc<MenuLayoutDef> = {
        const CONFIG_TEXT: &str = include_str!("../../assets/main_menu_layout.json5");
        let layout_def = json5::from_str(CONFIG_TEXT);
        Arc::new(layout_def.expect("Failed to load layout json"))
    };

    static ref MENU_BINDINGS: Arc<HashMap<String, MainMenuBindingDef>> = {
        const CONFIG_TEXT: &str = include_str!("../../assets/main_menu_bindings.json5");
        let binding_def = json5::from_str(CONFIG_TEXT);
        Arc::new(binding_def.expect("Failed to load main menu bindings"))
    };
);

pub struct MainMenuStage {
    main_menu_texture: Texture2D,
    labels: HashMap<String, (i32, i32)>,
    buttons: HashMap<String, (i32, i32)>,
    button_hitboxes: HashMap<String, ((i32, i32), (i32, i32))>,
}

impl MainMenuStage {
    pub fn new() -> Self {
        let main_menu_texture = ATLAS_TEXTURE.clone();

        let mut labels: HashMap<String, (i32, i32)> = HashMap::new();
        let mut buttons: HashMap<String, (i32, i32)> = HashMap::new();
        let mut button_hitboxes: HashMap<String, ((i32, i32), (i32, i32))> = HashMap::new();

        for label_def in ATLAS_DEF.labels.iter() {
            let label = label_def.0.as_str();
            match MENU_LAYOUT.labels.get(label) {
                None => { continue; }
                Some(&[x, y]) => {
                    let (x, y) = (
                        x as i32 * ATLAS_DEF.tile_size[0] as i32,
                        y as i32 * ATLAS_DEF.tile_size[1] as i32
                    );
                    labels.insert(label.to_owned(), (x, y));
                }
            }
        }

        for button_def in ATLAS_DEF.buttons.iter() {
            let button = button_def.0.as_str();
            match MENU_LAYOUT.buttons.get(button) {
                None => { continue; }
                Some(&[x, y]) => {
                    let (x, y) = (
                        x as i32 * ATLAS_DEF.tile_size[0] as i32,
                        y as i32 * ATLAS_DEF.tile_size[1] as i32
                    );
                    let hitbox_left = x + ATLAS_DEF.tile_size[0] as i32;
                    let hitbox_w = (button_def.1.size[0] as i32 - 2) * ATLAS_DEF.tile_size[0] as i32;
                    let hitbox_top = y + ATLAS_DEF.tile_size[1] as i32;
                    let hitbox_h = (button_def.1.size[1] as i32 - 2) * ATLAS_DEF.tile_size[1] as i32;
                    let hitbox_right = hitbox_left + hitbox_w;
                    let hitbox_bottom = hitbox_top + hitbox_h;
                    buttons.insert(button.to_owned(), (x, y));
                    button_hitboxes.insert(button.to_owned(), (
                        (hitbox_left, hitbox_top),
                        (hitbox_right, hitbox_bottom)
                    ));
                }
            }
        }

        Self {
            main_menu_texture,
            labels,
            buttons,
            button_hitboxes
        }
    }

    fn is_button_active(&self, button_name: &str) -> bool {
        MENU_BINDINGS
        .get(button_name)
        .and_then(|btn_def| btn_def.condition.as_ref())
        .map(|cond| self.check_condition(cond.as_str()))
        .unwrap_or(true)
    }

    fn check_condition(&self, cond: &str) -> bool {
        match cond {
            // todo: activate donates when there is a reason to do so
            "donate_active" => false,
            "save_exists" => std::fs::exists("save.dat").unwrap_or(false),
            _ => false
        }
    }

    fn is_button_hovered(&self, button: &str) -> bool {
        let (mouse_x, mouse_y) = screen_utils::scaled_mouse_position();
        let Some(hit_info) = self.button_hitboxes.get(button) else {
            return false;
        };

        (mouse_x >= (hit_info.0.0 as f32) && mouse_x <= (hit_info.1.0 as f32)) &&
            (mouse_y >= (hit_info.0.1 as f32) && mouse_y <= (hit_info.1.1 as f32))
    }

    pub fn process(&mut self) -> AppStageStatus<MainMenuCommand> {
        if is_mouse_button_released(MouseButton::Left) {
            for (button_name, _) in self.buttons.iter() {
                if !self.is_button_active(button_name.as_str()) {
                    continue;
                }

                if self.is_button_hovered(button_name.as_str()) {
                    return AppStageStatus::Complete(
                        MENU_BINDINGS
                            .get(button_name.as_str())
                            .map(|it| it.command)
                            .unwrap_or_else(|| {
                                let error_text = format!("Command for button {button_name} not found");
                                panic!("{error_text}");
                            })
                    )
                }
            }
        }
        AppStageStatus::Continue
    }

    pub fn render(&self) {
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        draw_texture_ex(
            &self.main_menu_texture,
            0f32,
            0f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_utils::TARGET_WIDTH, screen_utils::TARGET_HEIGHT)),
                source: Some(Rect {
                    x: 0.0,
                    y: screen_utils::TARGET_HEIGHT,
                    w: screen_utils::TARGET_WIDTH,
                    h: screen_utils::TARGET_HEIGHT,
                }),
                ..Default::default()
            }
        );

        for (label_name, (x, y)) in self.labels.iter() {
            match ATLAS_DEF.labels.get(label_name.as_str()) {
                None => { continue; }
                Some(label_def) => {
                    let sub_rect_x = label_def.coords[0] as u32 * ATLAS_DEF.tile_size[0] as u32;
                    let sub_rect_y = label_def.coords[1] as u32 * ATLAS_DEF.tile_size[1] as u32;
                    let sub_rect_w = label_def.size[0] as u32 * ATLAS_DEF.tile_size[0] as u32;
                    let sub_rect_h = label_def.size[1] as u32 * ATLAS_DEF.tile_size[1] as u32;

                    let expected_width = sub_rect_w as f32;
                    let expected_height = sub_rect_h as f32;

                    draw_texture_ex(
                        &self.main_menu_texture,
                        *x as f32,
                        *y as f32,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(expected_width, expected_height)),
                            source: Some(Rect {
                                x: sub_rect_x as _,
                                y: sub_rect_y as _,
                                w: sub_rect_w as _,
                                h: sub_rect_h as _,
                            }),
                            ..Default::default()
                        }
                    );
                }
            }
        }

        for (button_name, (x, y)) in self.buttons.iter() {
            let hovered = self.is_button_hovered(&button_name);

            let key =
                if hovered && mouse_down { "clicked" } else if hovered { "hover" } else { "idle" };

            match ATLAS_DEF.buttons.get(button_name.as_str()) {
                None => { continue; }
                Some(btn_def) => {
                    if !self.is_button_active(button_name.as_str()) {
                        continue;
                    }
                    let sub_rect_x = btn_def.states[key][0] as u32 * ATLAS_DEF.tile_size[0] as u32;
                    let sub_rect_y = btn_def.states[key][1] as u32 * ATLAS_DEF.tile_size[1] as u32;
                    let sub_rect_w = btn_def.size[0] as u32 * ATLAS_DEF.tile_size[0] as u32;
                    let sub_rect_h = btn_def.size[1] as u32 * ATLAS_DEF.tile_size[1] as u32;

                    let expected_width = sub_rect_w as f32;
                    let expected_height = sub_rect_h as f32;

                    draw_texture_ex(
                        &self.main_menu_texture,
                        *x as f32,
                        *y as f32,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(expected_width, expected_height)),
                            source: Some(Rect {
                                x: sub_rect_x as _,
                                y: sub_rect_y as _,
                                w: sub_rect_w as _,
                                h: sub_rect_h as _,
                            }),
                            ..Default::default()
                        }
                    );
                }
            }
        }
    }
}