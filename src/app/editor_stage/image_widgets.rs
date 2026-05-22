use crate::game_config::floor_parts::FloorPartConfig;
use crate::game_config::items::ItemConfig;
use crate::graphics::{
    FloorGraphicsTileGroup, SPRITE_ATLAS_DEF, WANG_MASK_LOOKUP, WANG_MASK_NORTH_EAST,
    WANG_MASK_NORTH_WEST, WANG_MASK_SOUTH_EAST, WANG_MASK_SOUTH_WEST, WallGraphicsTileGroup,
};
use egui::{Align2, Color32, Rect, Response, StrokeKind, TextureId, Ui, Vec2, vec2, Sense, CornerRadius, Stroke, pos2, TextStyle};

#[derive(Clone, Copy, Debug)]
struct AtlasSpriteRect {
    /// Размер всего атласа в пикселях.
    pub atlas_size: Vec2,

    /// Прямоугольник спрайта внутри атласа, в пикселях.
    pub rect_px: Rect,
}

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
    let sprite_data = crate::graphics::SPRITE_ATLAS_DEF.sprites[sprite_name];
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
pub fn pivot_editor(
    ui: &mut Ui,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    item_config: &mut ItemConfig,
    zoom: f32,
) -> Response {
    let sprite_data = crate::graphics::SPRITE_ATLAS_DEF.sprites[&item_config.sprite_name];
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
            rect.left() + (item_config.sprite_pivot[0] as f32 + 0.5) * zoom,
            rect.top() + (item_config.sprite_pivot[1] as f32 + 0.5) * zoom,
        );

        ui.painter()
            .circle_filled(pivot_screen_pos, 4f32, Color32::RED);

        ui.painter().circle_stroke(
            pivot_screen_pos,
            5f32,
            Stroke::new(3f32, Color32::WHITE),
        );
    }

    if response.clicked() || response.dragged() {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if rect.contains(pointer_pos) {
                let local_x = ((pointer_pos.x - rect.left()) / zoom).floor() as u8;
                let local_y = ((pointer_pos.y - rect.top()) / zoom).floor() as u8;

                let max_x = sprite_size_px.x as u8 - 1;
                let max_y = sprite_size_px.y as u8 - 1;

                item_config.sprite_pivot[0] = local_x.clamp(0, max_x);
                item_config.sprite_pivot[1] = local_y.clamp(0, max_y);
            }
        }
    }

    response
}

const DIRT_MASK: [[bool; 5]; 5] = [
    [true, false, true, false, true],
    [true, true, false, false, true],
    [false, true, true, true, false],
    [true, false, false, true, true],
    [false, false, true, true, true],
];

pub fn floor_part_editor(
    ui: &mut Ui,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    floor_part_config: &mut FloorPartConfig,
    zoom: u8,
) -> Option<[usize; 2]> {
    let zoom = zoom.clamp(1, 8) as f32;

    let tile_size = crate::graphics::SPRITE_ATLAS_DEF
        .tile_size
        .map(|it| it as f32);
    let display_size = vec2(
        tile_size[0] * zoom * floor_part_config.floor_data[0].len() as f32,
        tile_size[1] * zoom * floor_part_config.floor_data.len() as f32,
    );

    let (rect, response) = ui.allocate_exact_size(display_size, Sense::click_and_drag());

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
            floor_part_config,
            zoom,
            tile_size,
            rect,
        );
        draw_walls(
            ui,
            texture_id,
            atlas_size,
            floor_part_config,
            zoom,
            tile_size,
            rect,
        );
    }

    let mut result = None;

    if response.clicked() || response.dragged() {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if rect.contains(pointer_pos) {
                let local_x =
                    ((pointer_pos.x - rect.left()) / zoom / tile_size[0]).floor() as usize;
                let local_y = ((pointer_pos.y - rect.top()) / zoom / tile_size[1]).floor() as usize;

                result = Some([local_x, local_y]);
            }
        }
    }

    result
}

fn draw_floors(
    ui: &mut Ui,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    floor_part_config: &FloorPartConfig,
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

    for j in 0..floor_part_config.floor_data.len() - 1 {
        for i in 0..floor_part_config.floor_data[j].len() - 1 {
            let pt = point_on_rect(i, j);
            let image_rect = Rect::from_min_max(
                pt,
                [pt.x + tile_size[0] * zoom, pt.y + tile_size[1] * zoom].into(),
            );

            let mut dirt_bitmask = 0;
            if DIRT_MASK[j][i] {
                dirt_bitmask = dirt_bitmask | WANG_MASK_NORTH_WEST;
            }
            if DIRT_MASK[j][i + 1] {
                dirt_bitmask = dirt_bitmask | WANG_MASK_NORTH_EAST;
            }
            if DIRT_MASK[j + 1][i] {
                dirt_bitmask = dirt_bitmask | WANG_MASK_SOUTH_WEST;
            }
            if DIRT_MASK[j + 1][i + 1] {
                dirt_bitmask = dirt_bitmask | WANG_MASK_SOUTH_EAST;
            }
            let offset = WANG_MASK_LOOKUP[dirt_bitmask];
            let dirt_coords = [
                base_dirt_coords[0] + offset[0],
                base_dirt_coords[1] + offset[1],
            ];
            let atlas_rect = AtlasSpriteRect::from_u16(
                atlas_size,
                [
                    dirt_coords[0] as u16 * tile_size[0] as u16,
                    dirt_coords[1] as u16 * tile_size[1] as u16,
                ],
                [tile_size[0] as u16, tile_size[1] as u16],
            );
            ui.painter()
                .image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);

            for (group, base_coords) in [
                (FloorGraphicsTileGroup::Lava, base_lava_coords),
                (FloorGraphicsTileGroup::Water, base_water_coords),
                (FloorGraphicsTileGroup::Tile, base_tile_coords),
            ] {
                let mut bitmask = 0;
                if floor_part_config.floor_data[j][i] == group {
                    bitmask = bitmask | WANG_MASK_NORTH_WEST;
                }
                if floor_part_config.floor_data[j][i + 1] == group {
                    bitmask = bitmask | WANG_MASK_NORTH_EAST;
                }
                if floor_part_config.floor_data[j + 1][i] == group {
                    bitmask = bitmask | WANG_MASK_SOUTH_WEST;
                }
                if floor_part_config.floor_data[j + 1][i + 1] == group {
                    bitmask = bitmask | WANG_MASK_SOUTH_EAST;
                }
                let offset = WANG_MASK_LOOKUP[bitmask];
                let coords = [base_coords[0] + offset[0], base_coords[1] + offset[1]];
                let atlas_rect = AtlasSpriteRect::from_u16(
                    atlas_size,
                    [
                        coords[0] as u16 * tile_size[0] as u16,
                        coords[1] as u16 * tile_size[1] as u16,
                    ],
                    [tile_size[0] as u16, tile_size[1] as u16],
                );
                ui.painter()
                    .image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);
            }
        }
    }
}

fn draw_walls(
    ui: &mut Ui,
    texture_id: TextureId,
    atlas_size: [u16; 2],
    floor_part_config: &FloorPartConfig,
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

    for j in 0..floor_part_config.floor_data.len() - 1 {
        for i in 0..floor_part_config.floor_data[j].len() - 1 {
            let pt = point_on_rect(i, j);
            let image_rect = Rect::from_min_max(
                pt,
                [pt.x + tile_size[0] * zoom, pt.y + tile_size[1] * zoom].into(),
            );

            for (group, base_coords) in [
                (WallGraphicsTileGroup::Sandstone, base_sandstone_coords),
                (WallGraphicsTileGroup::Rocks, base_rocks_coords),
                (WallGraphicsTileGroup::Bricks, base_bricks_coords),
            ] {
                let mut bitmask = 0;
                if floor_part_config.wall_data[j][i] == group {
                    bitmask = bitmask | WANG_MASK_NORTH_WEST;
                }
                if floor_part_config.wall_data[j][i + 1] == group {
                    bitmask = bitmask | WANG_MASK_NORTH_EAST;
                }
                if floor_part_config.wall_data[j + 1][i] == group {
                    bitmask = bitmask | WANG_MASK_SOUTH_WEST;
                }
                if floor_part_config.wall_data[j + 1][i + 1] == group {
                    bitmask = bitmask | WANG_MASK_SOUTH_EAST;
                }
                let offset = WANG_MASK_LOOKUP[bitmask];
                let coords = [base_coords[0] + offset[0], base_coords[1] + offset[1]];
                let atlas_rect = AtlasSpriteRect::from_u16(
                    atlas_size,
                    [
                        coords[0] as u16 * tile_size[0] as u16,
                        coords[1] as u16 * tile_size[1] as u16,
                    ],
                    [tile_size[0] as u16, tile_size[1] as u16],
                );
                ui.painter()
                    .image(texture_id, image_rect, atlas_rect.uv_rect(), Color32::WHITE);
            }
        }
    }
}

pub fn floor_part_selector_button(
    ui: &mut Ui,
    selected: bool,
    editor_name: &str,
    floor_part_config: &FloorPartConfig,
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

    const BUTTON_SIZE: f32 = 85f32;
    const BUTTON_PADDING: f32 = 5f32;

    let button_size = vec2(BUTTON_SIZE, BUTTON_SIZE);

    let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());

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

        let rounding = CornerRadius::same(2);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter()
            .rect_stroke(rect, rounding, stroke, StrokeKind::Inside);

        let mut point = rect.min + vec2(BUTTON_PADDING, BUTTON_PADDING);
        let cell_size = vec2(
            (BUTTON_SIZE - BUTTON_PADDING * 2f32) / 5f32,
            (BUTTON_SIZE - BUTTON_PADDING * 2f32) / 5f32,
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

        ui.painter().text(
            pos2(text_x, text_y),
            Align2::CENTER_CENTER,
            editor_name,
            TextStyle::Small.resolve(ui.style()),
            text_color,
        );
    }

    response
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

        let sprite_rect =
            SPRITE_ATLAS_DEF
                .sprites
                .get(&item_config.sprite_name)
                .map(|sprite_data| {
                    AtlasSpriteRect::from_u16(
                        atlas_size,
                        [
                            sprite_data.coords[0] as u16 * tile_size[0] as u16,
                            sprite_data.coords[1] as u16 * tile_size[1] as u16,
                        ],
                        [
                            sprite_data.size[0] as u16 * tile_size[0] as u16,
                            sprite_data.size[1] as u16 * tile_size[1] as u16,
                        ],
                    )
                });

        let y_step = (rect.max.y - rect.min.y) / 3f32;
        let editor_name_y = rect.min.y + y_step / 2f32;
        let name_y = editor_name_y + y_step;
        let rarity_y = name_y + y_step;

        if let Some(sprite_rect) = sprite_rect {
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
        } else {
            ui.painter().text(
                pos2(rect.min.x + 8f32, editor_name_y),
                Align2::LEFT_CENTER,
                editor_name,
                TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                pos2(rect.min.x + 8f32, name_y),
                Align2::LEFT_CENTER,
                &item_config.name,
                TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                pos2(rect.min.x + 8f32, rarity_y),
                Align2::LEFT_CENTER,
                item_config.item_rarity.display_name(),
                TextStyle::Button.resolve(ui.style()),
                text_color,
            );
        }
    }

    response
}
