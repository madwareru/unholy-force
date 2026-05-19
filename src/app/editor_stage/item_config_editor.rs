use egui::Ui;
use crate::app::editor_stage::{thick_selector_button, EditorStage};
use crate::assets::{AssetDb, AssetKind};

impl EditorStage {
    fn save_current_item_config(&self, asset_db: &mut AssetDb) {
        match (self.selected_item_config_id, &self.current_item_config) {
            (Some(id), Some(current_item_config)) => {
                let config_text = json5::to_string(current_item_config)
                    .expect("Failed to serialize default item config");
                asset_db.update_json5_asset(AssetKind::ItemConfig, id, &config_text);
            }
            _ => {}
        }
    }

    pub(crate) fn draw_item_selector(&mut self, ui: &mut Ui) {
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
                ).clicked() {
                    let mut default_item_config = crate::game_config::items::ItemConfig::default();
                    let config_text = json5::to_string(&default_item_config)
                        .expect("Failed to serialize default item config");
                    default_item_config.name = self.new_asset_name.clone();

                    self.current_item_config = Some(default_item_config);

                    let id =asset_db.create_json5_asset(
                        AssetKind::ItemConfig,
                        &self.new_asset_name,
                        &config_text
                    );
                    if self.selected_item_config_id.is_none() {
                        self.selected_item_config_id = Some(id);
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
                            let selected = self.selected_item_config_id
                                .map(|it| it.eq(&id))
                                .unwrap_or(false);
                            if thick_selector_button(
                                ui,
                                selected,
                                egui::Align2::LEFT_CENTER,
                                item_asset_name
                            ).clicked() {
                                self.selected_item_config_id = Some(id);
                            };
                        }
                    });
            },
            _ => {}
        };
    }
}