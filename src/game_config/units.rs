use serde::{Deserialize, Serialize};
use crate::game_config::{Config};
use crate::game_config::floors::LootTableEntry;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum UnitDanger {
    Harmless = 0,
    Weak = 1,
    Moderate = 2,
    Challenging = 3,
    Horror = 4,
    Nightmare = 5
}
impl Default for UnitDanger {
    fn default() -> Self { UnitDanger::Harmless }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UnitConfig {
    pub name: String,
    pub description: String,
    pub sprite_name: String,
    pub sprite_pivot: [u8; 2],
    pub danger: UnitDanger,
    pub loot_table: Vec<LootTableEntry>
}

impl Config for UnitConfig {}