use egui::{PopupCloseBehavior, TextEdit, Ui};
use uuid::Uuid;
use crate::app::editor_stage::{EditorStage, UpdateState};
use crate::app::editor_stage::image_widgets::{atlas_sprite_button, pivot_editor, unit_selector_button, unit_visualizer};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::units::{UnitConfig, UnitDanger};
use crate::graphics::SPRITE_ATLAS_DEF;

#[derive(Default)]
pub struct UnitConfigEditorSection {
    unit_name_filter: String,
    selected_unit_config_id: Option<Uuid>,
    selected_unit_name: String,
    current_unit_config: Option<UnitConfig>,
}

impl EditorStage {
    fn update_current_unit_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&mut String, &mut UnitConfig) -> UpdateState
    ) {
        let section = &mut self.unit_section;
        let name = &mut section.selected_unit_name;
        let cur_unit = &mut section.current_unit_config;
        
        if let Some(current_unit_config) = cur_unit {
            if foo(name, current_unit_config) == UpdateState::Changed {
                match section.selected_unit_config_id {
                    Some(id) => {
                        let config_text = json5::to_string(current_unit_config)
                            .expect("Failed to serialize unit config");
                        asset_db.update_json5_asset(AssetKind::UnitConfig, id, &config_text);
                        asset_db.rename_asset(AssetKind::UnitConfig, id, &name);
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn draw_unit_selector(&mut self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;
        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                let full_width = ui.available_width();
                let available_height = ui.available_height() - ui.spacing().interact_size.y * 6f32;

                ui.horizontal(|ui| {
                    ui.label("Фильтр:");
                    ui.add(
                        TextEdit::singleline(&mut self.unit_section.unit_name_filter)
                            .desired_width(f32::INFINITY)
                    )
                });
                ui.add_space(4f32);
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut to_delete = None;

                        let units = asset_db.list_all_assets(AssetKind::UnitConfig);
                        for (id, unit_asset_name) in units {
                            let section = &mut self.unit_section;
                            if !section.unit_name_filter.is_empty() {
                                if !unit_asset_name.starts_with(&section.unit_name_filter) {
                                    continue;
                                }
                            }

                            let selected = section
                                .selected_unit_config_id
                                .map(|it| it.eq(&id))
                                .unwrap_or(false);

                            let config_text = asset_db.load_json5_asset(AssetKind::UnitConfig, id);
                            let unit_config: UnitConfig = json5::from_str(&config_text)
                                .expect("Failed to load unit config");

                            let response = unit_selector_button(
                                ui,
                                selected,
                                texture_id,
                                atlas_size,
                                unit_asset_name,
                                &unit_config,
                            );

                            let popup_id = ui.make_persistent_id(format!("выпадающее меню {}", id));

                            if response.clicked_by(egui::PointerButton::Primary) {
                                section.current_unit_config = Some(unit_config);
                                section.selected_unit_name.clear();
                                section.selected_unit_name += unit_asset_name;
                                section.selected_unit_config_id = Some(id);
                            } else if response.clicked_by(egui::PointerButton::Secondary) {
                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                            }

                            egui::popup_below_widget(
                                ui,
                                popup_id,
                                &response,
                                PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    ui.set_min_width(100f32);
                                    if ui.button("Удалить").clicked() {
                                        to_delete = Some(id);
                                        ui.memory_mut(|mem| mem.close_popup());
                                    }
                                }
                            );
                        }

                        if let Some(id) = to_delete {
                            let section = &mut self.unit_section;
                            match section.selected_unit_config_id {
                                Some(selected_id) if selected_id.eq(&id) => {
                                    section.selected_unit_config_id = None;
                                    section.current_unit_config = None;
                                }
                                _ => {}
                            }
                            asset_db.delete_asset(AssetKind::UnitConfig, id);
                        }
                    });

                if ui.add_sized(
                    [full_width, 24f32],
                    egui::Button::new("Создать персонажа")
                ).clicked() {
                    let default_unit_config = UnitConfig::default();
                    let config_text = json5::to_string(&default_unit_config)
                        .expect("Failed to serialize default unit config");

                    let section = &mut self.unit_section;
                    section.current_unit_config = Some(default_unit_config);

                    let id = asset_db.create_json5_asset(
                        AssetKind::UnitConfig,
                        "",
                        &config_text
                    );
                    section.selected_unit_name.clear();
                    section.selected_unit_config_id = Some(id);
                }

            }
            _ => {}
        }
    }

    pub(crate) fn draw_unit_preview_in_level(&self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;
        if let Some(unit_config) = &self.unit_section.current_unit_config {
            if unit_config.sprite_name.is_empty() {
                return;
            }
            ui.vertical(|ui| {
                ui.add_space(6f32);
                ui.group(|ui| {
                    ui.label("Предпросмотр на игровом поле:");
                    unit_visualizer(
                        ui,
                        texture_id,
                        atlas_size,
                        unit_config
                    )
                })
            });
        }
    }

    pub(crate) fn draw_unit_editor(&mut self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;

        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                self.update_current_unit_config(&mut asset_db, |unit_name, current_unit_config| {
                    let mut update_state = UpdateState::Unchanged;
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Название для редактора:");
                                if ui.add(TextEdit::singleline(unit_name).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Название:");
                                if ui.add(TextEdit::singleline(&mut current_unit_config.name).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Описание:");
                                if ui.add(TextEdit::multiline(&mut current_unit_config.description).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Изображение:");
                                let full_width = ui.available_width();

                                let response = ui.add_sized(
                                    [full_width, ui.spacing().interact_size.y],
                                    egui::Button::new(&current_unit_config.sprite_name),
                                );
                                let popup_id = ui.make_persistent_id("выбор изображения");
                                if response.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                egui::popup_below_widget(
                                    ui,
                                    popup_id,
                                    &response,
                                    PopupCloseBehavior::CloseOnClickOutside,
                                    |ui| {
                                        const COLUMNS_COUNT: usize = 6;
                                        ui.columns(COLUMNS_COUNT, |uis| {
                                            let mut current_column = 0;

                                            for sprite in crate::graphics::SPRITE_ATLAS_DEF.sprites.keys() {
                                                let sprite_name = sprite.as_str();

                                                let ui = &mut uis[current_column];
                                                ui.add_space(4f32);
                                                let response = atlas_sprite_button(
                                                    ui,
                                                    texture_id,
                                                    atlas_size,
                                                    sprite_name,
                                                    96f32,
                                                );

                                                if response.clicked() {
                                                    current_unit_config.sprite_name.clear();
                                                    current_unit_config.sprite_name += sprite_name;
                                                    update_state = UpdateState::Changed;
                                                    ui.memory_mut(|mem| mem.close_popup());
                                                }

                                                current_column = (current_column + 1) % COLUMNS_COUNT;
                                            }
                                        });
                                    },
                                );
                            });
                            let entry = SPRITE_ATLAS_DEF
                                .sprites
                                .get(&current_unit_config.sprite_name);
                            match entry {
                                None => {}
                                Some(sprite_data) => {
                                    let w = ui.available_width();
                                    let zoom = if sprite_data.size[0] == 0 {
                                        1f32
                                    } else {
                                        w / (sprite_data.size[0] as f32 * 16f32)
                                    };
                                    let old_pivot = current_unit_config.sprite_pivot;
                                    pivot_editor(
                                        ui,
                                        texture_id,
                                        atlas_size,
                                        current_unit_config,
                                        zoom,
                                    );
                                    if !old_pivot.eq(&current_unit_config.sprite_pivot) {
                                        update_state = UpdateState::Changed;
                                    }

                                    ui.horizontal(|ui| {
                                        ui.label("Опорная точка:");

                                        let available_width = ui.available_width();
                                        let slider_width = available_width / 2f32;

                                        if ui
                                            .add_sized(
                                                [slider_width, ui.spacing().interact_size.y],
                                                egui::Slider::new(
                                                    &mut current_unit_config.sprite_pivot[0],
                                                    0..=sprite_data.size[0] * 16 - 1,
                                                )
                                            )
                                            .changed()
                                        {
                                            update_state = UpdateState::Changed;
                                        }
                                        if ui
                                            .add_sized(
                                                [slider_width, ui.spacing().interact_size.y],
                                                egui::Slider::new(
                                                    &mut current_unit_config.sprite_pivot[1],
                                                    0..=sprite_data.size[1] * 16 - 1,
                                                )
                                            )
                                            .changed()
                                        {
                                            update_state = UpdateState::Changed;
                                        }
                                    });
                                }
                            }

                            ui.horizontal(|ui| {
                                ui.label("Опасность:");

                                egui::ComboBox::from_id_salt("danger")
                                    .selected_text(current_unit_config.danger.display_name())
                                    .show_ui(ui, |ui| {
                                        for v in [
                                            UnitDanger::Harmless,
                                            UnitDanger::Weak,
                                            UnitDanger::Moderate,
                                            UnitDanger::Challenging,
                                            UnitDanger::Horror,
                                            UnitDanger::Nightmare
                                        ] {
                                            if ui.selectable_value(
                                                &mut current_unit_config.danger,
                                                v,
                                                v.display_name()
                                            ).clicked() {
                                                update_state = UpdateState::Changed;
                                            }
                                        }
                                    });
                                if ui.add(
                                    egui::Checkbox::new(&mut current_unit_config.is_catchable, "Можно поймать")
                                ).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            })

                        })
                    });

                    update_state
                });
            }
            _ => {}
        }
    }

}