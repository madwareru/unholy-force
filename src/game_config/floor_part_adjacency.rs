use serde::{Deserialize, Serialize};
use crate::game_config::{Config, ConfigId};
use crate::game_config::floor_parts::FloorPartConfig;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FloorPartAdjacencyConfig {
    pub part: ConfigId<FloorPartConfig>,
    pub north_adjacent_parts: Vec<ConfigId<FloorPartConfig>>,
    pub west_adjacent_parts: Vec<ConfigId<FloorPartConfig>>,
    pub south_adjacent_parts: Vec<ConfigId<FloorPartConfig>>,
    pub east_adjacent_parts: Vec<ConfigId<FloorPartConfig>>,
}

impl Config for FloorPartAdjacencyConfig {}