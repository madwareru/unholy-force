use std::collections::{HashMap};
use bumpalo::{Bump, collections::Vec};
use crate::app::game_stage::{EntityId, GameWorld};
use crate::game_config::{ConfigId, ConfigProvider, ConfigProviderImpl};
use crate::game_config::effects::EffectConfig;

#[derive(Default)]
pub struct EffectEvaluatorRegistry {
    bump: Bump,
    provider: EffectEvaluatorProvider
}

#[derive(Default)]
pub struct EffectEvaluatorProvider {
    effects: HashMap<ConfigId<EffectConfig>, Box<dyn EffectEvaluator>>,
}
impl EffectEvaluatorProvider {
    fn get_or_create_effect_evaluator(
        &mut self,
        game_config_provider: &ConfigProvider,
        effect_config_id: ConfigId<EffectConfig>
    ) -> Option<&dyn EffectEvaluator> {
        if !self.effects.contains_key(&effect_config_id) {
            match game_config_provider.get_config(effect_config_id) {
                None => {
                    println!("Error! No effect config for {:?} found!!!", effect_config_id);
                }
                Some(effect_config) => {
                    match effect_config.create_evaluator() {
                        Some(effect_evaluator) => {
                            self.effects.insert(
                                effect_config_id,
                                effect_evaluator
                            );
                        }
                        _ => {
                            println!("Error! Failed to create effect evaluator for {:?}!", effect_config_id);
                        }
                    }
                }
            }
        }
        self.effects.get(&effect_config_id).map(|it| it.as_ref())
    }
}

impl EffectEvaluatorRegistry {
    pub fn create_effect(
        &mut self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_config_id: ConfigId<EffectConfig>,
        effect_context: EffectContext
    ) {
        let Some(evaluator) = self.provider.get_or_create_effect_evaluator(
            game_config_provider,
            effect_config_id
        ) else {
            return;
        };

        let bump = &self.bump;
        let mut queue = DelayedEffectQueue(Vec::new_in(bump));

        let effect_id = game_world.spawn((effect_config_id, effect_context));
        evaluator.setup(game_world, effect_id, &mut queue);

        let mut offset = 0;
        while offset < queue.0.len() {
            let Some(DelayedEffect { effect_config_id, effect_context }) = queue.0.get(offset).copied() else {
                continue;
            };
            offset += 1;

            let Some(evaluator) = self.provider.get_or_create_effect_evaluator(
                game_config_provider,
                effect_config_id
            ) else {
                continue;
            };

            let effect_id = game_world.spawn((effect_config_id, effect_context));
            evaluator.setup(game_world, effect_id, &mut queue);
        }
    }

    pub fn tick(
        &mut self,
        game_config_provider: &ConfigProvider,
        effect_id: EntityId,
        game_world: &mut GameWorld
    ) {
        let effect_config_id = {
            match game_world.get::<&ConfigId<EffectConfig>>(effect_id) {
                Ok(mechanic_setting) => *mechanic_setting,
                _ => panic!("No effect found for {:?}", effect_id)
            }
        };

        let Some(evaluator) = self.provider.get_or_create_effect_evaluator(
            game_config_provider,
            effect_config_id
        ) else {
            return;
        };

        let bump = &self.bump;
        let mut queue = DelayedEffectQueue(Vec::new_in(bump));

        if let EffectMechanicFlow::Complete = evaluator.tick(game_world, effect_id, &mut queue) {
            evaluator.on_destroy(game_world, effect_id, &mut queue);
            game_world.despawn(effect_id).expect("Failed to despawn effect");
        }

        let mut offset = 0;
        while offset < queue.0.len() {
            let Some(DelayedEffect { effect_config_id, effect_context }) = queue.0.get(offset).copied() else {
                continue;
            };
            offset += 1;

            let Some(evaluator) = self.provider.get_or_create_effect_evaluator(
                game_config_provider,
                effect_config_id
            ) else {
                continue;
            };

            let effect_id = game_world.spawn((effect_config_id, effect_context));
            evaluator.setup(game_world, effect_id, &mut queue);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct DelayedEffect {
    effect_config_id: ConfigId<EffectConfig>,
    effect_context: EffectContext
}

pub struct DelayedEffectQueue<'a>(Vec<'a, DelayedEffect>);
impl<'a> DelayedEffectQueue<'a> {
    pub fn push(&mut self, effect_config_id: ConfigId<EffectConfig>, effect_context: EffectContext) {
        self.0.push(DelayedEffect { effect_config_id, effect_context } );
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

pub trait EffectEvaluator {
    fn setup(
        &self,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    );
    fn on_destroy(
        &self,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    );
    fn tick(
        &self,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    ) -> EffectMechanicFlow;
}