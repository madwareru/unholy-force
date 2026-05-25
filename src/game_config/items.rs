use serde::{Deserialize, Serialize};
use crate::app::editor_stage::image_widgets::SpriteHolder;
use crate::game_config::{Config, ConfigId};
use crate::game_config::effects::EffectMechanicConfig;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
pub enum ItemRarity {
    #[default]
    Generic = 0,
    Rare = 1,
    Unique = 2,
    Legendary = 3
}
impl ItemRarity {
    pub fn display_name(self) -> &'static str {
        match self {
            ItemRarity::Generic => "Обычный",
            ItemRarity::Rare => "Редкий",
            ItemRarity::Unique => "Уникальный",
            ItemRarity::Legendary => "Былинный"
        }
    }
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ItemConfig {
    pub name: String,
    pub description: String,
    pub sprite_name : String,
    pub sprite_pivot: [u8; 2],
    pub item_rarity: ItemRarity,
    #[serde(default = "default_stack_limit")]
    pub stack_limit : u8,
    pub kind: ItemKind
}
impl Default for ItemConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            sprite_name: String::new(),
            sprite_pivot: [0, 0],
            item_rarity: ItemRarity::default(),
            stack_limit: default_stack_limit(),
            kind: ItemKind::default()
        }
    }
}
fn default_stack_limit() -> u8 { 1 }

impl SpriteHolder for ItemConfig {
    fn sprite_name(&self) -> &str {
        &self.sprite_name
    }

    fn sprite_pivot(&self) -> &[u8; 2] {
        &self.sprite_pivot
    }

    fn sprite_pivot_mut(&mut self) -> &mut [u8; 2] {
        &mut self.sprite_pivot
    }
}

impl Config for ItemConfig {}