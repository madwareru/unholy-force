use egui::Pos2;
use egui_snarl::NodeId;
use serde::{Deserialize, Serialize};
use tracing::error;
use crate::{
    app::game_stage::{EntityId, GameWorld},
    effect_mechanics::{
        EffectQueue,
        EffectFlow,
        EffectNodeImpl,
        EFFECT_GRAPH_TARGET,
        nodes::{
            SharedNodeData,
            ValueSource,
            get_effect_env_mut
        },
        EffectNode,
        nodes::get_memoized_parameter_value
    },
    game_config::{
        ConfigId,
        ConfigProvider,
        parameters::ParameterConfig
    },
};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaitTicksNode {
    shared_node_data: SharedNodeData,
    value_source: ValueSource,
    tick_count_parameter_id: ConfigId<ParameterConfig>,
    then_node: EffectNode
}
impl Into<EffectNode> for WaitTicksNode {
    fn into(self) -> EffectNode {
        EffectNode::WaitTicksNode(Box::new(self))
    }
}

impl WaitTicksNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        value_source: ValueSource,
        tick_count_parameter_id: ConfigId<ParameterConfig>,
        then_node: EffectNode
    ) -> Self {
        Self {
            shared_node_data,
            value_source,
            tick_count_parameter_id,
            then_node
        }
    }
}

impl EffectNodeImpl for WaitTicksNode {
    fn get_node_id(&self) -> NodeId { self.shared_node_data.node_id }

    fn get_node_pos(&self) -> Pos2 { self.shared_node_data.pos }

    fn tick(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) -> EffectFlow {
        const TICK_COUNT_HASH: &str = "tick_count_elapsed";

        let Some(ticks_to_wait) = get_memoized_parameter_value(
            self,
            game_config_provider,
            game_world,
            effect_id,
            self.value_source,
            self.tick_count_parameter_id
        ) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить количество тиков для ожидания завершилась неудачей"
            );
            return EffectFlow::Complete;
        };

        match get_effect_env_mut(game_world, effect_id) {
            Some(mut effect_env) => {
                let mut tick_count_elapsed = effect_env.get(self, TICK_COUNT_HASH).unwrap_or(0f32);
                if tick_count_elapsed < ticks_to_wait {
                    tick_count_elapsed += 1f32;
                    effect_env.set(self, TICK_COUNT_HASH, tick_count_elapsed);
                    return EffectFlow::Continue;
                }
            }
            _ => {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка обновить количество тиков для ожидания провалилась"
                );
                return EffectFlow::Complete;
            }
        }
        self.then_node.tick(game_config_provider, game_world, effect_id, effect_queue)
    }
}