use serde::{Deserialize, Serialize};
use crate::app::game_stage::{EntityId, GameWorld};
use crate::effect_mechanics::{EffectControlFlow, EffectNode, EffectNodeId, EffectNodeImpl, EffectQueue};
use crate::effect_mechanics::nodes::EffectNodeInfo;
use crate::game_config::ConfigProvider;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct JoinNode {
    then_node: Option<EffectNodeId>,
}
impl Into<EffectNode> for JoinNode {
    fn into(self) -> EffectNode {
        EffectNode::JoinNode(self)
    }
}

impl JoinNode {
    pub fn new(then_node: Option<EffectNodeId>) -> Self {
        Self { then_node }
    }
}

impl EffectNodeImpl for JoinNode {
    fn tick(
        &self,
        _node_info: EffectNodeInfo,
        _game_config_provider: &ConfigProvider,
        _game_world: &mut GameWorld,
        _effect_id: EntityId,
        _effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {
        self.then_node
            .map(|id| EffectControlFlow::AndThen(id))
            .unwrap_or(EffectControlFlow::Complete)
    }
}