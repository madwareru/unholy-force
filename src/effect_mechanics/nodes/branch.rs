use crate::{
    app::game_stage::{EntityId, GameWorld},
    effect_mechanics::{
        nodes::{get_value_source_entity_id, SharedNodeData, ValueSource},
        get_entity_parameter_value,
        DelayedEffectQueue,
        EffectFlow,
        EffectNode,
        EFFECT_GRAPH_TARGET
    },
    game_config::parameters::ParameterConfig,
    game_config::{ConfigId, ConfigProvider}
};
use egui::Pos2;
use egui_snarl::NodeId;
use tracing::error;

pub struct BranchNode {
    shared_node_data: SharedNodeData,
    value_source: ValueSource,
    condition_parameter_id: ConfigId<ParameterConfig>,
    then_node: Box<dyn EffectNode>,
    else_node: Box<dyn EffectNode>,
}

impl BranchNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        value_source: ValueSource,
        condition_parameter_id: ConfigId<ParameterConfig>,
        then_node: Box<dyn EffectNode>,
        else_node: Box<dyn EffectNode>
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

impl EffectNode for BranchNode {
    fn get_node_id(&self) -> NodeId { self.shared_node_data.node_id }

    fn get_node_pos(&self) -> Pos2 { self.shared_node_data.pos }

    fn tick(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    ) -> EffectFlow {
        let condition = {
            let Some(value_source_id) = get_value_source_entity_id(game_world, effect_id, self.value_source) else {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка проверить условие в узле ветвления провалилась"
                );
                return EffectFlow::Complete;
            };

            let Some(value) = get_entity_parameter_value(
                game_config_provider,
                game_world,
                value_source_id,
                self.condition_parameter_id
            ) else {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка проверить условие в узле ветвления провалилась"
                );
                return EffectFlow::Complete;
            };

            value
        };

        if condition.abs() <= f32::EPSILON {
            self.else_node.tick(game_config_provider, game_world, effect_id, delayed_effect_queue)
        } else {
            self.then_node.tick(game_config_provider, game_world, effect_id, delayed_effect_queue)
        }
    }
}