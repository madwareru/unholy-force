use egui::{Align, Align2, Button, StrokeKind, Ui};
use serde::Deserialize;
use uuid::Uuid;
use crate::app::app_stage::AppStageStatus;
use crate::app::editor_stage::floor_part_editor::FloorPartConfigEditorSection;
use crate::app::editor_stage::item_config_editor::ItemConfigEditorSection;
use crate::assets::AssetKind;
pub mod item_config_editor;
pub mod floor_part_editor;
pub mod image_widgets;

#[derive(Copy, Clone, Deserialize)]
pub enum EditorCommand {
    BackToMainMenu
}

#[derive(Copy, Clone, PartialEq)]
enum UpdateState {
    Unchanged,
    Changed,
}

pub struct EditorStage {
    atlas_texture: Option<egui::TextureHandle>,
    atlas_size: [u16; 2],
    current_file_kind: AssetKind,
    item_section: ItemConfigEditorSection,
    floor_part_section: FloorPartConfigEditorSection,
    selected_unit_config: Option<Uuid>,
    selected_fpa_config: Option<Uuid>,
    selected_floor_config: Option<Uuid>,
    selected_floor_graph_config: Option<Uuid>
}

impl EditorStage {
    pub fn new() -> Self {

        Self {
            atlas_texture: None,
            atlas_size: [0; 2],
            current_file_kind: AssetKind::UnitConfig,
            item_section: Default::default(),
            floor_part_section: Default::default(),
            selected_unit_config: None,
            selected_fpa_config: None,
            selected_floor_config: None,
            selected_floor_graph_config: None
        }
    }

    pub fn process(&mut self) -> AppStageStatus<EditorCommand> {
        let mut result_status = AppStageStatus::Continue;

        egui_macroquad::ui(|egui_ctx| {
            if self.atlas_texture.is_none() {
                let atlas_image = crate::graphics::SPRITE_ATLAS_TEXTURE.get_texture_data();

                let size = [atlas_image.width as usize, atlas_image.height as usize];

                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    size,
                    &atlas_image.bytes,
                );

                let atlas = egui_ctx.load_texture(
                    "sprite_atlas",
                    color_image,
                    egui::TextureOptions::NEAREST,
                );

                self.atlas_texture = Some(atlas);
                self.atlas_size = [atlas_image.width, atlas_image.height];
            }

            let screen_width = macroquad::prelude::screen_width();
            let preferred_central_width = match self.current_file_kind {
                AssetKind::ItemConfig => 600f32,
                AssetKind::UnitConfig => 450f32,
                AssetKind::FloorPart => 670f32,
                AssetKind::FloorPartAdjacency => 450f32,
                AssetKind::FloorConfig => 450f32,
                AssetKind::FloorFlowGraphConfig => 450f32,
            };

            let right_panel_width = screen_width - 300f32 - preferred_central_width;

            egui::SidePanel::left("Режим редактора и кнопки сохранения/выхода")
                .exact_width(300f32)
                .show(egui_ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(6f32);
                        ui.group(|ui| {
                            ui.columns(2, |ui| {
                                ui[0].vertical(|ui| {
                                    if thick_selector_button(
                                        ui,
                                        self.current_file_kind == AssetKind::UnitConfig,
                                        Align2::CENTER_CENTER,
                                        "Персонажи"
                                    ).clicked() {
                                        self.current_file_kind = AssetKind::UnitConfig;
                                    };
                                    ui.add_space(4f32);
                                    if thick_selector_button(
                                        ui,
                                        self.current_file_kind == AssetKind::FloorPart,
                                        Align2::CENTER_CENTER,
                                        "Части уровней"
                                    ).clicked() {
                                        self.current_file_kind = AssetKind::FloorPart;
                                    };
                                    ui.add_space(4f32);
                                    if thick_selector_button(
                                        ui,
                                        self.current_file_kind == AssetKind::FloorConfig,
                                        Align2::CENTER_CENTER,
                                        "Этажи"
                                    ).clicked() {
                                        self.current_file_kind = AssetKind::FloorConfig;
                                    };
                                });
                                ui[1].vertical(|ui| {
                                    if thick_selector_button(
                                        ui,
                                        self.current_file_kind == AssetKind::ItemConfig,
                                        Align2::CENTER_CENTER,
                                        "Предметы"
                                    ).clicked() {
                                        self.current_file_kind = AssetKind::ItemConfig;
                                    };
                                    ui.add_space(4f32);
                                    if thick_selector_button(
                                        ui,
                                        self.current_file_kind == AssetKind::FloorPartAdjacency,
                                        Align2::CENTER_CENTER,
                                        "Связи частей"
                                    ).clicked() {
                                        self.current_file_kind = AssetKind::FloorPartAdjacency;
                                    };
                                    ui.add_space(4f32);
                                    if thick_selector_button(
                                        ui,
                                        self.current_file_kind == AssetKind::FloorFlowGraphConfig,
                                        Align2::CENTER_CENTER,
                                        "Граф этажей"
                                    ).clicked() {
                                        self.current_file_kind = AssetKind::FloorFlowGraphConfig;
                                    };
                                });
                            });
                        });

                        ui.group(|ui| {
                            match self.current_file_kind {
                                AssetKind::ItemConfig => self.draw_item_selector(ui),
                                AssetKind::UnitConfig => self.draw_unit_selector(ui),
                                AssetKind::FloorPart => self.draw_floor_part_selector(ui),
                                AssetKind::FloorPartAdjacency => self.draw_fpa_selector(ui),
                                AssetKind::FloorConfig => self.draw_floor_selector(ui),
                                AssetKind::FloorFlowGraphConfig => self.draw_floor_graph_selector(ui),
                            }
                        });

                        ui.group(|ui| {
                            let full_width = ui.available_width();
                            if ui.add_sized(
                                [full_width, ui.spacing().interact_size.y],
                                Button::new("Сохранить изменения на диске")
                            ).clicked() {
                                match crate::assets::ASSET_DATABASE.lock() {
                                    Ok(mut asset_db) => {
                                        asset_db.flush_assets_to_disk();
                                    },
                                    _ => {}
                                };
                            }
                            if ui.add_sized(
                                [full_width, ui.spacing().interact_size.y],
                                Button::new("Вернуться в главное меню")
                            ).clicked() {
                                result_status = AppStageStatus::Complete(EditorCommand::BackToMainMenu);
                            }
                        });
                    });
                });

            egui::SidePanel::right("Дополнительная информация")
                .exact_width(right_panel_width)
                .show(egui_ctx, |ui| {
                    match self.current_file_kind {
                        AssetKind::ItemConfig => self.draw_item_preview_in_level(ui),
                        AssetKind::FloorPart => self.draw_floor_part_editor_tools(ui),
                        _ => {} // todo
                    }
                });

            egui::CentralPanel::default()
                .show(egui_ctx, |ui| {
                match self.current_file_kind {
                    AssetKind::ItemConfig => self.draw_item_editor(ui),
                    AssetKind::FloorPart => self.draw_floor_part_editor(ui),
                    _ => {} // todo
                }
            });
        });
        result_status
    }

    pub fn render(&self) {
        // todo: render here something relevant to editor logic
    }

    fn draw_unit_selector(&mut self, _ui: &mut Ui) {
        // todo
    }
    fn draw_fpa_selector(&mut self, _ui: &mut Ui) {
        // todo
    }
    fn draw_floor_selector(&mut self, _ui: &mut Ui) {
        // todo
    }
    fn draw_floor_graph_selector(&mut self, _ui: &mut Ui) {
        // todo
    }
}

fn thick_selector_button(
    ui: &mut Ui,
    selected: bool,
    align: Align2,
    text: &str,
) -> egui::Response {
    let desired_size = egui::vec2(
        ui.available_width(),
        ui.spacing().interact_size.y * 2f32,
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

        ui.painter().text(
            egui::pos2(
                match align {
                    Align2([Align::Min, _]) => rect.min.x + 8f32,
                    Align2([Align::Center, _]) => rect.center().x,
                    Align2([Align::Max, _]) => rect.max.x - 8f32,
                },
                rect.center().y
            ),
            align,
            text,
            egui::TextStyle::Button.resolve(ui.style()),
            text_color,
        );
    }

    response
}