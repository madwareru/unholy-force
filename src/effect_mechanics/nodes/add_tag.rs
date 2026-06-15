use egui::Pos2;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use crate::{
    game_config::{
        ConfigId,
        ConfigProvider,
        parameters::{ParameterConfig, TagConfig}
    },
    effect_mechanics::{
        nodes::{
            SharedNodeData,
            Holder
        },
        add_entity_tag_count,
        EffectQueue,
        EffectControlFlow,
        EffectNodeImpl,
        EFFECT_GRAPH_TARGET,
        EffectNode
    },
    app::game_stage::{EntityId, GameWorld},
};
use crate::effect_mechanics::{get_entity_parameter_value, EffectNodeId};
use crate::effect_mechanics::nodes::get_direction_entity_id;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct AddTagNode {
    shared_node_data: SharedNodeData,
    #[serde(default)]
    sign_flipped: bool,
    tag_holder: Holder,
    tag_config_id: ConfigId<TagConfig>,
    value_holder: Holder,
    value_parameter_id: ConfigId<ParameterConfig>,
    then_node: Option<EffectNodeId>,
}
impl Into<EffectNode> for AddTagNode {
    fn into(self) -> EffectNode {
        EffectNode::AddTagNode(self)
    }
}

impl AddTagNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        sign_flipped: bool,
        tag_holder: Holder,
        tag_config_id: ConfigId<TagConfig>,
        value_holder: Holder,
        value_parameter_id: ConfigId<ParameterConfig>,
        then_node: Option<EffectNodeId>,
    ) -> Self {
        Self {
            sign_flipped,
            shared_node_data,
            tag_holder,
            tag_config_id,
            value_holder,
            value_parameter_id,
            then_node,
        }
    }
}

impl EffectNodeImpl for AddTagNode {
    fn get_node_id(&self) -> EffectNodeId {
        self.shared_node_data.node_id
    }

    fn get_node_pos(&self) -> Pos2 {
        self.shared_node_data.pos
    }

    fn tick(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {
        let Some(value_source_id) = get_direction_entity_id(game_world, effect_id, self.value_holder) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Не удалось получить количество лычек для добавления"
            );
            return EffectControlFlow::Complete;
        };

        let Some(value) = get_entity_parameter_value(
            game_config_provider,
            game_world,
            value_source_id,
            self.value_parameter_id
        ) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Не удалось получить количество лычек для добавления"
            );
            return EffectControlFlow::Complete;
        };

        let Some(target_id) = get_direction_entity_id(game_world, effect_id, self.tag_holder) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить цель для выдачи лычек провалилась"
            );
            return EffectControlFlow::Complete;
        };

        if !add_entity_tag_count(
            game_config_provider,
            game_world,
            target_id,
            self.tag_config_id,
            if self.sign_flipped { -value } else { value },
            effect_queue
        ) {
            info!(
                target: EFFECT_GRAPH_TARGET,
                "Сущность, на которую предполагалось наложить лычки, перестала существовать. Цепочка прервана"
            );
            return EffectControlFlow::Complete;
        }

        self.then_node
            .map(|id| EffectControlFlow::AndThen(id))
            .unwrap_or(EffectControlFlow::Complete)
    }
}