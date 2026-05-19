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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub enum FloorGraphicsTileGroup {
    Dirt = 0,
    Tile = 1,
    Water = 2,
    Lava = 3,
}
impl Default for FloorGraphicsTileGroup {
    fn default() -> Self {
        FloorGraphicsTileGroup::Dirt
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Deserialize)]
pub enum WallGraphicsTileGroup {
    None = 0,
    Sandstone = 1,
    Rocks = 2,
    Bricks = 3,
}
impl Default for WallGraphicsTileGroup {
    fn default() -> Self {
        WallGraphicsTileGroup::Rocks
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