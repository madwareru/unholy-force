use serde::{Deserialize, Serialize};
use crate::effect_mechanics::EffectMechanicSetting;
use crate::game_config::Config;

#[derive(Serialize, Deserialize, Debug)]
pub struct EffectMechanicConfig {
    pub mechanic_setting: EffectMechanicSetting
}

impl Config for EffectMechanicConfig {}