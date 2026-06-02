use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use egui::{Button, PopupCloseBehavior, Stroke, TextEdit, Ui};
use egui_snarl::ui::{NodeLayout, PinPlacement, PinShape, SnarlStyle};
use uuid::Uuid;
use crate::app::editor_stage::EditorStage;
use crate::assets::AssetKind;
use crate::game_config::floor_flow_graph::{FloorFlowGraphConfig, FloorFlowGraphViewer, FloorFlowNode};

pub struct FloorFlowGraphEditorSection {
    config_name_filter: String,
    selected_config_id: Option<Uuid>,
    selected_config_name: String,
    current_config: Option<FloorFlowGraphConfig>,
    style: SnarlStyle,
}
impl Default for FloorFlowGraphEditorSection {
    fn default() -> Self {
        Self {
            config_name_filter: String::new(),
            selected_config_id: None,
            selected_config_name: String::new(),
            current_config: None,
            style: SnarlStyle {
                pin_placement: Some(PinPlacement::Edge),
                node_layout: Some(NodeLayout::Sandwich),
                pin_shape: Some(PinShape::Square),
                pin_stroke: Some(Stroke::NONE),
                ..Default::default()
            },
        }
    }
}

impl EditorStage {
    pub(crate) fn draw_floor_graph_selector(&mut self, ui: &mut Ui) {
        let mut asset_db = crate::assets::ASSET_DATABASE.lock()
            .expect("Failed to lock asset database");

        let full_width = ui.available_width();
        let available_height = ui.available_height() - ui.spacing().interact_size.y * 6f32;

        ui.horizontal(|ui| {
            ui.label("Фильтр:");
            ui.add(
                TextEdit::singleline(&mut self.floor_flow_graph_section.config_name_filter)
                    .desired_width(f32::INFINITY)
            )
        });
        ui.add_space(4f32);

        egui::ScrollArea::vertical()
            .max_height(available_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut to_delete = None;

                let graphs = asset_db.list_all_assets(AssetKind::FloorFlowGraphConfig);
                for (id, config_name) in graphs {
                    let section = &mut self.floor_flow_graph_section;
                    if !section.config_name_filter.is_empty() {
                        if !config_name.starts_with(&section.config_name_filter) {
                            continue;
                        }
                    }

                    let selected = section
                        .selected_config_id
                        .map(|it| it.eq(&id))
                        .unwrap_or(false);

                    let response = crate::app::editor_stage::thick_selector_button(
                        ui,
                        selected,
                        egui::Align2::LEFT_CENTER,
                        config_name,
                    );

                    let popup_id = ui.make_persistent_id(format!("выпадающее меню {}", id));

                    if response.clicked_by(egui::PointerButton::Primary) {
                        match section.selected_config_id {
                            Some(selected_id) if selected_id.eq(&id) => {}
                            _ => {
                                let config_text = asset_db.load_json5_asset(AssetKind::FloorFlowGraphConfig, id);
                                let config: FloorFlowGraphConfig = json5::from_str(&config_text)
                                    .expect("Failed to load floor flow graph config");
                                section.current_config = Some(config);
                                section.selected_config_name.clear();
                                section.selected_config_name += config_name;
                                section.selected_config_id = Some(id);
                            }
                        }
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
                        },
                    );
                }

                if let Some(id) = to_delete {
                    let section = &mut self.floor_flow_graph_section;
                    match section.selected_config_id {
                        Some(selected_id) if selected_id.eq(&id) => {
                            section.selected_config_id = None;
                            section.current_config = None;
                            section.selected_config_name.clear();
                        }
                        _ => {}
                    }
                    asset_db.delete_asset(AssetKind::FloorFlowGraphConfig, id);
                }
            });

        if ui.add_sized(
            [full_width, 24f32],
            Button::new("Создать граф этажей"),
        ).clicked() {
            let default_config = FloorFlowGraphConfig::default();
            let config_text = json5::to_string(&default_config)
                .expect("Failed to serialize default floor flow graph config");

            let section = &mut self.floor_flow_graph_section;
            section.current_config = Some(default_config);

            let id = asset_db.create_json5_asset(
                AssetKind::FloorFlowGraphConfig,
                "",
                &config_text,
            );
            section.selected_config_name.clear();
            section.selected_config_id = Some(id);
        }
    }

    pub(crate) fn draw_floor_flow_graph_editor(&mut self, ui: &mut Ui) {
        if self.floor_flow_graph_section.current_config.is_none() {
            return;
        }

        let section_id = self.floor_flow_graph_section.selected_config_id;
        let mut config_name = std::mem::take(&mut self.floor_flow_graph_section.selected_config_name);

        ui.horizontal(|ui| {
            ui.label("Название для редактора:");
            if ui.add(
                TextEdit::singleline(&mut config_name)
                    .desired_width(f32::INFINITY)
            ).changed() {
                if let Some(id) = section_id {
                    if let Ok(mut asset_db) = crate::assets::ASSET_DATABASE.lock() {
                        asset_db.rename_asset(AssetKind::FloorFlowGraphConfig, id, &config_name);
                    }
                }
            }
        });

        let section = &mut self.floor_flow_graph_section;
        section.selected_config_name = config_name;

        fn content_hash(config: &FloorFlowGraphConfig) -> u64 {
            let mut hasher = DefaultHasher::new();

            for (node_id, pos, node) in config.nodes_pos_ids() {
                node_id.0.hash(&mut hasher);
                pos.x.to_bits().hash(&mut hasher);
                pos.y.to_bits().hash(&mut hasher);
                match node {
                    FloorFlowNode::StartFloor(start) => {
                        0u8.hash(&mut hasher);
                        start.floor_id.hash(&mut hasher);
                    }
                    FloorFlowNode::Floor(floor) => {
                        1u8.hash(&mut hasher);
                        floor.floor_id.hash(&mut hasher);
                        floor.num_in_passages.hash(&mut hasher);
                    }
                }
            }

            for (out_pin, in_pin) in config.wires() {
                out_pin.node.0.hash(&mut hasher);
                out_pin.output.hash(&mut hasher);
                in_pin.node.0.hash(&mut hasher);
                in_pin.input.hash(&mut hasher);
            }

            hasher.finish()
        }

        if let Some(config) = &mut section.current_config {
            let hash_before = content_hash(config);

            config.show(
                &mut FloorFlowGraphViewer,
                &section.style,
                "floor_flow_graph",
                ui,
            );

            let hash_after = content_hash(config);
            if hash_before != hash_after {
                if let Some(id) = section_id {
                    if let Ok(mut asset_db) = crate::assets::ASSET_DATABASE.lock() {
                        asset_db.update_asset_mut(
                            AssetKind::FloorFlowGraphConfig,
                            id,
                            |buffer| json5::to_writer(buffer, config)
                        );
                    }
                }
            }
        }
    }
}
