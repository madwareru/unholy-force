use serde::{Deserialize, Serialize};
use crate::game_config::{Config, ConfigId};
use crate::game_config::effects::EffectMechanicConfig;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ItemRarity {
    Generic = 0,
    Rare = 1,
    Unique = 2,
    Legendary = 3
}
impl Default for ItemRarity {
    fn default() -> Self { Self::Generic }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum ItemEffect {
    Quest,
    Armor{ effect_mechanic: ConfigId<EffectMechanicConfig> },
    Weapon{ range: u8, effect_mechanic: ConfigId<EffectMechanicConfig> }
}
impl Default for ItemEffect {
    fn default() -> Self {
        Self::Quest
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ItemConfig {
    pub name: String,
    pub description: String,
    pub sprite_name : String,
    pub sprite_pivot: [u8; 2],
    pub item_rarity: ItemRarity,
    pub is_stackable : bool,
    pub effect: ItemEffect
}

impl Config for ItemConfig {}