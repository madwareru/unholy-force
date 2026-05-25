use serde::{Deserialize, Serialize};
use crate::app::editor_stage::image_widgets::SpriteHolder;
use crate::game_config::{Config};
use crate::game_config::floors::LootTableEntry;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
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
impl UnitDanger {
    pub fn from_id(id: u8) -> Self {
        match id {
            0 => UnitDanger::Harmless,
            1 => UnitDanger::Weak,
            2 => UnitDanger::Moderate,
            3 => UnitDanger::Challenging,
            4 => UnitDanger::Horror,
            _ => UnitDanger::Nightmare
        }
    }
    pub fn display_name(&self) -> &str {
        match self {
            UnitDanger::Harmless => "Безобидный",
            UnitDanger::Weak => "Слабый",
            UnitDanger::Moderate => "Рядовой",
            UnitDanger::Challenging => "Непростой",
            UnitDanger::Horror => "Устрашающий",
            UnitDanger::Nightmare => "Кошмарный"
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct UnitConfig {
    pub name: String,
    pub description: String,
    pub sprite_name: String,
    pub sprite_pivot: [u8; 2],
    pub danger: UnitDanger,
    #[serde(default)]
    pub is_catchable: bool,
}

impl SpriteHolder for UnitConfig {
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

impl Config for UnitConfig {}