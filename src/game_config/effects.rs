use serde::{Deserialize, Serialize};
use crate::effect_mechanics::EffectMechanicSetting;
use crate::game_config::Config;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EffectMechanicConfig {
    pub mechanic_name: EffectMechanicSetting
}

impl Config for EffectMechanicConfig {}