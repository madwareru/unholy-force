use egui::Pos2;
use egui_snarl::NodeId;
use serde::{Deserialize, Serialize};
use crate::{
    app::game_stage::{EntityId, GameWorld},
    effect_mechanics::{
        EffectQueue,
        EffectFlow,
        EffectNodeImpl,
        EffectNode,
        nodes::SharedNodeData
    },
    game_config::ConfigProvider
};

/// Особый узел, возникающий при случае, когда связь в графе не выставлена.
/// Не представлен отдельным узлом в эффект графе.
/// Наследует идентификатор и позицию родительского узла
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminatorNode {
    shared_node_data: SharedNodeData
}
impl Into<EffectNode> for TerminatorNode {
    fn into(self) -> EffectNode {
        EffectNode::TerminatorNode(Box::new(self))
    }
}
impl TerminatorNode {
    pub fn new(shared_node_data: SharedNodeData) -> Self {
        Self { shared_node_data }
    }
}

impl EffectNodeImpl for TerminatorNode {
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