use crate::{
    app::game_stage::{EntityId, GameWorld},
    effect_mechanics::{
        nodes::{SharedNodeData, ValueSource},
        EffectQueue,
        EffectFlow,
        EffectNodeImpl,
        EFFECT_GRAPH_TARGET,
        EffectNode,
        nodes::{get_memoized_parameter_value}
    },
    game_config::{
        parameters::ParameterConfig,
        ConfigId,
        ConfigProvider
    }
};
use egui::Pos2;
use egui_snarl::NodeId;
use serde::{Deserialize, Serialize};
use tracing::{error};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchNode {
    shared_node_data: SharedNodeData,
    value_source: ValueSource,
    condition_parameter_id: ConfigId<ParameterConfig>,
    then_node: EffectNode,
    else_node: EffectNode,
}
impl Into<EffectNode> for BranchNode {
    fn into(self) -> EffectNode {
        EffectNode::BranchNode(Box::new(self))
    }
}

impl BranchNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        value_source: ValueSource,
        condition_parameter_id: ConfigId<ParameterConfig>,
        then_node: EffectNode,
        else_node: EffectNode
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
    fn get_node_id(&self) -> NodeId { self.shared_node_data.node_id }

    fn get_node_pos(&self) -> Pos2 { self.shared_node_data.pos }

    fn tick(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) -> EffectFlow {
        let Some(condition) = get_memoized_parameter_value(
            self,
            game_config_provider,
            game_world,
            effect_id,
            self.value_source,
            self.condition_parameter_id
        ) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить значение условия завершилась неудачей"
            );
            return EffectFlow::Complete;
        };

        // При отрицательности значения считаем, что надо уйти в else ветку, иначе в then
        if condition > 0f32 {
            self.then_node.tick(game_config_provider, game_world, effect_id, effect_queue)
        } else {
            self.else_node.tick(game_config_provider, game_world, effect_id, effect_queue)
        }
    }
}