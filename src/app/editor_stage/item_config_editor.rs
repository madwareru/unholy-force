use copy_from_str::CopyFromStrExt;
use crate::app::editor_stage::reusable_image_widget::{AtlasSpriteRect, pivot_editor};
use crate::app::editor_stage::{EditorStage, thick_selector_button};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::items::{ItemConfig, ItemRarity};
use crate::graphics::SPRITE_ATLAS_DEF;
use egui::{Align, Align2, PointerButton, PopupCloseBehavior, StrokeKind, Ui};
use uuid::Uuid;

#[derive(Copy, Clone, PartialEq)]
enum UpdateState {
    Unchanged,
    Changed,
}

impl EditorStage {
    fn update_current_item_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&mut String, &mut ItemConfig) -> UpdateState,
    ) {
        let section = &mut self.item_section;
        let name = &mut section.selected_item_name;
        let cur_item = &mut section.current_item_config;
        if let Some(current_item_config) = cur_item {
            if foo(name, current_item_config) == UpdateState::Changed {
                match section.selected_item_config_id {
                    Some(id) => {
                        let config_text = json5::to_string(current_item_config)
                            .expect("Failed to serialize default item config");
                        asset_db.update_json5_asset(AssetKind::ItemConfig, id, &config_text);
                        asset_db.rename_asset(AssetKind::ItemConfig, id, &name);
                    }
                    _ => {}
                }
            }
        }
    }

    fn load_current_item_config(&mut self, asset_db: &AssetDb) {
        let section = &mut self.item_section;
        match section.selected_item_config_id {
            Some(id) => {
                let config_text = asset_db.load_json5_asset(AssetKind::ItemConfig, id);
                section.current_item_config =
                    Some(json5::from_str(&config_text).expect("Failed to load json5"));
            }
            _ => {}
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
                ui.add_sized(
                    [full_width, 24f32],
                    egui::TextEdit::singleline(&mut self.new_asset_name),
                );
                if ui
                    .add_sized([full_width, 24f32], egui::Button::new("+"))
                    .clicked()
                {
                    let mut default_item_config = ItemConfig::default();
                    default_item_config.name = self.new_asset_name.clone();
                    let config_text = json5::to_string(&default_item_config)
                        .expect("Failed to serialize default item config");

                    let section = &mut self.item_section;
                    section.current_item_config = Some(default_item_config);

                    let id = asset_db.create_json5_asset(
                        AssetKind::ItemConfig,
                        &self.new_asset_name,
                        &config_text,
                    );
                    section.selected_item_config_id = Some(id);
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
                                ui.horizontal(|ui| {
                                    let section = &mut self.item_section;
                                    let selected = section
                                        .selected_item_config_id
                                        .map(|it| it.eq(&id))
                                        .unwrap_or(false);

                                    let config_text = asset_db.load_json5_asset(AssetKind::ItemConfig, id);
                                    let item_config: ItemConfig = json5::from_str(&config_text)
                                        .expect("Failed to load json5");

                                    let response = item_selector_button(
                                        ui,
                                        selected,
                                        texture_id,
                                        atlas_size,
                                        item_asset_name,
                                        &item_config,
                                    );

                                    // let response = thick_selector_button(
                                    //     ui,
                                    //     selected,
                                    //     egui::Align2::LEFT_CENTER,
                                    //     item_asset_name,
                                    // );
                                    let popup_id =
                                        ui.make_persistent_id(format!("выпадающее меню {}", id));

                                    if response.clicked_by(PointerButton::Primary) {
                                        section.selected_item_name.clear();
                                        section.selected_item_name += item_asset_name;
                                        section.selected_item_config_id = Some(id);
                                        self.load_current_item_config(&asset_db);
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
                                });
                            }
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
            }
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
        let atlas_size = self.atlas_size;

        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                self.update_current_item_config(&mut asset_db, |item_name, current_item_config| {
                    let mut update_state = UpdateState::Unchanged;
                    ui.columns(3, |uis| {
                        uis[0].horizontal(|ui| {
                            ui.label("Название для редактора:");
                            if ui
                                .add(
                                    egui::TextEdit::singleline(item_name)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                update_state = UpdateState::Changed;
                            }
                        });
                        uis[0].horizontal(|ui| {
                            ui.label("Название:");
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut current_item_config.name)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                update_state = UpdateState::Changed;
                            }
                        });
                        uis[0].horizontal(|ui| {
                            ui.label("Описание:");
                            if ui
                                .add(
                                    egui::TextEdit::multiline(&mut current_item_config.description)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                update_state = UpdateState::Changed;
                            }
                        });

                        uis[0].horizontal(|ui| {
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
                                    ui.set_min_width(200f32);

                                    let sprite_names =
                                        crate::graphics::SPRITE_ATLAS_DEF.sprites.keys();

                                    for sprite_name in sprite_names {
                                        if ui.button(sprite_name.as_str()).clicked() {
                                            current_item_config.sprite_name = sprite_name.clone();
                                            update_state = UpdateState::Changed;
                                            ui.memory_mut(|mem| mem.close_popup());
                                            break;
                                        }
                                    }
                                },
                            );
                        });
                        let entry = SPRITE_ATLAS_DEF
                            .sprites
                            .get(&current_item_config.sprite_name);
                        match entry {
                            None => {}
                            Some(sprite_data) => {
                                let w = uis[0].available_width();
                                let zoom = if sprite_data.size[0] == 0 {
                                    1f32
                                } else {
                                    w / (sprite_data.size[0] as f32 * 16f32)
                                };
                                let old_pivot = current_item_config.sprite_pivot;
                                pivot_editor(
                                    &mut uis[0],
                                    texture_id,
                                    AtlasSpriteRect::from_u16(
                                        atlas_size,
                                        sprite_data.coords.map(|it| it as u16 * 16),
                                        sprite_data.size.map(|it| it as u16 * 16)
                                    ),
                                    &mut current_item_config.sprite_pivot,
                                    zoom,
                                );
                                if !old_pivot.eq(&current_item_config.sprite_pivot) {
                                    update_state = UpdateState::Changed;
                                }

                                uis[0].columns(3, |uis| {
                                    uis[0].label("Опорная точка:");
                                    if uis[1]
                                        .add(
                                            egui::Slider::new(
                                                &mut current_item_config.sprite_pivot[0],
                                                0..=sprite_data.size[0] * 16 - 1,
                                            )
                                            .show_value(false),
                                        )
                                        .changed()
                                    {
                                        update_state = UpdateState::Changed;
                                    }
                                    if uis[2]
                                        .add(
                                            egui::Slider::new(
                                                &mut current_item_config.sprite_pivot[1],
                                                0..=sprite_data.size[1] * 16 - 1,
                                            )
                                            .show_value(false),
                                        )
                                        .changed()
                                    {
                                        update_state = UpdateState::Changed;
                                    }
                                })
                            }
                        }

                        uis[0].horizontal(|ui|{
                            ui.label("Редкость:");

                            let full_width = ui.available_width();
                            egui::ComboBox::from_id_salt("rarity")
                                .width(full_width)
                                .selected_text(match current_item_config.item_rarity {
                                    ItemRarity::Generic => "Обычный",
                                    ItemRarity::Rare => "Редкий",
                                    ItemRarity::Unique => "Уникальный",
                                    ItemRarity::Legendary => "Былинный"
                                })
                                .show_ui(ui, |ui| {
                                    for (v, lbl) in [
                                        (ItemRarity::Generic, "Обычный"),
                                        (ItemRarity::Rare, "Редкий"),
                                        (ItemRarity::Unique, "Уникальный"),
                                        (ItemRarity::Legendary, "Былинный")
                                    ] {
                                        if ui.selectable_value(
                                            &mut current_item_config.item_rarity,
                                            v,
                                            lbl
                                        ).clicked() {
                                            update_state = UpdateState::Changed;
                                        }
                                    }
                                });
                        })
                    });
                    update_state
                });
            }
            _ => {}
        };
    }
}

fn item_selector_button(
    ui: &mut Ui,
    selected: bool,
    atlas_texture: egui::TextureId,
    atlas_size: [u16; 2],
    editor_name: &str,
    item_config: &ItemConfig
) -> egui::Response {
    let desired_size = egui::vec2(
        ui.available_width(),
        ui.spacing().interact_size.y * 4f32,
    );

    let (rect, response) = ui.allocate_exact_size(
        desired_size,
        egui::Sense::click(),
    );

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.bg_fill
        };

        let stroke = if selected {
            ui.visuals().selection.stroke
        } else {
            visuals.bg_stroke
        };

        let text_color = if selected {
            ui.visuals().selection.stroke.color
        } else {
            visuals.text_color()
        };

        let rounding = egui::CornerRadius::same(4);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter().rect_stroke(rect, rounding, stroke, StrokeKind::Inside);

        let sprite_rect = SPRITE_ATLAS_DEF
            .sprites.get(&item_config.sprite_name)
            .map(|sprite_data| {
                AtlasSpriteRect::from_u16(
                    atlas_size,
                    sprite_data.coords.map(|it| it as u16 * 16),
                    sprite_data.size.map(|it| it as u16 * 16)
                )
            });

        let y_step = (rect.max.y - rect.min.y) / 3f32;
        let editor_name_y = rect.min.y + y_step / 2f32;
        let name_y = editor_name_y + y_step;
        let rarity_y = name_y + y_step;

        if let Some(sprite_rect) = sprite_rect {
            let top = rect.min.y + 4f32;
            let bottom = rect.max.y - 4f32;

            let h = bottom - top;
            let zoom = h / sprite_rect.size_px().y;
            let w = sprite_rect.size_px().x * zoom;

            let sp_rect = egui::Rect::from_min_max(
                [rect.min.x + 4f32, rect.min.y + 4f32].into(),
                [rect.min.x + 4f32 + w, rect.min.y + 4f32 + h].into(),
            );

            ui.painter().image(
                atlas_texture,
                sp_rect,
                sprite_rect.uv_rect(),
                egui::Color32::WHITE,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + w + 8f32,
                    editor_name_y
                ),
                Align2::LEFT_CENTER,
                editor_name,
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + w + 8f32,
                    name_y
                ),
                Align2::LEFT_CENTER,
                &item_config.name,
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + w + 8f32,
                    rarity_y
                ),
                Align2::LEFT_CENTER,
                match item_config.item_rarity {
                    ItemRarity::Generic => "Обычный",
                    ItemRarity::Rare => "Редкий",
                    ItemRarity::Unique => "Уникальный",
                    ItemRarity::Legendary => "Былинный"
                },
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );
        } else {
            ui.painter().text(
                egui::pos2(
                    rect.min.x + 8f32,
                    editor_name_y
                ),
                Align2::LEFT_CENTER,
                editor_name,
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + 8f32,
                    name_y
                ),
                Align2::LEFT_CENTER,
                &item_config.name,
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + 8f32,
                    rarity_y
                ),
                Align2::LEFT_CENTER,
                match item_config.item_rarity {
                    ItemRarity::Generic => "Обычный",
                    ItemRarity::Rare => "Редкий",
                    ItemRarity::Unique => "Уникальный",
                    ItemRarity::Legendary => "Былинный"
                },
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );
        }
    }

    response
}