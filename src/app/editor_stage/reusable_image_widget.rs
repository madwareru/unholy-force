use egui::StrokeKind;

#[derive(Clone, Copy, Debug)]
pub struct AtlasSpriteRect {
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

pub fn pivot_editor(
    ui: &mut egui::Ui,
    texture_id: egui::TextureId,
    sprite: AtlasSpriteRect,
    pivot: &mut [u8; 2],
    zoom: f32,
) -> egui::Response {
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
            rect.left() + (pivot[0] as f32 + 0.5) * zoom,
            rect.top() + (pivot[1] as f32 + 0.5) * zoom,
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

                pivot[0] = local_x.clamp(0, max_x);
                pivot[1] = local_y.clamp(0, max_y);
            }
        }
    }

    response
}