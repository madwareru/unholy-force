use egui::Pos2;
use egui_snarl::NodeId;
use hecs::{Ref, RefMut};
use tracing::error;
use crate::app::game_stage::{EntityId, GameWorld};
use crate::effect_mechanics::{EffectContext, EffectEnv, EffectFlow, EntityValueHolder, GlobalValuesHolder, EFFECT_GRAPH_TARGET};

pub mod wait_ticks;
pub mod branch;
pub mod spawn_sub_effect;
pub mod add_tag;
pub mod terminator;

#[derive(Copy, Clone, Debug)]
pub struct SharedNodeData{
    pub node_id: NodeId,
    pub pos: Pos2,
}

#[derive(Copy, Clone, Debug)]
pub enum ValueSource {
    Global,
    Caster,
    Target
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