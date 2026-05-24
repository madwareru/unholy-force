use crate::app::editor_stage::image_widgets::{floor_part_id_button};
use crate::app::editor_stage::{EditorStage, UpdateState, thick_selector_button};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::floor_part_adjacency::FloorPartAdjacencyConfig;
use egui::{Align2, CollapsingHeader, Id, PointerButton, PopupCloseBehavior, Response, ScrollArea, TextEdit, Ui};
use uuid::Uuid;
use crate::game_config::ConfigId;
use crate::game_config::floor_parts::FloorPartConfig;

#[derive(Default)]
pub struct FloorPartAdjacencyConfigEditorSection {
    config_name_filter: String,
    selected_config_id: Option<Uuid>,
    selected_config_name: String,
    current_config: Option<FloorPartAdjacencyConfig>,
}

impl EditorStage {
    fn update_current_floor_part_adjacency_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&mut AssetDb, &mut String, &mut FloorPartAdjacencyConfig) -> UpdateState,
    ) {
        let section = &mut self.floor_part_adjacency_section;
        let name = &mut section.selected_config_name;
        let cur_item = &mut section.current_config;

        if let Some(current_item_config) = cur_item {
            if foo(asset_db, name, current_item_config) == UpdateState::Changed {
                match section.selected_config_id {
                    Some(id) => {
                        let config_text = json5::to_string(current_item_config)
                            .expect("Failed to serialize item config");
                        asset_db.update_json5_asset(
                            AssetKind::FloorPartAdjacencyConfig,
                            id,
                            &config_text,
                        );
                        asset_db.rename_asset(AssetKind::FloorPartAdjacencyConfig, id, &name);
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn draw_fpa_selector(&mut self, ui: &mut Ui) {
        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                let full_width = ui.available_width();
                let available_height = ui.available_height() - ui.spacing().interact_size.y * 6f32;

                ui.horizontal(|ui| {
                    ui.label("Фильтр:");
                    ui.add(
                        TextEdit::singleline(
                            &mut self.floor_part_adjacency_section.config_name_filter,
                        )
                        .desired_width(f32::INFINITY),
                    )
                });
                ui.add_space(4f32);
                ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut to_delete = None;

                        let units = asset_db.list_all_assets(AssetKind::FloorPartAdjacencyConfig);
                        for (id, config_name) in units {
                            let section = &mut self.floor_part_adjacency_section;
                            if !section.config_name_filter.is_empty() {
                                if !config_name.starts_with(&section.config_name_filter) {
                                    continue;
                                }
                            }

                            let selected = section
                                .selected_config_id
                                .map(|it| it.eq(&id))
                                .unwrap_or(false);

                            let config_text =
                                asset_db.load_json5_asset(AssetKind::FloorPartAdjacencyConfig, id);
                            let fpa_config: FloorPartAdjacencyConfig =
                                json5::from_str(&config_text).expect("Failed to load unit config");

                            let response = thick_selector_button(
                                ui,
                                selected,
                                Align2::LEFT_CENTER,
                                config_name,
                            );

                            let popup_id = ui.make_persistent_id(format!("выпадающее меню {}", id));

                            if response.clicked_by(PointerButton::Primary) {
                                section.current_config = Some(fpa_config);
                                section.selected_config_name.clear();
                                section.selected_config_name += config_name;
                                section.selected_config_id = Some(id);
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
                            let section = &mut self.floor_part_adjacency_section;
                            match section.selected_config_id {
                                Some(selected_id) if selected_id.eq(&id) => {
                                    section.selected_config_id = None;
                                    section.current_config = None;
                                    section.selected_config_name.clear();
                                }
                                _ => {}
                            }
                            asset_db.delete_asset(AssetKind::FloorPartAdjacencyConfig, id);
                        }
                    });

                if ui
                    .add_sized(
                        [full_width, 24f32],
                        egui::Button::new("Создать конфигурацию"),
                    )
                    .clicked()
                {
                    let default_unit_config = FloorPartAdjacencyConfig::default();
                    let config_text = json5::to_string(&default_unit_config)
                        .expect("Failed to serialize default unit config");

                    let section = &mut self.floor_part_adjacency_section;
                    section.current_config = Some(default_unit_config);

                    let id = asset_db.create_json5_asset(
                        AssetKind::FloorPartAdjacencyConfig,
                        "",
                        &config_text,
                    );
                    section.selected_config_name.clear();
                    section.selected_config_id = Some(id);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn draw_adjacency_visualizer(&self, ui: &mut Ui) {}

    pub(crate) fn draw_floor_part_adjacency_editor(&mut self, ui: &mut Ui) {
        let id = self.floor_part_adjacency_section
            .selected_config_id
            .unwrap_or(Uuid::nil());

        let neighbours_salt_n = format!("neighbours_n_{}", id);
        let neighbours_salt_s = format!("neighbours_s_{}", id);
        let neighbours_salt_w = format!("neighbours_w_{}", id);
        let neighbours_salt_e = format!("neighbours_e_{}", id);

        let mut asset_db = crate::assets::ASSET_DATABASE.lock().unwrap();
        self.update_current_floor_part_adjacency_config(
            &mut asset_db,
            |asset_db, fpa_name, fpa_config| {
                let mut update_state = UpdateState::Unchanged;
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label("Название для редактора:");
                        if ui
                            .add(TextEdit::singleline(fpa_name).desired_width(f32::INFINITY))
                            .changed()
                        {
                            update_state = UpdateState::Changed;
                        }
                        ui.label("Центральная часть:");
                        let (response, _) = floor_part_id_button(
                            ui,
                            false,
                            asset_db,
                            fpa_config.part,
                            68f32,
                            2f32
                        );

                        let popup_id = ui.make_persistent_id("выбор центральной части");
                        if response.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                        }
                        shared_floor_part_closure(ui, asset_db, popup_id, &response, |config_id| {
                            fpa_config.part = config_id;
                            update_state = UpdateState::Changed;
                        });

                        CollapsingHeader::new("Соседи с севера")
                            .id_salt(neighbours_salt_n)
                            .show(ui, |ui| {
                                ui.label("Связь можно удалить правой кнопкой мыши");
                                ScrollArea::vertical()
                                    .max_height(200f32)
                                    .show(ui, |ui| {
                                        const COLUMNS_COUNT: usize = 8;
                                        ui.columns(COLUMNS_COUNT, |uis| {
                                            let mut current_column = 0;

                                            let parts = fpa_config.north_adjacent_parts
                                                .iter()
                                                .copied()
                                                .collect::<Vec<_>>();

                                            for conf in parts {
                                                let ui = &mut uis[current_column];
                                                ui.add_space(4f32);
                                                let (response, _) = floor_part_id_button(
                                                    ui,
                                                    false,
                                                    asset_db,
                                                    conf,
                                                    48f32,
                                                    2f32
                                                );
                                                if response.clicked_by(PointerButton::Secondary) {
                                                    update_state = UpdateState::Changed;
                                                    fpa_config.north_adjacent_parts.remove(&conf);
                                                }
                                                current_column = (current_column + 1) % COLUMNS_COUNT;
                                            }
                                        });
                                    });

                                let popup_id = ui.make_persistent_id("Добавление связи с севера");
                                let response = ui.button("Добавить связь");
                                if response.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                shared_floor_part_closure(ui, asset_db, popup_id, &response, |config_id| {
                                    fpa_config.north_adjacent_parts.insert(config_id);
                                    update_state = UpdateState::Changed;
                                });
                            });

                        CollapsingHeader::new("Соседи с юга")
                            .id_salt(neighbours_salt_s)
                            .show(ui, |ui| {
                                ui.label("Связь можно удалить правой кнопкой мыши");
                                ScrollArea::vertical()
                                    .max_height(200f32)
                                    .show(ui, |ui| {
                                        const COLUMNS_COUNT: usize = 8;
                                        ui.columns(COLUMNS_COUNT, |uis| {
                                            let mut current_column = 0;

                                            let parts = fpa_config.south_adjacent_parts
                                                .iter()
                                                .copied()
                                                .collect::<Vec<_>>();

                                            for conf in parts {
                                                let ui = &mut uis[current_column];
                                                ui.add_space(4f32);
                                                let (response, _) = floor_part_id_button(
                                                    ui,
                                                    false,
                                                    asset_db,
                                                    conf,
                                                    48f32,
                                                    2f32
                                                );
                                                if response.clicked_by(PointerButton::Secondary) {
                                                    update_state = UpdateState::Changed;
                                                    fpa_config.south_adjacent_parts.remove(&conf);
                                                }
                                                current_column = (current_column + 1) % COLUMNS_COUNT;
                                            }
                                        });
                                    });
                                let popup_id = ui.make_persistent_id("Добавление связи с юга");
                                let response = ui.button("Добавить связь");
                                if response.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                shared_floor_part_closure(ui, asset_db, popup_id, &response, |config_id| {
                                    fpa_config.south_adjacent_parts.insert(config_id);
                                    update_state = UpdateState::Changed;
                                });
                            });


                        CollapsingHeader::new("Соседи с запада")
                            .id_salt(neighbours_salt_w)
                            .show(ui, |ui| {
                                ui.label("Связь можно удалить правой кнопкой мыши");
                                ScrollArea::vertical()
                                    .max_height(200f32)
                                    .show(ui, |ui| {
                                        const COLUMNS_COUNT: usize = 8;
                                        ui.columns(COLUMNS_COUNT, |uis| {
                                            let mut current_column = 0;

                                            let parts = fpa_config.west_adjacent_parts
                                                .iter()
                                                .copied()
                                                .collect::<Vec<_>>();

                                            for conf in parts {
                                                let ui = &mut uis[current_column];
                                                ui.add_space(4f32);
                                                let (response, _) = floor_part_id_button(
                                                    ui,
                                                    false,
                                                    asset_db,
                                                    conf,
                                                    48f32,
                                                    2f32
                                                );
                                                if response.clicked_by(PointerButton::Secondary) {
                                                    update_state = UpdateState::Changed;
                                                    fpa_config.west_adjacent_parts.remove(&conf);
                                                }
                                                current_column = (current_column + 1) % COLUMNS_COUNT;
                                            }
                                        });
                                    });
                                let popup_id = ui.make_persistent_id("Добавление связи с запада");
                                let response = ui.button("Добавить связь");
                                if response.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                shared_floor_part_closure(ui, asset_db, popup_id, &response, |config_id| {
                                    fpa_config.west_adjacent_parts.insert(config_id);
                                    update_state = UpdateState::Changed;
                                });
                            });


                        CollapsingHeader::new("Соседи с востока")
                            .id_salt(neighbours_salt_e)
                            .show(ui, |ui| {
                                ui.label("Связь можно удалить правой кнопкой мыши");
                                ScrollArea::vertical()
                                    .max_height(200f32)
                                    .show(ui, |ui| {
                                        const COLUMNS_COUNT: usize = 8;
                                        ui.columns(COLUMNS_COUNT, |uis| {
                                            let mut current_column = 0;

                                            let parts = fpa_config.east_adjacent_parts
                                                .iter()
                                                .copied()
                                                .collect::<Vec<_>>();

                                            for conf in parts {
                                                let ui = &mut uis[current_column];
                                                ui.add_space(4f32);
                                                let (response, _) = floor_part_id_button(
                                                    ui,
                                                    false,
                                                    asset_db,
                                                    conf,
                                                    48f32,
                                                    2f32
                                                );
                                                if response.clicked_by(PointerButton::Secondary) {
                                                    update_state = UpdateState::Changed;
                                                    fpa_config.east_adjacent_parts.remove(&conf);
                                                }
                                                current_column = (current_column + 1) % COLUMNS_COUNT;
                                            }
                                        });
                                    });
                                let popup_id = ui.make_persistent_id("Добавление связи с востока");
                                let response = ui.button("Добавить связь");
                                if response.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                shared_floor_part_closure(ui, asset_db, popup_id, &response, |config_id| {
                                    fpa_config.east_adjacent_parts.insert(config_id);
                                    update_state = UpdateState::Changed;
                                });
                            });
                    });
                });
                update_state
            },
        )
    }
}

fn shared_floor_part_closure(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    mut foo: impl FnMut(ConfigId<FloorPartConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.label("Для отмены выбора нажмите ESC");
            const COLUMNS_COUNT: usize = 5;
            ui.columns(COLUMNS_COUNT, |uis| {
                let mut current_column = 0;

                for (uuid, _) in asset_db.list_all_assets(AssetKind::FloorPartConfig) {
                    let config_id = ConfigId::from_uuid(uuid);
                    let ui = &mut uis[current_column];
                    ui.add_space(4f32);

                    let (response, _) = floor_part_id_button(
                        ui,
                        false,
                        asset_db,
                        config_id,
                        68f32,
                        2f32
                    );

                    if response.clicked() {
                        foo(config_id);
                        ui.memory_mut(|mem| mem.close_popup());
                    }

                    current_column = (current_column + 1) % COLUMNS_COUNT;
                }
            });
        },
    );
}