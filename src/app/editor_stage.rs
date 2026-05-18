use egui::{Align, Align2, StrokeKind, Ui};
use serde::Deserialize;
use uuid::Uuid;
use crate::app::app_stage::AppStageStatus;
use crate::assets::AssetKind;

#[derive(Copy, Clone, Deserialize)]
pub enum EditorCommand {
    BackToMainMenu
}

pub struct EditorStage {
    current_file_kind: AssetKind,
    selected_item_config: Option<Uuid>,
    selected_unit_config: Option<Uuid>,
    selected_floor_part: Option<Uuid>,
    selected_fpa_config: Option<Uuid>,
    selected_floor_config: Option<Uuid>,
    selected_floor_graph_config: Option<Uuid>,
    new_asset_name: String
}

impl EditorStage {
    pub fn new() -> Self {
        Self {
            current_file_kind: AssetKind::UnitConfig,
            selected_item_config: None,
            selected_unit_config: None,
            selected_floor_part: None,
            selected_fpa_config: None,
            selected_floor_config: None,
            selected_floor_graph_config: None,
            new_asset_name: String::new()
        }
    }

    pub fn process(&mut self) -> AppStageStatus<EditorCommand> {
        let mut result_status = AppStageStatus::Continue;

        egui_macroquad::ui(|egui_ctx| {
            egui::SidePanel::left("Режим редактора и кнопки сохранения/выхода")
                .show(egui_ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("Режим редактора:");
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
                        ui.separator();

                        match self.current_file_kind {
                            AssetKind::UnitConfig => self.draw_unit_selector(ui),
                            AssetKind::ItemConfig => self.draw_item_selector(ui),
                            AssetKind::FloorPart => self.draw_floor_part_selector(ui),
                            AssetKind::FloorPartAdjacency => self.draw_fpa_selector(ui),
                            AssetKind::FloorConfig => self.draw_floor_selector(ui),
                            AssetKind::FloorFlowGraphConfig => self.draw_floor_graph_selector(ui),
                        }

                        ui.separator();
                        if ui.button("Вернуться в главное меню").clicked() {
                            result_status = AppStageStatus::Complete(EditorCommand::BackToMainMenu);
                        }
                    });
                });
        });
        result_status
    }

    pub fn render(&self) {
        // todo: render here something relevant to editor logic
    }

    fn draw_item_selector(&mut self, ui: &mut Ui) {
        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                let full_width = ui.available_width();
                ui.add_sized(
                    [full_width, 24f32],
                    egui::TextEdit::singleline(&mut self.new_asset_name)
                );
                if ui.add_sized(
                    [full_width, 24f32],
                    egui::Button::new("+")
                ).clicked(){
                    let id =asset_db.create_json5_asset(
                        AssetKind::ItemConfig,
                        &self.new_asset_name,
                        "{}" // todo: сделать нормальное создание дефолтного ассета
                    );
                    if self.selected_item_config.is_none() {
                        self.selected_item_config = Some(id);
                    }
                    self.new_asset_name.clear();
                }
                ui.add_space(12f32);
                egui::ScrollArea::vertical()
                    .max_height(400f32)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let items = asset_db.list_all_assets(AssetKind::ItemConfig);
                        for (id, item_asset_name) in items {
                            let selected = self.selected_item_config
                                .map(|it| it.eq(&id))
                                .unwrap_or(false);
                            if thick_selector_button(
                                ui,
                                selected,
                                egui::Align2::LEFT_CENTER,
                                item_asset_name
                            ).clicked() {
                                self.selected_item_config = Some(id);
                            };
                        }
                });
            },
            _ => {}
        };
    }
    fn draw_unit_selector(&mut self, ui: &mut Ui) {
        // todo
    }
    fn draw_floor_part_selector(&mut self, ui: &mut Ui) {
        // todo
    }
    fn draw_fpa_selector(&mut self, ui: &mut Ui) {
        // todo
    }
    fn draw_floor_selector(&mut self, ui: &mut Ui) {
        // todo
    }
    fn draw_floor_graph_selector(&mut self, ui: &mut Ui) {
        // todo
    }
}

fn thick_selector_button(
    ui: &mut Ui,
    selected: bool,
    align: egui::Align2,
    text: &str,
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