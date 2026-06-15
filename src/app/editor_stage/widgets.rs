use crate::assets::{AssetDb, AssetKind};
use crate::game_config::ConfigId;
use crate::game_config::floor_part_adjacency::FloorPartAdjacencyConfig;
use crate::game_config::floor_parts::{FloorCellExtra, FloorPartConfig};
use crate::game_config::items::ItemConfig;
use crate::game_config::parameters::{ParameterConfig, TagConfig};
use crate::game_config::effects::EffectConfig;
use crate::game_config::units::UnitConfig;
use crate::graphics::{
    FloorGraphicsTileGroup, SPRITE_ATLAS_DEF, WANG_MASK_CLAMP_EAST_LOOKUP,
    WANG_MASK_CLAMP_NORTH_EAST_LOOKUP, WANG_MASK_CLAMP_NORTH_LOOKUP,
    WANG_MASK_CLAMP_NORTH_WEST_LOOKUP, WANG_MASK_CLAMP_SOUTH_EAST_LOOKUP,
    WANG_MASK_CLAMP_SOUTH_LOOKUP, WANG_MASK_CLAMP_SOUTH_WEST_LOOKUP, WANG_MASK_CLAMP_WEST_LOOKUP,
    WANG_MASK_LOOKUP, WANG_MASK_NORTH_EAST, WANG_MASK_NORTH_WEST, WANG_MASK_SOUTH_EAST,
    WANG_MASK_SOUTH_WEST, WallGraphicsTileGroup,
};
use egui::text::LayoutJob;
use egui::{Align, Align2, Color32, CornerRadius, Layout, Rect, Response, Sense, Stroke, StrokeKind, TextStyle, TextureId, Ui, UiBuilder, Vec2, pos2, vec2, Id, PopupCloseBehavior, Button};
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
struct AtlasSpriteRect {
    /// Размер всего атласа в пикселях.
    pub atlas_size: Vec2,

    /// Прямоугольник спрайта внутри атласа, в пикселях.
    pub rect_px: Rect,
}

const MOCK_FLOOR_DATA: FloorPartConfig = {
    const ALL_DIRT: [FloorGraphicsTileGroup; 5] = [FloorGraphicsTileGroup::Dirt; 5];
    const TILES_INSIDE: [FloorGraphicsTileGroup; 5] = [
        FloorGraphicsTileGroup::Dirt,
        FloorGraphicsTileGroup::Tile,
        FloorGraphicsTileGroup::Tile,
        FloorGraphicsTileGroup::Tile,
        FloorGraphicsTileGroup::Dirt,
    ];
    FloorPartConfig {
        floor_data: [ALL_DIRT, TILES_INSIDE, TILES_INSIDE, TILES_INSIDE, ALL_DIRT],
        wall_data: [[WallGraphicsTileGroup::None; 5]; 5],
        extra_data: [[FloorCellExtra::None; 5]; 5],
    }
};

impl AtlasSpriteRect {
    pub fn from_u16(atlas_size: [u16; 2], [x, y]: [u16; 2], [w, h]: [u16; 2]) -> Self {
        AtlasSpriteRect {
            atlas_size: atlas_size.map(|it| it as f32).into(),
            rect_px: Rect::from_min_max(
                [x as f32, y as f32].into(),
                [(x + w) as f32, (y + h) as f32].into(),
            ),
        }
    }

    pub fn size_px(&self) -> Vec2 {
        self.rect_px.size()
    }

    pub fn uv_rect(&self) -> Rect {
        Rect::from_min_max(
            pos2(
                self.rect_px.min.x / self.atlas_size.x,
                self.rect_px.min.y / self.atlas_size.y,
            ),
            pos2(
                self.rect_px.max.x / self.atlas_size.x,
                self.rect_px.max.y / self.atlas_size.y,
            ),
        )
    }
}

pub fn atlas_sprite_button(
    ui: &mut Ui,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    sprite_name: &str,
    size: f32,
) -> Response {
    let sprite_data = crate::graphics::SPRITE_ATLAS_DEF.get_sprite_def(sprite_name);
    let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

    let atlas_rect = AtlasSpriteRect::from_u16(
        atlas_size,
        [
            sprite_data.coords[0] as u16 * tile_size[0] as u16,
            sprite_data.coords[1] as u16 * tile_size[1] as u16,
        ],
        [
            sprite_data.size[0] as u16 * tile_size[0] as u16,
            sprite_data.size[1] as u16 * tile_size[1] as u16,
        ],
    );

    let sprite_size_px = atlas_rect.size_px();

    let button_size = vec2(size, size);

    let image_size = if sprite_size_px.x >= sprite_size_px.y {
        let aspect = sprite_size_px.x / sprite_size_px.y;
        vec2(button_size.x, button_size.y / aspect)
    } else {
        let aspect = sprite_size_px.y / sprite_size_px.x;
        vec2(button_size.x / aspect, button_size.y)
    };
    let offset_x = (button_size.x - image_size.x) / 2f32;
    let offset_y = (button_size.y - image_size.y) / 2f32;

    let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(4),
            ui.style().interact(&response).bg_fill,
        );
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            ui.style().interact(&response).bg_stroke,
            StrokeKind::Inside,
        );

        let image_rect = Rect::from_min_max(
            [rect.min.x + offset_x, rect.min.y + offset_y].into(),
            [
                rect.min.x + offset_x + image_size.x,
                rect.min.y + offset_y + image_size.y,
            ]
            .into(),
        );

        ui.painter().image(
            atlas_texture,
            image_rect,
            atlas_rect.uv_rect(),
            Color32::WHITE,
        );

        let text_color = ui.style().interact(&response).text_color();
        let text_x = rect.center().x;
        let text_y = rect.max.y - ui.spacing().interact_size.y * 0.5f32;

        ui.painter().text(
            pos2(text_x, text_y),
            Align2::CENTER_CENTER,
            sprite_name,
            TextStyle::Small.resolve(ui.style()),
            text_color,
        );
    }

    response
}

pub trait SpriteHolder {
    fn sprite_name(&self) -> &str;
    fn sprite_pivot(&self) -> &[u8; 2];
    fn sprite_pivot_mut(&mut self) -> &mut [u8; 2];
}

pub fn sprite_pivot_editor<Holder: SpriteHolder>(
    ui: &mut Ui,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    sprite_holder: &mut Holder,
    zoom: f32,
) -> Response {
    let sprite_data = crate::graphics::SPRITE_ATLAS_DEF.get_sprite_def(sprite_holder.sprite_name());
    let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

    let sprite = AtlasSpriteRect::from_u16(
        atlas_size,
        [
            sprite_data.coords[0] as u16 * tile_size[0] as u16,
            sprite_data.coords[1] as u16 * tile_size[1] as u16,
        ],
        [
            sprite_data.size[0] as u16 * tile_size[0] as u16,
            sprite_data.size[1] as u16 * tile_size[1] as u16,
        ],
    );

    let sprite_size_px = sprite.size_px();

    let display_size = vec2(sprite_size_px.x * zoom, sprite_size_px.y * zoom);

    let (rect, response) = ui.allocate_exact_size(display_size, Sense::click_and_drag());

    if ui.is_rect_visible(rect) {
        ui.painter()
            .image(texture_id, rect, sprite.uv_rect(), Color32::WHITE);

        ui.painter().rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            StrokeKind::Inside,
        );

        let pivot_screen_pos = pos2(
            rect.left() + (sprite_holder.sprite_pivot_mut()[0] as f32 + 0.5) * zoom,
            rect.top() + (sprite_holder.sprite_pivot_mut()[1] as f32 + 0.5) * zoom,
        );

        ui.painter()
            .circle_filled(pivot_screen_pos, 4f32, Color32::RED);

        ui.painter()
            .circle_stroke(pivot_screen_pos, 5f32, Stroke::new(3f32, Color32::WHITE));
    }

    if response.clicked() || response.dragged() {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if rect.contains(pointer_pos) {
                let local_x = ((pointer_pos.x - rect.left()) / zoom).floor() as u8;
                let local_y = ((pointer_pos.y - rect.top()) / zoom).floor() as u8;

                let max_x = sprite_size_px.x as u8 - 1;
                let max_y = sprite_size_px.y as u8 - 1;

                sprite_holder.sprite_pivot_mut()[0] = local_x.clamp(0, max_x);
                sprite_holder.sprite_pivot_mut()[1] = local_y.clamp(0, max_y);
            }
        }
    }

    response
}

pub fn sprite_holder_visualizer<Holder: SpriteHolder>(
    ui: &mut Ui,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    sprite_holder: &Holder,
) {
    let tile_size = crate::graphics::SPRITE_ATLAS_DEF
        .tile_size
        .map(|it| it as f32);
    let display_size = vec2(
        tile_size[0] * MOCK_FLOOR_DATA.width() as f32,
        tile_size[1] * MOCK_FLOOR_DATA.height() as f32,
    );
    let available_width = ui.available_width();
    let zoom = available_width / display_size.x;
    let display_size = display_size * zoom;

    let (rect, _) = ui.allocate_exact_size(display_size, Sense::empty());
    if ui.is_rect_visible(rect) {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            StrokeKind::Inside,
        );

        draw_floors(
            ui,
            texture_id,
            atlas_size,
            &MOCK_FLOOR_DATA,
            zoom,
            tile_size,
            rect,
        );

        let central_point = rect.center();
        let sprite_data = crate::graphics::SPRITE_ATLAS_DEF.get_sprite_def(
            sprite_holder.sprite_name()
        );
        let sprite_size = [
            sprite_data.size[0] as u16 * tile_size[0] as u16,
            sprite_data.size[1] as u16 * tile_size[1] as u16,
        ];
        let sprite = AtlasSpriteRect::from_u16(
            atlas_size,
            [
                sprite_data.coords[0] as u16 * tile_size[0] as u16,
                sprite_data.coords[1] as u16 * tile_size[1] as u16,
            ],
            sprite_size,
        );

        let image_size = [sprite_size[0] as f32 * zoom, sprite_size[1] as f32 * zoom].into();
        let image_pos = pos2(
            central_point.x - sprite_holder.sprite_pivot()[0] as f32 * zoom,
            central_point.y - sprite_holder.sprite_pivot()[1] as f32 * zoom,
        );

        let image_rect = Rect::from_min_max(image_pos, image_pos + image_size);
        ui.painter()
            .image(texture_id, image_rect, sprite.uv_rect(), Color32::WHITE);
    }
}

pub trait EditableFloorData {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get_floor_data(&self, coords: [usize; 2]) -> &FloorGraphicsTileGroup;
    fn get_floor_data_mut(&mut self, coords: [usize; 2]) -> &mut FloorGraphicsTileGroup;
    fn get_wall_data(&self, coords: [usize; 2]) -> &WallGraphicsTileGroup;
    fn get_wall_data_mut(&mut self, coords: [usize; 2]) -> &mut WallGraphicsTileGroup;
    fn get_cell_extra_data(&self, coords: [usize; 2]) -> &FloorCellExtra;
    fn get_cell_extra_data_mut(&mut self, coords: [usize; 2]) -> &mut FloorCellExtra;
}

pub fn floor_data_holder_editor(
    ui: &mut Ui,
    asset_db: &AssetDb,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    data: &impl EditableFloorData,
    zoom: u8,
) -> Option<[usize; 2]> {
    let zoom = zoom.clamp(1, 8) as f32;

    let tile_size = crate::graphics::SPRITE_ATLAS_DEF
        .tile_size
        .map(|it| it as f32);
    let display_size = vec2(
        tile_size[0] * zoom * data.width() as f32,
        tile_size[1] * zoom * data.height() as f32,
    );

    let (rect, response) = ui.allocate_exact_size(display_size, Sense::click_and_drag());

    if ui.is_rect_visible(rect) {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            StrokeKind::Inside,
        );

        draw_floors(ui, texture_id, atlas_size, data, zoom, tile_size, rect);
        draw_extra(ui, asset_db, texture_id, atlas_size, data, zoom, tile_size, rect);
        draw_walls(ui, texture_id, atlas_size, data, zoom, tile_size, rect);
    }

    let mut result = None;

    if let Some(hover_pos) = response.hover_pos() {
        if rect.contains(hover_pos) {
            let local_x = ((hover_pos.x - rect.left()) / zoom / tile_size[0]).floor() as usize;
            let local_y = ((hover_pos.y - rect.top()) / zoom / tile_size[1]).floor() as usize;

            let pt = rect.min
                + vec2(
                    local_x as f32 * tile_size[0] * zoom,
                    local_y as f32 * tile_size[1] * zoom,
                );

            let selection_rect =
                Rect::from_min_max(pt, pt + vec2(tile_size[0] * zoom, tile_size[1] * zoom));
            let mut selection_stroke = ui.visuals().selection.stroke;
            selection_stroke.width = 4.0;
            ui.painter().rect_stroke(
                selection_rect,
                CornerRadius::same(2 * zoom as u8),
                selection_stroke,
                StrokeKind::Inside,
            );

            if response.clicked() || response.dragged() {
                result = Some([local_x, local_y]);
            }
        }
    }

    result
}

const DIRT_MASK: [[bool; 5]; 5] = [
    [true, false, true, false, true],
    [true, true, false, false, true],
    [false, true, true, true, false],
    [true, false, false, true, true],
    [false, false, true, true, true],
];

const fn offset_coords(base_coords: [u8; 2], offset: [u8; 2]) -> [u8; 2] {
    [base_coords[0] + offset[0], base_coords[1] + offset[1]]
}

fn sprite_rect(atlas_size: [u16; 2], tile_size: [u16; 2], coords: [u8; 2]) -> AtlasSpriteRect {
    AtlasSpriteRect::from_u16(
        atlas_size,
        [
            coords[0] as u16 * tile_size[0],
            coords[1] as u16 * tile_size[1],
        ],
        [tile_size[0], tile_size[1]],
    )
}

fn get_coords_north(
    base_coords: [u8; 2],
    atlas_size: [u16; 2],
    tile_size: [f32; 2],
    mask: usize,
    mut image_rect: Rect,
    zoom: f32,
) -> (AtlasSpriteRect, Rect) {
    let coords = offset_coords(base_coords, WANG_MASK_CLAMP_NORTH_LOOKUP[mask]);
    let mut atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);
    atlas_rect.rect_px.min.y += tile_size[1] / 2f32;
    image_rect.min.y -= tile_size[1] * zoom / 2f32;
    image_rect.max.y -= tile_size[1] * zoom;
    (atlas_rect, image_rect)
}
fn get_coords_north_west(
    base_coords: [u8; 2],
    atlas_size: [u16; 2],
    tile_size: [f32; 2],
    mask: usize,
    mut image_rect: Rect,
    zoom: f32,
) -> (AtlasSpriteRect, Rect) {
    let coords = offset_coords(base_coords, WANG_MASK_CLAMP_NORTH_WEST_LOOKUP[mask]);
    let mut atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);
    atlas_rect.rect_px.min.y += tile_size[1] / 2f32;
    atlas_rect.rect_px.min.x += tile_size[0] / 2f32;
    image_rect.min.y -= tile_size[1] * zoom / 2f32;
    image_rect.max.y -= tile_size[1] * zoom;
    image_rect.min.x -= tile_size[0] * zoom / 2f32;
    image_rect.max.x -= tile_size[0] * zoom;
    (atlas_rect, image_rect)
}
fn get_coords_north_east(
    base_coords: [u8; 2],
    atlas_size: [u16; 2],
    tile_size: [f32; 2],
    mask: usize,
    mut image_rect: Rect,
    zoom: f32,
) -> (AtlasSpriteRect, Rect) {
    let coords = offset_coords(base_coords, WANG_MASK_CLAMP_NORTH_EAST_LOOKUP[mask]);
    let mut atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);
    atlas_rect.rect_px.min.y += tile_size[1] / 2f32;
    atlas_rect.rect_px.max.x -= tile_size[0] / 2f32;
    image_rect.min.y -= tile_size[1] * zoom / 2f32;
    image_rect.max.y -= tile_size[1] * zoom;
    image_rect.max.x += tile_size[0] * zoom / 2f32;
    image_rect.min.x += tile_size[0] * zoom;
    (atlas_rect, image_rect)
}

fn get_coords_south(
    base_coords: [u8; 2],
    atlas_size: [u16; 2],
    tile_size: [f32; 2],
    mask: usize,
    mut image_rect: Rect,
    zoom: f32,
) -> (AtlasSpriteRect, Rect) {
    let coords = offset_coords(base_coords, WANG_MASK_CLAMP_SOUTH_LOOKUP[mask]);
    let mut atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);
    atlas_rect.rect_px.max.y -= tile_size[1] / 2f32;
    image_rect.max.y += tile_size[1] * zoom / 2f32;
    image_rect.min.y += tile_size[1] * zoom;
    (atlas_rect, image_rect)
}

fn get_coords_south_west(
    base_coords: [u8; 2],
    atlas_size: [u16; 2],
    tile_size: [f32; 2],
    mask: usize,
    mut image_rect: Rect,
    zoom: f32,
) -> (AtlasSpriteRect, Rect) {
    let coords = offset_coords(base_coords, WANG_MASK_CLAMP_SOUTH_WEST_LOOKUP[mask]);
    let mut atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);
    atlas_rect.rect_px.max.y -= tile_size[1] / 2f32;
    atlas_rect.rect_px.min.x += tile_size[0] / 2f32;
    image_rect.max.y += tile_size[1] * zoom / 2f32;
    image_rect.min.y += tile_size[1] * zoom;
    image_rect.min.x -= tile_size[0] * zoom / 2f32;
    image_rect.max.x -= tile_size[0] * zoom;
    (atlas_rect, image_rect)
}

fn get_coords_south_east(
    base_coords: [u8; 2],
    atlas_size: [u16; 2],
    tile_size: [f32; 2],
    mask: usize,
    mut image_rect: Rect,
    zoom: f32,
) -> (AtlasSpriteRect, Rect) {
    let coords = offset_coords(base_coords, WANG_MASK_CLAMP_SOUTH_EAST_LOOKUP[mask]);
    let mut atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);
    atlas_rect.rect_px.max.y -= tile_size[1] / 2f32;
    atlas_rect.rect_px.max.x -= tile_size[0] / 2f32;
    image_rect.max.y += tile_size[1] * zoom / 2f32;
    image_rect.min.y += tile_size[1] * zoom;
    image_rect.max.x += tile_size[0] * zoom / 2f32;
    image_rect.min.x += tile_size[0] * zoom;
    (atlas_rect, image_rect)
}

fn get_coords_east(
    base_coords: [u8; 2],
    atlas_size: [u16; 2],
    tile_size: [f32; 2],
    mask: usize,
    mut image_rect: Rect,
    zoom: f32,
) -> (AtlasSpriteRect, Rect) {
    let coords = offset_coords(base_coords, WANG_MASK_CLAMP_EAST_LOOKUP[mask]);
    let mut atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);
    atlas_rect.rect_px.max.x -= tile_size[0] / 2f32;
    image_rect.max.x += tile_size[0] * zoom / 2f32;
    image_rect.min.x += tile_size[0] * zoom;
    (atlas_rect, image_rect)
}

fn get_coords_west(
    base_coords: [u8; 2],
    atlas_size: [u16; 2],
    tile_size: [f32; 2],
    mask: usize,
    mut image_rect: Rect,
    zoom: f32,
) -> (AtlasSpriteRect, Rect) {
    let coords = offset_coords(base_coords, WANG_MASK_CLAMP_WEST_LOOKUP[mask]);
    let mut atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);
    atlas_rect.rect_px.min.x += tile_size[0] / 2f32;
    image_rect.min.x -= tile_size[0] * zoom / 2f32;
    image_rect.max.x -= tile_size[0] * zoom;
    (atlas_rect, image_rect)
}

#[derive(Copy, Clone, Debug)]
pub enum NeighbourData<'a, TNeighbours: IntoIterator<Item = &'a ConfigId<FloorPartConfig>> + Copy> {
    SingleFocus(ConfigId<FloorPartConfig>),
    Multiple(TNeighbours),
}

pub fn visualize_floor_part_adjacency_in_rect<
    'a,
    TNeighbours: IntoIterator<Item = &'a ConfigId<FloorPartConfig>> + Copy,
>(
    ui: &'a mut Ui,
    rect: Rect,
    asset_db: &AssetDb,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    central_part: ConfigId<FloorPartConfig>,
    north_neighbours: NeighbourData<'a, TNeighbours>,
    south_neighbours: NeighbourData<'a, TNeighbours>,
    west_neighbours: NeighbourData<'a, TNeighbours>,
    east_neighbours: NeighbourData<'a, TNeighbours>,
    zoom: f32,
) {
    fn square_side(count: usize) -> usize {
        if count == 0 {
            return 0;
        }

        let mut side = 1;

        while side * side < count {
            side += 1;
        }

        side
    }

    let tile_size = crate::graphics::SPRITE_ATLAS_DEF
        .tile_size
        .map(|it| it as f32);

    if !asset_db.has_asset(AssetKind::FloorPartConfig, central_part.uuid) {
        return;
    }

    let north_neighbour_count = match north_neighbours {
        NeighbourData::SingleFocus(cfg_id) => {
            if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                1
            } else {
                0
            }
        }
        NeighbourData::Multiple(north_neighbours) => north_neighbours
            .into_iter()
            .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
            .count(),
    };
    let north_sides = square_side(north_neighbour_count);

    let south_neighbour_count = match south_neighbours {
        NeighbourData::SingleFocus(cfg_id) => {
            if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                1
            } else {
                0
            }
        }
        NeighbourData::Multiple(south_neighbours) => south_neighbours
            .into_iter()
            .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
            .count(),
    };
    let south_sides = square_side(south_neighbour_count);

    let west_neighbour_count = match west_neighbours {
        NeighbourData::SingleFocus(cfg_id) => {
            if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                1
            } else {
                0
            }
        }
        NeighbourData::Multiple(west_neighbours) => west_neighbours
            .into_iter()
            .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
            .count(),
    };
    let west_sides = square_side(west_neighbour_count);

    let east_neighbour_count = match east_neighbours {
        NeighbourData::SingleFocus(cfg_id) => {
            if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                1
            } else {
                0
            }
        }
        NeighbourData::Multiple(east_neighbours) => east_neighbours
            .into_iter()
            .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
            .count(),
    };
    let east_sides = square_side(east_neighbour_count);

    let bytes = asset_db.load_asset(AssetKind::FloorPartConfig, central_part.uuid);
    let central_part =
        FloorPartConfig::load_from_slice(bytes).expect("Failed to load floor part config");

    let central_rect =
        Rect::from_min_max(rect.min + rect.size() / 3f32, rect.max - rect.size() / 3f32);

    let north_rect = Rect::from_min_max(
        central_rect.min - vec2(0f32, central_rect.height()),
        central_rect.max - vec2(0f32, central_rect.height()),
    );
    let north_sub_rect_size = if north_sides <= 1 {
        north_rect.size()
    } else {
        north_rect.size() / north_sides as f32
    };

    let south_rect = Rect::from_min_max(
        central_rect.min + vec2(0f32, central_rect.height()),
        central_rect.max + vec2(0f32, central_rect.height()),
    );
    let south_sub_rect_size = if south_sides <= 1 {
        south_rect.size()
    } else {
        south_rect.size() / south_sides as f32
    };

    let west_rect = Rect::from_min_max(
        central_rect.min - vec2(central_rect.width(), 0f32),
        central_rect.max - vec2(central_rect.width(), 0f32),
    );
    let west_sub_rect_size = if west_sides <= 1 {
        west_rect.size()
    } else {
        west_rect.size() / west_sides as f32
    };

    let east_rect = Rect::from_min_max(
        central_rect.min + vec2(central_rect.width(), 0f32),
        central_rect.max + vec2(central_rect.width(), 0f32),
    );
    let east_sub_rect_size = if east_sides <= 1 {
        east_rect.size()
    } else {
        east_rect.size() / east_sides as f32
    };

    draw_floors(
        ui,
        texture_id,
        atlas_size,
        &central_part,
        zoom,
        tile_size,
        central_rect,
    );
    draw_walls(
        ui,
        texture_id,
        atlas_size,
        &central_part,
        zoom,
        tile_size,
        central_rect,
    );

    let north_sub_rect = Rect::from_min_max(
        pos2(north_rect.min.x, north_rect.max.y - north_sub_rect_size.y),
        pos2(north_rect.min.x + north_sub_rect_size.x, north_rect.max.y),
    );
    let north_sub_rect_step_small = vec2(north_sub_rect_size.x, 0f32);
    let north_sub_rect_step_big = vec2(
        -north_sub_rect_size.x * north_sides as f32,
        -north_sub_rect_size.y,
    );

    let south_sub_rect = Rect::from_min_max(
        pos2(south_rect.min.x, south_rect.min.y),
        pos2(
            south_rect.min.x + south_sub_rect_size.x,
            south_rect.max.y + south_sub_rect_size.y,
        ),
    );
    let south_sub_rect_step_small = vec2(south_sub_rect_size.x, 0f32);
    let south_sub_rect_step_big = vec2(
        -south_sub_rect_size.x * south_sides as f32,
        south_sub_rect_size.y,
    );

    let west_sub_rect = Rect::from_min_max(
        pos2(west_rect.max.x - west_sub_rect_size.x, west_rect.min.y),
        pos2(west_rect.max.x, west_rect.min.y + west_sub_rect_size.y),
    );
    let west_sub_rect_step_small = vec2(0f32, west_sub_rect_size.y);
    let west_sub_rect_step_big = vec2(
        -west_sub_rect_size.x,
        -west_sub_rect_size.y * west_sides as f32,
    );

    let east_sub_rect = Rect::from_min_max(
        pos2(east_rect.min.x, east_rect.min.y),
        pos2(
            east_rect.min.x + east_sub_rect_size.x,
            east_rect.min.y + east_sub_rect_size.y,
        ),
    );
    let east_sub_rect_step_small = vec2(0f32, east_sub_rect_size.y);
    let east_sub_rect_step_big = vec2(
        east_sub_rect_size.x,
        -east_sub_rect_size.y * east_sides as f32,
    );

    do_neighbour_draw(
        ui,
        asset_db,
        texture_id,
        atlas_size,
        north_neighbours,
        south_neighbours,
        west_neighbours,
        east_neighbours,
        zoom,
        tile_size,
        north_sides,
        south_sides,
        west_sides,
        east_sides,
        north_sub_rect,
        north_sub_rect_step_small,
        north_sub_rect_step_big,
        south_sub_rect,
        south_sub_rect_step_small,
        south_sub_rect_step_big,
        west_sub_rect,
        west_sub_rect_step_small,
        west_sub_rect_step_big,
        east_sub_rect,
        east_sub_rect_step_small,
        east_sub_rect_step_big,
    );
}

fn do_neighbour_draw<'a, TNeighbours: IntoIterator<Item = &'a ConfigId<FloorPartConfig>> + Copy>(
    ui: &mut Ui,
    asset_db: &AssetDb,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    north_neighbours: NeighbourData<'a, TNeighbours>,
    south_neighbours: NeighbourData<'a, TNeighbours>,
    west_neighbours: NeighbourData<'a, TNeighbours>,
    east_neighbours: NeighbourData<'a, TNeighbours>,
    zoom: f32,
    tile_size: [f32; 2],
    north_sides: usize,
    south_sides: usize,
    west_sides: usize,
    east_sides: usize,
    north_sub_rect: Rect,
    north_sub_rect_step_small: Vec2,
    north_sub_rect_step_big: Vec2,
    south_sub_rect: Rect,
    south_sub_rect_step_small: Vec2,
    south_sub_rect_step_big: Vec2,
    west_sub_rect: Rect,
    west_sub_rect_step_small: Vec2,
    west_sub_rect_step_big: Vec2,
    east_sub_rect: Rect,
    east_sub_rect_step_small: Vec2,
    east_sub_rect_step_big: Vec2,
) {
    for (mut sub_rect, step_small, step_big, neighbours, sides) in [
        (
            north_sub_rect,
            north_sub_rect_step_small,
            north_sub_rect_step_big,
            north_neighbours,
            north_sides,
        ),
        (
            south_sub_rect,
            south_sub_rect_step_small,
            south_sub_rect_step_big,
            south_neighbours,
            south_sides,
        ),
        (
            west_sub_rect,
            west_sub_rect_step_small,
            west_sub_rect_step_big,
            west_neighbours,
            west_sides,
        ),
        (
            east_sub_rect,
            east_sub_rect_step_small,
            east_sub_rect_step_big,
            east_neighbours,
            east_sides,
        ),
    ] {
        let mut offset = 0;
        match neighbours {
            NeighbourData::SingleFocus(cfg_id) => {
                if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                    let bytes = asset_db.load_asset(AssetKind::FloorPartConfig, cfg_id.uuid);
                    let side_part = FloorPartConfig::load_from_slice(bytes)
                        .expect("Failed to load FloorPartConfig");

                    draw_floors(
                        ui, texture_id, atlas_size, &side_part, zoom, tile_size, sub_rect,
                    );
                    draw_walls(
                        ui, texture_id, atlas_size, &side_part, zoom, tile_size, sub_rect,
                    );
                }
            }
            NeighbourData::Multiple(neighbours) => {
                for side_part in neighbours
                    .into_iter()
                    .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
                    .map(|cfg_id| asset_db.load_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
                    .map(|bytes| {
                        FloorPartConfig::load_from_slice(bytes)
                            .expect("Failed to load FloorPartConfig")
                    })
                {
                    draw_floors(
                        ui,
                        texture_id,
                        atlas_size,
                        &side_part,
                        zoom / sides as f32,
                        tile_size,
                        sub_rect,
                    );
                    draw_walls(
                        ui,
                        texture_id,
                        atlas_size,
                        &side_part,
                        zoom / sides as f32,
                        tile_size,
                        sub_rect,
                    );

                    sub_rect =
                        Rect::from_min_max(sub_rect.min + step_small, sub_rect.max + step_small);
                    offset += 1;
                    if offset == sides {
                        offset = 0;
                        sub_rect =
                            Rect::from_min_max(sub_rect.min + step_big, sub_rect.max + step_big);
                    }
                }
            }
        }
    }
}

pub fn visualize_floor_part_adjacency<
    'a,
    TNeighbours: IntoIterator<Item = &'a ConfigId<FloorPartConfig>> + Copy,
>(
    ui: &'a mut Ui,
    asset_db: &AssetDb,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    central_part: ConfigId<FloorPartConfig>,
    north_neighbours: NeighbourData<'a, TNeighbours>,
    south_neighbours: NeighbourData<'a, TNeighbours>,
    west_neighbours: NeighbourData<'a, TNeighbours>,
    east_neighbours: NeighbourData<'a, TNeighbours>,
    zoom: f32,
) {
    fn square_side(count: usize) -> usize {
        if count == 0 {
            return 0;
        }

        let mut side = 1;

        while side * side < count {
            side += 1;
        }

        side
    }

    let tile_size = crate::graphics::SPRITE_ATLAS_DEF
        .tile_size
        .map(|it| it as f32);

    if !asset_db.has_asset(AssetKind::FloorPartConfig, central_part.uuid) {
        return;
    }

    let north_neighbour_count = match north_neighbours {
        NeighbourData::SingleFocus(cfg_id) => {
            if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                1
            } else {
                0
            }
        }
        NeighbourData::Multiple(north_neighbours) => north_neighbours
            .into_iter()
            .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
            .count(),
    };
    let north_sides = square_side(north_neighbour_count);

    let south_neighbour_count = match south_neighbours {
        NeighbourData::SingleFocus(cfg_id) => {
            if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                1
            } else {
                0
            }
        }
        NeighbourData::Multiple(south_neighbours) => south_neighbours
            .into_iter()
            .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
            .count(),
    };
    let south_sides = square_side(south_neighbour_count);

    let west_neighbour_count = match west_neighbours {
        NeighbourData::SingleFocus(cfg_id) => {
            if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                1
            } else {
                0
            }
        }
        NeighbourData::Multiple(west_neighbours) => west_neighbours
            .into_iter()
            .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
            .count(),
    };
    let west_sides = square_side(west_neighbour_count);

    let east_neighbour_count = match east_neighbours {
        NeighbourData::SingleFocus(cfg_id) => {
            if asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid) {
                1
            } else {
                0
            }
        }
        NeighbourData::Multiple(east_neighbours) => east_neighbours
            .into_iter()
            .filter(|cfg_id| asset_db.has_asset(AssetKind::FloorPartConfig, cfg_id.uuid))
            .count(),
    };
    let east_sides = square_side(east_neighbour_count);

    let bytes = asset_db.load_asset(AssetKind::FloorPartConfig, central_part.uuid);
    let central_part =
        FloorPartConfig::load_from_slice(bytes).expect("Failed to load floor part config");

    // Мы будем рисовать часть уровня в центре и наборы частей по краям от неё,
    // вписанные в квадраты равного с ней размера
    let display_size = vec2(
        tile_size[0] * central_part.floor_data[0].len() as f32 * 3f32 * zoom,
        tile_size[1] * central_part.floor_data.len() as f32 * 3f32 * zoom,
    );

    let (rect, _) = ui.allocate_exact_size(display_size, Sense::empty());
    if ui.is_rect_visible(rect) {
        let central_rect =
            Rect::from_min_max(rect.min + rect.size() / 3f32, rect.max - rect.size() / 3f32);

        let north_rect = Rect::from_min_max(
            central_rect.min - vec2(0f32, central_rect.height()),
            central_rect.max - vec2(0f32, central_rect.height()),
        );
        let north_sub_rect_size = if north_sides <= 1 {
            north_rect.size()
        } else {
            north_rect.size() / north_sides as f32
        };

        let south_rect = Rect::from_min_max(
            central_rect.min + vec2(0f32, central_rect.height()),
            central_rect.max + vec2(0f32, central_rect.height()),
        );
        let south_sub_rect_size = if south_sides <= 1 {
            south_rect.size()
        } else {
            south_rect.size() / south_sides as f32
        };

        let west_rect = Rect::from_min_max(
            central_rect.min - vec2(central_rect.width(), 0f32),
            central_rect.max - vec2(central_rect.width(), 0f32),
        );
        let west_sub_rect_size = if west_sides <= 1 {
            west_rect.size()
        } else {
            west_rect.size() / west_sides as f32
        };

        let east_rect = Rect::from_min_max(
            central_rect.min + vec2(central_rect.width(), 0f32),
            central_rect.max + vec2(central_rect.width(), 0f32),
        );
        let east_sub_rect_size = if east_sides <= 1 {
            east_rect.size()
        } else {
            east_rect.size() / east_sides as f32
        };

        draw_floors(
            ui,
            texture_id,
            atlas_size,
            &central_part,
            zoom,
            tile_size,
            central_rect,
        );
        draw_walls(
            ui,
            texture_id,
            atlas_size,
            &central_part,
            zoom,
            tile_size,
            central_rect,
        );

        let north_sub_rect = Rect::from_min_max(
            pos2(north_rect.min.x, north_rect.max.y - north_sub_rect_size.y),
            pos2(north_rect.min.x + north_sub_rect_size.x, north_rect.max.y),
        );
        let north_sub_rect_step_small = vec2(north_sub_rect_size.x, 0f32);
        let north_sub_rect_step_big = vec2(
            -north_sub_rect_size.x * north_sides as f32,
            -north_sub_rect_size.y,
        );

        let south_sub_rect = Rect::from_min_max(
            pos2(south_rect.min.x, south_rect.min.y),
            pos2(
                south_rect.min.x + south_sub_rect_size.x,
                south_rect.max.y + south_sub_rect_size.y,
            ),
        );
        let south_sub_rect_step_small = vec2(south_sub_rect_size.x, 0f32);
        let south_sub_rect_step_big = vec2(
            -south_sub_rect_size.x * south_sides as f32,
            south_sub_rect_size.y,
        );

        let west_sub_rect = Rect::from_min_max(
            pos2(west_rect.max.x - west_sub_rect_size.x, west_rect.min.y),
            pos2(west_rect.max.x, west_rect.min.y + west_sub_rect_size.y),
        );
        let west_sub_rect_step_small = vec2(0f32, west_sub_rect_size.y);
        let west_sub_rect_step_big = vec2(
            -west_sub_rect_size.x,
            -west_sub_rect_size.y * west_sides as f32,
        );

        let east_sub_rect = Rect::from_min_max(
            pos2(east_rect.min.x, east_rect.min.y),
            pos2(
                east_rect.min.x + east_sub_rect_size.x,
                east_rect.min.y + east_sub_rect_size.y,
            ),
        );
        let east_sub_rect_step_small = vec2(0f32, east_sub_rect_size.y);
        let east_sub_rect_step_big = vec2(
            east_sub_rect_size.x,
            -east_sub_rect_size.y * east_sides as f32,
        );

        do_neighbour_draw(
            ui,
            asset_db,
            texture_id,
            atlas_size,
            north_neighbours,
            south_neighbours,
            west_neighbours,
            east_neighbours,
            zoom,
            tile_size,
            north_sides,
            south_sides,
            west_sides,
            east_sides,
            north_sub_rect,
            north_sub_rect_step_small,
            north_sub_rect_step_big,
            south_sub_rect,
            south_sub_rect_step_small,
            south_sub_rect_step_big,
            west_sub_rect,
            west_sub_rect_step_small,
            west_sub_rect_step_big,
            east_sub_rect,
            east_sub_rect_step_small,
            east_sub_rect_step_big,
        );
    }
}

fn draw_floors(
    ui: &mut Ui,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    data: &impl EditableFloorData,
    zoom: f32,
    tile_size: [f32; 2],
    rect: Rect,
) {
    let point_on_rect = |x: usize, y: usize| {
        pos2(
            rect.left() + (x as f32 + 0.5) * tile_size[0] * zoom,
            rect.top() + (y as f32 + 0.5) * tile_size[1] * zoom,
        )
    };

    let base_dirt_coords = crate::graphics::SPRITE_ATLAS_DEF.floor_tile_groups
        [&FloorGraphicsTileGroup::Dirt]
        .base_coords;
    let base_tile_coords = crate::graphics::SPRITE_ATLAS_DEF.floor_tile_groups
        [&FloorGraphicsTileGroup::Tile]
        .base_coords;
    let base_lava_coords = crate::graphics::SPRITE_ATLAS_DEF.floor_tile_groups
        [&FloorGraphicsTileGroup::Lava]
        .base_coords;
    let base_water_coords = crate::graphics::SPRITE_ATLAS_DEF.floor_tile_groups
        [&FloorGraphicsTileGroup::Water]
        .base_coords;

    for j in 0..data.height() - 1 {
        for i in 0..data.width() - 1 {
            let pt = point_on_rect(i, j);
            let image_rect = Rect::from_min_max(
                pt,
                [pt.x + tile_size[0] * zoom, pt.y + tile_size[1] * zoom].into(),
            );

            let mut dirt_bitmask = 0;
            if DIRT_MASK[j % 5][i % 5] {
                dirt_bitmask = dirt_bitmask | WANG_MASK_NORTH_WEST;
            }
            if DIRT_MASK[j % 5][(i + 1) % 5] {
                dirt_bitmask = dirt_bitmask | WANG_MASK_NORTH_EAST;
            }
            if DIRT_MASK[(j + 1) % 5][i % 5] {
                dirt_bitmask = dirt_bitmask | WANG_MASK_SOUTH_WEST;
            }
            if DIRT_MASK[(j + 1) % 5][(i + 1) % 5] {
                dirt_bitmask = dirt_bitmask | WANG_MASK_SOUTH_EAST;
            }
            let dirt_coords = offset_coords(base_dirt_coords, WANG_MASK_LOOKUP[dirt_bitmask]);
            let atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), dirt_coords);

            ui.painter()
                .image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);
            if j == 0 {
                let (atlas_rect, image_rect_n) = get_coords_north(
                    base_dirt_coords,
                    atlas_size,
                    tile_size,
                    dirt_bitmask,
                    image_rect,
                    zoom,
                );
                ui.painter().image(
                    texture_id,
                    image_rect_n,
                    atlas_rect.uv_rect(),
                    Color32::WHITE,
                );
                if i == 0 {
                    let (atlas_rect, image_rect_nw) = get_coords_north_west(
                        base_dirt_coords,
                        atlas_size,
                        tile_size,
                        dirt_bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect_nw,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );
                }
                if i == data.width() - 2 {
                    let (atlas_rect, image_rect_ne) = get_coords_north_east(
                        base_dirt_coords,
                        atlas_size,
                        tile_size,
                        dirt_bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect_ne,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );
                }
            }
            if j == data.height() - 2 {
                let (atlas_rect, image_rect_s) = get_coords_south(
                    base_dirt_coords,
                    atlas_size,
                    tile_size,
                    dirt_bitmask,
                    image_rect,
                    zoom,
                );
                ui.painter().image(
                    texture_id,
                    image_rect_s,
                    atlas_rect.uv_rect(),
                    Color32::WHITE,
                );

                if i == 0 {
                    let (atlas_rect, image_rect_sw) = get_coords_south_west(
                        base_dirt_coords,
                        atlas_size,
                        tile_size,
                        dirt_bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect_sw,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );
                }
                if i == data.width() - 2 {
                    let (atlas_rect, image_rect_se) = get_coords_south_east(
                        base_dirt_coords,
                        atlas_size,
                        tile_size,
                        dirt_bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect_se,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );
                }
            }
            if i == 0 {
                let (atlas_rect, image_rect) = get_coords_west(
                    base_dirt_coords,
                    atlas_size,
                    tile_size,
                    dirt_bitmask,
                    image_rect,
                    zoom,
                );
                ui.painter()
                    .image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);
            }
            if i == data.width() - 2 {
                let (atlas_rect, image_rect) = get_coords_east(
                    base_dirt_coords,
                    atlas_size,
                    tile_size,
                    dirt_bitmask,
                    image_rect,
                    zoom,
                );
                ui.painter()
                    .image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);
            }

            for (group, base_coords) in [
                (FloorGraphicsTileGroup::Lava, base_lava_coords),
                (FloorGraphicsTileGroup::Water, base_water_coords),
                (FloorGraphicsTileGroup::Tile, base_tile_coords),
            ] {
                let mut bitmask = 0;
                if *data.get_floor_data([i, j]) == group {
                    bitmask = bitmask | WANG_MASK_NORTH_WEST;
                }
                if *data.get_floor_data([i + 1, j]) == group {
                    bitmask = bitmask | WANG_MASK_NORTH_EAST;
                }
                if *data.get_floor_data([i, j + 1]) == group {
                    bitmask = bitmask | WANG_MASK_SOUTH_WEST;
                }
                if *data.get_floor_data([i + 1, j + 1]) == group {
                    bitmask = bitmask | WANG_MASK_SOUTH_EAST;
                }
                let coords = offset_coords(base_coords, WANG_MASK_LOOKUP[bitmask]);
                let atlas_rect = sprite_rect(atlas_size, tile_size.map(|it| it as u16), coords);

                ui.painter()
                    .image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);

                if j == 0 {
                    let (atlas_rect, image_rect_n) = get_coords_north(
                        base_coords,
                        atlas_size,
                        tile_size,
                        bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect_n,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );

                    if i == 0 {
                        let (atlas_rect, image_rect_nw) = get_coords_north_west(
                            base_coords,
                            atlas_size,
                            tile_size,
                            bitmask,
                            image_rect,
                            zoom,
                        );
                        ui.painter().image(
                            texture_id,
                            image_rect_nw,
                            atlas_rect.uv_rect(),
                            Color32::WHITE,
                        );
                    }
                    if i == data.width() - 2 {
                        let (atlas_rect, image_rect_ne) = get_coords_north_east(
                            base_coords,
                            atlas_size,
                            tile_size,
                            bitmask,
                            image_rect,
                            zoom,
                        );
                        ui.painter().image(
                            texture_id,
                            image_rect_ne,
                            atlas_rect.uv_rect(),
                            Color32::WHITE,
                        );
                    }
                }
                if j == data.height() - 2 {
                    let (atlas_rect, image_rect_s) = get_coords_south(
                        base_coords,
                        atlas_size,
                        tile_size,
                        bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect_s,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );

                    if i == 0 {
                        let (atlas_rect, image_rect_sw) = get_coords_south_west(
                            base_coords,
                            atlas_size,
                            tile_size,
                            bitmask,
                            image_rect,
                            zoom,
                        );
                        ui.painter().image(
                            texture_id,
                            image_rect_sw,
                            atlas_rect.uv_rect(),
                            Color32::WHITE,
                        );
                    }
                    if i == data.width() - 2 {
                        let (atlas_rect, image_rect_se) = get_coords_south_east(
                            base_coords,
                            atlas_size,
                            tile_size,
                            bitmask,
                            image_rect,
                            zoom,
                        );
                        ui.painter().image(
                            texture_id,
                            image_rect_se,
                            atlas_rect.uv_rect(),
                            Color32::WHITE,
                        );
                    }
                }
                if i == 0 {
                    let (atlas_rect, image_rect) = get_coords_west(
                        base_coords,
                        atlas_size,
                        tile_size,
                        bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );
                }
                if i == data.width() - 2 {
                    let (atlas_rect, image_rect) = get_coords_east(
                        base_coords,
                        atlas_size,
                        tile_size,
                        bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}

fn draw_extra(
    ui: &mut Ui,
    asset_db: &AssetDb,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    data: &impl EditableFloorData,
    zoom: f32,
    tile_size: [f32; 2],
    rect: Rect,
) {
    let point_on_rect = |x: usize, y: usize, [pivot_x, pivot_y]: [u16; 2]| {
        pos2(
            rect.left() - pivot_x as f32 * zoom + (x as f32 + 0.5) * tile_size[0] * zoom,
            rect.top() - pivot_y as f32 * zoom + (y as f32 + 0.5) * tile_size[1] * zoom,
        )
    };

    for j in 0..data.height() {
        for i in 0..data.width() {
            let extra_data = *data.get_cell_extra_data([i, j]);

            let tile_size = [tile_size[0] as u16, tile_size[1] as u16];

            // Если не обозначено иное, выбираем сдвиг на тайл по горизонтали и вертикали,
            // так как все тайлы для порталов и меток имеют размер 2x2
            let default_pivot = SPRITE_ATLAS_DEF.tile_size.map(|it| it as u16);

            let (portal, label, pivot) = match extra_data {
                FloorCellExtra::None => (None, None, default_pivot),
                FloorCellExtra::SpawnUnitHint(unit_danger) => {
                    (
                        Some(SPRITE_ATLAS_DEF.get_sprite_def("Портал")),
                        Some(SPRITE_ATLAS_DEF.get_sprite_def(unit_danger.display_name())),
                        default_pivot
                    )
                }
                FloorCellExtra::SpawnLootHint(item_rarity) => {
                    (
                        Some(SPRITE_ATLAS_DEF.get_sprite_def("Портал")),
                        Some(SPRITE_ATLAS_DEF.get_sprite_def(item_rarity.display_name())),
                        default_pivot
                    )
                }
                FloorCellExtra::SpawnUnit(unit_config_id) => {
                    if !asset_db.has_asset(AssetKind::UnitConfig, unit_config_id.uuid) {
                        (None, None, default_pivot)
                    } else {
                        let text = asset_db.load_json5_asset(
                            AssetKind::UnitConfig,
                            unit_config_id.uuid
                        );
                        let config: UnitConfig = json5::from_str(text)
                            .expect("Failed to deserialize UnitConfig");
                        (
                            Some(SPRITE_ATLAS_DEF.get_sprite_def(&config.sprite_name)),
                            None,
                            config.sprite_pivot.map(|it| it as u16)
                        )
                    }
                }
                FloorCellExtra::SpawnLoot(item_config_id) => {
                    if !asset_db.has_asset(AssetKind::ItemConfig, item_config_id.uuid) {
                        (None, None, default_pivot)
                    } else {
                        let text = asset_db.load_json5_asset(
                            AssetKind::ItemConfig,
                            item_config_id.uuid
                        );
                        let config: ItemConfig = json5::from_str(text)
                            .expect("Failed to deserialize UnitConfig");
                        (
                            Some(SPRITE_ATLAS_DEF.get_sprite_def(&config.sprite_name)),
                            None,
                            config.sprite_pivot.map(|it| it as u16)
                        )
                    }
                }
                FloorCellExtra::LadderDownHint => {
                    (Some(SPRITE_ATLAS_DEF.get_sprite_def("Лестница вниз")), None, default_pivot)
                }
                FloorCellExtra::LadderUpHint => {
                    (Some(SPRITE_ATLAS_DEF.get_sprite_def("Лестница вверх")), None, default_pivot)
                }
                FloorCellExtra::PlayerStartHint => {
                    (
                        Some(SPRITE_ATLAS_DEF.get_sprite_def("Портал")),
                        Some(SPRITE_ATLAS_DEF.get_sprite_def("Старт")),
                        default_pivot
                    )
                }
                FloorCellExtra::TriggerEffect(_) => {
                    // todo: придумать как показать для эффекта какой именно это эффект
                    (Some(SPRITE_ATLAS_DEF.get_sprite_def("Эффект")), None, default_pivot)
                }
            };

            if let Some(portal) = portal {
                let atlas_rect = AtlasSpriteRect::from_u16(
                    atlas_size,
                    [
                        portal.coords[0] as u16 * tile_size[0],
                        portal.coords[1] as u16 * tile_size[1],
                    ],
                    [
                        portal.size[0] as u16 * tile_size[0],
                        portal.size[1] as u16 * tile_size[1]
                    ],
                );

                let pt = point_on_rect(i, j, pivot);
                let image_rect = Rect::from_min_max(
                    pt,
                    [pt.x + atlas_rect.rect_px.width() * zoom, pt.y + atlas_rect.rect_px.height() * zoom].into(),
                );

                ui.painter().image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);
            }

            if let Some(label) = label {
                let atlas_rect = AtlasSpriteRect::from_u16(
                    atlas_size,
                    [
                        label.coords[0] as u16 * tile_size[0],
                        label.coords[1] as u16 * tile_size[1],
                    ],
                    [
                        label.size[0] as u16 * tile_size[0],
                        label.size[1] as u16 * tile_size[1]
                    ],
                );
                // надпись рисуется со сдвигом на один тайл вверх
                let mut pivot = pivot;
                pivot[1] += tile_size[1];
                let pt = point_on_rect(i, j, pivot);
                let image_rect = Rect::from_min_max(
                    pt,
                    [pt.x + atlas_rect.rect_px.width() * zoom, pt.y + atlas_rect.rect_px.height() * zoom].into(),
                );
                ui.painter().image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);
            }
        }
    }
}


fn draw_walls(
    ui: &mut Ui,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    data: &impl EditableFloorData,
    zoom: f32,
    tile_size: [f32; 2],
    rect: Rect,
) {
    let point_on_rect = |x: usize, y: usize| {
        pos2(
            rect.left() + (x as f32 + 0.5) * tile_size[0] * zoom,
            rect.top() + (y as f32 + 0.5) * tile_size[1] * zoom,
        )
    };

    let base_sandstone_coords = crate::graphics::SPRITE_ATLAS_DEF.wall_tile_groups
        [&WallGraphicsTileGroup::Sandstone]
        .base_coords;
    let base_rocks_coords = crate::graphics::SPRITE_ATLAS_DEF.wall_tile_groups
        [&WallGraphicsTileGroup::Rocks]
        .base_coords;
    let base_bricks_coords = crate::graphics::SPRITE_ATLAS_DEF.wall_tile_groups
        [&WallGraphicsTileGroup::Bricks]
        .base_coords;

    for j in 0..data.height() - 1 {
        for i in 0..data.width() - 1 {
            let pt = point_on_rect(i, j);
            let image_rect = Rect::from_min_max(
                pt,
                [pt.x + tile_size[0] * zoom, pt.y + tile_size[1] * zoom].into(),
            );

            let tile_size = [tile_size[0] as u16, tile_size[1] as u16];

            for (group, base_coords) in [
                (WallGraphicsTileGroup::Sandstone, base_sandstone_coords),
                (WallGraphicsTileGroup::Rocks, base_rocks_coords),
                (WallGraphicsTileGroup::Bricks, base_bricks_coords),
            ] {
                let mut bitmask = 0;
                if *data.get_wall_data([i, j]) == group {
                    bitmask = bitmask | WANG_MASK_NORTH_WEST;
                }
                if *data.get_wall_data([i + 1, j]) == group {
                    bitmask = bitmask | WANG_MASK_NORTH_EAST;
                }
                if *data.get_wall_data([i, j + 1]) == group {
                    bitmask = bitmask | WANG_MASK_SOUTH_WEST;
                }
                if *data.get_wall_data([i + 1, j + 1]) == group {
                    bitmask = bitmask | WANG_MASK_SOUTH_EAST;
                }
                let coords = offset_coords(base_coords, WANG_MASK_LOOKUP[bitmask]);
                let atlas_rect = sprite_rect(atlas_size, tile_size, coords);
                ui.painter()
                    .image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);
                if j == 0 {
                    let (atlas_rect, image_rect_n) = get_coords_north(
                        base_coords,
                        atlas_size,
                        tile_size.map(|it| it as f32),
                        bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect_n,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );

                    if i == 0 {
                        let (atlas_rect, image_rect_nw) = get_coords_north_west(
                            base_coords,
                            atlas_size,
                            tile_size.map(|it| it as f32),
                            bitmask,
                            image_rect,
                            zoom,
                        );
                        ui.painter().image(
                            texture_id,
                            image_rect_nw,
                            atlas_rect.uv_rect(),
                            Color32::WHITE,
                        );
                    }
                    if i == data.width() - 2 {
                        let (atlas_rect, image_rect_ne) = get_coords_north_east(
                            base_coords,
                            atlas_size,
                            tile_size.map(|it| it as f32),
                            bitmask,
                            image_rect,
                            zoom,
                        );
                        ui.painter().image(
                            texture_id,
                            image_rect_ne,
                            atlas_rect.uv_rect(),
                            Color32::WHITE,
                        );
                    }
                }
                if j == data.height() - 2 {
                    let (atlas_rect, image_rect_s) = get_coords_south(
                        base_coords,
                        atlas_size,
                        tile_size.map(|it| it as f32),
                        bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect_s,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );

                    if i == 0 {
                        let (atlas_rect, image_rect_sw) = get_coords_south_west(
                            base_coords,
                            atlas_size,
                            tile_size.map(|it| it as f32),
                            bitmask,
                            image_rect,
                            zoom,
                        );
                        ui.painter().image(
                            texture_id,
                            image_rect_sw,
                            atlas_rect.uv_rect(),
                            Color32::WHITE,
                        );
                    }
                    if i == data.width() - 2 {
                        let (atlas_rect, image_rect_se) = get_coords_south_east(
                            base_coords,
                            atlas_size,
                            tile_size.map(|it| it as f32),
                            bitmask,
                            image_rect,
                            zoom,
                        );
                        ui.painter().image(
                            texture_id,
                            image_rect_se,
                            atlas_rect.uv_rect(),
                            Color32::WHITE,
                        );
                    }
                }
                if i == 0 {
                    let (atlas_rect, image_rect) = get_coords_west(
                        base_coords,
                        atlas_size,
                        tile_size.map(|it| it as f32),
                        bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );
                }
                if i == data.width() - 2 {
                    let (atlas_rect, image_rect) = get_coords_east(
                        base_coords,
                        atlas_size,
                        tile_size.map(|it| it as f32),
                        bitmask,
                        image_rect,
                        zoom,
                    );
                    ui.painter().image(
                        texture_id,
                        image_rect,
                        atlas_rect.uv_rect(),
                        Color32::WHITE,
                    );
                }
            }
        }
    }
}

fn floor_part_button(
    ui: &mut Ui,
    selected: bool,
    editor_name: &str,
    floor_part_config: &FloorPartConfig,
    button_size: f32,
    button_padding: f32,
) -> Response {
    fn get_floor_color(group: FloorGraphicsTileGroup) -> Color32 {
        match group {
            FloorGraphicsTileGroup::Dirt => Color32::from_rgb(91, 43, 48),
            FloorGraphicsTileGroup::Tile => Color32::from_rgb(163, 145, 142),
            FloorGraphicsTileGroup::Water => Color32::from_rgb(54, 118, 167),
            FloorGraphicsTileGroup::Lava => Color32::from_rgb(217, 117, 54),
        }
    }

    fn get_wall_color(group: WallGraphicsTileGroup) -> Option<Color32> {
        match group {
            WallGraphicsTileGroup::None => None,
            WallGraphicsTileGroup::Sandstone => Some(Color32::from_rgb(63, 71, 84)),
            WallGraphicsTileGroup::Rocks => Some(Color32::BLACK),
            WallGraphicsTileGroup::Bricks => Some(Color32::from_rgb(83, 128, 128)),
        }
    }

    let (rect, response) = ui.allocate_exact_size(vec2(button_size, button_size), Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.bg_fill
        };

        let stroke = if selected {
            ui.visuals().selection.stroke
        } else {
            visuals.bg_stroke
        };

        let rounding = CornerRadius::same(2);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);

        let mut point = rect.min + vec2(button_padding, button_padding);
        let cell_size = vec2(
            (button_size - button_padding * 2f32) / 5f32,
            (button_size - button_padding * 2f32) / 5f32,
        );
        for j in 0..5 {
            for i in 0..5 {
                let floor_color = get_floor_color(floor_part_config.floor_data[j][i]);
                let wall_color = get_wall_color(floor_part_config.wall_data[j][i]);
                let sub_rect = Rect::from_min_max(point, point + cell_size);
                ui.painter()
                    .rect_filled(sub_rect, CornerRadius::ZERO, floor_color);
                if let Some(wall_color) = wall_color {
                    let sub_rect = Rect::from_min_max(
                        sub_rect.min + vec2(2f32, 2f32),
                        sub_rect.max - vec2(2f32, 2f32),
                    );
                    ui.painter()
                        .rect_filled(sub_rect, CornerRadius::same(2), wall_color);
                }
                point.x += cell_size.x;
            }
            point.x -= cell_size.x * 5f32;
            point.y += cell_size.y;
        }

        let text_x = rect.center().x;
        let text_y = rect.max.y - ui.spacing().interact_size.y * 0.5f32;

        // Контрастная обводка для текста:
        for j in -1..=1 {
            for i in -1..=1 {
                if i * j == 0 {
                    continue;
                }
                let added_vec = vec2(i as f32, j as f32);
                ui.painter().text(
                    pos2(text_x, text_y) + added_vec,
                    Align2::CENTER_CENTER,
                    editor_name,
                    TextStyle::Small.resolve(ui.style()),
                    Color32::BLACK,
                );
            }
        }

        ui.painter().text(
            pos2(text_x, text_y),
            Align2::CENTER_CENTER,
            editor_name,
            TextStyle::Small.resolve(ui.style()),
            Color32::WHITE,
        );
    }

    response
}

pub fn floor_part_id_button(
    ui: &mut Ui,
    selected: bool,
    asset_db: &AssetDb,
    floor_part_config_id: ConfigId<FloorPartConfig>,
    button_size: f32,
    button_padding: f32,
) -> (Response, Option<FloorPartConfig>) {
    if asset_db.has_asset(AssetKind::FloorPartConfig, floor_part_config_id.uuid) {
        let editor_name =
            asset_db.asset_name(AssetKind::FloorPartConfig, floor_part_config_id.uuid);
        let config_bytes =
            asset_db.load_asset(AssetKind::FloorPartConfig, floor_part_config_id.uuid);
        let floor_part_config = FloorPartConfig::load_from_slice(config_bytes)
            .expect("Failed to load floor part config");

        let response = floor_part_button(
            ui,
            selected,
            editor_name,
            &floor_part_config,
            button_size,
            button_padding,
        );

        (response, Some(floor_part_config))
    } else {
        (
            broken_uuid_button(ui, [button_size, button_size], floor_part_config_id.uuid),
            None,
        )
    }
}

pub fn broken_uuid_button(ui: &mut Ui, button_size: [f32; 2], uuid: Uuid) -> Response {
    let (rect, response) = ui.allocate_exact_size(button_size.into(), Sense::click());
    if ui.is_rect_visible(rect) {
        let rounding = CornerRadius::same(4);

        ui.painter().rect_filled(
            rect,
            rounding,
            if uuid.is_nil() {
                Color32::DARK_GRAY
            } else {
                Color32::MAGENTA
            },
        );

        ui.painter().rect_stroke(
            rect,
            rounding,
            ui.style().interact(&response).bg_stroke,
            StrokeKind::Inside,
        );

        let text = if uuid.is_nil() {
            "Нет ссылки"
        } else {
            "Битая ссылка"
        };

        let text_width = rect.width() * 0.9;

        let mut job = LayoutJob::simple(
            text.to_owned(),
            TextStyle::Small.resolve(ui.style()),
            Color32::WHITE,
            text_width,
        );

        job.halign = Align::Center;

        let galley = ui.painter().layout_job(job);

        // Текст внутри galley выравнивается таким образом, что минимум у него
        // имеет отрицательные координаты по `x`, но минимум по `y` это 0, то
        // есть по сути там лежит [-`w`/2..`w`/2, 0..`h`], по этой причине для `x`
        // приходится делать поправку на `w`/2, таким образом текст корректно
        // выравнивается
        let galley_position = rect.center() + vec2(galley.size().x / 2f32, 0f32);

        let text_rect = Rect::from_center_size(galley_position, galley.size());

        ui.painter().galley(text_rect.min, galley, Color32::WHITE);
    }
    response
}

pub fn broken_uuid_button_small(ui: &mut Ui, uuid: Uuid) -> Response {
    let text = if uuid.is_nil() {
        "Нет ссылки"
    } else {
        "Битая ссылка"
    };
    let color = if uuid.is_nil() {
        Color32::DARK_GRAY
    } else {
        Color32::MAGENTA
    };
    ui.add(Button::new(text).fill(color))
}

pub fn fpa_id_button(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    preview_size: f32,
    fpa_config_id: ConfigId<FloorPartAdjacencyConfig>,
) -> Response {
    if !asset_db.has_asset(AssetKind::FloorPartAdjacencyConfig, fpa_config_id.uuid) {
        let size = [ui.available_width(), ui.spacing().interact_size.y * 4f32];
        broken_uuid_button(ui, size, fpa_config_id.uuid)
    } else {
        let config_text =
            asset_db.load_json5_asset(AssetKind::FloorPartAdjacencyConfig, fpa_config_id.uuid);
        let config_name =
            asset_db.asset_name(AssetKind::FloorPartAdjacencyConfig, fpa_config_id.uuid);
        let config: FloorPartAdjacencyConfig =
            json5::from_str(config_text).expect("Failed to parse item config");
        fpa_button(
            ui,
            asset_db,
            selected,
            atlas_texture,
            atlas_size,
            preview_size,
            config_name,
            &config,
        )
    }
}

pub fn fpa_button(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    preview_size: f32,
    editor_name: &str,
    config: &FloorPartAdjacencyConfig,
) -> Response {
    let padding_size = ui.spacing().item_spacing.x;
    let button_width = preview_size + padding_size * 2f32;

    let (rect, response) = ui.allocate_exact_size(vec2(button_width, button_width), Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.bg_fill
        };

        let stroke = if selected {
            ui.visuals().selection.stroke
        } else {
            visuals.bg_stroke
        };

        let text_color = if selected {
            ui.visuals().selection.stroke.color
        } else {
            visuals.text_color()
        };

        let rounding = CornerRadius::same(4);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);

        let zoom_one_width = crate::graphics::SPRITE_ATLAS_DEF.tile_size[0] as f32 * 15f32;
        let zoom = preview_size / zoom_one_width;

        let rect_min = rect.min + vec2(padding_size, padding_size);
        let rect_max = rect.max - vec2(padding_size, padding_size);

        let rect = Rect::from_min_max(rect_min, rect_max);

        visualize_floor_part_adjacency_in_rect(
            ui,
            rect,
            asset_db,
            atlas_texture,
            atlas_size,
            config.part,
            NeighbourData::Multiple(&config.north_adjacent_parts),
            NeighbourData::Multiple(&config.south_adjacent_parts),
            NeighbourData::Multiple(&config.west_adjacent_parts),
            NeighbourData::Multiple(&config.east_adjacent_parts),
            zoom,
        );

        let text_pos = pos2(
            rect.center().x,
            rect.max.y - ui.spacing().interact_size.y * 0.5f32,
        );

        let text_style = match preview_size {
            x if x >= 240f32 => TextStyle::Heading.resolve(ui.style()),
            x if x >= 180f32 => TextStyle::Button.resolve(ui.style()),
            x if x >= 120f32 => TextStyle::Body.resolve(ui.style()),
            _ => TextStyle::Small.resolve(ui.style()),
        };

        for j in -1..=1 {
            for i in -1..=1 {
                if i * j == 0 {
                    continue;
                }
                let added_vec = vec2(i as f32, j as f32);
                ui.painter().text(
                    text_pos + added_vec,
                    Align2::CENTER_CENTER,
                    editor_name,
                    text_style.clone(),
                    Color32::BLACK,
                );
            }
        }

        ui.painter().text(
            text_pos,
            Align2::CENTER_CENTER,
            editor_name,
            text_style,
            text_color,
        );
    }

    response
}

pub fn item_config_id_button(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    unit_config_id: ConfigId<ItemConfig>,
) -> Response {
    if !asset_db.has_asset(AssetKind::ItemConfig, unit_config_id.uuid) {
        let size = [ui.available_width(), ui.spacing().interact_size.y * 4f32];
        broken_uuid_button(ui, size, unit_config_id.uuid)
    } else {
        let config_text = asset_db.load_json5_asset(AssetKind::ItemConfig, unit_config_id.uuid);
        let config_name = asset_db.asset_name(AssetKind::ItemConfig, unit_config_id.uuid);
        let config: ItemConfig = json5::from_str(config_text).expect("Failed to parse item config");
        item_selector_button(
            ui,
            selected,
            atlas_texture,
            atlas_size,
            config_name,
            &config,
        )
    }
}

pub fn item_selector_button(
    ui: &mut Ui,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    editor_name: &str,
    item_config: &ItemConfig,
) -> Response {
    let desired_size = vec2(ui.available_width(), ui.spacing().interact_size.y * 4f32);

    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.bg_fill
        };

        let stroke = if selected {
            ui.visuals().selection.stroke
        } else {
            visuals.bg_stroke
        };

        let text_color = if selected {
            ui.visuals().selection.stroke.color
        } else {
            visuals.text_color()
        };

        let rounding = CornerRadius::same(4);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);
        let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

        let sprite_data = SPRITE_ATLAS_DEF.get_sprite_def(&item_config.sprite_name);

        let sprite_rect =AtlasSpriteRect::from_u16(
            atlas_size,
            [
                sprite_data.coords[0] as u16 * tile_size[0] as u16,
                sprite_data.coords[1] as u16 * tile_size[1] as u16,
            ],
            [
                sprite_data.size[0] as u16 * tile_size[0] as u16,
                sprite_data.size[1] as u16 * tile_size[1] as u16,
            ],
        );

        let y_step = (rect.max.y - rect.min.y) / 3f32;
        let editor_name_y = rect.min.y + y_step / 2f32;
        let name_y = editor_name_y + y_step;
        let rarity_y = name_y + y_step;

        let top = rect.min.y + 4f32;
        let bottom = rect.max.y - 4f32;

        let h = bottom - top;
        let zoom = h / sprite_rect.size_px().y;
        let w = sprite_rect.size_px().x * zoom;

        let sp_rect = Rect::from_min_max(
            [rect.min.x + 4f32, rect.min.y + 4f32].into(),
            [rect.min.x + 4f32 + w, rect.min.y + 4f32 + h].into(),
        );

        ui.painter().image(
            atlas_texture,
            sp_rect,
            sprite_rect.uv_rect(),
            Color32::WHITE,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, editor_name_y),
            Align2::LEFT_CENTER,
            editor_name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, name_y),
            Align2::LEFT_CENTER,
            &item_config.name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, rarity_y),
            Align2::LEFT_CENTER,
            item_config.item_rarity.display_name(),
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );
    }

    response
}

pub fn unit_config_id_button(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    unit_config_id: ConfigId<UnitConfig>,
) -> Response {
    if !asset_db.has_asset(AssetKind::UnitConfig, unit_config_id.uuid) {
        let size = [ui.available_width(), ui.spacing().interact_size.y * 4f32];
        broken_uuid_button(ui, size, unit_config_id.uuid)
    } else {
        let config_text = asset_db.load_json5_asset(AssetKind::UnitConfig, unit_config_id.uuid);
        let config_name = asset_db.asset_name(AssetKind::UnitConfig, unit_config_id.uuid);
        let config: UnitConfig = json5::from_str(config_text).expect("Failed to parse unit config");
        unit_selector_button(
            ui,
            selected,
            atlas_texture,
            atlas_size,
            config_name,
            &config,
        )
    }
}

pub fn unit_selector_button(
    ui: &mut Ui,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    editor_name: &str,
    unit_config: &UnitConfig,
) -> Response {
    let desired_size = vec2(ui.available_width(), ui.spacing().interact_size.y * 4f32);

    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.bg_fill
        };

        let stroke = if selected {
            ui.visuals().selection.stroke
        } else {
            visuals.bg_stroke
        };

        let text_color = if selected {
            ui.visuals().selection.stroke.color
        } else {
            visuals.text_color()
        };

        let rounding = CornerRadius::same(4);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);
        let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

        let sprite_data = SPRITE_ATLAS_DEF.get_sprite_def(&unit_config.sprite_name);

        let sprite_rect =AtlasSpriteRect::from_u16(
            atlas_size,
            [
                sprite_data.coords[0] as u16 * tile_size[0] as u16,
                sprite_data.coords[1] as u16 * tile_size[1] as u16,
            ],
            [
                sprite_data.size[0] as u16 * tile_size[0] as u16,
                sprite_data.size[1] as u16 * tile_size[1] as u16,
            ],
        );

        let y_step = (rect.max.y - rect.min.y) / 3f32;
        let editor_name_y = rect.min.y + y_step / 2f32;
        let name_y = editor_name_y + y_step;
        let rarity_y = name_y + y_step;

        let top = rect.min.y + 4f32;
        let bottom = rect.max.y - 4f32;

        let h = bottom - top;
        let zoom = h / sprite_rect.size_px().y;
        let w = sprite_rect.size_px().x * zoom;

        let sp_rect = Rect::from_min_max(
            [rect.min.x + 4f32, rect.min.y + 4f32].into(),
            [rect.min.x + 4f32 + w, rect.min.y + 4f32 + h].into(),
        );

        ui.painter().image(
            atlas_texture,
            sp_rect,
            sprite_rect.uv_rect(),
            Color32::WHITE,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, editor_name_y),
            Align2::LEFT_CENTER,
            editor_name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, name_y),
            Align2::LEFT_CENTER,
            &unit_config.name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, rarity_y),
            Align2::LEFT_CENTER,
            unit_config.danger.display_name(),
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );
    }

    response
}

pub fn parameter_config_id_button(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    parameter_config_id: ConfigId<ParameterConfig>,
) -> Response {
    if !asset_db.has_asset(AssetKind::ParameterConfig, parameter_config_id.uuid) {
        let size = [ui.available_width(), ui.spacing().interact_size.y * 4f32];
        broken_uuid_button(ui, size, parameter_config_id.uuid)
    } else {
        let config_text = asset_db.load_json5_asset(AssetKind::ParameterConfig, parameter_config_id.uuid);
        let config_name = asset_db.asset_name(AssetKind::ParameterConfig, parameter_config_id.uuid);
        let config: ParameterConfig = json5::from_str(config_text).expect("Failed to parse parameter config");
        parameter_selector_button(
            ui,
            selected,
            atlas_texture,
            atlas_size,
            config_name,
            &config,
        )
    }
}

pub fn parameter_config_id_button_small(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    parameter_config_id: ConfigId<ParameterConfig>,
) -> Response {
    if !asset_db.has_asset(AssetKind::ParameterConfig, parameter_config_id.uuid) {
        broken_uuid_button_small(ui, parameter_config_id.uuid)
    } else {
        let config_text = asset_db.load_json5_asset(AssetKind::ParameterConfig, parameter_config_id.uuid);
        let config: ParameterConfig = json5::from_str(config_text).expect("Failed to parse parameter config");
        ui.add(Button::new(format!("{{{}}}", &config.bound_name)).selected(selected))
    }
}

pub fn tag_config_id_button_small(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    parameter_config_id: ConfigId<TagConfig>,
) -> Response {
    if !asset_db.has_asset(AssetKind::TagConfig, parameter_config_id.uuid) {
        broken_uuid_button_small(ui, parameter_config_id.uuid)
    } else {
        let config_text = asset_db.load_json5_asset(AssetKind::TagConfig, parameter_config_id.uuid);
        let config: TagConfig = json5::from_str(config_text).expect("Failed to parse parameter config");
        ui.add(Button::new(format!("[{}]", &config.bound_name)).selected(selected))
    }
}

pub fn parameter_selector_button(
    ui: &mut Ui,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    editor_name: &str,
    parameter_config: &ParameterConfig,
) -> Response {
    let desired_size = vec2(ui.available_width(), ui.spacing().interact_size.y * 4f32);

    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.bg_fill
        };

        let stroke = if selected {
            ui.visuals().selection.stroke
        } else {
            visuals.bg_stroke
        };

        let text_color = if selected {
            ui.visuals().selection.stroke.color
        } else {
            visuals.text_color()
        };

        let rounding = CornerRadius::same(4);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);
        let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

        let sprite_data = SPRITE_ATLAS_DEF.get_sprite_def(&parameter_config.sprite_name);

        let sprite_rect = AtlasSpriteRect::from_u16(
            atlas_size,
            [
                sprite_data.coords[0] as u16 * tile_size[0] as u16,
                sprite_data.coords[1] as u16 * tile_size[1] as u16,
            ],
            [
                sprite_data.size[0] as u16 * tile_size[0] as u16,
                sprite_data.size[1] as u16 * tile_size[1] as u16,
            ],
        );

        let y_step = (rect.max.y - rect.min.y) / 3f32;
        let editor_name_y = rect.min.y + y_step / 2f32;
        let name_y = editor_name_y + y_step;
        let bound_name_y = name_y + y_step;

        let top = rect.min.y + 4f32;
        let bottom = rect.max.y - 4f32;

        let h = bottom - top;
        let zoom = h / sprite_rect.size_px().y;
        let w = sprite_rect.size_px().x * zoom;

        let sp_rect = Rect::from_min_max(
            [rect.min.x + 4f32, rect.min.y + 4f32].into(),
            [rect.min.x + 4f32 + w, rect.min.y + 4f32 + h].into(),
        );

        ui.painter().image(
            atlas_texture,
            sp_rect,
            sprite_rect.uv_rect(),
            Color32::WHITE,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, editor_name_y),
            Align2::LEFT_CENTER,
            editor_name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, name_y),
            Align2::LEFT_CENTER,
            &parameter_config.name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, bound_name_y),
            Align2::LEFT_CENTER,
            &parameter_config.bound_name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );
    }

    response
}

pub fn tag_config_id_button(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    tag_config_id: ConfigId<TagConfig>,
) -> Response {
    if !asset_db.has_asset(AssetKind::TagConfig, tag_config_id.uuid) {
        let size = [ui.available_width(), ui.spacing().interact_size.y * 4f32];
        broken_uuid_button(ui, size, tag_config_id.uuid)
    } else {
        let config_text = asset_db.load_json5_asset(AssetKind::TagConfig, tag_config_id.uuid);
        let config_name = asset_db.asset_name(AssetKind::TagConfig, tag_config_id.uuid);
        let config: TagConfig = json5::from_str(config_text).expect("Failed to parse tag config");
        tag_selector_button(
            ui,
            selected,
            atlas_texture,
            atlas_size,
            config_name,
            &config,
        )
    }
}

pub fn tag_selector_button(
    ui: &mut Ui,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    editor_name: &str,
    tag_config: &TagConfig,
) -> Response {
    let desired_size = vec2(ui.available_width(), ui.spacing().interact_size.y * 4f32);

    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.bg_fill
        };

        let stroke = if selected {
            ui.visuals().selection.stroke
        } else {
            visuals.bg_stroke
        };

        let text_color = if selected {
            ui.visuals().selection.stroke.color
        } else {
            visuals.text_color()
        };

        let rounding = CornerRadius::same(4);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);
        let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

        let sprite_data = SPRITE_ATLAS_DEF.get_sprite_def(&tag_config.sprite_name);

        let sprite_rect = AtlasSpriteRect::from_u16(
            atlas_size,
            [
                sprite_data.coords[0] as u16 * tile_size[0] as u16,
                sprite_data.coords[1] as u16 * tile_size[1] as u16,
            ],
            [
                sprite_data.size[0] as u16 * tile_size[0] as u16,
                sprite_data.size[1] as u16 * tile_size[1] as u16,
            ],
        );

        let y_step = (rect.max.y - rect.min.y) / 3f32;
        let editor_name_y = rect.min.y + y_step / 2f32;
        let name_y = editor_name_y + y_step;
        let bound_name_y = name_y + y_step;

        let top = rect.min.y + 4f32;
        let bottom = rect.max.y - 4f32;

        let h = bottom - top;
        let zoom = h / sprite_rect.size_px().y;
        let w = sprite_rect.size_px().x * zoom;

        let sp_rect = Rect::from_min_max(
            [rect.min.x + 4f32, rect.min.y + 4f32].into(),
            [rect.min.x + 4f32 + w, rect.min.y + 4f32 + h].into(),
        );

        ui.painter().image(
            atlas_texture,
            sp_rect,
            sprite_rect.uv_rect(),
            Color32::WHITE,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, editor_name_y),
            Align2::LEFT_CENTER,
            editor_name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, name_y),
            Align2::LEFT_CENTER,
            &tag_config.name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, bound_name_y),
            Align2::LEFT_CENTER,
            &tag_config.bound_name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );
    }

    response
}

#[inline]
pub fn split_2_horizontal<R>(
    ui: &mut Ui,
    ratio: f32,
    add_contents: impl FnOnce(&mut [Ui; 2]) -> R,
) -> Option<R> {
    let ratio = ratio.clamp(0f32, 1f32);
    let spacing = ui.spacing().item_spacing.x;
    let columns_width = ui.available_width() - spacing;
    if columns_width <= 0f32 {
        return None;
    }

    let column_width_0 = columns_width * ratio;
    let column_width_1 = columns_width - column_width_0;
    let top_left = ui.cursor().min;

    let mut columns = [
        {
            let pos = top_left;
            let child_rect = Rect::from_min_max(
                pos,
                pos2(pos.x + column_width_0, ui.max_rect().right_bottom().y),
            );
            let mut column_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(child_rect)
                    .layout(Layout::top_down_justified(Align::LEFT)),
            );
            column_ui.set_width(column_width_0);
            column_ui
        },
        {
            let pos = pos2(top_left.x + column_width_0 + spacing, top_left.y);
            let child_rect = Rect::from_min_max(
                pos,
                pos2(pos.x + column_width_1, ui.max_rect().right_bottom().y),
            );
            let mut column_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(child_rect)
                    .layout(Layout::top_down_justified(Align::LEFT)),
            );
            column_ui.set_width(column_width_1);
            column_ui
        },
    ];
    let result = add_contents(&mut columns);
    let mut max_height = 0.0;
    for column in &columns {
        max_height = column.min_size().y.max(max_height);
    }

    let total_required_width = columns_width + spacing;

    let size = vec2(ui.available_width().max(total_required_width), max_height);
    ui.advance_cursor_after_rect(Rect::from_min_size(top_left, size));
    Some(result)
}

pub fn unit_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<UnitConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                for (uuid, _) in asset_db.list_all_assets(AssetKind::UnitConfig) {
                    let config_id = ConfigId::from_uuid(uuid);
                    ui.add_space(4f32);

                    let response = unit_config_id_button(
                        ui,
                        asset_db,
                        false,
                        atlas_texture,
                        atlas_size,
                        config_id,
                    );

                    if response.clicked() {
                        foo(config_id);
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        },
    );
}

pub fn effect_config_id_button(
    ui: &mut Ui,
    asset_db: &AssetDb,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    effect_config_id: ConfigId<EffectConfig>,
) -> Response {
    if !asset_db.has_asset(AssetKind::EffectConfig, effect_config_id.uuid) {
        let size = [ui.available_width(), ui.spacing().interact_size.y * 4f32];
        broken_uuid_button(ui, size, effect_config_id.uuid)
    } else {
        let config_text = asset_db.load_json5_asset(AssetKind::EffectConfig, effect_config_id.uuid);
        let config_name = asset_db.asset_name(AssetKind::EffectConfig, effect_config_id.uuid);
        let config: EffectConfig = json5::from_str(config_text).expect("Failed to parse effect config");
        effect_selector_button(
            ui,
            selected,
            atlas_texture,
            atlas_size,
            config_name,
            &config,
        )
    }
}

pub fn effect_selector_button(
    ui: &mut Ui,
    selected: bool,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    editor_name: &str,
    effect_config: &EffectConfig,
) -> Response {
    let desired_size = vec2(ui.available_width(), ui.spacing().interact_size.y * 4f32);

    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        let fill = if selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.bg_fill
        };

        let stroke = if selected {
            ui.visuals().selection.stroke
        } else {
            visuals.bg_stroke
        };

        let text_color = if selected {
            ui.visuals().selection.stroke.color
        } else {
            visuals.text_color()
        };

        let rounding = CornerRadius::same(4);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);
        let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

        let sprite_data = SPRITE_ATLAS_DEF.get_sprite_def(effect_config.sprite_name());

        let sprite_rect = AtlasSpriteRect::from_u16(
            atlas_size,
            [
                sprite_data.coords[0] as u16 * tile_size[0] as u16,
                sprite_data.coords[1] as u16 * tile_size[1] as u16,
            ],
            [
                sprite_data.size[0] as u16 * tile_size[0] as u16,
                sprite_data.size[1] as u16 * tile_size[1] as u16,
            ],
        );

        let y_step = (rect.max.y - rect.min.y) / 3f32;
        let editor_name_y = rect.min.y + y_step / 2f32;
        let description_y = editor_name_y + y_step;

        let top = rect.min.y + 4f32;
        let bottom = rect.max.y - 4f32;

        let h = bottom - top;
        let zoom = h / sprite_rect.size_px().y;
        let w = sprite_rect.size_px().x * zoom;

        let sp_rect = Rect::from_min_max(
            [rect.min.x + 4f32, rect.min.y + 4f32].into(),
            [rect.min.x + 4f32 + w, rect.min.y + 4f32 + h].into(),
        );

        ui.painter().image(
            atlas_texture,
            sp_rect,
            sprite_rect.uv_rect(),
            Color32::WHITE,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, editor_name_y),
            Align2::LEFT_CENTER,
            editor_name,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );

        ui.painter().text(
            pos2(rect.min.x + w + 8f32, description_y),
            Align2::LEFT_CENTER,
            &effect_config.description,
            TextStyle::Button.resolve(ui.style()),
            text_color,
        );
    }

    response
}

pub fn effect_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<EffectConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                for (uuid, _) in asset_db.list_all_assets(AssetKind::EffectConfig) {
                    let config_id = ConfigId::from_uuid(uuid);
                    ui.add_space(4f32);

                    let response = effect_config_id_button(
                        ui,
                        asset_db,
                        false,
                        atlas_texture,
                        atlas_size,
                        config_id,
                    );

                    if response.clicked() {
                        foo(config_id);
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        },
    );
}

pub fn item_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<ItemConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                for (uuid, _) in asset_db.list_all_assets(AssetKind::ItemConfig) {
                    let config_id = ConfigId::from_uuid(uuid);
                    ui.add_space(4f32);

                    let response = item_config_id_button(
                        ui,
                        asset_db,
                        false,
                        atlas_texture,
                        atlas_size,
                        config_id,
                    );

                    if response.clicked() {
                        foo(config_id);
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        },
    );
}

pub fn parameter_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<ParameterConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                for (uuid, _) in asset_db.list_all_assets(AssetKind::ParameterConfig) {
                    let config_id = ConfigId::from_uuid(uuid);
                    ui.add_space(4f32);

                    let response = parameter_config_id_button(
                        ui,
                        asset_db,
                        false,
                        atlas_texture,
                        atlas_size,
                        config_id,
                    );

                    if response.clicked() {
                        foo(config_id);
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        },
    );
}

pub fn tag_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<TagConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                for (uuid, _) in asset_db.list_all_assets(AssetKind::TagConfig) {
                    let config_id = ConfigId::from_uuid(uuid);
                    ui.add_space(4f32);

                    let response = tag_config_id_button(
                        ui,
                        asset_db,
                        false,
                        atlas_texture,
                        atlas_size,
                        config_id,
                    );

                    if response.clicked() {
                        foo(config_id);
                        ui.memory_mut(|mem| mem.close_popup());
                    }
                }
            });
        },
    );
}

pub fn fpa_selector_popup(
    ui: &mut Ui,
    asset_db: &AssetDb,
    popup_id: Id,
    response: &Response,
    atlas_texture: TextureId,
    atlas_size: [u16; 2],
    mut foo: impl FnMut(ConfigId<FloorPartAdjacencyConfig>) -> ()
) {
    egui::popup_below_widget(
        ui,
        popup_id,
        response,
        PopupCloseBehavior::IgnoreClicks,
        |ui| {
            ui.set_min_width(300f32);
            ui.label("Для отмены выбора нажмите ESC");
            ui.vertical(|ui|{
                const NUM_COLUMNS: usize = 4;
                let mut offset = 0;
                ui.columns(NUM_COLUMNS, |uis| {
                    for (uuid, _) in asset_db.list_all_assets(AssetKind::FloorPartAdjacencyConfig) {
                        let ui = &mut uis[offset];
                        offset = (offset + 1) % NUM_COLUMNS;
                        let config_id = ConfigId::from_uuid(uuid);
                        ui.add_space(4f32);

                        let response = fpa_id_button(
                            ui,
                            asset_db,
                            false,
                            atlas_texture,
                            atlas_size,
                            60f32,
                            config_id,
                        );

                        if response.clicked() {
                            foo(config_id);
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                    }
                });
            });
        },
    );
}
