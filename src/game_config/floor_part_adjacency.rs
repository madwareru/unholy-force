use serde::{Deserialize, Serialize};
use crate::game_config::{Config, ConfigId};
use crate::game_config::floor_parts::FloorPartConfig;

/// Настройка смежности для части уровня
/// Для одной и той же части уровня может
/// существовать одновременно несколько настроек,
/// они могут сливаться в одну или использоваться
/// по-отдельности в зависимости от того, как будет
/// настроен этаж
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FloorPartAdjacencyConfig {
    pub part: ConfigId<FloorPartConfig>,
    pub north_adjacent_parts: Vec<ConfigId<FloorPartConfig>>,
    pub west_adjacent_parts: Vec<ConfigId<FloorPartConfig>>,
    pub south_adjacent_parts: Vec<ConfigId<FloorPartConfig>>,
    pub east_adjacent_parts: Vec<ConfigId<FloorPartConfig>>,
}

impl Config for FloorPartAdjacencyConfig {}