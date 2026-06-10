use egui::{Align, Align2, Button, StrokeKind, Ui};
use serde::Deserialize;
use crate::{
    app::{
        app_stage::AppStageStatus,
        editor_stage::{
            effect_config_editor::EffectConfigEditorSection,
            floor_part_editor::FloorPartConfigEditorSection,
            item_config_editor::ItemConfigEditorSection,
            parameter_config_editor::ParameterConfigEditorSection,
            tag_config_editor::TagConfigEditorSection,
            unit_config_editor::UnitConfigEditorSection
        }
    },
    assets::AssetKind
};
use crate::app::editor_stage::floor_config_editor::FloorConfigEditorSection;
use crate::app::editor_stage::floor_flow_graph_config_editor::FloorFlowGraphEditorSection;
use crate::app::editor_stage::floor_part_adjacency_config_editor::FloorPartAdjacencyConfigEditorSection;

pub mod unit_config_editor;
pub mod item_config_editor;
pub mod floor_part_editor;
pub mod floor_part_adjacency_config_editor;
pub mod floor_config_editor;
pub mod floor_flow_graph_config_editor;
pub mod tag_config_editor;
pub mod parameter_config_editor;
pub mod effect_config_editor;
pub mod widgets;
pub mod text_completion;
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
    unit_section: UnitConfigEditorSection,
    item_section: ItemConfigEditorSection,
    floor_part_section: FloorPartConfigEditorSection,
    floor_part_adjacency_section: FloorPartAdjacencyConfigEditorSection,
    floor_section: FloorConfigEditorSection,
    floor_flow_graph_section: FloorFlowGraphEditorSection,
    tag_section: TagConfigEditorSection,
    parameter_section: ParameterConfigEditorSection,
    effect_section: EffectConfigEditorSection,
}

impl EditorStage {
    pub fn new() -> Self {

        Self {
            atlas_texture: None,
            atlas_size: [0; 2],
            current_file_kind: AssetKind::UnitConfig,
            unit_section: Default::default(),
            item_section: Default::default(),
            floor_part_section: Default::default(),
            floor_part_adjacency_section: Default::default(),
            floor_section: Default::default(),
            floor_flow_graph_section: Default::default(),
            tag_section: Default::default(),
            parameter_section: Default::default(),
            effect_section: Default::default(),
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
                AssetKind::ItemConfig => 510f32,
                AssetKind::UnitConfig => 510f32,
                AssetKind::FloorPartConfig => 670f32,
                AssetKind::FloorPartAdjacencyConfig => 450f32,
                AssetKind::FloorConfig => 670f32,
                AssetKind::FloorFlowGraphConfig => f32::INFINITY,
                AssetKind::ParameterConfig => 450f32,
                AssetKind::TagConfig => 450f32,
                AssetKind::EffectConfig => 450f32,
                AssetKind::GameGonfig => 450f32,
            };

            let right_panel_width = screen_width - 300f32 - preferred_central_width;

            egui::SidePanel::left("Режим редактора и кнопки сохранения/выхода")
                .exact_width(300f32)
                .show(egui_ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.add_space(6f32);
                        ui.group(|ui| {
                            const COLUMN_COUNT: usize = 2;

                            ui.columns(COLUMN_COUNT, |ui| {
                                let mut offset = 0;
                                for kind in [
                                    AssetKind::UnitConfig,
                                    AssetKind::ItemConfig,
                                    AssetKind::FloorPartConfig,
                                    AssetKind::FloorPartAdjacencyConfig,
                                    AssetKind::FloorConfig,
                                    AssetKind::FloorFlowGraphConfig,
                                    AssetKind::ParameterConfig,
                                    AssetKind::TagConfig,
                                    AssetKind::EffectConfig,
                                ] {
                                    let ui = &mut ui[offset];
                                    offset = (offset + 1) % COLUMN_COUNT;
                                    if thick_selector_button(
                                        ui,
                                        self.current_file_kind == kind,
                                        Align2::CENTER_CENTER,
                                        kind.editor_label()
                                    ).clicked() {
                                        self.current_file_kind = kind;
                                    };
                                }
                            });
                        });

                        ui.group(|ui| {
                            match self.current_file_kind {
                                AssetKind::ItemConfig => self.draw_item_selector(ui),
                                AssetKind::UnitConfig => self.draw_unit_selector(ui),
                                AssetKind::FloorPartConfig => self.draw_floor_part_selector(ui),
                                AssetKind::FloorPartAdjacencyConfig => self.draw_fpa_selector(ui),
                                AssetKind::FloorConfig => self.draw_floor_config_selector(ui),
                                AssetKind::FloorFlowGraphConfig => self.draw_floor_graph_selector(ui),
                                AssetKind::ParameterConfig => self.draw_parameter_selector(ui),
                                AssetKind::TagConfig => self.draw_tag_selector(ui),
                                AssetKind::EffectConfig => self.draw_effect_selector(ui),
                                AssetKind::GameGonfig => todo!()
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

            if right_panel_width > 0f32 {
                egui::SidePanel::right("Дополнительная информация")
                    .exact_width(right_panel_width)
                    .show(egui_ctx, |ui| {
                        match self.current_file_kind {
                            AssetKind::UnitConfig => self.draw_unit_preview_in_level(ui),
                            AssetKind::ItemConfig => self.draw_item_preview_in_level(ui),
                            AssetKind::FloorPartConfig => self.draw_floor_part_editor_tools(ui),
                            AssetKind::FloorPartAdjacencyConfig => self.draw_adjacency_visualizer(ui),
                            AssetKind::FloorConfig => self.draw_floor_editor_tools(ui),
                            AssetKind::TagConfig => {},
                            AssetKind::ParameterConfig => {},
                            AssetKind::EffectConfig => self.draw_effect_preview_in_level(ui),
                            _ => {}
                        }
                    });
            }

            egui::CentralPanel::default()
                .show(egui_ctx, |ui| {
                match self.current_file_kind {
                    AssetKind::UnitConfig => self.draw_unit_editor(ui),
                    AssetKind::ItemConfig => self.draw_item_editor(ui),
                    AssetKind::FloorPartConfig => self.draw_floor_part_editor(ui),
                    AssetKind::FloorPartAdjacencyConfig => self.draw_floor_part_adjacency_editor(ui),
                    AssetKind::FloorConfig => self.draw_floor_config_editor(ui),
                    AssetKind::FloorFlowGraphConfig => self.draw_floor_flow_graph_editor(ui),
                    AssetKind::TagConfig => self.draw_tag_editor(ui),
                    AssetKind::ParameterConfig => self.draw_parameter_editor(ui),
                    AssetKind::EffectConfig => self.draw_effect_editor(ui),
                    _ => {} // todo
                }
            });
        });
        result_status
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