use std::collections::{HashMap};
use bumpalo::{Bump, collections::Vec};
use serde::{Deserialize, Serialize};
use crate::app::game_stage::{EntityId, GameWorld};

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum EffectMechanicSetting {
    SimpleAttack
}

#[derive(Default)]
pub struct EffectMechanicRegistry {
    bump: Bump,
    provider: EffectMechanicProvider
}

#[derive(Default)]
pub struct EffectMechanicProvider {
    mechanics: HashMap<EffectMechanicSetting, Box<dyn EffectMechanic>>
}
impl EffectMechanicProvider {
    fn get_or_create_effect_mechanic(
        &mut self,
        setting: EffectMechanicSetting
    ) -> &dyn EffectMechanic {
        if !self.mechanics.contains_key(&setting) {
            self.mechanics.insert(
                setting,
                match setting {
                    EffectMechanicSetting::SimpleAttack => todo!()
                }
            );
        }
        let Some(mechanic) = self.mechanics.get(&setting).map(|it| it.as_ref()) else {
            unreachable!()
        };
        mechanic
    }
}

impl EffectMechanicRegistry {
    pub fn create_effect(
        &mut self,
        game_world: &mut GameWorld,
        setting: EffectMechanicSetting,
        effect_context: EffectContext
    ) {
        let mechanic = self.provider.get_or_create_effect_mechanic(setting);
        let effect_id = game_world.spawn((setting, effect_context));
        mechanic.setup(game_world, effect_id);
    }

    pub fn tick(
        &mut self,
        effect_id: EntityId,
        game_world: &mut GameWorld
    ) {
        let bump = &self.bump;
        let mechanic_setting = {
            match game_world.get::<&EffectMechanicSetting>(effect_id) {
                Ok(mechanic_setting) => *mechanic_setting,
                _ => panic!("No effect found for {:?}", effect_id)
            }
        };

        let mut queue = DelayedEffectQueue(Vec::new_in(bump));
        let mechanic = self.provider.get_or_create_effect_mechanic(mechanic_setting);
        if let EffectMechanicFlow::Complete = mechanic.tick(game_world, effect_id, &mut queue) {
            mechanic.on_destroy(game_world, effect_id);
            game_world.despawn(effect_id).expect("Failed to despawn effect");
        }
        for DelayedEffect { mechanic_setting, effect_context } in queue.drain() {
            let effect_id = game_world.spawn((mechanic_setting, effect_context));
            let mechanic = self.provider.get_or_create_effect_mechanic(mechanic_setting);
            mechanic.setup(game_world, effect_id);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct DelayedEffect {
    mechanic_setting: EffectMechanicSetting,
    effect_context: EffectContext
}

pub struct DelayedEffectQueue<'a>(Vec<'a, DelayedEffect>);
impl<'a> DelayedEffectQueue<'a> {
    pub fn push(&mut self, mechanic_setting: EffectMechanicSetting, effect_context: EffectContext) {
        self.0.push(DelayedEffect { mechanic_setting, effect_context } );
    }

    fn drain(&mut self) -> impl Iterator<Item = DelayedEffect> + '_ {
        self.0.drain(..)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EffectContext {
    WorldOnly,
    CasterOnly { id: EntityId },
    TargetOnly { id: EntityId },
    CasterAndTarget { caster_id: EntityId, target_id: EntityId }
}

pub enum EffectMechanicFlow {
    Continue,
    Complete
}

pub trait EffectMechanic {
    fn setup(&self, _game_world: &mut GameWorld, _effect_id: EntityId) {}
    fn on_destroy(&self, _game_world: &mut GameWorld, _effect_id: EntityId ) {}
    fn tick(
        &self,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    ) -> EffectMechanicFlow;
}