use std::hash::{DefaultHasher, Hash, Hasher};
use egui::{pos2, Button, Color32, Frame, Pos2, TextureId, Ui};
use egui::ahash::{HashMap, HashMapExt};
use egui_snarl::{InPin, NodeId, OutPin, OutPinId, Snarl};
use egui_snarl::ui::{PinInfo, SnarlPin, SnarlStyle, SnarlViewer, WireStyle};
use serde::{Deserialize, Serialize};
use crate::{
    assets::{AssetDb},
    app::editor_stage::widgets::{parameter_config_id_button_small, parameter_selector_popup, tag_config_id_button_small, tag_selector_popup},
    effect_mechanics::{
        EffectRoot,
        nodes::{
            Holder,
            add_tag::AddTagNode,
            SharedNodeData,
            spawn_sub_effect::SpawnSubEffectNode,
            wait_cond::WaitForConditionNode,
            wait_ticks::WaitTicksNode,
            branch::BranchNode
        },
        EffectNode,
    },
    game_config::{
        Config,
        ConfigId,
        parameters::{ParameterConfig, TagConfig}
    },
};
use crate::app::editor_stage::widgets::{effect_config_id_button, effect_selector_popup, SpriteHolder};
use crate::effect_mechanics::EffectNodeId;
use crate::effect_mechanics::nodes::join::JoinNode;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EffectConfig {
    pub description: String,
    pub sprite_name: String,
    pub sprite_pivot: [u8; 2],
    snarl: Snarl<EffectGraphNode>,
    compiled_root: EffectRoot,
}

impl SpriteHolder for EffectConfig {
    fn sprite_name(&self) -> &str {
        &self.sprite_name.as_str()
    }

    fn sprite_pivot(&self) -> &[u8; 2] {
        &self.sprite_pivot
    }

    fn sprite_pivot_mut(&mut self) -> &mut [u8; 2] {
        &mut self.sprite_pivot
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EffectGraphNode {
    EntryPoint(EntryPointEffectGraphNode),
    AddTag(AddTagEffectGraphNode),
    DecTag(AddTagEffectGraphNode),
    Branch(BranchEffectGraphNode),
    Join(JoinEffectGraphNode),
    WaitForCondition(WaitForConditionEffectGraphNode),
    WaitForTicks(WaitForTicksEffectGraphNode),
    SpawnSubEffect(SpawnSubEffectGraphNode),
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntryPointEffectGraphNode;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct JoinEffectGraphNode;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct AddTagEffectGraphNode {
    value_holder: Holder,
    value_parameter_id: ConfigId<ParameterConfig>,
    tag_holder: Holder,
    tag_config_id: ConfigId<TagConfig>,
}
impl AddTagEffectGraphNode {
    pub fn value_holder(&self) -> Holder {
        self.value_holder
    }
    pub fn value_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.value_parameter_id
    }
    pub fn tag_holder(&self) -> Holder {
        self.tag_holder
    }
    pub fn tag_id(&self) -> ConfigId<TagConfig> {
        self.tag_config_id
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BranchEffectGraphNode {
    value_source: Holder,
    condition_parameter_id: ConfigId<ParameterConfig>,
}
impl BranchEffectGraphNode {
    pub fn value_source(&self) -> Holder {
        self.value_source
    }
    pub fn condition_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.condition_parameter_id
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WaitForConditionEffectGraphNode {
    value_source: Holder,
    condition_parameter_id: ConfigId<ParameterConfig>,
}
impl WaitForConditionEffectGraphNode {
    pub fn value_source(&self) -> Holder {
        self.value_source
    }
    pub fn condition_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.condition_parameter_id
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WaitForTicksEffectGraphNode {
    value_source: Holder,
    tick_count_parameter_id: ConfigId<ParameterConfig>,
}
impl WaitForTicksEffectGraphNode {
    pub fn value_source(&self) -> Holder {
        self.value_source
    }
    pub fn tick_count_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.tick_count_parameter_id
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SpawnSubEffectGraphNode {
    effect_config_id: ConfigId<EffectConfig>,
    caster: Holder,
    target: Holder,
}
impl SpawnSubEffectGraphNode {
    pub fn effect_config_id(&self) -> ConfigId<EffectConfig> {
        self.effect_config_id
    }
    pub fn caster(&self) -> Holder {
        self.caster
    }
    pub fn target(&self) -> Holder {
        self.target
    }
}

const ENTRY_NODE_COLOR: Color32 = Color32::from_rgb(177 / 3, 93 / 3, 62 / 3);
const SUSPENDABLE_NODE_COLOR: Color32 = Color32::from_rgb(158 / 3, 177 / 3, 62 / 3);
const CONTROL_FLOW_NODE_COLOR: Color32 = Color32::from_rgb(62 / 3, 152 / 3, 177 / 3);
const NODE_COLOR: Color32 = Color32::from_rgb(217 / 3, 117 / 3, 54 / 3);
const PIN_COLOR: Color32 = Color32::from_rgb(217, 117, 54);

pub struct EffectGraphViewer<'a> {
    asset_db: &'a AssetDb,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
}
impl<'a> EffectGraphViewer<'a> {
    pub fn new(
        asset_db: &'a AssetDb,
        atlas_texture: TextureId,
        atlas_size: [u16; 2]
    ) -> Self {
        Self { asset_db, atlas_texture, atlas_size }
    }
}

impl<'a> SnarlViewer<EffectGraphNode> for EffectGraphViewer<'a> {
    fn title(&mut self, node: &EffectGraphNode) -> String {
        match node {
            EffectGraphNode::EntryPoint(_) => "Эффект".to_owned(),
            EffectGraphNode::AddTag(_) => "Добавь".to_owned(),
            EffectGraphNode::DecTag(_) => "Убавь".to_owned(),
            EffectGraphNode::Branch(_) => "Развилка".to_owned(),
            EffectGraphNode::WaitForCondition(_) => "Жди возможности".to_owned(),
            EffectGraphNode::WaitForTicks(_) => "Жди отсчёта".to_owned(),
            EffectGraphNode::SpawnSubEffect(_) => "Стреляй".to_owned(),
            EffectGraphNode::Join(_) => "Объединение".to_owned()
        }
    }

    fn header_frame(
        &mut self,
        frame: Frame,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        snarl: &Snarl<EffectGraphNode>
    ) -> Frame {
        match snarl[node] {
            | EffectGraphNode::EntryPoint(_) => frame.fill(ENTRY_NODE_COLOR),
            | EffectGraphNode::WaitForTicks(_)
            | EffectGraphNode::WaitForCondition(_) => frame.fill(SUSPENDABLE_NODE_COLOR),
            | EffectGraphNode::Branch(_)
            | EffectGraphNode::Join(_) => frame.fill(CONTROL_FLOW_NODE_COLOR),
            _ => frame.fill(NODE_COLOR),
        }
    }

    fn inputs(&mut self, node: &EffectGraphNode) -> usize {
        match node {
            EffectGraphNode::EntryPoint(_) => 0,
            EffectGraphNode::Join(_) => 2,
            _ => 1,
        }
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<EffectGraphNode>
    ) -> impl SnarlPin + 'static {
        match snarl[pin.id.node] {
            EffectGraphNode::EntryPoint(_) => {
                unreachable!("EntryPoint has no inputs")
            }
            _ => match &*pin.remotes {
                [] => {
                    ui.label("Нет данных");
                    PinInfo::square().with_fill(PIN_COLOR)
                },
                [_] => PinInfo::square()
                    .with_fill(PIN_COLOR)
                    .with_wire_style(WireStyle::Bezier5),
                _ => unreachable!(),
            }
        }
    }

    fn outputs(&mut self, node: &EffectGraphNode) -> usize {
        match node {
            EffectGraphNode::EntryPoint(_) => 3,
            EffectGraphNode::AddTag(_) => 1,
            EffectGraphNode::DecTag(_) => 1,
            EffectGraphNode::Branch(_) => 2,
            EffectGraphNode::WaitForCondition(_) => 1,
            EffectGraphNode::WaitForTicks(_) => 1,
            EffectGraphNode::SpawnSubEffect(_) => 1,
            EffectGraphNode::Join(_) => 1
        }
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<EffectGraphNode>
    ) -> impl SnarlPin + 'static {
        ui.label(match (&snarl[pin.id.node], pin.id.output) {
            (EffectGraphNode::EntryPoint(_), 0) => "Установка",
            (EffectGraphNode::EntryPoint(_), 1) => "Шаг",
            (EffectGraphNode::EntryPoint(_), 2) => "Очистка",
            (EffectGraphNode::Branch(_), 0) => "Да",
            (EffectGraphNode::Branch(_), 1) => "Нет",
            _ => "Далее",
        });
        PinInfo::square()
            .with_fill(PIN_COLOR)
            .with_wire_style(WireStyle::Bezier5)
    }

    fn has_body(&mut self, _node: &EffectGraphNode) -> bool { true }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<EffectGraphNode>
    ) {
        let Some(node_data) = snarl.get_node_mut(node) else {
            return;
        };

        let texture_id = self.atlas_texture;
        let atlas_size = self.atlas_size;

        match node_data {
            EffectGraphNode::EntryPoint(_) => {}
            EffectGraphNode::Join(_) => {}
              EffectGraphNode::AddTag(data)
            | EffectGraphNode::DecTag(data) => {
                ui.vertical(|ui| {
                    ui.label("Лычка:");
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt("tag_holder")
                            .selected_text(data.tag_holder.display_name())
                            .show_ui(ui, |ui| {
                                for v in [Holder::Global, Holder::Caster, Holder::Target ] {
                                    ui.selectable_value(
                                        &mut data.tag_holder,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                        ui.label(".");
                        let response = tag_config_id_button_small(
                            ui,
                            &self.asset_db,
                            false,
                            data.tag_config_id
                        );
                        let popup_id = ui.make_persistent_id("Выбор лычки");
                        if response.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                        }
                        tag_selector_popup(
                            ui,
                            &self.asset_db,
                            popup_id,
                            &response,
                            texture_id,
                            atlas_size,
                            |config_id| data.tag_config_id = config_id
                        );
                    });

                    ui.label("Значение:");
                    ui.horizontal(|ui|{
                        egui::ComboBox::from_id_salt("value_holder")
                            .selected_text(data.value_holder.display_name())
                            .show_ui(ui, |ui| {
                                for v in [Holder::Global, Holder::Caster, Holder::Target ] {
                                    ui.selectable_value(
                                        &mut data.value_holder,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                        ui.label(".");
                        let response = parameter_config_id_button_small(
                            ui,
                            &self.asset_db,
                            false,
                            data.value_parameter_id
                        );
                        let popup_id = ui.make_persistent_id("Выбор черты для значения количества лычек");
                        if response.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                        }
                        parameter_selector_popup(
                            ui,
                            &self.asset_db,
                            popup_id,
                            &response,
                            texture_id,
                            atlas_size,
                            |config_id| data.value_parameter_id = config_id
                        );
                    });
                });
            }
            EffectGraphNode::Branch(data) => {
                ui.vertical(|ui| {
                    ui.label("Условие:");
                    ui.horizontal(|ui|{
                        egui::ComboBox::from_id_salt("value_source")
                            .selected_text(data.value_source.display_name())
                            .show_ui(ui, |ui| {
                                for v in [Holder::Global, Holder::Caster, Holder::Target ] {
                                    ui.selectable_value(
                                        &mut data.value_source,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                        ui.label(".");
                        let response = parameter_config_id_button_small(
                            ui,
                            &self.asset_db,
                            false,
                            data.condition_parameter_id
                        );
                        let popup_id = ui.make_persistent_id("Выбор черты для условия ветвления");
                        if response.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                        }
                        parameter_selector_popup(
                            ui,
                            &self.asset_db,
                            popup_id,
                            &response,
                            texture_id,
                            atlas_size,
                            |config_id| data.condition_parameter_id = config_id
                        );
                    });
                });
            }
            EffectGraphNode::WaitForCondition(data) => {
                ui.vertical(|ui| {
                    ui.label("Условие:");
                    ui.horizontal(|ui|{
                        egui::ComboBox::from_id_salt("value_source")
                            .selected_text(data.value_source.display_name())
                            .show_ui(ui, |ui| {
                                for v in [Holder::Global, Holder::Caster, Holder::Target ] {
                                    ui.selectable_value(
                                        &mut data.value_source,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                        ui.label(".");
                        let response = parameter_config_id_button_small(
                            ui,
                            &self.asset_db,
                            false,
                            data.condition_parameter_id
                        );
                        let popup_id = ui.make_persistent_id("Выбор черты для условия ожидания");
                        if response.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                        }
                        parameter_selector_popup(
                            ui,
                            &self.asset_db,
                            popup_id,
                            &response,
                            texture_id,
                            atlas_size,
                            |config_id| data.condition_parameter_id = config_id
                        );
                    });
                });
            }
            EffectGraphNode::WaitForTicks(data) => {
                ui.vertical(|ui| {
                    ui.label("Количество шагов:");
                    ui.horizontal(|ui|{
                        egui::ComboBox::from_id_salt("value_source")
                            .selected_text(data.value_source.display_name())
                            .show_ui(ui, |ui| {
                                for v in [Holder::Global, Holder::Caster, Holder::Target ] {
                                    ui.selectable_value(
                                        &mut data.value_source,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                        ui.label(".");
                        let response = parameter_config_id_button_small(
                            ui,
                            &self.asset_db,
                            false,
                            data.tick_count_parameter_id
                        );
                        let popup_id = ui.make_persistent_id("Выбор черты для количества шагов");
                        if response.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                        }
                        parameter_selector_popup(
                            ui,
                            &self.asset_db,
                            popup_id,
                            &response,
                            texture_id,
                            atlas_size,
                            |config_id| data.tick_count_parameter_id = config_id
                        );
                    });
                });
            }
            EffectGraphNode::SpawnSubEffect(data) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui|{
                        egui::ComboBox::from_id_salt("caster")
                            .selected_text(data.caster.display_name())
                            .show_ui(ui, |ui| {
                                for v in [Holder::Global, Holder::Caster, Holder::Target ] {
                                    ui.selectable_value(&mut data.caster, v, v.display_name());
                                }
                            });
                        ui.label("->");
                        egui::ComboBox::from_id_salt("target")
                            .selected_text(data.target.display_name())
                            .show_ui(ui, |ui| {
                                for v in [Holder::Global, Holder::Caster, Holder::Target ] {
                                    ui.selectable_value(&mut data.target, v, v.display_name());
                                }
                            });
                    });
                    let response = effect_config_id_button(
                        ui,
                        &self.asset_db,
                        false,
                        texture_id,
                        atlas_size,
                        data.effect_config_id
                    );
                    let popup_id = ui.make_persistent_id("Выбор эффекта");
                    if response.clicked() {
                        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                    }
                    effect_selector_popup(
                        ui,
                        &self.asset_db,
                        popup_id,
                        &response,
                        texture_id,
                        atlas_size,
                        |config_id| data.effect_config_id = config_id
                    );
                });
            }
        }
    }

    fn has_graph_menu(&mut self, _pos: Pos2, _snarl: &mut Snarl<EffectGraphNode>) -> bool { true }

    fn show_graph_menu(
        &mut self,
        pos: Pos2,
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<EffectGraphNode>
    ) {
        ui.label("Добавить узел");
        ui.group(|ui| {
            ui.label("Контроль управления");
            if ui.add(Button::new("Развилка").fill(CONTROL_FLOW_NODE_COLOR)).clicked(){
                snarl.insert_node(pos, EffectGraphNode::Branch(Default::default()));
                ui.close_menu();
            }
            if ui.add(Button::new("Объединение").fill(CONTROL_FLOW_NODE_COLOR)).clicked(){
                snarl.insert_node(pos, EffectGraphNode::Join(Default::default()));
                ui.close_menu();
            }
        });
        ui.group(|ui| {
            ui.label("Продолжительные");
            if ui.add(Button::new("Жди возможности").fill(SUSPENDABLE_NODE_COLOR)).clicked(){
                snarl.insert_node(pos, EffectGraphNode::WaitForCondition(Default::default()));
                ui.close_menu();
            }
            if ui.add(Button::new("Жди отсчёта").fill(SUSPENDABLE_NODE_COLOR)).clicked(){
                snarl.insert_node(pos, EffectGraphNode::WaitForTicks(Default::default()));
                ui.close_menu();
            }
        });
        ui.group(|ui|{
            ui.label("Мгновенные");
            if ui.add(Button::new("Добавь").fill(NODE_COLOR)).clicked(){
                snarl.insert_node(pos, EffectGraphNode::AddTag(Default::default()));
                ui.close_menu();
            }
            if ui.add(Button::new("Убавь").fill(NODE_COLOR)).clicked(){
                snarl.insert_node(pos, EffectGraphNode::DecTag(Default::default()));
                ui.close_menu();
            }
            if ui.add(Button::new("Стреляй").fill(NODE_COLOR)).clicked(){
                snarl.insert_node(pos, EffectGraphNode::SpawnSubEffect(Default::default()));
                ui.close_menu();
            }
        });
    }

    fn has_node_menu(&mut self, _node: &EffectGraphNode) -> bool { true }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        _scale: f32,
        snarl: &mut Snarl<EffectGraphNode>
    ) {
        ui.label("Меню узла");
        if ui.button("Удалить узел").clicked() {
            snarl.remove_node(node);
            ui.close_menu();
        }
    }

    #[inline]
    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<EffectGraphNode>) {
        snarl.drop_outputs(from.id);
        snarl.drop_inputs(to.id);
        snarl.connect(from.id, to.id);
    }
}

impl EffectConfig {
    pub fn new() -> Self {
        let mut snarl = Snarl::new();
        let pos = pos2(0f32, 0f32);
        snarl.insert_node(
            pos,
            EffectGraphNode::EntryPoint(EntryPointEffectGraphNode)
        );
        Self {
            description: Default::default(),
            sprite_name: Default::default(),
            sprite_pivot: Default::default(),
            snarl,
            compiled_root: EffectRoot::new(Vec::new(), None, None, None)
        }
    }

    pub fn root_node(&self) -> EffectRoot {
        self.compiled_root.clone()
    }
    pub fn edit_snarl(
        &mut self,
        ui: &mut Ui,
        asset_db: &AssetDb,
        texture_id: TextureId,
        atlas_size: [u16; 2],
        style: &SnarlStyle,
    ) -> bool {
        fn content_hash(snarl: &Snarl<EffectGraphNode>) -> u64 {
            let mut hasher = DefaultHasher::new();

            for (node_id, pos, node) in snarl.nodes_pos_ids() {
                node_id.0.hash(&mut hasher);
                pos.x.to_bits().hash(&mut hasher);
                pos.y.to_bits().hash(&mut hasher);
                match node {
                    EffectGraphNode::EntryPoint(_) => {
                        0u8.hash(&mut hasher);
                    }
                    EffectGraphNode::AddTag(add_tag) => {
                        1u8.hash(&mut hasher);
                        add_tag.value_holder().hash(&mut hasher);
                        add_tag.value_parameter_id().hash(&mut hasher);
                        add_tag.tag_id().hash(&mut hasher);
                    }
                    EffectGraphNode::Branch(branch) => {
                        2u8.hash(&mut hasher);
                        branch.value_source().hash(&mut hasher);
                        branch.condition_parameter_id().hash(&mut hasher);
                    }
                    EffectGraphNode::WaitForCondition(wait_for) => {
                        3u8.hash(&mut hasher);
                        wait_for.value_source().hash(&mut hasher);
                        wait_for.condition_parameter_id().hash(&mut hasher);
                    }
                    EffectGraphNode::WaitForTicks(wait_ticks) => {
                        4u8.hash(&mut hasher);
                        wait_ticks.value_source().hash(&mut hasher);
                        wait_ticks.tick_count_parameter_id().hash(&mut hasher);
                    }
                    EffectGraphNode::SpawnSubEffect(spawn_sub_effect) => {
                        5u8.hash(&mut hasher);
                        spawn_sub_effect.effect_config_id().hash(&mut hasher);
                    }
                    EffectGraphNode::Join(_) => {
                        6u8.hash(&mut hasher);
                    }
                    EffectGraphNode::DecTag(dec_tag) => {
                        7u8.hash(&mut hasher);
                        dec_tag.value_holder().hash(&mut hasher);
                        dec_tag.value_parameter_id().hash(&mut hasher);
                        dec_tag.tag_id().hash(&mut hasher);
                    }
                }
            }

            for (out_pin, in_pin) in snarl.wires() {
                out_pin.node.0.hash(&mut hasher);
                out_pin.output.hash(&mut hasher);
                in_pin.node.0.hash(&mut hasher);
                in_pin.input.hash(&mut hasher);
            }

            hasher.finish()
        }

        let snarl = &mut self.snarl;
        let mut viewer = EffectGraphViewer::new(asset_db, texture_id, atlas_size);

        let hash_before = content_hash(snarl);

        snarl.show(&mut viewer, style, "floor_flow_graph", ui);

        let hash_after = content_hash(snarl);
        let is_changed = !hash_after.eq(&hash_before);
        if is_changed {
            self.recompile_root_node();
        }

        is_changed
    }

    fn recompile_root_node(&mut self) {
        fn parse_or_none(
            snarl: &Snarl<EffectGraphNode>,
            memoizer: &mut HashMap<NodeId, EffectNodeId>,
            nodes: &mut Vec<EffectNode>,
            node_id: NodeId,
            id: usize
        ) -> Option<EffectNodeId> {
            match snarl.out_pin(OutPinId { node: node_id, output: id }).remotes.as_slice() {
                [remote] =>
                    match memoizer.get(&remote.node).copied() {
                        Some(memoized_id) => Some(memoized_id),
                        _ => {
                            let id = parse(snarl, memoizer, nodes, remote.node);
                            memoizer.insert(remote.node, id);
                            Some(id)
                        }
                    }
                _ => None,
            }
        }

        fn parse(
            snarl: &Snarl<EffectGraphNode>,
            memoizer: &mut HashMap<NodeId, EffectNodeId>,
            nodes: &mut Vec<EffectNode>,
            node_id: NodeId
        ) -> EffectNodeId {
            let new_node = match &snarl[node_id] {
                EffectGraphNode::EntryPoint(_) => unreachable!(),
                EffectGraphNode::AddTag(add_tag_data) => {
                    let then_id = parse_or_none(snarl, memoizer, nodes, node_id, 0);
                    AddTagNode::new(
                        SharedNodeData {
                            node_id: EffectNodeId::new(nodes.len() as u32),
                            pos: snarl.get_node_info(node_id)
                                .map(|it| it.pos)
                                .expect("There should be a node info"),
                        },
                        false,
                        add_tag_data.tag_holder(),
                        add_tag_data.tag_id(),
                        add_tag_data.value_holder(),
                        add_tag_data.value_parameter_id(),
                        then_id
                    ).into()
                },
                EffectGraphNode::DecTag(dec_tag_data) => {
                    let then_id = parse_or_none(snarl, memoizer, nodes, node_id, 0);
                    AddTagNode::new(
                        SharedNodeData {
                            node_id: EffectNodeId::new(nodes.len() as u32),
                            pos: snarl.get_node_info(node_id)
                                .map(|it| it.pos)
                                .expect("There should be a node info"),
                        },
                        true,
                        dec_tag_data.tag_holder(),
                        dec_tag_data.tag_id(),
                        dec_tag_data.value_holder(),
                        dec_tag_data.value_parameter_id(),
                        then_id
                    ).into()
                },
                EffectGraphNode::Branch(branch_data) => {
                    let then_id = parse_or_none(snarl, memoizer, nodes, node_id, 0);
                    let else_id = parse_or_none(snarl, memoizer, nodes, node_id, 1);
                    BranchNode::new(
                        SharedNodeData {
                            node_id: EffectNodeId::new(nodes.len() as u32),
                            pos: snarl.get_node_info(node_id)
                                .map(|it| it.pos)
                                .expect("There should be a node info"),
                        },
                        branch_data.value_source(),
                        branch_data.condition_parameter_id(),
                        then_id,
                        else_id
                    ).into()
                },
                EffectGraphNode::Join(_) => {
                    let then_id = parse_or_none(snarl, memoizer, nodes, node_id, 0);
                    JoinNode::new(
                        SharedNodeData {
                            node_id: EffectNodeId::new(nodes.len() as u32),
                            pos: snarl.get_node_info(node_id)
                                .map(|it| it.pos)
                                .expect("There should be a node info"),
                        },
                        then_id
                    ).into()
                },
                EffectGraphNode::WaitForCondition(wait_cond_data) => {
                    let then_id = parse_or_none(snarl, memoizer, nodes, node_id, 0);
                    WaitForConditionNode::new(
                        SharedNodeData {
                            node_id: EffectNodeId::new(nodes.len() as u32),
                            pos: snarl.get_node_info(node_id)
                                .map(|it| it.pos)
                                .expect("There should be a node info"),
                        },
                        wait_cond_data.value_source(),
                        wait_cond_data.condition_parameter_id(),
                        then_id
                    ).into()
                },
                EffectGraphNode::WaitForTicks(wait_ticks_data) => {
                    let then_id = parse_or_none(snarl, memoizer, nodes, node_id, 0);
                    WaitTicksNode::new(
                        SharedNodeData {
                            node_id: EffectNodeId::new(nodes.len() as u32),
                            pos: snarl.get_node_info(node_id)
                                .map(|it| it.pos)
                                .expect("There should be a node info"),
                        },
                        wait_ticks_data.value_source(),
                        wait_ticks_data.tick_count_parameter_id(),
                        then_id
                    ).into()
                },
                EffectGraphNode::SpawnSubEffect(spawn_sub_effect_data) => {
                    let then_id = parse_or_none(snarl, memoizer, nodes, node_id, 0);
                    SpawnSubEffectNode::new(
                        SharedNodeData {
                            node_id: EffectNodeId::new(nodes.len() as u32),
                            pos: snarl.get_node_info(node_id)
                                .map(|it| it.pos)
                                .expect("There should be a node info"),
                        },
                        spawn_sub_effect_data.effect_config_id(),
                        spawn_sub_effect_data.caster(),
                        spawn_sub_effect_data.target(),
                        then_id
                    ).into()
                },
            };
            let id = EffectNodeId::new(nodes.len() as u32);
            nodes.push(new_node);
            id
        }

        let (entry_node_id, entry_node) = self.snarl
            .node_ids()
            .find(|it| matches!(it.1, EffectGraphNode::EntryPoint(_)))
            .expect("No entry point found");

        let EffectGraphNode::EntryPoint(_) = entry_node else { unreachable!() };
        let mut nodes = Vec::new();
        let mut memoizer = HashMap::new();
        let setup = parse_or_none(&self.snarl, &mut memoizer, &mut nodes, entry_node_id, 0);
        let tick = parse_or_none(&self.snarl, &mut memoizer, &mut nodes, entry_node_id, 1);
        let on_destroy = parse_or_none(&self.snarl, &mut memoizer, &mut nodes, entry_node_id, 2);

        self.compiled_root = EffectRoot::new(nodes, setup, tick, on_destroy);
    }
}


impl Config for EffectConfig {}