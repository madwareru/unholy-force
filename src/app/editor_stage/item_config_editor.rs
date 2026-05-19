use egui::{PointerButton, PopupCloseBehavior, Ui};
use macroquad::math::Rect;
use crate::app::editor_stage::{thick_selector_button, EditorStage};
use crate::app::editor_stage::reusable_image_widget::{pivot_editor, AtlasSpriteRect};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::items::ItemConfig;
use crate::graphics::{SpriteGraphicsDef, SPRITE_ATLAS_DEF, SPRITE_ATLAS_TEXTURE};

#[derive(Copy, Clone, PartialEq)]
enum UpdateState {
    Unchanged,
    Changed,
}

impl EditorStage {

    fn update_current_item_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&mut ItemConfig) -> UpdateState
    ) {
        if let Some(current_item_config) = &mut self.current_item_config {
            if foo(current_item_config) == UpdateState::Changed {
                match self.selected_item_config_id{
                    Some(id) => {
                        let config_text = json5::to_string(current_item_config)
                            .expect("Failed to serialize default item config");
                        asset_db.update_json5_asset(AssetKind::ItemConfig, id, &config_text);
                    }
                    _ => {}
                }
            }
        }
    }

    fn load_current_item_config(&mut self, asset_db: &AssetDb) {
        match (self.selected_item_config_id) {
            (Some(id)) => {
                let config_text = asset_db.load_json5_asset(AssetKind::ItemConfig, id);
                self.current_item_config = Some(json5::from_str(&config_text).expect("Failed to load json5"));
            }
            _ => {}
        }
    }

    pub(crate) fn draw_item_selector(&mut self, ui: &mut Ui) {
        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                let full_width = ui.available_width();
                ui.add_sized(
                    [full_width, 24f32],
                    egui::TextEdit::singleline(&mut self.new_asset_name)
                );
                if ui.add_sized(
                    [full_width, 24f32],
                    egui::Button::new("+")
                ).clicked() {
                    let mut default_item_config = crate::game_config::items::ItemConfig::default();
                    default_item_config.name = self.new_asset_name.clone();
                    let config_text = json5::to_string(&default_item_config)
                        .expect("Failed to serialize default item config");

                    self.current_item_config = Some(default_item_config);

                    let id =asset_db.create_json5_asset(
                        AssetKind::ItemConfig,
                        &self.new_asset_name,
                        &config_text
                    );
                    self.selected_item_config_id = Some(id);
                    self.new_asset_name.clear();
                }
                ui.add_space(12f32);
                egui::ScrollArea::vertical()
                    .max_height(400f32)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut to_delete = None;
                        {
                            let items = asset_db.list_all_assets(AssetKind::ItemConfig);
                            for (id, item_asset_name) in items {
                                ui.horizontal(|ui|{
                                    {
                                        let selected = self.selected_item_config_id
                                            .map(|it| it.eq(&id))
                                            .unwrap_or(false);

                                        let response = thick_selector_button(
                                            ui,
                                            selected,
                                            egui::Align2::LEFT_CENTER,
                                            item_asset_name
                                        );
                                        let popup_id = ui
                                            .make_persistent_id(format!("выпадающее меню {}", id));

                                        if response.clicked_by(PointerButton::Primary) {
                                            self.selected_item_config_id = Some(id);
                                            self.load_current_item_config(&asset_db);
                                        } else if response.clicked_by(PointerButton::Secondary) {
                                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                        }

                                        egui::popup_below_widget(ui, popup_id, &response, PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                            ui.set_min_width(100f32);
                                            if ui.button("Удалить").clicked() {
                                                to_delete = Some(id);
                                                ui.memory_mut(|mem| mem.close_popup());
                                            }
                                        });
                                    }
                                });
                            }
                        }
                        if let Some(id) = to_delete {
                            match self.selected_item_config_id {
                                Some(selected_id) if selected_id.eq(&id) => {
                                    self.selected_item_config_id = None;
                                    self.current_item_config = None;
                                }
                                _ => {}
                            }
                            asset_db.delete_asset(AssetKind::ItemConfig, id);
                        }
                    });
            },
            _ => {}
        };
    }

    pub(crate) fn draw_item_editor(&mut self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };

        match (crate::assets::ASSET_DATABASE.lock()) {
            Ok(mut asset_db) => {
                self.update_current_item_config(&mut asset_db, |current_item_config| {
                    let mut update_state = UpdateState::Unchanged;
                    ui.columns(2, |uis| {
                        uis[0].horizontal(|ui| {
                            ui.label("Название:");
                            if ui.add(egui::TextEdit::singleline(&mut current_item_config.name)
                                .desired_width(f32::INFINITY)).changed() {
                                update_state = UpdateState::Changed;
                            }
                        });
                        uis[0].horizontal(|ui| {
                            ui.label("Описание:");
                            if ui.add(egui::TextEdit::multiline(&mut current_item_config.description).desired_width(f32::INFINITY)).changed() {
                                update_state = UpdateState::Changed;
                            }
                        });

                        uis[0].horizontal(|ui| {
                            ui.label("Изображение:");
                            ui.add(egui::TextEdit::singleline(&mut current_item_config.sprite_name)
                                .desired_width(f32::INFINITY)
                                .interactive(false));
                            let response = ui.button("...");
                            let popup_id = ui.make_persistent_id("выбор изображения");
                            if response.clicked() {
                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                            }
                            egui::popup_below_widget(ui, popup_id, &response, PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                ui.set_min_width(200f32);

                                let sprite_names = crate::graphics::SPRITE_ATLAS_DEF
                                    .sprites
                                    .keys();

                                for sprite_name in sprite_names {
                                    if ui.button(sprite_name.as_str()).clicked() {

                                        current_item_config.sprite_name = sprite_name.clone();
                                        update_state = UpdateState::Changed;
                                        ui.memory_mut(|mem| mem.close_popup());
                                        break;
                                    }
                                }
                            });
                        });
                        let entry = SPRITE_ATLAS_DEF.sprites.get(&current_item_config.sprite_name);
                        match entry {
                            None => {}
                            Some(sprite_data) => {
                                let zoom = match (sprite_data.size[0]).max(sprite_data.size[1]) {
                                    1 => 25f32,
                                    2 => 12f32,
                                    3 => 8f32,
                                    4 => 6f32,
                                    5 => 8f32,
                                    6 => 4f32,
                                    7 => 3f32,
                                    8 => 3f32,
                                    9 => 2f32,
                                    10 => 2f32,
                                    _ => 1f32
                                };
                                let [x, y] = sprite_data.coords.map(|it| it as u16 * 16);
                                let [w, h] = sprite_data.size.map(|it| it as u16 * 16);
                                let old_pivot = current_item_config.sprite_pivot;
                                pivot_editor(
                                    &mut uis[0],
                                    texture_id,
                                    AtlasSpriteRect {
                                        atlas_size: [640f32, 360f32].into(),
                                        rect_px: egui::Rect::from_min_max(
                                            [x as f32, y as f32].into(),
                                            [(x + w) as f32, (y + h) as f32].into()
                                        ),
                                    },
                                    &mut current_item_config.sprite_pivot,
                                    zoom
                                );
                                if !old_pivot.eq(&current_item_config.sprite_pivot) {
                                    update_state = UpdateState::Changed;
                                }
                            }
                        }
                    });
                    update_state
                });
            },
            _ => {}
        };

    }
}