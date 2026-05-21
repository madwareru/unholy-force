use egui::{Align2, StrokeKind, Ui};
use crate::game_config::items::ItemConfig;
use crate::graphics::{SPRITE_ATLAS_DEF};

#[derive(Clone, Copy, Debug)]
struct AtlasSpriteRect {
    /// Размер всего атласа в пикселях.
    pub atlas_size: egui::Vec2,

    /// Прямоугольник спрайта внутри атласа, в пикселях.
    pub rect_px: egui::Rect,
}

impl AtlasSpriteRect {
    pub fn from_u16(
        atlas_size: [u16; 2],
        [x, y]: [u16; 2],
        [w, h]: [u16; 2]
    ) -> Self {
        AtlasSpriteRect {
            atlas_size: atlas_size.map(|it| it as f32).into(),
            rect_px: egui::Rect::from_min_max(
                [x as f32, y as f32].into(),
                [(x + w) as f32, (y + h) as f32].into(),
            ),
        }
    }

    pub fn size_px(&self) -> egui::Vec2 {
        self.rect_px.size()
    }

    pub fn uv_rect(&self) -> egui::Rect {
        egui::Rect::from_min_max(
            egui::pos2(
                self.rect_px.min.x / self.atlas_size.x,
                self.rect_px.min.y / self.atlas_size.y,
            ),
            egui::pos2(
                self.rect_px.max.x / self.atlas_size.x,
                self.rect_px.max.y / self.atlas_size.y,
            ),
        )
    }
}

pub fn atlas_sprite_button(
    ui: &mut Ui,
    atlas_texture: egui::TextureId,
    atlas_size: [u16; 2],
    sprite_name: &str,
    size: f32,
) -> egui::Response {
    let sprite_data = crate::graphics::SPRITE_ATLAS_DEF.sprites[sprite_name];
    let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

    let atlas_rect = AtlasSpriteRect::from_u16(
        atlas_size,
        [
            sprite_data.coords[0] as u16 * tile_size[0] as u16,
            sprite_data.coords[1] as u16 * tile_size[1] as u16
        ],
        [
            sprite_data.size[0] as u16 * tile_size[0] as u16,
            sprite_data.size[1] as u16 * tile_size[1] as u16
        ]
    );

    let sprite_size_px = atlas_rect.size_px();

    let button_size = egui::vec2(size, size);

    let image_size = if sprite_size_px.x >= sprite_size_px.y {
        let aspect = sprite_size_px.x / sprite_size_px.y;
        egui::vec2(
            button_size.x,
            button_size.y / aspect,
        )
    } else {
        let aspect = sprite_size_px.y / sprite_size_px.x;
        egui::vec2(
            button_size.x / aspect,
            button_size.y,
        )
    };
    let offset_x = (button_size.x - image_size.x) / 2f32;
    let offset_y = (button_size.y - image_size.y) / 2f32;

    let (rect, response) = ui.allocate_exact_size(
        button_size,
        egui::Sense::click(),
    );

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(4),
            ui.style().interact(&response).bg_fill
        );
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(4),
            ui.style().interact(&response).bg_stroke,
            StrokeKind::Inside
        );

        let image_rect = egui::Rect::from_min_max(
            [rect.min.x + offset_x, rect.min.y + offset_y].into(),
            [rect.min.x + offset_x + image_size.x, rect.min.y + offset_y + image_size.y].into(),
        );

        ui.painter().image(
            atlas_texture,
            image_rect,
            atlas_rect.uv_rect(),
            egui::Color32::WHITE,
        );

        let text_color = ui.style().interact(&response).text_color();
        let text_x = rect.center().x;
        let text_y = rect.max.y - ui.spacing().interact_size.y * 0.5f32;

        ui.painter().text(
            egui::pos2(text_x, text_y),
            Align2::CENTER_CENTER,
            sprite_name,
            egui::TextStyle::Small.resolve(ui.style()),
            text_color,
        );
    }

    response
}

pub fn pivot_editor(
    ui: &mut Ui,
    texture_id: egui::TextureId,
    atlas_size: [u16; 2],
    item_config: &mut ItemConfig,
    zoom: f32,
) -> egui::Response {
    let sprite_data = crate::graphics::SPRITE_ATLAS_DEF.sprites[&item_config.sprite_name];
    let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

    let sprite = AtlasSpriteRect::from_u16(
        atlas_size,
        [
            sprite_data.coords[0] as u16 * tile_size[0] as u16,
            sprite_data.coords[1] as u16 * tile_size[1] as u16
        ],
        [
            sprite_data.size[0] as u16 * tile_size[0] as u16,
            sprite_data.size[1] as u16 * tile_size[1] as u16
        ]
    );

    let sprite_size_px = sprite.size_px();

    let display_size = egui::vec2(
        sprite_size_px.x * zoom,
        sprite_size_px.y * zoom,
    );

    let (rect, response) = ui.allocate_exact_size(
        display_size,
        egui::Sense::click_and_drag(),
    );

    if ui.is_rect_visible(rect) {
        ui.painter().image(
            texture_id,
            rect,
            sprite.uv_rect(),
            egui::Color32::WHITE,
        );

        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::ZERO,
            egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
            StrokeKind::Inside
        );

        let pivot_screen_pos = egui::pos2(
            rect.left() + (item_config.sprite_pivot[0] as f32 + 0.5) * zoom,
            rect.top() + (item_config.sprite_pivot[1] as f32 + 0.5) * zoom,
        );

        ui.painter().circle_filled(
            pivot_screen_pos,
            4f32,
            egui::Color32::RED,
        );

        ui.painter().circle_stroke(
            pivot_screen_pos,
            5f32,
            egui::Stroke::new(3f32, egui::Color32::WHITE),
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

pub fn item_selector_button(
    ui: &mut Ui,
    selected: bool,
    atlas_texture: egui::TextureId,
    atlas_size: [u16; 2],
    editor_name: &str,
    item_config: &ItemConfig
) -> egui::Response {
    let desired_size = egui::vec2(
        ui.available_width(),
        ui.spacing().interact_size.y * 4f32,
    );

    let (rect, response) = ui.allocate_exact_size(
        desired_size,
        egui::Sense::click(),
    );

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

        let rounding = egui::CornerRadius::same(4);

        ui.painter().rect_filled(rect, rounding, fill);
        ui.painter().rect_stroke(rect, rounding, stroke, StrokeKind::Inside);
        let tile_size = crate::graphics::SPRITE_ATLAS_DEF.tile_size;

        let sprite_rect = SPRITE_ATLAS_DEF
            .sprites.get(&item_config.sprite_name)
            .map(|sprite_data| {
                AtlasSpriteRect::from_u16(
                    atlas_size,
                    [
                        sprite_data.coords[0] as u16 * tile_size[0] as u16,
                        sprite_data.coords[1] as u16 * tile_size[1] as u16
                    ],
                    [
                        sprite_data.size[0] as u16 * tile_size[0] as u16,
                        sprite_data.size[1] as u16 * tile_size[1] as u16
                    ]
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

            let sp_rect = egui::Rect::from_min_max(
                [rect.min.x + 4f32, rect.min.y + 4f32].into(),
                [rect.min.x + 4f32 + w, rect.min.y + 4f32 + h].into(),
            );

            ui.painter().image(
                atlas_texture,
                sp_rect,
                sprite_rect.uv_rect(),
                egui::Color32::WHITE,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + w + 8f32,
                    editor_name_y
                ),
                Align2::LEFT_CENTER,
                editor_name,
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + w + 8f32,
                    name_y
                ),
                Align2::LEFT_CENTER,
                &item_config.name,
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + w + 8f32,
                    rarity_y
                ),
                Align2::LEFT_CENTER,
                item_config.item_rarity.display_name(),
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );
        } else {
            ui.painter().text(
                egui::pos2(
                    rect.min.x + 8f32,
                    editor_name_y
                ),
                Align2::LEFT_CENTER,
                editor_name,
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + 8f32,
                    name_y
                ),
                Align2::LEFT_CENTER,
                &item_config.name,
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );

            ui.painter().text(
                egui::pos2(
                    rect.min.x + 8f32,
                    rarity_y
                ),
                Align2::LEFT_CENTER,
                item_config.item_rarity.display_name(),
                egui::TextStyle::Button.resolve(ui.style()),
                text_color,
            );
        }
    }

    response
}