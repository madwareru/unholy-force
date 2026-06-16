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
        nodes::{EffectNodeInfo}
    },
    game_config::{
        ConfigId,
        ConfigProvider,
        effects::EffectConfig
    }
};
use crate::effect_mechanics::{EffectContext, EffectNodeId};
use crate::effect_mechanics::nodes::{get_direction_entity_id, Holder};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SpawnSubEffectNode{
    effect_config_id: ConfigId<EffectConfig>,
    caster: Holder,
    target: Holder,
    then_node: Option<EffectNodeId>
}
impl Into<EffectNode> for SpawnSubEffectNode {
    fn into(self) -> EffectNode {
        EffectNode::SpawnSubEffectNode(self)
    }
}

impl SpawnSubEffectNode {
    pub fn new(
        effect_config_id: ConfigId<EffectConfig>,
        caster: Holder,
        target: Holder,
        then_node: Option<EffectNodeId>
    ) -> Self {
        Self { effect_config_id, caster, target, then_node }
    }
}

impl EffectNodeImpl for SpawnSubEffectNode{
    fn tick(
        &self,
        _node_info: EffectNodeInfo,
        _game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {
        let Some(caster_id) = get_direction_entity_id(game_world, effect_id, self.caster) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить источник для эффекта провалилась"
            );
            return EffectControlFlow::Complete;
        };

        let Some(target_id) = get_direction_entity_id(game_world, effect_id, self.target) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить приёмник для эффекта провалилась"
            );
            return EffectControlFlow::Complete;
        };

        effect_queue.push(self.effect_config_id, EffectContext::new(caster_id, target_id));

        self.then_node
            .map(|id| EffectControlFlow::AndThen(id))
            .unwrap_or(EffectControlFlow::Complete)
    }
}