use crate::effect_mechanics::{EffectEvaluatorRegistry};
use crate::game_config::ConfigProvider;

pub mod floor_generator;
pub mod grid_math;

pub type EntityId = hecs::Entity;
pub type GameWorld = hecs::World;

pub struct GameContext {
    pub world: GameWorld,
    pub effect_mechanic_registry: EffectEvaluatorRegistry,
    pub game_config_provider: ConfigProvider,
}