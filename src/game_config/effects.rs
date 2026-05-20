use serde::{Deserialize, Serialize};
use crate::effect_mechanics::EffectMechanicTag;
use crate::game_config::Config;

#[derive(Serialize, Deserialize, Debug)]
pub struct EffectMechanicConfig {
    pub mechanic_setting: EffectMechanicTag
}

impl Config for EffectMechanicConfig {}