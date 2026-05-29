use serde::{Deserialize, Serialize};
use crate::app::editor_stage::image_widgets::{EditableFloorData, FloorDataHolderConst, FloorTilesHolderConst, WallTilesHolderConst};
use crate::game_config::{Config, ConfigId};
use crate::game_config::floor_part_adjacency::FloorPartAdjacencyConfig;
use crate::game_config::items::ItemConfig;
use crate::game_config::units::UnitConfig;
use crate::graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup};

pub trait GeneratedFloorSize {
    const PARTS_WIDTH: usize;
    const PARTS_HEIGHT: usize;
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AuthoredFloorSize15x15 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 15]; 15]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 15]; 15]>,
}
impl FloorTilesHolderConst<15, 15> for AuthoredFloorSize15x15 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 15]; 15] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 15]; 15] {
        self.floor_data.floor_data_mut()
    }
}
impl WallTilesHolderConst<15, 15> for AuthoredFloorSize15x15 {
    fn wall_data(&self) -> &[[WallGraphicsTileGroup; 15]; 15] {
        self.wall_data.wall_data()
    }
    fn wall_data_mut(&mut self) -> &mut [[WallGraphicsTileGroup; 15]; 15] {
        self.wall_data.wall_data_mut()
    }   
}
impl FloorDataHolderConst<15, 15> for AuthoredFloorSize15x15 {}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AuthoredFloorSize20x20 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 20]; 20]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 20]; 20]>,
}
impl FloorTilesHolderConst<20, 20> for AuthoredFloorSize20x20 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 20]; 20] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 20]; 20] {
        self.floor_data.floor_data_mut()
    }
}
impl WallTilesHolderConst<20, 20> for AuthoredFloorSize20x20 {
    fn wall_data(&self) -> &[[WallGraphicsTileGroup; 20]; 20] {
        self.wall_data.wall_data()
    }
    fn wall_data_mut(&mut self) -> &mut [[WallGraphicsTileGroup; 20]; 20] {
        self.wall_data.wall_data_mut()
    }
}
impl FloorDataHolderConst<20, 20> for AuthoredFloorSize20x20 {}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AuthoredFloorSize25x25 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 25]; 25]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 25]; 25]>,
}
impl FloorTilesHolderConst<25, 25> for AuthoredFloorSize25x25 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 25]; 25] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 25]; 25] {
        self.floor_data.floor_data_mut()
    }
}
impl WallTilesHolderConst<25, 25> for AuthoredFloorSize25x25 {
    fn wall_data(&self) -> &[[WallGraphicsTileGroup; 25]; 25] {
        self.wall_data.wall_data()
    }
    fn wall_data_mut(&mut self) -> &mut [[WallGraphicsTileGroup; 25]; 25] {
        self.wall_data.wall_data_mut()
    }
}
impl FloorDataHolderConst<25, 25> for AuthoredFloorSize25x25 {}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AuthoredFloorSize30x30 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 30]; 30]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 30]; 30]>,
}
impl FloorTilesHolderConst<30, 30> for AuthoredFloorSize30x30 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 30]; 30] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 30]; 30] {
        self.floor_data.floor_data_mut()
    }
}
impl WallTilesHolderConst<30, 30> for AuthoredFloorSize30x30 {
    fn wall_data(&self) -> &[[WallGraphicsTileGroup; 30]; 30] {
        self.wall_data.wall_data()
    }
    fn wall_data_mut(&mut self) -> &mut [[WallGraphicsTileGroup; 30]; 30] {
        self.wall_data.wall_data_mut()
    }
}
impl FloorDataHolderConst<30, 30> for AuthoredFloorSize30x30 {}

#[derive(Clone, Debug)]
pub struct FloorSize40x40 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 40]; 40]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 40]; 40]>,
}
impl Default for FloorSize40x40 {
    fn default() -> Self {
        Self {
            floor_data: Box::new([[FloorGraphicsTileGroup::default(); 40]; 40]),
            wall_data: Box::new([[WallGraphicsTileGroup::default(); 40]; 40]),
        }
    }
}
impl FloorTilesHolderConst<40, 40> for FloorSize40x40 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 40]; 40] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 40]; 40] {
        self.floor_data.floor_data_mut()
    }
}
impl WallTilesHolderConst<40, 40> for FloorSize40x40 {
    fn wall_data(&self) -> &[[WallGraphicsTileGroup; 40]; 40] {
        self.wall_data.wall_data()
    }
    fn wall_data_mut(&mut self) -> &mut [[WallGraphicsTileGroup; 40]; 40] {
        self.wall_data.wall_data_mut()
    }
}
impl FloorDataHolderConst<40, 40> for FloorSize40x40 {}

#[derive(Clone, Debug)]
pub struct FloorSize60x60 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 60]; 60]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 60]; 60]>,
}
impl Default for FloorSize60x60 {
    fn default() -> Self {
        Self {
            floor_data: Box::new([[FloorGraphicsTileGroup::default(); 60]; 60]),
            wall_data: Box::new([[WallGraphicsTileGroup::default(); 60]; 60]),
        }
    }
}
impl FloorTilesHolderConst<60, 60> for FloorSize60x60 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 60]; 60] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 60]; 60] {
        self.floor_data.floor_data_mut()
    }
}
impl WallTilesHolderConst<60, 60> for FloorSize60x60 {
    fn wall_data(&self) -> &[[WallGraphicsTileGroup; 60]; 60] {
        self.wall_data.wall_data()
    }
    fn wall_data_mut(&mut self) -> &mut [[WallGraphicsTileGroup; 60]; 60] {
        self.wall_data.wall_data_mut()
    }
}
impl FloorDataHolderConst<60, 60> for FloorSize60x60 {}

#[derive(Clone, Debug)]
pub struct FloorSize80x80 {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 80]; 80]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 80]; 80]>,
}
impl Default for FloorSize80x80 {
    fn default() -> Self {
        Self {
            floor_data: Box::new([[FloorGraphicsTileGroup::default(); 80]; 80]),
            wall_data: Box::new([[WallGraphicsTileGroup::default(); 80]; 80]),
        }
    }
}
impl FloorTilesHolderConst<80, 80> for FloorSize80x80 {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 80]; 80] {
        self.floor_data.floor_data()
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 80]; 80] {
        self.floor_data.floor_data_mut()
    }
}
impl WallTilesHolderConst<80, 80> for FloorSize80x80 {
    fn wall_data(&self) -> &[[WallGraphicsTileGroup; 80]; 80] {
        self.wall_data.wall_data()
    }
    fn wall_data_mut(&mut self) -> &mut [[WallGraphicsTileGroup; 80]; 80] {
        self.wall_data.wall_data_mut()
    }
}
impl FloorDataHolderConst<80, 80> for FloorSize80x80 {}

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
pub struct PartsSize<const W: usize, const H: usize>;
impl<const W: usize, const H: usize> GeneratedFloorSize for PartsSize<W, H> {
    const PARTS_WIDTH: usize = W;
    const PARTS_HEIGHT: usize = H;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FloorVariantTag {
    Authored15x15,
    Authored20x20,
    Authored25x25,
    Authored30x30,
    Generated15x15,
    Generated20x20,
    Generated25x25,
    Generated30x30,
    Generated40x40,
    Generated60x60,
    Generated80x80,
}
impl FloorVariantTag {
    pub fn editor_label(&self) -> &'static str {
        match self {
            | FloorVariantTag::Authored15x15 => "Ручной 15x15",
            | FloorVariantTag::Authored20x20 => "Ручной 20x20",
            | FloorVariantTag::Authored25x25 => "Ручной 25x25",
            | FloorVariantTag::Authored30x30 => "Ручной 30x30",
            | FloorVariantTag::Generated15x15 => "Сборный 15x15",
            | FloorVariantTag::Generated20x20 => "Сборный 20x20",
            | FloorVariantTag::Generated25x25 => "Сборный 25x25",
            | FloorVariantTag::Generated30x30 => "Сборный 30x30",
            | FloorVariantTag::Generated40x40 => "Сборный 40x40",
            | FloorVariantTag::Generated60x60 => "Сборный 60x60",
            | FloorVariantTag::Generated80x80 => "Сборный 80x80",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AuthoredFloor {
    Size15x15(AuthoredFloorSize15x15),
    Size20x20(AuthoredFloorSize20x20),
    Size25x25(AuthoredFloorSize25x25),
    Size30x30(AuthoredFloorSize30x30),
}

impl EditableFloorData for AuthoredFloor {
    fn width(&self) -> usize {
        match self {
            AuthoredFloor::Size15x15(_) => { 15 }
            AuthoredFloor::Size20x20(_) => { 20 }
            AuthoredFloor::Size25x25(_) => { 25 }
            AuthoredFloor::Size30x30(_) => { 30 }
        }
    }

    fn height(&self) -> usize {
        match self {
            AuthoredFloor::Size15x15(_) => { 15 }
            AuthoredFloor::Size20x20(_) => { 20 }
            AuthoredFloor::Size25x25(_) => { 25 }
            AuthoredFloor::Size30x30(_) => { 30 }
        }
    }

    fn get_floor_data(&self, [x, y]: [usize; 2]) -> &FloorGraphicsTileGroup {
        match self {
            AuthoredFloor::Size15x15(data) => {
                &data.floor_data()[y][x]
            }
            AuthoredFloor::Size20x20(data) => {
                &data.floor_data()[y][x]
            }
            AuthoredFloor::Size25x25(data) => {
                &data.floor_data()[y][x]
            }
            AuthoredFloor::Size30x30(data) => {
                &data.floor_data()[y][x]
            }
        }
    }

    fn get_floor_data_mut(&mut self, [x, y]: [usize; 2]) -> &mut FloorGraphicsTileGroup {
        match self {
            AuthoredFloor::Size15x15(data) => {
                &mut data.floor_data_mut()[y][x]
            }
            AuthoredFloor::Size20x20(data) => {
                &mut data.floor_data_mut()[y][x]
            }
            AuthoredFloor::Size25x25(data) => {
                &mut data.floor_data_mut()[y][x]
            }
            AuthoredFloor::Size30x30(data) => {
                &mut data.floor_data_mut()[y][x]
            }
        }
    }

    fn get_wall_data(&self, [x, y]: [usize; 2]) -> &WallGraphicsTileGroup {
        match self {
            AuthoredFloor::Size15x15(data) => {
                &data.wall_data()[y][x]
            }
            AuthoredFloor::Size20x20(data) => {
                &data.wall_data()[y][x]
            }
            AuthoredFloor::Size25x25(data) => {
                &data.wall_data()[y][x]
            }
            AuthoredFloor::Size30x30(data) => {
                &data.wall_data()[y][x]
            }
        }
    }

    fn get_wall_data_mut(&mut self, [x, y]: [usize; 2]) -> &mut WallGraphicsTileGroup {
        match self {
            AuthoredFloor::Size15x15(data) => {
                &mut data.wall_data_mut()[y][x]
            }
            AuthoredFloor::Size20x20(data) => {
                &mut data.wall_data_mut()[y][x]
            }
            AuthoredFloor::Size25x25(data) => {
                &mut data.wall_data_mut()[y][x]
            }
            AuthoredFloor::Size30x30(data) => {
                &mut data.wall_data_mut()[y][x]
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum GeneratedFloor {
    Size15x15(PartsSize<3, 3>),
    Size20x20(PartsSize<4, 4>),
    Size25x25(PartsSize<5, 5>),
    Size30x30(PartsSize<6, 6>),
    Size40x40(PartsSize<8, 8>),
    Size60x60(PartsSize<12, 12>),
    Size80x80(PartsSize<16, 16>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FloorVariant {
    Authored(AuthoredFloor),
    Generated(GeneratedFloor)
}
impl Default for FloorVariant {
    fn default() -> Self {
        Self::Generated(GeneratedFloor::Size15x15(PartsSize::<3, 3>::default()))
    }
}
impl FloorVariant {
    pub fn is_generated(&self) -> bool {
        match self {
            | FloorVariant::Authored(_) => false,
            _ => true
        }
    }
    pub fn get_tag(&self) -> FloorVariantTag {
        match self {
            | FloorVariant::Authored(AuthoredFloor::Size15x15(_)) => FloorVariantTag::Authored15x15,
            | FloorVariant::Authored(AuthoredFloor::Size20x20(_)) => FloorVariantTag::Authored20x20,
            | FloorVariant::Authored(AuthoredFloor::Size25x25(_)) => FloorVariantTag::Authored25x25,
            | FloorVariant::Authored(AuthoredFloor::Size30x30(_)) => FloorVariantTag::Authored30x30,
            | FloorVariant::Generated(GeneratedFloor::Size15x15(_)) => FloorVariantTag::Generated15x15,
            | FloorVariant::Generated(GeneratedFloor::Size20x20(_)) => FloorVariantTag::Generated20x20,
            | FloorVariant::Generated(GeneratedFloor::Size25x25(_)) => FloorVariantTag::Generated25x25,
            | FloorVariant::Generated(GeneratedFloor::Size30x30(_)) => FloorVariantTag::Generated30x30,
            | FloorVariant::Generated(GeneratedFloor::Size40x40(_)) => FloorVariantTag::Generated40x40,
            | FloorVariant::Generated(GeneratedFloor::Size60x60(_)) => FloorVariantTag::Generated60x60,
            | FloorVariant::Generated(GeneratedFloor::Size80x80(_)) => FloorVariantTag::Generated80x80,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct FloorConfig {
    /// Название этажа в игре
    pub name: String,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_table: Vec<LootTableEntry>,
    pub floor_variant: FloorVariant,
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
}

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
pub struct SpawnTableEntry {
    pub unit_config: ConfigId<UnitConfig>,
    pub weight: u32
}

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug)]
pub struct LootTableEntry {
    pub item_config: ConfigId<ItemConfig>,
    pub weight: u32
}

impl Config for FloorConfig {}