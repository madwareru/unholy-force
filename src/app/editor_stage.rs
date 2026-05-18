use serde::Deserialize;
use crate::app::app_stage::AppStageStatus;

#[derive(Copy, Clone, Deserialize)]
pub enum EditorCommand {
    BackToMainMenu
}

pub struct EditorStage {

}

impl EditorStage {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn process(&mut self) -> AppStageStatus<EditorCommand> {
        let mut result_status = AppStageStatus::Continue;
        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad")
                .show(egui_ctx, |ui| {
                    ui.vertical(|ui| {
                        if ui.button("Back to main menu").clicked() {
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
}