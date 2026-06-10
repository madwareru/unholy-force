use std::rc::Rc;
use egui::{PopupCloseBehavior, Stroke, TextEdit, Ui};
use egui_snarl::ui::{NodeLayout, PinPlacement, PinShape, SnarlStyle};
use uuid::Uuid;
use crate::{
    app::{
        editor_stage::{
            EditorStage,
            UpdateState,
            widgets::{atlas_sprite_button, effect_selector_button, sprite_pivot_editor}
        }
    },
    assets::{AssetDb, AssetKind},
    game_config::effects::EffectConfig,
    graphics::SPRITE_ATLAS_DEF
};

pub struct EffectConfigEditorSection {
    effect_name_filter: String,
    selected_effect_config_id: Option<Uuid>,
    selected_effect_name: String,
    current_effect_config: Option<EffectConfig>,
    style: Rc<SnarlStyle>,
}

impl Default for EffectConfigEditorSection {
    fn default() -> Self {
        Self {
            effect_name_filter: String::new(),
            selected_effect_config_id: None,
            selected_effect_name: String::new(),
            current_effect_config: None,
            style: Rc::new(SnarlStyle {
                pin_placement: Some(PinPlacement::Edge),
                node_layout: Some(NodeLayout::Sandwich),
                pin_shape: Some(PinShape::Square),
                pin_stroke: Some(Stroke::NONE),
                ..Default::default()
            }),
        }
    }
}

impl EditorStage {
    fn update_current_effect_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(&AssetDb, &mut String, &mut EffectConfig) -> UpdateState,
    ) {
        let section = &mut self.effect_section;
        let name = &mut section.selected_effect_name;
        let cur_effect = &mut section.current_effect_config;

        if let Some(current_effect_config) = cur_effect {
            if foo(asset_db, name, current_effect_config) == UpdateState::Changed {
                if let Some(id) = section.selected_effect_config_id {
                    asset_db.update_asset_mut(
                        AssetKind::EffectConfig,
                        id,
                        |buffer| json5::to_writer(buffer, current_effect_config),
                    );
                    asset_db.rename_asset(AssetKind::EffectConfig, id, name);
                }
            }
        }
    }

    pub(crate) fn draw_effect_selector(&mut self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;

        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                let full_width = ui.available_width();
                let available_height =
                    ui.available_height() - ui.spacing().interact_size.y * 6f32;

                ui.horizontal(|ui| {
                    ui.label("Фильтр:");
                    ui.add(
                        TextEdit::singleline(&mut self.effect_section.effect_name_filter)
                            .desired_width(f32::INFINITY),
                    )
                });
                ui.add_space(4f32);

                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut to_delete = None;

                        let effects = asset_db.list_all_assets(AssetKind::EffectConfig);
                        for (id, effect_asset_name) in effects {
                            let section = &mut self.effect_section;
                            if !section.effect_name_filter.is_empty() {
                                if !effect_asset_name.starts_with(&section.effect_name_filter) {
                                    continue;
                                }
                            }

                            let selected = section
                                .selected_effect_config_id
                                .map(|it| it.eq(&id))
                                .unwrap_or(false);

                            let config_text =
                                asset_db.load_json5_asset(AssetKind::EffectConfig, id);
                            let effect_config: EffectConfig = json5::from_str(&config_text)
                                .expect("Failed to load effect config");

                            let response = effect_selector_button(
                                ui,
                                selected,
                                texture_id,
                                atlas_size,
                                effect_asset_name,
                                &effect_config,
                            );

                            let popup_id =
                                ui.make_persistent_id(format!("выпадающее меню {}", id));

                            if response.clicked_by(egui::PointerButton::Primary) {
                                section.current_effect_config = Some(effect_config);
                                section.selected_effect_name.clear();
                                section.selected_effect_name += effect_asset_name;
                                section.selected_effect_config_id = Some(id);
                            } else if response.clicked_by(egui::PointerButton::Secondary) {
                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                            }

                            egui::popup_below_widget(
                                ui,
                                popup_id,
                                &response,
                                PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    ui.set_min_width(100f32);
                                    if ui.button("Удалить").clicked() {
                                        to_delete = Some(id);
                                        ui.memory_mut(|mem| mem.close_popup());
                                    }
                                },
                            );
                        }

                        if let Some(id) = to_delete {
                            let section = &mut self.effect_section;
                            match section.selected_effect_config_id {
                                Some(selected_id) if selected_id.eq(&id) => {
                                    section.selected_effect_config_id = None;
                                    section.current_effect_config = None;
                                }
                                _ => {}
                            }
                            asset_db.delete_asset(AssetKind::EffectConfig, id);
                        }
                    });

                if ui
                    .add_sized(
                        [full_width, 24f32],
                        egui::Button::new("Создать механику эффекта"),
                    )
                    .clicked()
                {
                    let default_config = EffectConfig::new();
                    let config_text = json5::to_string(&default_config)
                        .expect("Failed to serialize default effect config");

                    let section = &mut self.effect_section;
                    section.current_effect_config = Some(default_config);

                    let id = asset_db.create_json5_asset(
                        AssetKind::EffectConfig,
                        "",
                        &config_text,
                    );
                    section.selected_effect_name.clear();
                    section.selected_effect_config_id = Some(id);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn draw_effect_editor(&mut self, ui: &mut Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;
        let style = Rc::clone(&self.effect_section.style);

        let mut asset_db = crate::assets::ASSET_DATABASE.lock()
            .expect("Failed to lock asset database");

        self.update_current_effect_config(
            &mut asset_db,
            |asset_db, effect_name, current_effect_config| {
                let mut update_state = UpdateState::Unchanged;
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Название для редактора:");
                            if ui
                                .add(
                                    TextEdit::singleline(effect_name)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                update_state = UpdateState::Changed;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Описание:");
                            if ui
                                .add(
                                    TextEdit::multiline(
                                        &mut current_effect_config.description,
                                    )
                                        .desired_width(f32::INFINITY),
                                )
                                .changed()
                            {
                                update_state = UpdateState::Changed;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Изображение:");
                            let full_width = ui.available_width();

                            let response = ui.add_sized(
                                [full_width, ui.spacing().interact_size.y],
                                egui::Button::new(&current_effect_config.sprite_name),
                            );
                            let popup_id =
                                ui.make_persistent_id("выбор изображения");
                            if response.clicked() {
                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                            }
                            egui::popup_below_widget(
                                ui,
                                popup_id,
                                &response,
                                PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    const COLUMNS_COUNT: usize = 6;
                                    ui.columns(COLUMNS_COUNT, |uis| {
                                        let mut current_column = 0;

                                        for sprite in
                                            crate::graphics::SPRITE_ATLAS_DEF
                                                .sprite_keys()
                                        {
                                            let sprite_name = sprite.as_str();

                                            let ui = &mut uis[current_column];
                                            ui.add_space(4f32);
                                            let response = atlas_sprite_button(
                                                ui,
                                                texture_id,
                                                atlas_size,
                                                sprite_name,
                                                96f32,
                                            );

                                            if response.clicked() {
                                                current_effect_config
                                                    .sprite_name
                                                    .clear();
                                                current_effect_config
                                                    .sprite_name
                                                    += sprite_name;
                                                update_state = UpdateState::Changed;
                                                ui.memory_mut(|mem| {
                                                    mem.close_popup()
                                                });
                                            }

                                            current_column =
                                                (current_column + 1) % COLUMNS_COUNT;
                                        }
                                    });
                                },
                            );
                        });

                        let sprite_data = SPRITE_ATLAS_DEF
                            .get_sprite_def(&current_effect_config.sprite_name);

                        let zoom = 4f32;
                        let old_pivot = current_effect_config.sprite_pivot;
                        sprite_pivot_editor(
                            ui,
                            texture_id,
                            atlas_size,
                            current_effect_config,
                            zoom,
                        );
                        if !old_pivot.eq(&current_effect_config.sprite_pivot) {
                            update_state = UpdateState::Changed;
                        }

                        ui.horizontal(|ui| {
                            ui.label("Опорная точка:");

                            let available_width = ui.available_width();
                            let slider_width = available_width / 2f32;

                            if ui
                                .add_sized(
                                    [slider_width, ui.spacing().interact_size.y],
                                    egui::Slider::new(
                                        &mut current_effect_config.sprite_pivot[0],
                                        0..=sprite_data.size[0] * 16 - 1,
                                    ),
                                )
                                .changed()
                            {
                                update_state = UpdateState::Changed;
                            }
                            if ui
                                .add_sized(
                                    [slider_width, ui.spacing().interact_size.y],
                                    egui::Slider::new(
                                        &mut current_effect_config.sprite_pivot[1],
                                        0..=sprite_data.size[1] * 16 - 1,
                                    ),
                                )
                                .changed()
                            {
                                update_state = UpdateState::Changed;
                            }
                        });
                    });
                    ui.group(|ui| {
                        if current_effect_config.edit_snarl(
                            ui,
                            &asset_db,
                            texture_id,
                            atlas_size,
                            &style,
                        ) {
                            update_state = UpdateState::Changed;
                        }
                    })
                });

                update_state
            },
        );
    }
}
