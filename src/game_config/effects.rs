use serde::{Deserialize, Serialize};
use crate::effect_mechanics::EffectEvaluator;
use crate::game_config::Config;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EffectConfig {
}

impl EffectConfig {
    pub(crate) fn create_evaluator(&self) -> Option<Box<dyn EffectEvaluator>> {
        todo!("Реализовать создание вычислителя эффекта")
    }
}

impl Config for EffectConfig {}