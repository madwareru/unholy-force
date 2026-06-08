use egui::Pos2;
use egui_snarl::NodeId;
use crate::app::game_stage::{EntityId, GameWorld};
use crate::effect_mechanics::{EffectQueue, EffectFlow, EffectNode};
use crate::effect_mechanics::nodes::SharedNodeData;
use crate::game_config::ConfigProvider;

pub struct TerminatorNode {
    shared_node_data: SharedNodeData
}

impl EffectNode for TerminatorNode {
    fn get_node_id(&self) -> NodeId {
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
    ) -> EffectFlow {
        EffectFlow::Complete
    }
}