use serde::{Deserialize, Serialize};
use crate::effect_mechanics::EffectRoot;
use crate::game_config::Config;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EffectConfig {
}

impl EffectConfig {
    pub(crate) fn try_create_root(&self) -> Option<EffectRoot> {
        todo!("Реализовать создание корневого узла эффекта")
    }
}

impl Config for EffectConfig {}