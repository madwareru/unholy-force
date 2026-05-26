use serde::{Deserialize, Serialize};
use crate::app::editor_stage::image_widgets::FloorTilesHolder;
use crate::game_config::{Config, ConfigId};
use crate::game_config::floor_part_adjacency::FloorPartAdjacencyConfig;
use crate::game_config::items::ItemConfig;
use crate::game_config::units::UnitConfig;
use crate::graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup};

pub trait FloorSize {
    const WIDTH: usize;
    const HEIGHT: usize;
}

pub trait GeneratedFloorSize: FloorSize {
    const PARTS_WIDTH: usize;
    const PARTS_HEIGHT: usize;
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AuthoredFloorSize15x15 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 15]; 15]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 15]; 15]>,
}
impl FloorSize for AuthoredFloorSize15x15 {
    const WIDTH: usize = 15;
    const HEIGHT: usize = 15;
}
impl FloorTilesHolder<15, 15> for AuthoredFloorSize15x15 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 15]; 15] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 15]; 15] {
        self.floor_data.floor_data_mut()
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AuthoredFloorSize20x20 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 20]; 20]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 20]; 20]>,
}
impl FloorSize for AuthoredFloorSize20x20 {
    const WIDTH: usize = 20;
    const HEIGHT: usize = 20;
}
impl FloorTilesHolder<20, 20> for AuthoredFloorSize20x20 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 20]; 20] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 20]; 20] {
        self.floor_data.floor_data_mut()
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AuthoredFloorSize25x25 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 25]; 25]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 25]; 25]>,
}
impl FloorSize for AuthoredFloorSize25x25 {
    const WIDTH: usize = 25;
    const HEIGHT: usize = 25;
}
impl FloorTilesHolder<25, 25> for AuthoredFloorSize25x25 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 25]; 25] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 25]; 25] {
        self.floor_data.floor_data_mut()
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AuthoredFloorSize30x30 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 30]; 30]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 30]; 30]>,
}
impl FloorSize for AuthoredFloorSize30x30 {
    const WIDTH: usize = 30;
    const HEIGHT: usize = 30;
}
impl FloorTilesHolder<30, 30> for AuthoredFloorSize30x30 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 30]; 30] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 30]; 30] {
        self.floor_data.floor_data_mut()
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
pub struct GeneratedFloor<const W: usize, const H: usize>;
impl<const W: usize, const H: usize> FloorSize for GeneratedFloor<W, H> {
    const WIDTH: usize = W * 5;
    const HEIGHT: usize = H * 5;
}
impl<const W: usize, const H: usize> GeneratedFloorSize for GeneratedFloor<W, H> {
    const PARTS_WIDTH: usize = W;
    const PARTS_HEIGHT: usize = H;
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FloorVariant {
    Authored15x15(AuthoredFloorSize15x15),
    Authored20x20(AuthoredFloorSize20x20),
    Authored25x25(AuthoredFloorSize25x25),
    Authored30x30(AuthoredFloorSize30x30),
    Generated15x15(GeneratedFloor<3, 3>),
    Generated20x20(GeneratedFloor<4, 4>),
    Generated25x25(GeneratedFloor<5, 5>),
    Generated30x30(GeneratedFloor<6, 6>),
    Generated40x40(GeneratedFloor<8, 8>),
    Generated60x60(GeneratedFloor<12, 12>),
    Generated80x80(GeneratedFloor<16, 16>),
}
impl Default for FloorVariant {
    fn default() -> Self {
        Self::Generated15x15(GeneratedFloor::<3, 3>::default())
    }
}
impl FloorVariant {
    pub fn is_generated(&self) -> bool {
        match self {
            | FloorVariant::Authored15x15(_)
            | FloorVariant::Authored20x20(_)
            | FloorVariant::Authored25x25(_)
            | FloorVariant::Authored30x30(_) => false,
            _ => true
        }
    }
}

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