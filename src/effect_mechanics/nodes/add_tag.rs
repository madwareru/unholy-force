use egui::Pos2;
use egui_snarl::NodeId;
use tracing::error;
use crate::app::game_stage::{EntityId, GameWorld};
use crate::effect_mechanics::{add_entity_tag_count, get_entity_parameter_value, DelayedEffectQueue, EffectFlow, EffectNode, EFFECT_GRAPH_TARGET};
use crate::effect_mechanics::nodes::{get_effect_context, get_value_source_entity_id, SharedNodeData, ValueSource};
use crate::game_config::{ConfigId, ConfigProvider};
use crate::game_config::parameters::{ParameterConfig, TagConfig};

pub struct AddTagNode {
    shared_node_data: SharedNodeData,
    value_source: ValueSource,
    value_parameter_id: ConfigId<ParameterConfig>,
    tag_config_id: ConfigId<TagConfig>,
    then_node: Box<dyn EffectNode>,
}

impl AddTagNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        value_source: ValueSource,
        value_parameter_id: ConfigId<ParameterConfig>,
        tag_config_id: ConfigId<TagConfig>,
        then_node: Box<dyn EffectNode>,
    ) -> Self {
        Self {
            shared_node_data,
            value_source,
            value_parameter_id,
            tag_config_id,
            then_node,
        }
    }
}

impl EffectNode for AddTagNode {
    fn get_node_id(&self) -> NodeId {
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
        delayed_effect_queue: &mut DelayedEffectQueue
    ) -> EffectFlow {
        let value = {
            let Some(value_source_id) = get_value_source_entity_id(game_world, effect_id, self.value_source) else {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка получить значение для установки лычки провалилась"
                );
                return EffectFlow::Complete;
            };

            let Some(value) = get_entity_parameter_value(
                game_config_provider,
                game_world,
                value_source_id,
                self.value_parameter_id
            ) else {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка получить значение для установки лычки провалилась"
                );
                return EffectFlow::Complete;
            };

            value
        };

        let target_id = match get_effect_context(game_world, effect_id) {
            Some(effect_context) => effect_context.target_id,
            None => {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка получить цель для выдачи лычки провалилась"
                );
                return EffectFlow::Complete;
            }
        };

        add_entity_tag_count(
            game_config_provider,
            game_world,
            target_id,
            self.tag_config_id,
            value,
            delayed_effect_queue
        );

        self.then_node.tick(game_config_provider, game_world, effect_id, delayed_effect_queue)
    }
}