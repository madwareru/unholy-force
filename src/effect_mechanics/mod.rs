use std::collections::{HashMap};
use bumpalo::{Bump, collections::Vec};
use serde::{Deserialize, Serialize};
use crate::app::game_stage::{EntityId, GameWorld};

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum EffectMechanicTag {
    SimpleAttack
}

#[derive(Default)]
pub struct EffectMechanicRegistry {
    bump: Bump,
    provider: EffectMechanicProvider
}

#[derive(Default)]
pub struct EffectMechanicProvider {
    mechanics: HashMap<EffectMechanicTag, Box<dyn EffectMechanic>>
}
impl EffectMechanicProvider {
    fn get_or_create_effect_mechanic(
        &mut self,
        tag: EffectMechanicTag
    ) -> &dyn EffectMechanic {
        if !self.mechanics.contains_key(&tag) {
            self.mechanics.insert(
                tag,
                match tag {
                    EffectMechanicTag::SimpleAttack => todo!()
                }
            );
        }
        let Some(mechanic) = self.mechanics.get(&tag).map(|it| it.as_ref()) else {
            unreachable!()
        };
        mechanic
    }
}

impl EffectMechanicRegistry {
    pub fn create_effect(
        &mut self,
        game_world: &mut GameWorld,
        tag: EffectMechanicTag,
        effect_context: EffectContext
    ) {
        let mechanic = self.provider.get_or_create_effect_mechanic(tag);
        let effect_id = game_world.spawn((tag, effect_context));
        mechanic.setup(game_world, effect_id);
    }

    pub fn tick(
        &mut self,
        effect_id: EntityId,
        game_world: &mut GameWorld
    ) {
        let bump = &self.bump;
        let tag = {
            match game_world.get::<&EffectMechanicTag>(effect_id) {
                Ok(tag) => *tag,
                _ => panic!("No effect found for {:?}", effect_id)
            }
        };

        let mut queue = DelayedEffectQueue(Vec::new_in(bump));
        let mechanic = self.provider.get_or_create_effect_mechanic(tag);
        if let EffectMechanicFlow::Complete = mechanic.tick(game_world, effect_id, &mut queue) {
            mechanic.on_destroy(game_world, effect_id);
            game_world.despawn(effect_id).expect("Failed to despawn effect");
        }
        for DelayedEffect { tag, effect_context } in queue.drain() {
            let effect_id = game_world.spawn((tag, effect_context));
            let mechanic = self.provider.get_or_create_effect_mechanic(tag);
            mechanic.setup(game_world, effect_id);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct DelayedEffect {
    tag: EffectMechanicTag,
    effect_context: EffectContext
}

pub struct DelayedEffectQueue<'a>(Vec<'a, DelayedEffect>);
impl<'a> DelayedEffectQueue<'a> {
    pub fn push(&mut self, tag: EffectMechanicTag, effect_context: EffectContext) {
        self.0.push(DelayedEffect { tag, effect_context } );
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