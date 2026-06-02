use crate::app::editor_stage::image_widgets::{sprite_pivot_editor, item_selector_button, atlas_sprite_button, sprite_holder_visualizer};
use crate::app::editor_stage::{EditorStage, UpdateState};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::items::{ItemConfig, ItemRarity};
use crate::graphics::SPRITE_ATLAS_DEF;
use egui::{PointerButton, PopupCloseBehavior, TextEdit, Ui};
use uuid::Uuid;

const STACK_INFO_TEXT: &str = "При значении 1 предмет не складывается в стопку";

#[derive(Default)]
pub struct ItemConfigEditorSection {
    item_name_filter: String,
    selected_item_config_id: Option<Uuid>,
    selected_item_name: String,
    current_item_config: Option<ItemConfig>,
}

impl EditorStage {
    fn update_current_item_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&mut String, &mut ItemConfig) -> UpdateState
    ) {
        let section = &mut self.item_section;
        let name = &mut section.selected_item_name;
        let cur_item = &mut section.current_item_config;

        if let Some(current_item_config) = cur_item {
            if foo(name, current_item_config) == UpdateState::Changed {
                match section.selected_item_config_id {
                    Some(id) => {
                        asset_db.update_asset_mut(
                            AssetKind::ItemConfig,
                            id,
                            |buffer| json5::to_writer(buffer, current_item_config)
                        );
                        asset_db.rename_asset(AssetKind::ItemConfig, id, &name);
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn draw_item_selector(&mut self, ui: &mut Ui) {
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
                        TextEdit::singleline(&mut self.item_section.item_name_filter)
                            .desired_width(f32::INFINITY)
                    )
                });
                ui.add_space(4f32);

                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut to_delete = None;

                        let items = asset_db.list_all_assets(AssetKind::ItemConfig);
                        for (id, item_asset_name) in items {
                            let section = &mut self.item_section;
                            if !section.item_name_filter.is_empty() {
                                if !item_asset_name.starts_with(&section.item_name_filter) {
                                    continue;
                                }
                            }

                            let selected = section
                                .selected_item_config_id
                                .map(|it| it.eq(&id))
                                .unwrap_or(false);

                            let config_text = asset_db.load_json5_asset(AssetKind::ItemConfig, id);
                            let item_config: ItemConfig = json5::from_str(&config_text)
                                .expect("Failed to load item config");

                            let response = item_selector_button(
                                ui,
                                selected,
                                texture_id,
                                atlas_size,
                                item_asset_name,
                                &item_config,
                            );

                            let popup_id =
                                ui.make_persistent_id(format!("выпадающее меню {}", id));

                            if response.clicked_by(PointerButton::Primary) {
                                section.current_item_config = Some(item_config);
                                section.selected_item_name.clear();
                                section.selected_item_name += item_asset_name;
                                section.selected_item_config_id = Some(id);
                            } else if response.clicked_by(PointerButton::Secondary) {
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
                                },
                            );
                        }

                        if let Some(id) = to_delete {
                            let section = &mut self.item_section;
                            match section.selected_item_config_id {
                                Some(selected_id) if selected_id.eq(&id) => {
                                    section.selected_item_config_id = None;
                                    section.current_item_config = None;
                                }
                                _ => {}
                            }
                            asset_db.delete_asset(AssetKind::ItemConfig, id);
                        }
                    });

                if ui
                    .add_sized([full_width, 24f32], egui::Button::new("Создать предмет"))
                    .clicked()
                {
                    let default_item_config = ItemConfig::default();
                    let config_text = json5::to_string(&default_item_config)
                        .expect("Failed to serialize default item config");

                    let section = &mut self.item_section;
                    section.current_item_config = Some(default_item_config);

                    let id = asset_db.create_json5_asset(
                        AssetKind::ItemConfig,
                        "",
                        &config_text,
                    );
                    section.selected_item_name.clear();
                    section.selected_item_config_id = Some(id);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn draw_item_preview_in_level(&self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;
        if let Some(item_config) = &self.item_section.current_item_config {
            if item_config.sprite_name.is_empty() {
                return;
            }
            ui.vertical(|ui| {
                ui.add_space(6f32);
                ui.group(|ui| {
                    ui.label("Предпросмотр на игровом поле:");
                    sprite_holder_visualizer(
                        ui,
                        texture_id,
                        atlas_size,
                        item_config
                    )
                });
            });
        }
    }

    pub(crate) fn draw_item_editor(&mut self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;

        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                self.update_current_item_config(&mut asset_db, |item_name, current_item_config| {
                    let mut update_state = UpdateState::Unchanged;
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Название для редактора:");
                                if ui.add(TextEdit::singleline(item_name).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Название:");
                                if ui.add(TextEdit::singleline(&mut current_item_config.name).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Описание:");
                                if ui.add(TextEdit::multiline(&mut current_item_config.description).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Изображение:");
                                let full_width = ui.available_width();

                                let response = ui.add_sized(
                                    [full_width, ui.spacing().interact_size.y],
                                    egui::Button::new(&current_item_config.sprite_name),
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
                                                let response = atlas_sprite_button(
                                                    ui,
                                                    texture_id,
                                                    atlas_size,
                                                    sprite_name,
                                                    96f32,
                                                );

                                                if response.clicked() {
                                                    current_item_config.sprite_name.clear();
                                                    current_item_config.sprite_name += sprite_name;
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
                                &current_item_config.sprite_name
                            );

                            let w = ui.available_width();
                            let zoom = if sprite_data.size[0] == 0 {
                                1f32
                            } else {
                                w / (sprite_data.size[0] as f32 * 16f32)
                            };
                            let old_pivot = current_item_config.sprite_pivot;
                            sprite_pivot_editor(
                                ui,
                                texture_id,
                                atlas_size,
                                current_item_config,
                                zoom,
                            );
                            if !old_pivot.eq(&current_item_config.sprite_pivot) {
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
                                            &mut current_item_config.sprite_pivot[0],
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
                                            &mut current_item_config.sprite_pivot[1],
                                            0..=sprite_data.size[1] * 16 - 1,
                                        )
                                    )
                                    .changed()
                                {
                                    update_state = UpdateState::Changed;
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Редкость:");

                                egui::ComboBox::from_id_salt("rarity")
                                    .selected_text(current_item_config.item_rarity.display_name())
                                    .show_ui(ui, |ui| {
                                        for v in [
                                            ItemRarity::Generic,
                                            ItemRarity::Rare,
                                            ItemRarity::Unique,
                                            ItemRarity::Legendary
                                        ] {
                                            if ui.selectable_value(
                                                &mut current_item_config.item_rarity,
                                                v,
                                                v.display_name()
                                            ).clicked() {
                                                update_state = UpdateState::Changed;
                                            }
                                        }
                                    });

                                ui.label("Максимум в стопке:");
                                if ui.add(
                                    egui::Slider::new(&mut current_item_config.stack_limit, 1..=100)
                                ).on_hover_text(STACK_INFO_TEXT).changed() {
                                    update_state = UpdateState::Changed
                                }
                            });
                        });
                    });
                    update_state
                });
            }
            _ => {}
        };
    }
}