use egui::{PopupCloseBehavior, TextEdit, Ui};
use uuid::Uuid;
use crate::app::editor_stage::{EditorStage, UpdateState};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::parameters::{TagConfig, PARAMETER_CACHE};
use crate::app::editor_stage::image_widgets::{sprite_pivot_editor, tag_selector_button};
use crate::game_config::ConfigId;
use crate::graphics::SPRITE_ATLAS_DEF;

#[derive(Default)]
pub struct TagConfigEditorSection {
    tag_name_filter: String,
    selected_tag_config_id: Option<Uuid>,
    selected_tag_name: String,
    current_tag_config: Option<TagConfig>,
}

impl EditorStage {
    fn update_current_tag_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&AssetDb, ConfigId<TagConfig>, &mut String, &mut TagConfig) -> UpdateState
    ) {
        let section = &mut self.tag_section;
        let name = &mut section.selected_tag_name;
        let cur_tag = &mut section.current_tag_config;

        if let (Some(uuid), Some(current_tag_config)) = (section.selected_tag_config_id, cur_tag) {
            if foo(asset_db, ConfigId::from_uuid(uuid), name, current_tag_config) == UpdateState::Changed {
                match section.selected_tag_config_id {
                    Some(id) => {
                        asset_db.update_asset_mut(
                            AssetKind::TagConfig,
                            id,
                            |buffer| json5::to_writer(buffer, &current_tag_config)
                        );
                        asset_db.rename_asset(AssetKind::TagConfig, id, &name);
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn draw_tag_selector(&mut self, ui: &mut Ui) {
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
                        TextEdit::singleline(&mut self.tag_section.tag_name_filter)
                            .desired_width(f32::INFINITY)
                    )
                });
                ui.add_space(4f32);
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut to_delete = None;

                        let tags = asset_db.list_all_assets(AssetKind::TagConfig);
                        for (id, tag_asset_name) in tags {
                            let section = &mut self.tag_section;
                            if !section.tag_name_filter.is_empty() {
                                if !tag_asset_name.starts_with(&section.tag_name_filter) {
                                    continue;
                                }
                            }

                            let selected = section
                                .selected_tag_config_id
                                .map(|it| it.eq(&id))
                                .unwrap_or(false);

                            let config_text = asset_db.load_json5_asset(AssetKind::TagConfig, id);
                            let tag_config: TagConfig = json5::from_str(&config_text)
                                .expect("Failed to load tag config");

                            let response = tag_selector_button(
                                ui,
                                selected,
                                texture_id,
                                atlas_size,
                                tag_asset_name,
                                &tag_config,
                            );

                            let popup_id = ui.make_persistent_id(format!("выпадающее меню {}", id));

                            if response.clicked_by(egui::PointerButton::Primary) {
                                section.current_tag_config = Some(tag_config);
                                section.selected_tag_name.clear();
                                section.selected_tag_name += tag_asset_name;
                                section.selected_tag_config_id = Some(id);
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
                            let section = &mut self.tag_section;
                            match section.selected_tag_config_id {
                                Some(selected_id) if selected_id.eq(&id) => {
                                    if let Some(current_config) = &section.current_tag_config {
                                        if let Ok(mut cache) = PARAMETER_CACHE.lock() {
                                            cache.flush_tag_id(&current_config.bound_name);
                                        }
                                    }

                                    section.selected_tag_config_id = None;
                                    section.current_tag_config = None;
                                }
                                _ => {
                                    if let Ok(mut cache) = PARAMETER_CACHE.lock() {
                                        let config_text = asset_db.load_json5_asset(AssetKind::TagConfig, id);
                                        let config_to_delete: TagConfig = json5::from_str(&config_text)
                                            .expect("Failed to load tag config");
                                        cache.flush_tag_id(&config_to_delete.bound_name);
                                    }
                                }
                            }
                            asset_db.delete_asset(AssetKind::TagConfig, id);
                        }
                    });

                if ui.add_sized(
                    [full_width, 24f32],
                    egui::Button::new("Создать лычку")
                ).clicked() {
                    let default_tag_config = TagConfig::default();
                    let config_text = json5::to_string(&default_tag_config)
                        .expect("Failed to serialize default tag config");

                    let section = &mut self.tag_section;
                    section.current_tag_config = Some(default_tag_config);

                    let id = asset_db.create_json5_asset(
                        AssetKind::TagConfig,
                        "",
                        &config_text
                    );
                    section.selected_tag_name.clear();
                    section.selected_tag_config_id = Some(id);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn draw_tag_editor(&mut self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;

        // let effect_entries: Vec<EffectMechanicEntry> =
        //
        //
        //     if let Ok(db) = crate::assets::ASSET_DATABASE.lock() {
        //
        //
        //         db.list_all_assets(AssetKind::EffectConfig)
        //             .map(|(uuid, _)| {
        //                 let text = db.load_json5_asset(AssetKind::EffectConfig, uuid);
        //                 let config: EffectConfig = json5::from_str(&text)
        //                     .expect("Failed to load effect mechanic config");
        //                 EffectMechanicEntry {
        //                     uuid,
        //                     label: format!("{:?}", config.mechanic_name),
        //                 }
        //             })
        //             .collect()
        //     } else {
        //         Vec::new()
        //     };

        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                self.update_current_tag_config(&mut asset_db, |asset_db, current_config_id, tag_name, current_tag_config| {
                    let mut update_state = UpdateState::Unchanged;
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Название для редактора:");
                                if ui.add(TextEdit::singleline(tag_name).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Имя для формул:");
                                let old_name = current_tag_config.bound_name.clone();
                                if ui.add(TextEdit::singleline(&mut current_tag_config.bound_name).desired_width(f32::INFINITY)).changed() {
                                    if let Ok(mut cache) = PARAMETER_CACHE.lock() {
                                        cache.flush_tag_id(&old_name);
                                    }
                                    update_state = UpdateState::Changed;
                                }
                            });
                            let duplicate_errors = current_tag_config
                                .check_duplicate_bound_name(current_config_id, asset_db);
                            if !duplicate_errors.is_empty() {
                                ui.colored_label(egui::Color32::RED, duplicate_errors);
                            }
                            ui.horizontal(|ui| {
                                ui.label("Название:");
                                if ui.add(TextEdit::singleline(&mut current_tag_config.name).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Описание:");
                                if ui.add(TextEdit::multiline(&mut current_tag_config.description).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Изображение:");
                                let full_width = ui.available_width();

                                let response = ui.add_sized(
                                    [full_width, ui.spacing().interact_size.y],
                                    egui::Button::new(&current_tag_config.sprite_name),
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

                                            for sprite in crate::graphics::SPRITE_ATLAS_DEF.sprite_keys() {
                                                let sprite_name = sprite.as_str();

                                                let ui = &mut uis[current_column];
                                                ui.add_space(4f32);
                                                let response = crate::app::editor_stage::image_widgets::atlas_sprite_button(
                                                    ui,
                                                    texture_id,
                                                    atlas_size,
                                                    sprite_name,
                                                    96f32,
                                                );

                                                if response.clicked() {
                                                    current_tag_config.sprite_name.clear();
                                                    current_tag_config.sprite_name += sprite_name;
                                                    update_state = UpdateState::Changed;
                                                    ui.memory_mut(|mem| mem.close_popup());
                                                }

                                                current_column = (current_column + 1) % COLUMNS_COUNT;
                                            }
                                        });
                                    },
                                );
                            });

                            let sprite_data = SPRITE_ATLAS_DEF.get_sprite_def(
                                &current_tag_config.sprite_name
                            );

                            let w = ui.available_width();
                            let zoom = if sprite_data.size[0] == 0 {
                                1f32
                            } else {
                                w / (sprite_data.size[0] as f32 * 16f32)
                            };
                            let old_pivot = current_tag_config.sprite_pivot;
                            sprite_pivot_editor(
                                ui,
                                texture_id,
                                atlas_size,
                                current_tag_config,
                                zoom,
                            );
                            if !old_pivot.eq(&current_tag_config.sprite_pivot) {
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
                                            &mut current_tag_config.sprite_pivot[0],
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
                                            &mut current_tag_config.sprite_pivot[1],
                                            0..=sprite_data.size[1] * 16 - 1,
                                        )
                                    )
                                    .changed()
                                {
                                    update_state = UpdateState::Changed;
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Эффект при наложении:");
                                let full_width = ui.available_width();
                                let effect_label = match current_tag_config.effect_mechanic {
                                    Some(id) if !id.uuid.is_nil() => {
                                        if !asset_db.has_asset(AssetKind::EffectConfig, id.uuid) {
                                            "Не найден"
                                        } else {
                                            asset_db.asset_name(AssetKind::EffectConfig, id.uuid)
                                        }
                                    }
                                    _ => "Нет"
                                };
                                let response = ui.add_sized(
                                    [full_width, ui.spacing().interact_size.y],
                                    egui::Button::new(effect_label),
                                );
                                let popup_id = ui.make_persistent_id("выбор эффекта");
                                if response.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                egui::popup_below_widget(
                                    ui,
                                    popup_id,
                                    &response,
                                    PopupCloseBehavior::CloseOnClickOutside,
                                    |ui| {
                                        if ui.button("Сбросить").clicked() {
                                            current_tag_config.effect_mechanic = None;
                                            update_state = UpdateState::Changed;
                                            ui.memory_mut(|mem| mem.close_popup());
                                        }
                                        ui.separator();
                                        for (uuid, name) in asset_db.list_all_assets(AssetKind::EffectConfig) {
                                            if ui.button(name).clicked() {
                                                current_tag_config.effect_mechanic = Some(
                                                    ConfigId::from_uuid(uuid)
                                                );
                                                update_state = UpdateState::Changed;
                                                ui.memory_mut(|mem| mem.close_popup());
                                            }
                                        }
                                    },
                                );
                            });
                        })
                    });

                    update_state
                });
            }
            _ => {}
        }
    }
}
