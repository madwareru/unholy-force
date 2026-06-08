use egui::Pos2;
use egui_snarl::NodeId;
use tracing::error;
use crate::app::game_stage::{EntityId, GameWorld};
use crate::effect_mechanics::{DelayedEffectQueue, EffectFlow, EffectNode, EFFECT_GRAPH_TARGET};
use crate::effect_mechanics::nodes::{get_effect_context, get_effect_env, get_effect_env_mut, SharedNodeData};
use crate::game_config::{ConfigId, ConfigProvider};
use crate::game_config::effects::EffectConfig;

pub struct SpawnSubEffectNode{
    shared_node_data: SharedNodeData,
    effect_config_id: ConfigId<EffectConfig>,
    then_node: Box<dyn EffectNode>
}

impl SpawnSubEffectNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        effect_config_id: ConfigId<EffectConfig>,
        then_node: Box<dyn EffectNode>
    ) -> Self {
        Self { shared_node_data, effect_config_id, then_node }
    }
}

impl EffectNode for SpawnSubEffectNode{
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
        const EFFECT_HAS_SPAWN_HASH: &str = "effect_has_spawn";

        let effect_has_spawn = match get_effect_env(game_world, effect_id) {
            Some(effect_env) => effect_env.get(self, EFFECT_HAS_SPAWN_HASH).is_some(),
            _ => {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка проверить статус порождения эффекта провалилась"
                );
                return EffectFlow::Complete;
            }
        };

        if !effect_has_spawn {
            match get_effect_context(game_world, effect_id) {
                Some(effect_context) => {
                    delayed_effect_queue.push(self.effect_config_id, *effect_context);
                }
                _ => {
                    error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка породить эффект провалилась"
                );
                    return EffectFlow::Complete;
                }
            }

            get_effect_env_mut(game_world, effect_id)
                .map(|mut effect_env| effect_env.set(self, EFFECT_HAS_SPAWN_HASH, 1f32));
        }

        self.then_node.tick(game_config_provider, game_world, effect_id, delayed_effect_queue)
    }
}