use egui::Pos2;
use egui_snarl::NodeId;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use crate::{
    game_config::{
        ConfigId,
        ConfigProvider,
        parameters::{ParameterConfig, TagConfig}
    },
    effect_mechanics::{
        nodes::{
            get_effect_context,
            get_effect_env,
            get_effect_env_mut,
            get_memoized_parameter_value,
            SharedNodeData,
            ValueSource
        },
        add_entity_tag_count,
        EffectQueue,
        EffectFlow,
        EffectNodeImpl,
        EFFECT_GRAPH_TARGET,
        EffectNode
    },
    app::game_stage::{EntityId, GameWorld},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddTagNode {
    shared_node_data: SharedNodeData,
    value_source: ValueSource,
    value_parameter_id: ConfigId<ParameterConfig>,
    tag_config_id: ConfigId<TagConfig>,
    then_node: EffectNode,
}
impl Into<EffectNode> for AddTagNode {
    fn into(self) -> EffectNode {
        EffectNode::AddTagNode(Box::new(self))
    }
}

impl AddTagNode {
    pub fn new(
        shared_node_data: SharedNodeData,
        value_source: ValueSource,
        value_parameter_id: ConfigId<ParameterConfig>,
        tag_config_id: ConfigId<TagConfig>,
        then_node: EffectNode,
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

impl EffectNodeImpl for AddTagNode {
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
        effect_queue: &mut EffectQueue
    ) -> EffectFlow {
        const TAG_GIVEN_HASH: &str = "tag_is_given";

        let Some(value) = get_memoized_parameter_value(
            self,
            game_config_provider,
            game_world,
            effect_id,
            self.value_source,
            self.value_parameter_id
        ) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Не удалось получить количество лычек для добавления"
            );
            return EffectFlow::Complete;
        };

        let tag_given = match get_effect_env(game_world, effect_id) {
            Some(effect_env) => effect_env.get(self, TAG_GIVEN_HASH).is_some(),
            _ => {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка проверить статус выдачи лычки провалилась"
                );
                return EffectFlow::Complete;
            }
        };

        if !tag_given {
            let target_id = match get_effect_context(game_world, effect_id) {
                Some(effect_context) => effect_context.target_id,
                None => {
                    error!(
                        target: EFFECT_GRAPH_TARGET,
                        "Попытка получить цель для выдачи лычек провалилась"
                    );
                    return EffectFlow::Complete;
                }
            };

            if !add_entity_tag_count(
                game_config_provider,
                game_world,
                target_id,
                self.tag_config_id,
                value,
                effect_queue
            ) {
                info!(
                    target: EFFECT_GRAPH_TARGET,
                    "Сущность, на которую предполагалось наложить лычки, перестала существовать. Цепочка прервана"
                );
                return EffectFlow::Complete;
            }

            get_effect_env_mut(game_world, effect_id)
                .map(|mut effect_env| effect_env.set(self, TAG_GIVEN_HASH, 1f32));
        }

        self.then_node.tick(game_config_provider, game_world, effect_id, effect_queue)
    }
}