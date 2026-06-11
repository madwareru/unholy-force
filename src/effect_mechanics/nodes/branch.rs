use crate::{
    app::game_stage::{EntityId, GameWorld},
    effect_mechanics::{
        nodes::{SharedNodeData, ValueSource},
        EffectQueue,
        EffectControlFlow,
        EffectNodeImpl,
        EFFECT_GRAPH_TARGET,
        EffectNode,
    },
    game_config::{
        parameters::ParameterConfig,
        ConfigId,
        ConfigProvider
    }
};
use egui::Pos2;
use serde::{Deserialize, Serialize};
use tracing::{error};
use crate::effect_mechanics::{get_entity_parameter_value, EffectNodeId};
use crate::effect_mechanics::nodes::get_value_source_entity_id;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BranchNode {
    shared_node_data: SharedNodeData,
    value_source: ValueSource,
    condition_parameter_id: ConfigId<ParameterConfig>,
    then_node: Option<EffectNodeId>,
    else_node: Option<EffectNodeId>,
}
impl Into<EffectNode> for BranchNode {
    fn into(self) -> EffectNode {
        EffectNode::BranchNode(self)
    }
}

impl BranchNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        value_source: ValueSource,
        condition_parameter_id: ConfigId<ParameterConfig>,
        then_node: Option<EffectNodeId>,
        else_node: Option<EffectNodeId>
    ) -> Self {
        Self {
            shared_node_data,
            value_source,
            condition_parameter_id,
            then_node,
            else_node
        }
    }
}

impl EffectNodeImpl for BranchNode {
    fn get_node_id(&self) -> EffectNodeId { self.shared_node_data.node_id }

    fn get_node_pos(&self) -> Pos2 { self.shared_node_data.pos }

    fn tick(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        _effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {

        let Some(value_source_id) = get_value_source_entity_id(game_world, effect_id, self.value_source) else {
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

        // При отрицательности значения или при нуле считаем, что надо уйти в else ветку, иначе в then
        if condition > 0f32 {
            self.then_node
        } else {
            self.else_node
        }.map(|id| EffectControlFlow::AndThen(id))
            .unwrap_or(EffectControlFlow::Complete)
    }
}