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
            EffectNodeInfo,
            Holder
        },
        add_entity_tag_count,
        EffectQueue,
        EffectControlFlow,
        EffectNodeImpl,
        EFFECT_GRAPH_TARGET,
        EffectNode
    },
    app::game_stage::{EntityId, GameWorld},
};
use crate::effect_mechanics::{get_entity_parameter_value, EffectNodeId};
use crate::effect_mechanics::nodes::get_direction_entity_id;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct AddTagNode {
    #[serde(default)]
    flags: u8,
    tag_config_id: ConfigId<TagConfig>,
    value_parameter_id: ConfigId<ParameterConfig>,
    then_node: Option<EffectNodeId>,
}
impl Into<EffectNode> for AddTagNode {
    fn into(self) -> EffectNode {
        EffectNode::AddTagNode(self)
    }
}

impl AddTagNode {
    fn sign_flipped(&self) -> bool {
        self.flags & 0b0000_0001 == 0b0000_0001
    }
    fn tag_holder(&self) -> Holder {
        match (self.flags & 0b0000_0110) >> 1 {
            0b01 => Holder::Caster,
            0b10 => Holder::Target,
            _ => Holder::Global
        }
    }
    fn value_holder(&self) -> Holder {
        match (self.flags & 0b0001_1000) >> 3 {
            0b01 => Holder::Caster,
            0b10 => Holder::Target,
            _ => Holder::Global
        }
    }

    pub fn new(
        sign_flipped: bool,
        tag_holder: Holder,
        tag_config_id: ConfigId<TagConfig>,
        value_holder: Holder,
        value_parameter_id: ConfigId<ParameterConfig>,
        then_node: Option<EffectNodeId>,
    ) -> Self {
        let flags = 0u8;
        let flags = if !sign_flipped { flags } else { flags | 0b0000_0001 };
        let flags = match tag_holder {
            Holder::Global => flags,
            Holder::Caster => flags | 0b00_01_0,
            Holder::Target => flags | 0b00_10_0,
        };
        let flags = match value_holder {
            Holder::Global => flags,
            Holder::Caster => flags | 0b01_00_0,
            Holder::Target => flags | 0b10_00_0,
        };
        Self {
            flags,
            tag_config_id,
            value_parameter_id,
            then_node,
        }
    }
}

impl EffectNodeImpl for AddTagNode {
    fn tick(
        &self,
        _node_info: EffectNodeInfo,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {
        let Some(value_source_id) = get_direction_entity_id(game_world, effect_id, self.value_holder()) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Не удалось получить количество лычек для добавления"
            );
            return EffectControlFlow::Complete;
        };

        let Some(value) = get_entity_parameter_value(
            game_config_provider,
            game_world,
            value_source_id,
            self.value_parameter_id
        ) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Не удалось получить количество лычек для добавления"
            );
            return EffectControlFlow::Complete;
        };

        let Some(target_id) = get_direction_entity_id(game_world, effect_id, self.tag_holder()) else {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить цель для выдачи лычек провалилась"
            );
            return EffectControlFlow::Complete;
        };

        if !add_entity_tag_count(
            game_config_provider,
            game_world,
            target_id,
            self.tag_config_id,
            if self.sign_flipped() { -value } else { value },
            effect_queue
        ) {
            info!(
                target: EFFECT_GRAPH_TARGET,
                "Сущность, на которую предполагалось наложить лычки, перестала существовать. Цепочка прервана"
            );
            return EffectControlFlow::Complete;
        }

        self.then_node
            .map(|id| EffectControlFlow::AndThen(id))
            .unwrap_or(EffectControlFlow::Complete)
    }
}