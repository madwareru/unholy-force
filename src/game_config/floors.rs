use serde::{Deserialize, Serialize};
use crate::game_config::{Config, ConfigId};
use crate::game_config::floor_parts::FloorPartConfig;
use crate::game_config::items::ItemConfig;
use crate::game_config::units::UnitConfig;

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct FloorConfig {
    pub available_parts: Vec<ConfigId<FloorPartConfig>>,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_table: Vec<LootTableEntry>,
}

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
pub struct SpawnTableEntry {
    pub unit_config: Option<ConfigId<UnitConfig>>,
    pub weight: u32
}

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
pub struct LootTableEntry {
    pub item_config: Option<ConfigId<ItemConfig>>,
    pub weight: u32
}

impl Config for FloorConfig {}