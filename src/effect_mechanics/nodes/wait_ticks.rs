use serde::{Deserialize, Serialize};
use tracing::error;
use crate::{
    app::game_stage::{EntityId, GameWorld},
    effect_mechanics::{
        EffectQueue,
        EffectControlFlow,
        EffectNodeImpl,
        EFFECT_GRAPH_TARGET,
        nodes::{
            EffectNodeInfo,
            Holder,
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
use crate::effect_mechanics::EffectNodeId;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WaitTicksNode {
    value_source: Holder,
    tick_count_parameter_id: ConfigId<ParameterConfig>,
    then_node: Option<EffectNodeId>
}
impl Into<EffectNode> for WaitTicksNode {
    fn into(self) -> EffectNode {
        EffectNode::WaitTicksNode(self)
    }
}

impl WaitTicksNode {
    pub fn new(
        value_source: Holder,
        tick_count_parameter_id: ConfigId<ParameterConfig>,
        then_node: Option<EffectNodeId>
    ) -> Self {
        Self {
            value_source,
            tick_count_parameter_id,
            then_node
        }
    }
}

impl EffectNodeImpl for WaitTicksNode {
    fn tick(
        &self,
        node_info: EffectNodeInfo,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        _effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {
        const TICK_COUNT_HASH: &str = "tick_count_elapsed";

        // Так как данный узел может быть проигран множество раз,
        // для корректной его работы количество тиков, которое нужно ждать,
        // вычисляется только один раз через механизм мемоизации
        let Some(ticks_to_wait) = get_memoized_parameter_value(
            node_info,
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
            return EffectControlFlow::Complete;
        };

        match get_effect_env_mut(game_world, effect_id) {
            Some(mut effect_env) => {
                let mut tick_count_elapsed = effect_env.get(node_info, TICK_COUNT_HASH).unwrap_or(0f32);
                if tick_count_elapsed < ticks_to_wait {
                    tick_count_elapsed += 1f32;
                    effect_env.set(node_info, TICK_COUNT_HASH, tick_count_elapsed);
                    return EffectControlFlow::Suspend;
                }
            }
            _ => {
                error!(
                    target: EFFECT_GRAPH_TARGET,
                    "Попытка обновить количество тиков для ожидания провалилась"
                );
                return EffectControlFlow::Complete;
            }
        }

        self.then_node
            .map(|id| EffectControlFlow::AndThen(id))
            .unwrap_or(EffectControlFlow::Complete)
    }
}