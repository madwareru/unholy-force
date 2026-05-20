use serde::{Deserialize, Serialize};
use crate::game_config::{Config, ConfigId};
use crate::game_config::effects::EffectMechanicConfig;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
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
pub enum ItemKind {
    Quest,
    Armor{ equip_effect: ConfigId<EffectMechanicConfig> },
    Weapon{ attack_effect: ConfigId<EffectMechanicConfig> },
    Consumable{ use_effect: ConfigId<EffectMechanicConfig> }
}
impl Default for ItemKind {
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
    pub kind: ItemKind
}

impl Config for ItemConfig {}