use egui::Pos2;
use serde::{Deserialize, Serialize};
use tracing::error;
use crate::{
    app::game_stage::{EntityId, GameWorld},
    effect_mechanics::{
        EffectQueue,
        EffectControlFlow,
        EffectNodeImpl,
        EFFECT_GRAPH_TARGET,
        EffectNode,
        nodes::{get_effect_context, SharedNodeData}
    },
    game_config::{
        ConfigId,
        ConfigProvider,
        effects::EffectConfig
    }
};
use crate::effect_mechanics::EffectNodeId;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpawnSubEffectNode{
    shared_node_data: SharedNodeData,
    effect_config_id: ConfigId<EffectConfig>,
    then_node: Option<EffectNodeId>
}
impl Into<EffectNode> for SpawnSubEffectNode {
    fn into(self) -> EffectNode {
        EffectNode::SpawnSubEffectNode(self)
    }
}

impl SpawnSubEffectNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        effect_config_id: ConfigId<EffectConfig>,
        then_node: Option<EffectNodeId>
    ) -> Self {
        Self { shared_node_data, effect_config_id, then_node }
    }
}

impl EffectNodeImpl for SpawnSubEffectNode{
    fn get_node_id(&self) -> EffectNodeId {
        self.shared_node_data.node_id
    }

    fn get_node_pos(&self) -> Pos2 {
        self.shared_node_data.pos
    }

    fn tick(
        &self,
        _game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {
        match get_effect_context(game_world, effect_id) {
            Some(effect_context) => {
                effect_queue.push(self.effect_config_id, *effect_context);
            }
            _ => {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка породить эффект провалилась"
                );
                return EffectControlFlow::Complete;
            }
        }

        self.then_node
            .map(|id| EffectControlFlow::AndThen(id))
            .unwrap_or(EffectControlFlow::Complete)
    }
}