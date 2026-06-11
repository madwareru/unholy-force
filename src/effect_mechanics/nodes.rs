use egui::Pos2;
use egui_snarl::NodeId;
use hecs::{Ref, RefMut};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use crate::{
    effect_mechanics::{
        get_entity_parameter_value,
        EffectContext,
        EffectEnv,
        EffectNodeImpl,
        EntityValueHolder,
        GlobalValuesHolder,
        EFFECT_GRAPH_TARGET
    },
    app::game_stage::{EntityId, GameWorld},
    game_config::{
        ConfigId,
        ConfigProvider,
        parameters::ParameterConfig
    }
};
use crate::effect_mechanics::EffectNodeId;

pub mod wait_ticks;
pub mod wait_cond;
pub mod branch;
pub mod spawn_sub_effect;
pub mod add_tag;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SharedNodeData{
    pub node_id: EffectNodeId,
    pub pos: Pos2,
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Hash)]
pub enum ValueSource {
    #[default]
    Global,
    Caster,
    Target
}
impl ValueSource {
    pub fn display_name(&self) -> &'static str {
        match self {
            ValueSource::Global => "Глобальное",
            ValueSource::Caster => "Источник",
            ValueSource::Target => "Приёмник"
        }
    }
}
pub fn get_effect_context(game_world: &GameWorld, effect_id: EntityId) -> Option<Ref<'_, EffectContext>> {
    match game_world.get::<&EffectContext>(effect_id) {
        Ok(context) => Some(context),
        _ => {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить контекст эффекта для {:?} провалилась",
                effect_id
            );
            None
        }
    }
}

pub fn get_memoized_parameter_value<N: EffectNodeImpl>(
    node: &N,
    game_config_provider: &ConfigProvider,
    game_world: &GameWorld,
    effect_id: EntityId,
    value_source: ValueSource,
    parameter_config_id: ConfigId<ParameterConfig>
) -> Option<f32> {
    let value_source_id = get_value_source_entity_id(game_world, effect_id, value_source)?;
    let memoized_value = get_effect_env(game_world, effect_id)?
        .get(node, parameter_config_id);

    let value = match memoized_value {
        Some(value) => value,
        None => {
            let value = get_entity_parameter_value(
                game_config_provider,
                game_world,
                value_source_id,
                parameter_config_id
            )?;

            get_effect_env_mut(game_world, effect_id)
                .map(|mut effect_env| effect_env.set(node, parameter_config_id, value));

            value
        }
    };

    Some(value)
}

pub fn get_effect_env(game_world: &GameWorld, effect_id: EntityId) -> Option<Ref<'_, EffectEnv>> {
    match game_world.get::<&EffectEnv>(effect_id) {
        Ok(env) => Some(env),
        _ => {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить окружение эффекта для {:?} провалилась",
                effect_id
            );
            None
        }
    }
}

pub fn get_effect_env_mut(game_world: &GameWorld, effect_id: EntityId) -> Option<RefMut<'_, EffectEnv>> {
    match game_world.get::<&mut EffectEnv>(effect_id) {
        Ok(env) => Some(env),
        _ => {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить окружение эффекта для {:?} провалилась",
                effect_id
            );
            None
        }
    }
}

pub fn get_value_holder(game_world: &GameWorld, entity_id: EntityId) -> Option<Ref<'_, EntityValueHolder>> {
    match game_world.get::<&EntityValueHolder>(entity_id) {
        Ok(context) => Some(context),
        _ => {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить хранилище значений для сущности {:?} провалилась",
                entity_id
            );
            None
        }
    }
}

pub fn get_value_holder_mut(game_world: &mut GameWorld, entity_id: EntityId) -> Option<RefMut<'_, EntityValueHolder>> {
    match game_world.get::<&mut EntityValueHolder>(entity_id) {
        Ok(context) => Some(context),
        _ => {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить хранилище значений для сущности {:?} провалилась",
                entity_id
            );
            None
        }
    }
}

pub fn get_value_source_entity_id(
    game_world: &GameWorld,
    effect_id: EntityId,
    value_source: ValueSource
) -> Option<EntityId>  {
    match get_effect_context(game_world, effect_id) {
        Some(context) => match value_source {
            ValueSource::Global => GlobalValuesHolder::get_entity_id(game_world),
            ValueSource::Caster => Some(context.caster_id),
            ValueSource::Target => Some(context.target_id)
        },
        _ => {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить источник значений эффекта для {:?} провалилась",
                effect_id
            );
            None
        }
    }
}

pub fn effect_context_is_expired(game_world: &GameWorld, effect_id: EntityId) -> bool {
    match get_effect_context(game_world, effect_id) {
        Some(context) =>  {
            if game_world.entity(context.caster_id).is_err() {
                info!(
                    target: EFFECT_GRAPH_TARGET,
                    "Источник эффекта {:?} перестал существовать",
                    effect_id
                );
                true
            } else if game_world.entity(context.target_id).is_err() {
                info!(
                    target: EFFECT_GRAPH_TARGET,
                    "Приёмник эффекта {:?} перестал существовать",
                    effect_id
                );
                true
            } else {
                false
            }
        },
        _ => {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка получить источник значений эффекта для {:?} провалилась",
                effect_id
            );
            true
        }
    }
}