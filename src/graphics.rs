use std::collections::HashMap;
use std::sync::Arc;
use lazy_static::lazy_static;
use serde::Deserialize;
use macroquad::prelude::*;

lazy_static!(
    pub static ref SPRITE_ATLAS_TEXTURE: Texture2D = {
        let main_menu_texture = Texture2D::from_file_with_format(
            include_bytes!("../assets/sprites.png"),
            None
        );
        main_menu_texture.set_filter(FilterMode::Nearest);
        main_menu_texture
    };

    pub static ref SPRITE_ATLAS_DEF: Arc<SpriteGraphicsAtlasDef> = {
        const CONFIG_TEXT: &str = include_str!("../assets/sprite_atlas.json5");
        let atlas_def = json5::from_str(CONFIG_TEXT);
        Arc::new(atlas_def.expect("Failed to load sprite atlas json"))
    };
);

pub const WANG_MASK_NORTH_EAST: usize = 0b0001;
pub const WANG_MASK_NORTH_WEST: usize = 0b0010;
pub const WANG_MASK_SOUTH_EAST: usize = 0b0100;
pub const WANG_MASK_SOUTH_WEST: usize = 0b1000;

const fn wang_mask_to_tile_offset_lookup() -> [[u8; 2]; 16] {
    let mut lookup = [[0, 3]; 16];
    lookup[WANG_MASK_SOUTH_EAST] = [2, 2];
    lookup[WANG_MASK_SOUTH_WEST] = [3, 2];
    lookup[WANG_MASK_NORTH_EAST] = [2, 3];
    lookup[WANG_MASK_NORTH_WEST] = [3, 3];
    lookup[WANG_MASK_SOUTH_WEST | WANG_MASK_SOUTH_EAST] = [0, 0];
    lookup[WANG_MASK_SOUTH_WEST | WANG_MASK_NORTH_EAST] = [1, 0];
    lookup[WANG_MASK_SOUTH_WEST | WANG_MASK_NORTH_WEST] = [1, 1];
    lookup[WANG_MASK_NORTH_EAST | WANG_MASK_NORTH_WEST] = [0, 2];
    lookup[WANG_MASK_NORTH_WEST | WANG_MASK_SOUTH_EAST] = [1, 2];
    lookup[WANG_MASK_NORTH_EAST | WANG_MASK_SOUTH_EAST] = [1, 3];
    lookup[WANG_MASK_SOUTH_WEST | WANG_MASK_NORTH_WEST | WANG_MASK_NORTH_EAST] = [2, 0];
    lookup[WANG_MASK_SOUTH_EAST | WANG_MASK_NORTH_WEST | WANG_MASK_NORTH_EAST] = [3, 0];
    lookup[WANG_MASK_SOUTH_EAST | WANG_MASK_SOUTH_WEST | WANG_MASK_NORTH_WEST] = [2, 1];
    lookup[WANG_MASK_SOUTH_EAST | WANG_MASK_SOUTH_WEST | WANG_MASK_NORTH_EAST] = [3, 1];
    lookup[WANG_MASK_SOUTH_EAST | WANG_MASK_SOUTH_WEST | WANG_MASK_NORTH_WEST | WANG_MASK_NORTH_EAST] = [0, 1];
    lookup
}

pub const WANG_MASK_LOOKUP: [[u8; 2]; 16] = wang_mask_to_tile_offset_lookup();

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize, Default)]
pub enum FloorGraphicsTileGroup {
    #[default]
    Dirt = 0,
    Tile = 1,
    Water = 2,
    Lava = 3,
}
impl FloorGraphicsTileGroup {
    pub fn get_name(self) -> &'static str {
        match self {
            FloorGraphicsTileGroup::Dirt => "Земля",
            FloorGraphicsTileGroup::Tile => "Плитка",
            FloorGraphicsTileGroup::Water => "Вода",
            FloorGraphicsTileGroup::Lava => "Лава"
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize, Default)]
pub enum WallGraphicsTileGroup {
    None = 0,
    Sandstone = 1,
    #[default]
    Rocks = 2,
    Bricks = 3,
}
impl WallGraphicsTileGroup {
    pub fn get_name(self) -> &'static str {
        match self {
            WallGraphicsTileGroup::None => "Свободно",
            WallGraphicsTileGroup::Sandstone => "Песчаник",
            WallGraphicsTileGroup::Rocks => "Камни",
            WallGraphicsTileGroup::Bricks => "Кладка"
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub enum VisibilityGraphicsTileGroup {
    Unvisible = 0,
    FogOfVar = 1
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct TileGroupGraphicsDef {
    pub base_coords: [u8; 2]
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct SpriteGraphicsDef {
    pub coords: [u8; 2],
    pub size: [u8; 2]
}

#[derive(Deserialize)]
pub struct SpriteGraphicsAtlasDef {
    pub tile_size: [u8; 2],
    pub floor_tile_groups: HashMap<FloorGraphicsTileGroup, TileGroupGraphicsDef>,
    pub wall_tile_groups: HashMap<WallGraphicsTileGroup, TileGroupGraphicsDef>,
    pub visibility_tile_groups: HashMap<VisibilityGraphicsTileGroup, TileGroupGraphicsDef>,
    pub sprites: HashMap<String, SpriteGraphicsDef>,
}