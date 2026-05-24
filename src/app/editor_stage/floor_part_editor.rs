use crate::app::editor_stage::image_widgets::{floor_part_editor, floor_part_selector_button};
use crate::app::editor_stage::{thick_selector_button, EditorStage, UpdateState};
use crate::assets::{AssetDb, AssetKind};
use crate::game_config::floor_parts::FloorPartConfig;
use crate::graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup};
use egui::{Align2, PointerButton, PopupCloseBehavior, TextEdit, Ui};
use uuid::Uuid;

#[derive(Default)]
pub struct FloorPartConfigEditorSection {
    floor_part_name_filter: String,
    selected_floor_part_config_id: Option<Uuid>,
    selected_floor_part_name: String,
    current_floor_part_config: Option<FloorPartConfig>,
    tools_sub_section: FloorPartToolsSubSection,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub enum FloorPartEditorTool {
    #[default]
    PlaceFloor,
    PlaceWall,
}

#[derive(Copy, Clone, Default)]
pub struct FloorPartToolsSubSection {
    current_tool: FloorPartEditorTool,
    floor_tile_group: FloorGraphicsTileGroup,
    wall_tile_group: WallGraphicsTileGroup,
}

impl EditorStage {
    fn update_current_floor_part_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&mut String, &mut FloorPartConfig) -> UpdateState,
    ) {
        let section = &mut self.floor_part_section;
        let name = &mut section.selected_floor_part_name;
        let cur_part = &mut section.current_floor_part_config;
        if let Some(current_floor_part_config) = cur_part {
            if foo(name, current_floor_part_config) == UpdateState::Changed {
                match section.selected_floor_part_config_id {
                    Some(id) => {
                        let mut buffer: Vec<u8> = Vec::new();
                        current_floor_part_config.write(&mut buffer)
                            .expect("Failed to serialize floor part config");
                        asset_db.update_asset(AssetKind::FloorPartConfig, id, &buffer);
                        asset_db.rename_asset(AssetKind::FloorPartConfig, id, &name);
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn draw_floor_part_selector(&mut self, ui: &mut Ui) {
        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                let full_width = ui.available_width();
                let available_height = ui.available_height() - ui.spacing().interact_size.y * 6f32;

                ui.horizontal(|ui| {
                    ui.label("Фильтр:");
                    ui.add(
                        TextEdit::singleline(&mut self.floor_part_section.floor_part_name_filter)
                            .desired_width(f32::INFINITY)
                    )
                });
                ui.add_space(4f32);

                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        const NUM_COLUMNS: usize = 3;
                        let mut to_delete = None;
                        ui.columns(NUM_COLUMNS, |uis| {
                            let items = asset_db.list_all_assets(AssetKind::FloorPartConfig);
                            let mut offset = 0;
                            for (id, asset_name) in items {
                                let ui = &mut uis[offset];
                                let section = &mut self.floor_part_section;
                                if !section.floor_part_name_filter.is_empty() {
                                    if !asset_name.starts_with(&section.floor_part_name_filter) {
                                        continue;
                                    }
                                }
                                offset = (offset + 1) % NUM_COLUMNS;

                                let selected = section
                                    .selected_floor_part_config_id
                                    .map(|it| it.eq(&id))
                                    .unwrap_or(false);

                                let config_bytes = asset_db.load_asset(AssetKind::FloorPartConfig, id);
                                let floor_part_config = FloorPartConfig::load_from_slice(config_bytes)
                                    .expect("Failed to load floor part config");

                                let response = floor_part_selector_button(
                                    ui,
                                    selected,
                                    asset_name,
                                    &floor_part_config
                                );

                                let popup_id = ui.make_persistent_id(format!("выпадающее меню {}", id));

                                if response.clicked_by(PointerButton::Primary) {
                                    section.current_floor_part_config = Some(floor_part_config);
                                    section.selected_floor_part_name.clear();
                                    section.selected_floor_part_name += asset_name;
                                    section.selected_floor_part_config_id = Some(id);
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
                        });
                        if let Some(id) = to_delete {
                            let section = &mut self.floor_part_section;
                            match section.selected_floor_part_config_id {
                                Some(selected_id) if selected_id.eq(&id) => {
                                    section.selected_floor_part_config_id = None;
                                    section.current_floor_part_config = None;
                                }
                                _ => {}
                            }
                            asset_db.delete_asset(AssetKind::FloorPartConfig, id);
                        }
                    });

                if ui
                    .add_sized([full_width, 24f32], egui::Button::new("Создать часть этажа"))
                    .clicked()
                {
                    let default_floor_part_config = FloorPartConfig::default();
                    let mut buffer: Vec<u8> = Vec::new();
                    default_floor_part_config.write(&mut buffer)
                        .expect("Failed to serialize default floor part config");

                    let section = &mut self.floor_part_section;
                    section.current_floor_part_config = Some(default_floor_part_config);

                    let id = asset_db.create_asset(
                        AssetKind::FloorPartConfig,
                        "",
                        &buffer,
                    );
                    section.selected_floor_part_config_id = Some(id);
                }
            }
            _ => {}
        };
    }

    pub(crate) fn draw_floor_part_editor(&mut self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;
        let current_tool_section = self.floor_part_section.tools_sub_section;

        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                self.update_current_floor_part_config(&mut asset_db, |floor_part_name, current_floor_part_config| {
                    let mut update_state = UpdateState::Unchanged;
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Название для редактора:");
                                if ui.add(TextEdit::singleline(floor_part_name).desired_width(f32::INFINITY)).changed() {
                                    update_state = UpdateState::Changed;
                                }
                            });

                            let full_width = ui.available_width();
                            let full_height = ui.available_height();
                            let padding = (full_height - full_width) / 2f32;
                            ui.add_space(padding);
                            if let Some([x, y]) = floor_part_editor(
                                ui,
                                texture_id,
                                atlas_size,
                                current_floor_part_config,
                                8
                            ) {
                                match current_tool_section.current_tool {
                                    FloorPartEditorTool::PlaceFloor => {
                                        current_floor_part_config.floor_data[y][x] =
                                            current_tool_section.floor_tile_group;
                                    }
                                    FloorPartEditorTool::PlaceWall => {
                                        current_floor_part_config.wall_data[y][x] =
                                            current_tool_section.wall_tile_group;
                                    }
                                }
                                update_state = UpdateState::Changed;
                            }
                            ui.add_space(padding);
                        });
                    });
                    update_state
                });
            }
            _ => {}
        };
    }

    pub(crate) fn draw_floor_part_editor_tools(&mut self, ui: &mut Ui) {
        let sub_section = &mut self.floor_part_section.tools_sub_section;
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