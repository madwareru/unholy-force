use serde::{Deserialize, Serialize};
use crate::app::game_stage::{EntityId, GameWorld};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum EffectMechanicSetting {
    SimpleAttack
}

pub trait EffectMechanicFactory {
    fn make(
        &self,
        game_world: &GameWorld,
        self_id: EntityId
    ) -> Box<dyn EffectMechanic>;
}

pub enum EffectMechanicFlow {
    Pending,
    Complete
}

pub trait EffectMechanic {
    fn setup(
        &mut self,
        _game_world: &mut GameWorld,
        _caster_id: Option<EntityId>,
        _target_id: Option<EntityId>
    ) {}
    fn on_destroy(
        &mut self,
        _game_world: &mut GameWorld,
        _caster_id: Option<EntityId>,
        _target_id: Option<EntityId>
    ) {}
    fn tick(
        &mut self,
        game_world: &mut GameWorld,
        target_id: EntityId
    ) -> EffectMechanicFlow;
}