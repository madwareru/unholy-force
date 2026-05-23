use serde::{Deserialize, Serialize};
use crate::game_config::{Config, ConfigId};
use crate::game_config::floor_part_adjacency::FloorPartAdjacencyConfig;
use crate::game_config::items::ItemConfig;
use crate::game_config::units::UnitConfig;

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct FloorConfig {
    /// Название этажа в игре
    pub name: String,
    /// Используемые на этаже правила смежности.
    /// Из этих правил получаются сразу и наборы частей
    /// и наборы смежных с ними. Через такой подход
    /// можно варьировать, с какими именно частями может
    /// стыковаться часть этажа, когда она относится
    /// именно к этому конкретному этажу. В итоге
    /// получаем больший контроль, может ли в определённом
    /// наборе тайлов случайно затесаться высокоуровневый
    /// монстр или редкий лут. Или тайная комната
    pub available_parts: Vec<ConfigId<FloorPartAdjacencyConfig>>,
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