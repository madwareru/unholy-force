use egui::Pos2;
use serde::{Deserialize, Serialize};
use tracing::error;
use crate::{
    app::game_stage::{EntityId, GameWorld},
    effect_mechanics::{
        get_entity_parameter_value,
        EffectControlFlow,
        EffectNodeImpl,
        EffectQueue,
        EFFECT_GRAPH_TARGET,
        nodes::{get_value_source_entity_id, SharedNodeData, ValueSource},
        EffectNode
    },
    game_config::{
        ConfigId,
        ConfigProvider,
        parameters::ParameterConfig
    },
};
use crate::effect_mechanics::EffectNodeId;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WaitForConditionNode {
    shared_node_data: SharedNodeData,
    value_source: ValueSource,
    condition_parameter_id: ConfigId<ParameterConfig>,
    then_node: Option<EffectNodeId>,
}
impl Into<EffectNode> for WaitForConditionNode {
    fn into(self) -> EffectNode {
        EffectNode::WaitForConditionNode(self)
    }
}

impl WaitForConditionNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        value_source: ValueSource,
        condition_parameter_id: ConfigId<ParameterConfig>,
        then_node: Option<EffectNodeId>
    ) -> Self {
        Self {
            shared_node_data,
            value_source,
            condition_parameter_id,
            then_node,
        }
    }
}

impl EffectNodeImpl for WaitForConditionNode {
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
        _effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {
        let Some(value_source_id) = get_value_source_entity_id(game_world, effect_id, self.value_source) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить значение условия завершилась неудачей"
            );
            return EffectControlFlow::Complete;
        };

        let Some(condition) = get_entity_parameter_value(
            game_config_provider,
            game_world,
            value_source_id,
            self.condition_parameter_id
        ) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить значение условия завершилась неудачей"
            );
            return EffectControlFlow::Complete;
        };

        if condition <= 0f32 {
            return EffectControlFlow::Suspend;
        }
        
        self.then_node
            .map(|id| EffectControlFlow::AndThen(id))
            .unwrap_or(EffectControlFlow::Complete)
    }
}