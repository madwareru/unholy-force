use egui::Pos2;
use serde::{Deserialize, Serialize};
use crate::app::game_stage::{EntityId, GameWorld};
use crate::effect_mechanics::{EffectControlFlow, EffectNode, EffectNodeId, EffectNodeImpl, EffectQueue};
use crate::effect_mechanics::nodes::SharedNodeData;
use crate::game_config::ConfigProvider;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct JoinNode {
    shared_node_data: SharedNodeData,
    then_node: Option<EffectNodeId>,
}
impl Into<EffectNode> for JoinNode {
    fn into(self) -> EffectNode {
        EffectNode::JoinNode(self)
    }
}

impl JoinNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        then_node: Option<EffectNodeId>,
    ) -> Self {
        Self { shared_node_data, then_node }
    }
}

impl EffectNodeImpl for JoinNode {
    fn get_node_id(&self) -> EffectNodeId {
        self.shared_node_data.node_id
    }

    fn get_node_pos(&self) -> Pos2 {
        self.shared_node_data.pos
    }

    fn tick(
        &self, 
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