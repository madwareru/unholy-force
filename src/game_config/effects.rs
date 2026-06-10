use std::hash::{DefaultHasher, Hash, Hasher};
use egui::{pos2, Color32, Frame, Pos2, TextureId, Ui};
use egui_snarl::{InPin, NodeId, OutPin, OutPinId, Snarl};
use egui_snarl::ui::{PinInfo, SnarlPin, SnarlStyle, SnarlViewer, WireStyle};
use serde::{Deserialize, Serialize};
use crate::{
    assets::{AssetDb},
    app::editor_stage::widgets::{parameter_config_id_button, parameter_selector_popup, tag_config_id_button, tag_selector_popup},
    effect_mechanics::{
        EffectRoot,
        nodes::{
            ValueSource,
            add_tag::AddTagNode,
            SharedNodeData,
            spawn_sub_effect::SpawnSubEffectNode,
            terminator::TerminatorNode,
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
use crate::app::editor_stage::widgets::SpriteHolder;

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
    Branch(BranchEffectGraphNode),
    WaitForCondition(WaitForConditionEffectGraphNode),
    WaitForTicks(WaitForTicksEffectGraphNode),
    SpawnSubEffect(SpawnSubEffectGraphNode),
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntryPointEffectGraphNode {
    comment: String,
}
impl EntryPointEffectGraphNode {
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct AddTagEffectGraphNode {
    value_source: ValueSource,
    value_parameter_id: ConfigId<ParameterConfig>,
    tag_config_id: ConfigId<TagConfig>,
    comment: String,
}
impl AddTagEffectGraphNode {
    pub fn value_source(&self) -> ValueSource {
        self.value_source
    }
    pub fn value_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.value_parameter_id
    }
    pub fn tag_id(&self) -> ConfigId<TagConfig> {
        self.tag_config_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BranchEffectGraphNode {
    value_source: ValueSource,
    condition_parameter_id: ConfigId<ParameterConfig>,
    comment: String,
}
impl BranchEffectGraphNode {
    pub fn value_source(&self) -> ValueSource {
        self.value_source
    }
    pub fn condition_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.condition_parameter_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WaitForConditionEffectGraphNode {
    value_source: ValueSource,
    condition_parameter_id: ConfigId<ParameterConfig>,
    comment: String,
}
impl WaitForConditionEffectGraphNode {
    pub fn value_source(&self) -> ValueSource {
        self.value_source
    }
    pub fn condition_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.condition_parameter_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WaitForTicksEffectGraphNode {
    value_source: ValueSource,
    tick_count_parameter_id: ConfigId<ParameterConfig>,
    comment: String,
}
impl WaitForTicksEffectGraphNode {
    pub fn value_source(&self) -> ValueSource {
        self.value_source
    }
    pub fn tick_count_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.tick_count_parameter_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SpawnSubEffectGraphNode {
    effect_config_id: ConfigId<EffectConfig>,
    comment: String,
}
impl SpawnSubEffectGraphNode {
    pub fn effect_config_id(&self) -> ConfigId<EffectConfig> {
        self.effect_config_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

const ENTRY_NODE_COLOR: Color32 = Color32::from_rgb(177 / 3, 93 / 3, 62 / 3);
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
            EffectGraphNode::EntryPoint(_) => "Точка входа".to_owned(),
            EffectGraphNode::AddTag(_) => "Добавить/снять лычки".to_owned(),
            EffectGraphNode::Branch(_) => "Распутье".to_owned(),
            EffectGraphNode::WaitForCondition(_) => "Ждать возможности".to_owned(),
            EffectGraphNode::WaitForTicks(_) => "Отсчитать и продолжить".to_owned(),
            EffectGraphNode::SpawnSubEffect(_) => "Спровоцировать эффект".to_owned(),
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
            EffectGraphNode::EntryPoint(_) => frame.fill(ENTRY_NODE_COLOR),
            _ => frame.fill(NODE_COLOR),
        }
    }

    fn inputs(&mut self, node: &EffectGraphNode) -> usize {
        match node {
            EffectGraphNode::EntryPoint(_) => 0,
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
                [remote] => match snarl[remote.node] {
                    EffectGraphNode::EntryPoint(_) => {
                        ui.label(match remote.output {
                            0 => "Точка входа (установка)",
                            1 => "Точка входа (шаг)",
                            2 => "Точка входа (очистка)",
                            _ => unreachable!()
                        });
                        PinInfo::square()
                            .with_fill(PIN_COLOR)
                            .with_wire_style(WireStyle::Bezier5)
                    }
                    EffectGraphNode::AddTag(_) => {
                        ui.label("Лычки");
                        PinInfo::square()
                            .with_fill(PIN_COLOR)
                            .with_wire_style(WireStyle::Bezier5)
                    }
                    EffectGraphNode::Branch(_) => {
                        ui.label(match remote.output {
                            0 => "Распутье (да)",
                            1 => "Распутье (нет)",
                            _ => unreachable!()
                        });
                        PinInfo::square()
                            .with_fill(PIN_COLOR)
                            .with_wire_style(WireStyle::Bezier5)
                    }
                    EffectGraphNode::WaitForCondition(_) => {
                        ui.label("Выжидание");
                        PinInfo::square()
                            .with_fill(PIN_COLOR)
                            .with_wire_style(WireStyle::Bezier5)
                    }
                    EffectGraphNode::WaitForTicks(_) => {
                        ui.label("Отсчёт");
                        PinInfo::square()
                            .with_fill(PIN_COLOR)
                            .with_wire_style(WireStyle::Bezier5)
                    }
                    EffectGraphNode::SpawnSubEffect(_) => {
                        ui.label("Провокация");
                        PinInfo::square()
                            .with_fill(PIN_COLOR)
                            .with_wire_style(WireStyle::Bezier5)
                    }
                },
                _ => unreachable!(),
            }
        }
    }

    fn outputs(&mut self, node: &EffectGraphNode) -> usize {
        match node {
            EffectGraphNode::EntryPoint(_) => 3,
            EffectGraphNode::AddTag(_) => 1,
            EffectGraphNode::Branch(_) => 2,
            EffectGraphNode::WaitForCondition(_) => 1,
            EffectGraphNode::WaitForTicks(_) => 1,
            EffectGraphNode::SpawnSubEffect(_) => 1,
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
            EffectGraphNode::EntryPoint(data) => {
                ui.vertical(|ui| {
                    ui.label("Комментарий:");
                    ui.text_edit_multiline(&mut data.comment);
                });
            }
            EffectGraphNode::AddTag(data) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui|{
                        ui.label("Источник значения:");
                        egui::ComboBox::from_id_salt("value_source")
                            .selected_text(data.value_source.display_name())
                            .show_ui(ui, |ui| {
                                for v in [ValueSource::Global, ValueSource::Caster, ValueSource::Target ] {
                                    ui.selectable_value(
                                        &mut data.value_source,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                    });

                    ui.label("Количество лычек:");
                    let response = parameter_config_id_button(
                        ui,
                        &self.asset_db,
                        false,
                        texture_id,
                        atlas_size,
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

                    ui.label("Лычка:");
                    let response = tag_config_id_button(
                        ui,
                        &self.asset_db,
                        false,
                        texture_id,
                        atlas_size,
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

                    ui.label("Комментарий:");
                    ui.text_edit_multiline(&mut data.comment);
                });
            }
            EffectGraphNode::Branch(data) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui|{
                        ui.label("Источник значения:");
                        egui::ComboBox::from_id_salt("value_source")
                            .selected_text(data.value_source.display_name())
                            .show_ui(ui, |ui| {
                                for v in [ValueSource::Global, ValueSource::Caster, ValueSource::Target ] {
                                    ui.selectable_value(
                                        &mut data.value_source,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                    });

                    ui.label("Условие:");
                    let response = parameter_config_id_button(
                        ui,
                        &self.asset_db,
                        false,
                        texture_id,
                        atlas_size,
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

                    ui.label("Комментарий:");
                    ui.text_edit_multiline(&mut data.comment);
                });
            }
            EffectGraphNode::WaitForCondition(data) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui|{
                        ui.label("Источник значения:");
                        egui::ComboBox::from_id_salt("value_source")
                            .selected_text(data.value_source.display_name())
                            .show_ui(ui, |ui| {
                                for v in [ValueSource::Global, ValueSource::Caster, ValueSource::Target ] {
                                    ui.selectable_value(
                                        &mut data.value_source,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                    });

                    ui.label("Условие:");
                    let response = parameter_config_id_button(
                        ui,
                        &self.asset_db,
                        false,
                        texture_id,
                        atlas_size,
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

                    ui.label("Комментарий:");
                    ui.text_edit_multiline(&mut data.comment);
                });
            }
            EffectGraphNode::WaitForTicks(data) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui|{
                        ui.label("Источник значения:");
                        egui::ComboBox::from_id_salt("value_source")
                            .selected_text(data.value_source.display_name())
                            .show_ui(ui, |ui| {
                                for v in [ValueSource::Global, ValueSource::Caster, ValueSource::Target ] {
                                    ui.selectable_value(
                                        &mut data.value_source,
                                        v,
                                        v.display_name()
                                    );
                                }
                            });
                    });

                    ui.label("Количество шагов:");
                    let response = parameter_config_id_button(
                        ui,
                        &self.asset_db,
                        false,
                        texture_id,
                        atlas_size,
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

                    ui.label("Комментарий:");
                    ui.text_edit_multiline(&mut data.comment);
                });
            }
            EffectGraphNode::SpawnSubEffect(data) => {
                ui.vertical(|ui| {
                    // todo: выбор эффекта тут добавить
                    ui.label("Комментарий:");
                    ui.text_edit_multiline(&mut data.comment);
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
        if ui.button("Добавить/снять лычки").clicked(){
            snarl.insert_node(pos, EffectGraphNode::AddTag(Default::default()));
            ui.close_menu();
        } else if ui.button("Распутье").clicked(){
            snarl.insert_node(pos, EffectGraphNode::Branch(Default::default()));
            ui.close_menu();
        } else if ui.button("Ждать возможности").clicked(){
            snarl.insert_node(pos, EffectGraphNode::WaitForCondition(Default::default()));
            ui.close_menu();
        } else if ui.button("Отсчитать и продолжить").clicked(){
            snarl.insert_node(pos, EffectGraphNode::WaitForTicks(Default::default()));
            ui.close_menu();
        } else if ui.button("Спровоцировать эффект").clicked(){
            snarl.insert_node(pos, EffectGraphNode::SpawnSubEffect(Default::default()));
            ui.close_menu();
        }
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
        let entry_node_id = snarl.insert_node(
            pos,
            EffectGraphNode::EntryPoint(
                EntryPointEffectGraphNode {
                    comment: "".to_string(),
                }
            )
        );
        let shared_node_data = SharedNodeData {
            node_id: entry_node_id,
            pos
        };
        Self {
            description: Default::default(),
            sprite_name: Default::default(),
            sprite_pivot: Default::default(),
            snarl,
            compiled_root: EffectRoot::new(
                TerminatorNode::new(shared_node_data).into(),
                TerminatorNode::new(shared_node_data).into(),
                TerminatorNode::new(shared_node_data).into()
            )
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
                    EffectGraphNode::EntryPoint(entry) => {
                        0u8.hash(&mut hasher);
                        entry.comment().hash(&mut hasher);
                    }
                    EffectGraphNode::AddTag(add_tag) => {
                        1u8.hash(&mut hasher);
                        add_tag.value_source().hash(&mut hasher);
                        add_tag.value_parameter_id().hash(&mut hasher);
                        add_tag.tag_id().hash(&mut hasher);
                        add_tag.comment().hash(&mut hasher);
                    }
                    EffectGraphNode::Branch(branch) => {
                        2u8.hash(&mut hasher);
                        branch.value_source().hash(&mut hasher);
                        branch.condition_parameter_id().hash(&mut hasher);
                        branch.comment().hash(&mut hasher);
                    }
                    EffectGraphNode::WaitForCondition(wait_for) => {
                        3u8.hash(&mut hasher);
                        wait_for.value_source().hash(&mut hasher);
                        wait_for.condition_parameter_id().hash(&mut hasher);
                        wait_for.comment().hash(&mut hasher);
                    }
                    EffectGraphNode::WaitForTicks(wait_ticks) => {
                        4u8.hash(&mut hasher);
                        wait_ticks.value_source().hash(&mut hasher);
                        wait_ticks.tick_count_parameter_id().hash(&mut hasher);
                        wait_ticks.comment().hash(&mut hasher);
                    }
                    EffectGraphNode::SpawnSubEffect(spawn_sub_effect) => {
                        5u8.hash(&mut hasher);
                        spawn_sub_effect.effect_config_id().hash(&mut hasher);
                        spawn_sub_effect.comment().hash(&mut hasher);
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
        fn parse(node_id: NodeId, snarl: &Snarl<EffectGraphNode>) -> EffectNode {
            let shared_data = SharedNodeData {
                node_id,
                pos: snarl.get_node_info(node_id)
                    .map(|it| it.pos)
                    .expect("There should be a node info"),
            };
            match &snarl[node_id] {
                EffectGraphNode::EntryPoint(_) => {
                    unreachable!("There should be no entrypoint node during traversal")
                }
                EffectGraphNode::AddTag(add_tag_data) => {
                    let then_pin = snarl.out_pin(OutPinId { node: node_id, output: 0 });
                    let then_node = match &then_pin.remotes[..] {
                        [] => TerminatorNode::new(shared_data).into(),
                        [then_remote] => {
                            parse(then_remote.node, snarl)
                        },
                        _ => unreachable!("There should be at most one then pin")
                    };
                    AddTagNode::new(
                        shared_data,
                        add_tag_data.value_source(),
                        add_tag_data.value_parameter_id(),
                        add_tag_data.tag_id(),
                        then_node
                    ).into()
                }
                EffectGraphNode::Branch(branch_data) => {
                    let then_pin = snarl.out_pin(OutPinId { node: node_id, output: 0 });
                    let then_node = match &then_pin.remotes[..] {
                        [] => TerminatorNode::new(shared_data).into(),
                        [then_remote] => {
                            parse(then_remote.node, snarl)
                        },
                        _ => unreachable!("There should be at most one then pin")
                    };

                    let else_pin = snarl.out_pin(OutPinId { node: node_id, output: 0 });
                    let else_node = match &else_pin.remotes[..] {
                        [] => TerminatorNode::new(shared_data).into(),
                        [else_remote] => {
                            parse(else_remote.node, snarl)
                        },
                        _ => unreachable!("There should be at most one else pin")
                    };

                    BranchNode::new(
                        shared_data,
                        branch_data.value_source(),
                        branch_data.condition_parameter_id(),
                        then_node,
                        else_node
                    ).into()
                }
                EffectGraphNode::WaitForCondition(wait_cond_data) => {
                    let then_pin = snarl.out_pin(OutPinId { node: node_id, output: 0 });
                    let then_node = match &then_pin.remotes[..] {
                        [] => TerminatorNode::new(shared_data).into(),
                        [then_remote] => {
                            parse(then_remote.node, snarl)
                        },
                        _ => unreachable!("There should be at most one then pin")
                    };
                    WaitForConditionNode::new(
                        shared_data,
                        wait_cond_data.value_source(),
                        wait_cond_data.condition_parameter_id(),
                        then_node
                    ).into()
                }
                EffectGraphNode::WaitForTicks(wait_ticks_data) => {
                    let then_pin = snarl.out_pin(OutPinId { node: node_id, output: 0 });
                    let then_node = match &then_pin.remotes[..] {
                        [] => TerminatorNode::new(shared_data).into(),
                        [then_remote] => {
                            parse(then_remote.node, snarl)
                        },
                        _ => unreachable!("There should be at most one then pin")
                    };
                    WaitTicksNode::new(
                        shared_data,
                        wait_ticks_data.value_source(),
                        wait_ticks_data.tick_count_parameter_id(),
                        then_node
                    ).into()
                }
                EffectGraphNode::SpawnSubEffect(spawn_sub_effect_data) => {
                    let then_pin = snarl.out_pin(OutPinId { node: node_id, output: 0 });
                    let then_node = match &then_pin.remotes[..] {
                        [] => TerminatorNode::new(shared_data).into(),
                        [then_remote] => {
                            parse(then_remote.node, snarl)
                        },
                        _ => unreachable!("There should be at most one then pin")
                    };
                    SpawnSubEffectNode::new(
                        shared_data,
                        spawn_sub_effect_data.effect_config_id(),
                        then_node
                    ).into()
                }
            }
        }

        let (entry_node_id, entry_node) = self.snarl
            .node_ids()
            .find(|it| matches!(it.1, EffectGraphNode::EntryPoint(_)))
            .expect("No entry point found");

        let EffectGraphNode::EntryPoint(_) = entry_node else { unreachable!() };
        let shared_data = SharedNodeData {
            node_id: entry_node_id,
            pos: self.snarl.get_node_info(entry_node_id)
                .map(|it| it.pos)
                .expect("There should be a node info"),
        };

        let setup_pin = self.snarl.out_pin(OutPinId { node: entry_node_id, output: 0 });
        let tick_pin = self.snarl.out_pin(OutPinId { node: entry_node_id, output: 1 });
        let on_destroy_pin = self.snarl.out_pin(OutPinId { node: entry_node_id, output: 2 });
        let setup_node = match &setup_pin.remotes[..] {
            [] => TerminatorNode::new(shared_data).into(),
            [setup_remote] => {
                parse(setup_remote.node, &self.snarl)
            },
            _ => unreachable!("there should be at most one setup pin"),
        };

        let tick_node = match &tick_pin.remotes[..] {
            [] => TerminatorNode::new(shared_data).into(),
            [tick_remote] => {
                parse(tick_remote.node, &self.snarl)
            },
            _ => unreachable!("there should be at most one tick pin")
        };

        let on_destroy_node = match &on_destroy_pin.remotes[..] {
            [] => TerminatorNode::new(shared_data).into(),
            [on_destroy_remote] => {
                parse(on_destroy_remote.node, &self.snarl)
            },
            _ => unreachable!("there should be at most one on_destroy pin"),
        };

        self.compiled_root = EffectRoot::new(setup_node, tick_node, on_destroy_node);
    }
}

impl Config for EffectConfig {}