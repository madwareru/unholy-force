use crate::effect_mechanics::{EffectMechanicRegistry};

pub mod floor_generator;

pub type EntityId = hecs::Entity;
pub type GameWorld = hecs::World;

pub struct GameContext {
    pub world: GameWorld,
    pub effect_mechanic_registry: EffectMechanicRegistry
}