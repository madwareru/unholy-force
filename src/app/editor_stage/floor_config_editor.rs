use egui::{Align2, Button, CollapsingHeader, Id, PointerButton, PopupCloseBehavior, Response, ScrollArea, TextEdit, TextureId, Ui};
use uuid::Uuid;
use crate::app::editor_stage::{thick_selector_button, EditorStage, UpdateState};
use crate::app::editor_stage::floor_part_editor::{FloorPartEditorTool, FloorPartToolsSubSection};
use crate::app::editor_stage::image_widgets::{floor_data_holder_editor, fpa_id_button, item_config_id_button, split_2_horizontal, unit_config_id_button, EditableFloorData};
use crate::app::game_stage::floor_generator::{generate, FloorGeneratorResult};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::ConfigId;
use crate::game_config::floor_part_adjacency::FloorPartAdjacencyConfig;
use crate::game_config::floors::{FloorConfig, FloorVariant, FloorVariantTag, LootTableEntry, SpawnTableEntry, AuthoredFloor, GeneratedFloor};
use crate::game_config::items::ItemConfig;
use crate::game_config::units::UnitConfig;
use crate::graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup};

#[derive(Default)]
pub struct FloorConfigEditorSection {
    config_name_filter: String,
    selected_config_id: Option<Uuid>,
    selected_config_name: String,
    current_config: Option<FloorConfig>,
    tools_sub_section: FloorPartToolsSubSection,
    generated_floor: Option<FloorGeneratorResult>,
}

impl EditorStage {
    fn update_current_floor_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&AssetDb, &mut String, &mut FloorConfig) -> UpdateState,
    ) {
        let section = &mut self.floor_section;
        let name = &mut section.selected_config_name;
        let current_config = &mut section.current_config;

        if let Some(current_config) = current_config {
            if foo(asset_db, name, current_config) == UpdateState::Changed {
                match section.selected_config_id {
                    Some(id) => {
                        let config_text = json5::to_string(&current_config)
                            .expect("Failed to serialize floor config");
                        asset_db.update_json5_asset(AssetKind::FloorConfig, id, &config_text);
                        asset_db.rename_asset(AssetKind::FloorConfig, id, &name);
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn draw_floor_config_selector(&mut self, ui: &mut Ui) {
        let mut asset_db = crate::assets::ASSET_DATABASE.lock()
            .expect("Failed to lock asset database");

        let full_width = ui.available_width();
        let available_height = ui.available_height() - ui.spacing().interact_size.y * 6f32;

        ui.horizontal(|ui| {
            ui.label("Фильтр:");
            ui.add(
                TextEdit::singleline(
                    &mut self.floor_section.config_name_filter,
                ).desired_width(f32::INFINITY),
            )
        });
        ui.add_space(4f32);

        ScrollArea::vertical()
            .max_height(available_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut to_delete = None;

                let units = asset_db.list_all_assets(AssetKind::FloorConfig);
                for (id, config_name) in units {
                    let section = &mut self.floor_section;
                    if !section.config_name_filter.is_empty() {
                        if !config_name.starts_with(&section.config_name_filter) {
                            continue;
                        }
                    }

                    let selected = section
                        .selected_config_id
                        .map(|it| it.eq(&id))
                        .unwrap_or(false);

                    let config_text = asset_db.load_json5_asset(AssetKind::FloorConfig, id);
                    let fpa_config: FloorConfig =
                        json5::from_str(&config_text).expect("Failed to load unit config");

                    let response = thick_selector_button(
                        ui,
                        selected,
                        Align2::LEFT_CENTER,
                        config_name,
                    );

                    let popup_id = ui.make_persistent_id(format!("выпадающее меню {}", id));

                    if response.clicked_by(PointerButton::Primary) {
                        match section.selected_config_id {
                            Some(selected_id) if selected_id.eq(&id) => {}
                            _ => {
                                section.current_config = Some(fpa_config);
                                section.selected_config_name.clear();
                                section.selected_config_name += config_name;
                                section.selected_config_id = Some(id);
                                section.generated_floor = None;
                            }
                        }

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
                    let section = &mut self.floor_section;
                    match section.selected_config_id {
                        Some(selected_id) if selected_id.eq(&id) => {
                            section.selected_config_id = None;
                            section.current_config = None;
                            section.generated_floor = None;
                            section.selected_config_name.clear();
                        }
                        _ => {}
                    }
                    asset_db.delete_asset(AssetKind::FloorConfig, id);
                }
            });

        if ui
            .add_sized(
                [full_width, 24f32],
                Button::new("Создать этаж"),
            )
            .clicked()
        {
            let default_config = FloorConfig::default();
            let config_text = json5::to_string(&default_config)
                .expect("Failed to serialize default floor config");

            let section = &mut self.floor_section;
            section.current_config = Some(default_config);

            let id = asset_db.create_json5_asset(
                AssetKind::FloorConfig,
                "",
                &config_text,
            );
            section.selected_config_name.clear();
            section.selected_config_id = Some(id);
        }
    }

    pub(crate) fn draw_floor_config_editor(&mut self, ui: &mut Ui) {
        let texture_id: TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };

        let atlas_size = self.atlas_size;
        let current_tool_section = self.floor_section.tools_sub_section;

        let mut asset_db = crate::assets::ASSET_DATABASE.lock()
            .expect("Failed to lock asset database");

        let floor_config_id = self.floor_section
            .selected_config_id
            .unwrap_or_default();

        let loot_table_salt = format!("floor_loot_table_{}", floor_config_id);
        let spawn_table_salt = format!("floor_spawn_table_{}", floor_config_id);
        let adjacency_rule_table_salt = format!("floor_adjacency_rule_table_{}", floor_config_id);

        let mut generate_requested = false;
        self.update_current_floor_config(&mut asset_db, |asset_db, floor_name, current_floor_config| {
            let mut update_state = UpdateState::Unchanged;

            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Название для редактора:");
                        if ui.add(TextEdit::singleline(floor_name).desired_width(f32::INFINITY)).changed() {
                            update_state = UpdateState::Changed;
                        }
                    });

                    ui.columns_const(|[loot_ui, spawn_ui]| {
                        CollapsingHeader::new("Возможные предметы на этаже")
                            .id_salt(loot_table_salt)
                            .show(loot_ui, |ui| {
                                let available_width = ui.available_width();
                                ScrollArea::vertical()
                                    .max_height(400f32)
                                    .max_width(available_width - 10f32)
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        let mut loot_table = std::mem::take(&mut current_floor_config.loot_table);
                                        let mut to_delete = None;

                                        split_2_horizontal(
                                            ui,
                                            0.8f32,
                                            |[button_column_ui, weight_delete_column_ui]| {
                                                for (index, entry) in loot_table.iter_mut().enumerate() {
                                                    item_config_id_button(
                                                        button_column_ui,
                                                        asset_db,
                                                        false,
                                                        texture_id,
                                                        atlas_size,
                                                        entry.item_config
                                                    );
                                                    weight_delete_column_ui.vertical(
                                                        |ui| {
                                                            ui.horizontal(|ui| {
                                                                ui.label("Вес:");
                                                                if ui.add(
                                                                    egui::DragValue::new(&mut entry.weight).range(0..=100)
                                                                ).changed() {
                                                                    update_state = UpdateState::Changed;
                                                                }
                                                            });
                                                            let max_width = ui.available_width();
                                                            if ui.add_sized(
                                                                [
                                                                    max_width,
                                                                    ui.spacing().interact_size.y * 2.8f32,
                                                                ],
                                                                Button::new("Убрать")
                                                            ).clicked() {
                                                                to_delete = Some(index);
                                                            }
                                                        }
                                                    );
                                                }
                                            }
                                        );
                                        if let Some(to_delete) = to_delete {
                                            loot_table.remove(to_delete);
                                            update_state = UpdateState::Changed;
                                        }
                                        current_floor_config.loot_table = loot_table;
                                    });

                                let popup_id = ui.make_persistent_id("Добавление записи для предмета");
                                let response = ui.button("Добавить предмет");
                                if response.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                item_selector_popup(ui, asset_db, popup_id, &response, texture_id, atlas_size, |config_id| {
                                    current_floor_config.loot_table.push(LootTableEntry {
                                        item_config: config_id,
                                        weight: 1,
                                    });
                                    update_state = UpdateState::Changed;
                                });
                            });

                        CollapsingHeader::new("Возможные персонажи на этаже")
                            .id_salt(spawn_table_salt)
                            .show(spawn_ui, |ui| {
                                let available_width = ui.available_width();
                                ScrollArea::vertical()
                                    .max_height(400f32)
                                    .max_width(available_width - 15f32)
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        let mut spawn_table = std::mem::take(&mut current_floor_config.spawn_table);
                                        let mut to_delete = None;

                                        split_2_horizontal(
                                            ui,
                                            0.8f32,
                                            |[button_column_ui, weight_delete_column_ui]| {
                                                for (index, entry) in spawn_table.iter_mut().enumerate() {
                                                    unit_config_id_button(
                                                        button_column_ui,
                                                        asset_db,
                                                        false,
                                                        texture_id,
                                                        atlas_size,
                                                        entry.unit_config
                                                    );
                                                    weight_delete_column_ui.vertical(
                                                        |ui| {
                                                            ui.horizontal(|ui| {
                                                                ui.label("Вес:");
                                                                if ui.add(
                                                                    egui::DragValue::new(&mut entry.weight).range(0..=100)
                                                                ).changed() {
                                                                    update_state = UpdateState::Changed;
                                                                }
                                                            });
                                                            let max_width = ui.available_width();
                                                            if ui.add_sized(
                                                                [
                                                                    max_width,
                                                                    ui.spacing().interact_size.y * 2.8f32,
                                                                ],
                                                                Button::new("Убрать")
                                                            ).clicked() {
                                                                to_delete = Some(index);
                                                            }
                                                        }
                                                    );
                                                }
                                            }
                                        );
                                        if let Some(to_delete) = to_delete {
                                            spawn_table.remove(to_delete);
                                            update_state = UpdateState::Changed;
                                        }
                                        current_floor_config.spawn_table = spawn_table;
                                    });

                                let popup_id = ui.make_persistent_id("Добавление записи для персонажа");
                                let response = ui.button("Добавить персонажа");
                                if response.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }
                                unit_selector_popup(ui, asset_db, popup_id, &response, texture_id, atlas_size, |config_id| {
                                    current_floor_config.spawn_table.push(SpawnTableEntry {
                                        unit_config: config_id,
                                        weight: 1,
                                    });
                                    update_state = UpdateState::Changed;
                                });
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Тип этажа:");

                        let mut floor_variant_tag = current_floor_config.floor_variant.get_tag();
                        let old_variant_tag = floor_variant_tag;

                        egui::ComboBox::from_id_salt("rarity")
                            .selected_text(floor_variant_tag.editor_label())
                            .show_ui(ui, |ui| {
                                for tag in [
                                    FloorVariantTag::Authored15x15,
                                    FloorVariantTag::Authored20x20,
                                    FloorVariantTag::Authored25x25,
                                    FloorVariantTag::Authored30x30,
                                    FloorVariantTag::Generated15x15,
                                    FloorVariantTag::Generated20x20,
                                    FloorVariantTag::Generated25x25,
                                    FloorVariantTag::Generated30x30,
                                    FloorVariantTag::Generated40x40,
                                    FloorVariantTag::Generated60x60,
                                    FloorVariantTag::Generated80x80,
                                ] {
                                    if ui.selectable_value(
                                        &mut floor_variant_tag,
                                        tag,
                                        tag.editor_label()
                                    ).clicked()  {
                                        if floor_variant_tag != old_variant_tag {
                                            current_floor_config.floor_variant =
                                                match floor_variant_tag {
                                                    FloorVariantTag::Authored15x15 =>
                                                        FloorVariant::Authored(
                                                            AuthoredFloor::Size15x15(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Authored20x20 =>
                                                        FloorVariant::Authored(
                                                            AuthoredFloor::Size20x20(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Authored25x25 =>
                                                        FloorVariant::Authored(
                                                            AuthoredFloor::Size25x25(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Authored30x30 =>
                                                        FloorVariant::Authored(
                                                            AuthoredFloor::Size30x30(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Generated15x15 =>
                                                        FloorVariant::Generated(
                                                            GeneratedFloor::Size15x15(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Generated20x20 =>
                                                        FloorVariant::Generated(
                                                            GeneratedFloor::Size20x20(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Generated25x25 =>
                                                        FloorVariant::Generated(
                                                            GeneratedFloor::Size25x25(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Generated30x30 =>
                                                        FloorVariant::Generated(
                                                            GeneratedFloor::Size30x30(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Generated40x40 =>
                                                        FloorVariant::Generated(
                                                            GeneratedFloor::Size40x40(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Generated60x60 =>
                                                        FloorVariant::Generated(
                                                            GeneratedFloor::Size60x60(
                                                                Default::default()
                                                            )
                                                        ),
                                                    FloorVariantTag::Generated80x80 =>
                                                        FloorVariant::Generated(
                                                            GeneratedFloor::Size80x80(
                                                                Default::default()
                                                            )
                                                        )
                                                };

                                            update_state = UpdateState::Changed;
                                        }
                                    }
                                }
                            });
                    });

                    match &mut current_floor_config.floor_variant {
                        FloorVariant::Authored(authored_floor) => {
                            let scale = match authored_floor {
                                AuthoredFloor::Size15x15(_) => 2,
                                AuthoredFloor::Size20x20(_) => 2,
                                AuthoredFloor::Size25x25(_) => 1,
                                AuthoredFloor::Size30x30(_) => 1,
                            };

                            let available_height = ui.available_height();
                            ScrollArea::vertical()
                                .max_height(available_height)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    if let Some([x, y]) = floor_data_holder_editor(
                                        ui,
                                        texture_id,
                                        atlas_size,
                                        authored_floor,
                                        scale
                                    ) {
                                        match current_tool_section.current_tool {
                                            FloorPartEditorTool::PlaceFloor => {
                                                *authored_floor.get_floor_data_mut([x, y]) =
                                                    current_tool_section.floor_tile_group;
                                            }
                                            FloorPartEditorTool::PlaceWall => {
                                                *authored_floor.get_wall_data_mut([x, y]) =
                                                    current_tool_section.wall_tile_group;
                                            }
                                        }
                                        update_state = UpdateState::Changed;
                                    }
                                });
                        }
                        _ => {
                            CollapsingHeader::new("Правила связности на этаже")
                                .id_salt(adjacency_rule_table_salt)
                                .show(ui, |ui| {
                                    let available_width = ui.available_width();
                                    ScrollArea::vertical()
                                        .max_height(400f32)
                                        .max_width(available_width - 15f32)
                                        .auto_shrink([false, true])
                                        .show(ui, |ui| {
                                            let mut available_parts = std::mem::take(&mut current_floor_config.available_parts);
                                            let mut to_delete = None;
                                            ui.label("Для удаления нажмите правой кнопкой мыши");
                                            const NUM_COLUMNS: usize = 6;
                                            ui.columns(NUM_COLUMNS, |uis| {
                                                let mut offset = 0;
                                                for (index, config_id) in available_parts.iter().enumerate() {
                                                    let ui = &mut uis[offset];
                                                    offset = (offset + 1) % NUM_COLUMNS;
                                                    let response = fpa_id_button(
                                                        ui,
                                                        asset_db,
                                                        false,
                                                        texture_id,
                                                        atlas_size,
                                                        80f32,
                                                        *config_id,
                                                    );
                                                    if response.clicked_by(PointerButton::Secondary) {
                                                        to_delete = Some(index);
                                                    }
                                                }
                                            });

                                            if let Some(to_delete) = to_delete {
                                                available_parts.remove(to_delete);
                                                update_state = UpdateState::Changed;
                                            }
                                            current_floor_config.available_parts = available_parts;
                                        });

                                    let popup_id = ui.make_persistent_id("Добавление правила связности");
                                    let response = ui.button("Добавить правило");
                                    if response.clicked() {
                                        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                    }
                                    fpa_selector_popup(ui, asset_db, popup_id, &response, texture_id, atlas_size, |config_id| {
                                        current_floor_config.available_parts.push(config_id);
                                        update_state = UpdateState::Changed;
                                    });
                                });

                            if ui.button("Тестовая генерация").clicked() {
                                generate_requested = true;
                            }
                        }
                    }
                });
            });

            update_state
        });
        if generate_requested {
            self.floor_section.generated_floor = generate(
                &asset_db,
                ConfigId::from_uuid(floor_config_id)
            );
        }
        if let Some(generated_result) = &self.floor_section.generated_floor {
            let available_height = ui.available_height();
            let available_width = ui.available_width();
            ScrollArea::vertical()
                .max_height(available_height)
                .max_width(available_width)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    floor_data_holder_editor(
                        ui,
                        texture_id,
                        atlas_size,
                        generated_result,
                        1
                    );
                }
            );
        }
    }

    pub(crate) fn draw_floor_editor_tools(&mut self, ui: &mut Ui) {
        match &self.floor_section.current_config {
            Some(cfg) if cfg.floor_variant.is_generated() => {
                return;
            }
            _ => {}
        }

        let sub_section = &mut self.floor_section.tools_sub_section;
        ui.vertical(|ui| {
            ui.add_space(6f32);
            ui.group(|ui| {
                if thick_selector_button(
                    ui,
                    sub_section.current_tool == FloorPartEditorTool::PlaceFloor,
                    Align2::CENTER_CENTER,
                    "Расстановка пола"
                ).clicked() {
                    sub_section.current_tool = FloorPartEditorTool::PlaceFloor;
                }
                if thick_selector_button(
                    ui,
                    sub_section.current_tool == FloorPartEditorTool::PlaceWall,
                    Align2::CENTER_CENTER,
                    "Расстановка стен"
                ).clicked() {
                    sub_section.current_tool = FloorPartEditorTool::PlaceWall;
                }
                match sub_section.current_tool {
                    FloorPartEditorTool::PlaceFloor => {
                        ui.columns(4, |uis| {
                            for (i, group) in [
                                FloorGraphicsTileGroup::Dirt,
                                FloorGraphicsTileGroup::Tile,
                                FloorGraphicsTileGroup::Water,
                                FloorGraphicsTileGroup::Lava,
                            ].iter().enumerate() {
                                if thick_selector_button(
                                    &mut uis[i],
                                    sub_section.floor_tile_group == *group,
                                    Align2::CENTER_CENTER,
                                    group.get_name()
                                ).clicked() {
                                    sub_section.floor_tile_group = *group;
                                }
                            }
                        });
                    }
                    FloorPartEditorTool::PlaceWall => {
                        ui.columns(4, |uis| {
                            for (i, group) in [
                                WallGraphicsTileGroup::None,
                                WallGraphicsTileGroup::Sandstone,
                                WallGraphicsTileGroup::Rocks,
                                WallGraphicsTileGroup::Bricks,
                            ].iter().enumerate() {
                                if thick_selector_button(
                                    &mut uis[i],
                                    sub_section.wall_tile_group == *group,
                                    Align2::CENTER_CENTER,
                                    group.get_name()
                                ).clicked() {
                                    sub_section.wall_tile_group = *group;
                                }
                            }
                        });
                    }
                }
            });
        });
    }
}

fn unit_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<UnitConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                for (uuid, _) in asset_db.list_all_assets(AssetKind::UnitConfig) {
                    let config_id = ConfigId::from_uuid(uuid);
                    ui.add_space(4f32);

                    let response = unit_config_id_button(
                        ui,
                        asset_db,
                        false,
                        atlas_texture,
                        atlas_size,
                        config_id,
                    );

                    if response.clicked() {
                        foo(config_id);
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        },
    );
}

fn item_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<ItemConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                for (uuid, _) in asset_db.list_all_assets(AssetKind::ItemConfig) {
                    let config_id = ConfigId::from_uuid(uuid);
                    ui.add_space(4f32);

                    let response = item_config_id_button(
                        ui,
                        asset_db,
                        false,
                        atlas_texture,
                        atlas_size,
                        config_id,
                    );

                    if response.clicked() {
                        foo(config_id);
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        },
    );
}

fn fpa_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<FloorPartAdjacencyConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                const NUM_COLUMNS: usize = 4;
                let mut offset = 0;
                ui.columns(NUM_COLUMNS, |uis| {
                    for (uuid, _) in asset_db.list_all_assets(AssetKind::FloorPartAdjacencyConfig) {
                        let ui = &mut uis[offset];
                        offset = (offset + 1) % NUM_COLUMNS;
                        let config_id = ConfigId::from_uuid(uuid);
                        ui.add_space(4f32);

                        let response = fpa_id_button(
                            ui,
                            asset_db,
                            false,
                            atlas_texture,
                            atlas_size,
                            60f32,
                            config_id,
                        );

                        if response.clicked() {
                            foo(config_id);
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                    }
                });
            });
        },
    );
}