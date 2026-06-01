use egui::{Color32, PopupCloseBehavior, Ui};
use egui_snarl::{InPin, NodeId, OutPin, Snarl};
use egui_snarl::ui::{AnyPins, PinInfo, SnarlPin, SnarlViewer, WireStyle};
use serde::{Deserialize, Serialize};
use crate::assets::{AssetKind, ASSET_DATABASE};
use crate::game_config::{Config, ConfigId};
use crate::game_config::floors::FloorConfig;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FloorFlowNode {
    StartFloor(StartFloorNode),
    Floor(FloorNode)
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct StartFloorNode {
    pub floor_id: ConfigId<FloorConfig>,
    cached_name: String,
}
impl StartFloorNode {
    fn floor_name(&self) -> &str {
        match self {
            StartFloorNode { floor_id, cached_name } =>
                if ConfigId::INVALID.eq(floor_id) { "Нет данных" } else { cached_name.as_str() },
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FloorNode {
    pub floor_id: ConfigId<FloorConfig>,
    pub num_in_passages: u8,
    cached_name: String,
}
impl FloorNode {
    fn floor_name(&self) -> &str {
        match self {
            FloorNode { floor_id, cached_name, .. } =>
                if ConfigId::INVALID.eq(floor_id) { "Нет данных" } else { cached_name.as_str() },
        }
    }
}

const START_FLOOR_COLOR: Color32 = Color32::from_rgb(177 / 3, 93 / 3, 62 / 3);
const FLOOR_COLOR: Color32 = Color32::from_rgb(158 / 3, 177 / 3, 62 / 3);
const FLOOR_PIN_COLOR: Color32 = Color32::from_rgb(158, 177, 62);

pub struct FloorFlowGraphViewer;

impl SnarlViewer<FloorFlowNode> for FloorFlowGraphViewer {
    fn title(&mut self, node: &FloorFlowNode) -> String {
        match node {
            FloorFlowNode::StartFloor(_) => "Стартовый этаж".to_owned(),
            FloorFlowNode::Floor(FloorNode{ num_in_passages, .. }) => match num_in_passages {
                1 => "Этаж с 1 входом".to_owned(),
                x => format!("Этаж с {x} входами")
            }
        }
    }

    fn header_frame(
        &mut self,
        frame: egui::Frame,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<FloorFlowNode>,
    ) -> egui::Frame {
        match snarl[node] {
            FloorFlowNode::StartFloor(_) => frame.fill(START_FLOOR_COLOR),
            FloorFlowNode::Floor(_) => frame.fill(FLOOR_COLOR),
        }
    }

    fn inputs(&mut self, node: &FloorFlowNode) -> usize {
        match node {
            FloorFlowNode::StartFloor(_) => 0,
            FloorFlowNode::Floor(FloorNode { num_in_passages, ..} ) => *num_in_passages as usize,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FloorFlowNode>,
    ) -> impl SnarlPin + 'static {
        match snarl[pin.id.node] {
            FloorFlowNode::StartFloor(_) => {
                unreachable!("Number node has no inputs")
            }
            FloorFlowNode::Floor(_) => {
                match &*pin.remotes {
                    [] => {
                        ui.label("None");
                        PinInfo::square().with_fill(FLOOR_PIN_COLOR)
                    },
                    [remote] => match snarl[remote.node] {
                        FloorFlowNode::StartFloor(ref data) => {
                            ui.label(data.floor_name());
                            PinInfo::square().with_fill(FLOOR_PIN_COLOR).with_wire_style(
                                WireStyle::Bezier5,
                            )
                        },
                        | FloorFlowNode::Floor(ref data) => {
                            ui.label(data.floor_name());
                            PinInfo::square().with_fill(FLOOR_PIN_COLOR).with_wire_style(
                                WireStyle::Bezier5,
                            )
                        }
                    },
                    _ => unreachable!(),
                }
            }
        }
    }

    fn outputs(&mut self, node: &FloorFlowNode) -> usize {
        match node {
            FloorFlowNode::StartFloor(_) => 1,
            FloorFlowNode::Floor(_) => 1,
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FloorFlowNode>,
    ) -> impl SnarlPin + 'static {
        match snarl[pin.id.node] {
            FloorFlowNode::StartFloor(ref mut value) => {
                let response = ui.button(value.floor_name());
                let popup_id = ui.make_persistent_id("выбор стартового этажа");
                if response.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }
                egui::popup_below_widget(
                    ui,
                    popup_id,
                    &response,
                    PopupCloseBehavior::CloseOnClickOutside,
                    |ui| {
                        if let Ok(asset_db) = ASSET_DATABASE.lock() {
                            let floor_config_assets = asset_db.list_all_assets(AssetKind::FloorConfig);
                            for (uuid, _) in floor_config_assets {
                                let asset_text = asset_db.load_json5_asset(AssetKind::FloorConfig, uuid);
                                let config: FloorConfig = json5::from_str(&asset_text).expect("Failed to load floor config");
                                if ui.button(&config.name).clicked() {
                                    value.floor_id = ConfigId::from_uuid(uuid);
                                    value.cached_name = config.name;
                                    ui.memory_mut(|mem| mem.close_popup());
                                }
                            }
                        }
                    },
                );
                PinInfo::square()
                    .with_fill(FLOOR_PIN_COLOR)
                    .with_wire_style(WireStyle::Bezier5)
            },
            FloorFlowNode::Floor(ref mut value) => {
                let response = ui.button(value.floor_name());
                let popup_id = ui.make_persistent_id("выбор этажа");
                if response.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }
                egui::popup_below_widget(
                    ui,
                    popup_id,
                    &response,
                    PopupCloseBehavior::CloseOnClickOutside,
                    |ui| {
                        if let Ok(asset_db) = ASSET_DATABASE.lock() {
                            let floor_config_assets = asset_db.list_all_assets(AssetKind::FloorConfig);
                            for (uuid, _) in floor_config_assets {
                                let asset_text = asset_db.load_json5_asset(AssetKind::FloorConfig, uuid);
                                let config: FloorConfig = json5::from_str(&asset_text).expect("Failed to load floor config");
                                if ui.button(&config.name).clicked() {
                                    value.floor_id = ConfigId::from_uuid(uuid);
                                    value.cached_name = config.name;
                                    ui.memory_mut(|mem| mem.close_popup());
                                }
                            }
                        }
                    },
                );
                PinInfo::square()
                    .with_fill(FLOOR_PIN_COLOR)
                    .with_wire_style(WireStyle::Bezier5)
            }
        }
    }

    fn has_body(&mut self, node: &FloorFlowNode) -> bool {
        matches!(node, FloorFlowNode::Floor(_))
    }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FloorFlowNode>
    ) {
        let Some(node_data) = snarl.get_node_mut(node) else {
            return;
        };
        match node_data {
            FloorFlowNode::StartFloor(_) => {}
            FloorFlowNode::Floor(data) => {
                ui.horizontal(|ui| {
                    ui.label("Количество входов:");
                    ui.add(egui::DragValue::new(&mut data.num_in_passages)
                        .range(1..=8)
                    );
                });
            }
        }
    }

    fn has_on_hover_popup(&mut self, _: &FloorFlowNode) -> bool {
        false
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<FloorFlowNode>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FloorFlowNode>,
    ) {
        ui.label("Добавить узел");
        if ui.button("Стартовый этаж").clicked() {
            snarl.insert_node(pos, FloorFlowNode::StartFloor(Default::default()));
            ui.close_menu();
        }
        if ui.button("Этаж с 1 входом").clicked() {
            snarl.insert_node(pos, FloorFlowNode::Floor(
                FloorNode { num_in_passages: 1, ..Default::default() }
            ));
            ui.close_menu();
        }
        if ui.button("Этаж с 2 входами").clicked() {
            snarl.insert_node(pos, FloorFlowNode::Floor(
                FloorNode { num_in_passages: 2, ..Default::default() }
            ));
            ui.close_menu();
        }
        if ui.button("Этаж с 3 входами").clicked() {
            snarl.insert_node(pos, FloorFlowNode::Floor(
                FloorNode { num_in_passages: 3, ..Default::default() }
            ));
            ui.close_menu();
        }
    }

    fn has_dropped_wire_menu(&mut self, _src_pins: AnyPins, _snarl: &mut Snarl<FloorFlowNode>) -> bool {
        false
    }

    fn has_node_menu(&mut self, _node: &FloorFlowNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<FloorFlowNode>,
    ) {
        ui.label("Node menu");
        if ui.button("Remove").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
    }

    #[inline]
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<FloorFlowNode>) {
        match (&snarl[from.id.node], &snarl[to.id.node]) {
            (_, FloorFlowNode::StartFloor(_)) => {
                unreachable!("StartFloor has no inputs")
            }
            (FloorFlowNode::StartFloor(_), _) => {}
            (FloorFlowNode::Floor(_), _) => {}
        }
        for &remote in &to.remotes {
            snarl.disconnect(remote, to.id);
        }
        snarl.connect(from.id, to.id);
    }
}

pub type FloorFlowGraphConfig = Snarl<FloorFlowNode>;

impl Config for FloorFlowGraphConfig {}