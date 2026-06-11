use bumpalo::Bump;
use crate::effect_mechanics::{EffectContext, EffectEnv, EffectRegistry, EffectQueue};
use crate::game_config::{ConfigProvider};

pub mod floor_generator;
pub mod grid_math;

pub type EntityId = hecs::Entity;
pub type GameWorld = hecs::World;

pub trait GameSystem {
    fn tick(
        &mut self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_queue: &mut EffectQueue
    );
}

pub struct GameContextBuilder {
    game_systems: Vec<Box<dyn GameSystem>>,
}
impl GameContextBuilder {
    pub fn new() -> Self {
        Self { game_systems: Vec::new() }
    }
    pub fn add_game_system<S: GameSystem + Default + 'static>(mut self) -> Self {
        self.game_systems.push(Box::new(S::default()));
        self
    }
    pub fn build(self) -> GameContext {
        GameContext {
            world: Default::default(),
            effect_registry: Default::default(),
            game_config_provider: ConfigProvider::make(),
            game_systems: self.game_systems,
            bump: Default::default(),
        }
    }
}

pub struct GameContext {
    world: GameWorld,
    effect_registry: EffectRegistry,
    game_config_provider: ConfigProvider,
    game_systems: Vec<Box<dyn GameSystem>>,
    bump: Bump
}

impl GameContext {
    pub fn tick(&mut self) {
        self.tick_game_systems();
        self.tick_effect_subsystem();
        self.reset_bump();
    }

    fn tick_game_systems(&mut self) {
        let mut effect_queue = EffectQueue::new_in(&self.bump);
        for game_system in self.game_systems.iter_mut() {
            game_system.tick(&self.game_config_provider, &mut self.world, &mut effect_queue);
        }
        for scheduled_effect in effect_queue.drain() {
            self.effect_registry.create_effect(
                &self.bump,
                &self.game_config_provider,
                &mut self.world,
                scheduled_effect.effect_config_id(),
                scheduled_effect.effect_context()
            );
        }
    }

    fn tick_effect_subsystem(&mut self) {
        let world = &mut self.world;
        let effect_registry = &mut self.effect_registry;
        let game_config_provider = &self.game_config_provider;

        let mut effect_entities = bumpalo::collections::Vec::new_in(&self.bump);
        for (entity_id, _, effect_ctx) in world.query::<(EntityId, &EffectEnv, &EffectContext)>().iter() {
            effect_entities.push((entity_id, effect_ctx.current_node_id));
        }

        for (effect_id, current_node_id) in effect_entities.drain(..) {
            let next_node_id = effect_registry.tick(&self.bump, game_config_provider, effect_id, current_node_id, world);
            if let Ok((effect_ctx,)) = world.query_one::<(&mut EffectContext,)>(effect_id).get() {
                effect_ctx.current_node_id = next_node_id;
            }
        }
    }

    fn reset_bump(&mut self) {
        self.bump.reset();
    }
}