use crate::{
    app::{
        editor_stage::{
            image_widgets::sprite_pivot_editor,
            EditorStage,
            UpdateState
        }
    },
    assets::{AssetDb, AssetKind},
    game_config::{
        parameters::{
            CompiledExpressionParameterNode,
            ParameterConfig,
            ParameterType,
            PARAMETER_CACHE
        },
        ConfigId
    },
    graphics::SPRITE_ATLAS_DEF
};
use egui::{PopupCloseBehavior, TextEdit};
use egui_code_editor::{CodeEditor, ColorTheme};
use uuid::Uuid;
use crate::app::editor_stage::text_completion::Completer;
use crate::game_config::parameters::{ParameterOperator, TagConfig};

#[derive(Default)]
pub struct ParameterConfigEditorSection {
    parameter_name_filter: String,
    selected_parameter_config_id: Option<Uuid>,
    selected_parameter_name: String,
    current_parameter_config: Option<ParameterConfig>,
    completer: Completer
}

impl EditorStage {
    fn update_current_parameter_config(
        &mut self,
        asset_db: &mut AssetDb,
        foo: impl FnOnce(
            &AssetDb,
            ConfigId<ParameterConfig>,
            &mut String,
            &mut ParameterConfig,
            &mut Completer,
        ) -> UpdateState,
    ) {
        let section = &mut self.parameter_section;
        let name = &mut section.selected_parameter_name;
        let cur_param = &mut section.current_parameter_config;
        let completer = &mut section.completer;

        if let (Some(config_id), Some(current_parameter_config)) = (section.selected_parameter_config_id, cur_param) {
            completer.clear_words();

            for standard_foo_name in ParameterOperator::standard_function_names() {
                completer.add_word(standard_foo_name);
            }

            for (id, _) in asset_db.list_all_assets(AssetKind::ParameterConfig) {
                if id.eq(&config_id) {
                    continue;
                }

                let config_text = asset_db.load_json5_asset(AssetKind::ParameterConfig, id);
                let config: ParameterConfig = json5::from_str(config_text)
                    .expect("Failed to load parameter config");
                let name = format!("{{{}}}", config.bound_name);
                completer.add_word(&name);
            }

            for (id, _) in asset_db.list_all_assets(AssetKind::TagConfig) {
                let config_text = asset_db.load_json5_asset(AssetKind::TagConfig, id);
                let config: TagConfig = json5::from_str(&config_text)
                    .expect("Failed to load tag config");
                let name = format!("[{}]", config.bound_name);
                completer.add_word(&name);
            }

            if foo(
                asset_db,
                ConfigId::from_uuid(config_id),
                name,
                current_parameter_config,
                completer
            ) == UpdateState::Changed {
                match section.selected_parameter_config_id {
                    Some(id) => {
                        asset_db.update_asset_mut(
                            AssetKind::ParameterConfig,
                            id,
                            |buffer| json5::to_writer(buffer, &current_parameter_config)
                        );
                        asset_db.rename_asset(AssetKind::ParameterConfig, id, &name);
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn draw_parameter_selector(&mut self, ui: &mut egui::Ui) {
        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                let full_width = ui.available_width();
                let available_height = ui.available_height() - ui.spacing().interact_size.y * 6f32;

                ui.horizontal(|ui| {
                    ui.label("Фильтр:");
                    ui.add(
                        TextEdit::singleline(&mut self.parameter_section.parameter_name_filter)
                            .desired_width(f32::INFINITY)
                    )
                });
                ui.add_space(4f32);
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut to_delete = None;

                        let parameters = asset_db.list_all_assets(AssetKind::ParameterConfig);
                        for (id, param_asset_name) in parameters {
                            let section = &mut self.parameter_section;
                            if !section.parameter_name_filter.is_empty() {
                                if !param_asset_name.starts_with(&section.parameter_name_filter) {
                                    continue;
                                }
                            }

                            let selected = section
                                .selected_parameter_config_id
                                .map(|it| it.eq(&id))
                                .unwrap_or(false);

                            let config_text =
                                asset_db.load_json5_asset(AssetKind::ParameterConfig, id);
                            let param_config: ParameterConfig = json5::from_str(&config_text)
                                .expect("Failed to load parameter config");

                            let response = ui.selectable_label(
                                selected,
                                if param_config.bound_name.is_empty() {
                                    param_asset_name.to_owned()
                                } else {
                                    format!("{{{}}} {}", param_config.bound_name, param_asset_name)
                                }
                            );

                            let popup_id =
                                ui.make_persistent_id(format!("выпадающее меню параметра {}", id));

                            if response.clicked_by(egui::PointerButton::Primary) {
                                section.current_parameter_config = Some(param_config);
                                section.selected_parameter_name.clear();
                                section.selected_parameter_name += param_asset_name;
                                section.selected_parameter_config_id = Some(id);
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
                                }
                            );
                        }

                        if let Some(id) = to_delete {
                            let section = &mut self.parameter_section;
                            match section.selected_parameter_config_id {
                                Some(selected_id) if selected_id.eq(&id) => {
                                    if let Some(current_config) = &section.current_parameter_config {
                                        if let Ok(mut cache) = PARAMETER_CACHE.lock() {
                                            cache.flush_parameter_id(&current_config.bound_name);
                                        }
                                    }
                                    section.selected_parameter_config_id = None;
                                    section.current_parameter_config = None;
                                }
                                _ => {
                                    if let Ok(mut cache) = PARAMETER_CACHE.lock() {
                                        let config_text = asset_db.load_json5_asset(AssetKind::ParameterConfig, id);
                                        let config_to_delete: ParameterConfig = json5::from_str(&config_text)
                                            .expect("Failed to load parameter config");
                                        cache.flush_parameter_id(&config_to_delete.bound_name);
                                    }
                                }
                            }
                            asset_db.delete_asset(AssetKind::ParameterConfig, id);
                        }
                    });

                if ui.add_sized(
                    [full_width, 24f32],
                    egui::Button::new("Создать черту")
                ).clicked() {
                    let default_param_config = ParameterConfig::default();
                    let config_text = json5::to_string(&default_param_config)
                        .expect("Failed to serialize default parameter config");

                    let section = &mut self.parameter_section;
                    section.current_parameter_config = Some(default_param_config);

                    let id = asset_db.create_json5_asset(
                        AssetKind::ParameterConfig,
                        "",
                        &config_text
                    );
                    section.selected_parameter_name.clear();
                    section.selected_parameter_config_id = Some(id);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn draw_parameter_editor(&mut self, ui: &mut egui::Ui) {
        let texture_id: egui::TextureId;
        if let Some(handle) = &self.atlas_texture {
            texture_id = handle.id();
        } else {
            unreachable!()
        };
        let atlas_size = self.atlas_size;

        match crate::assets::ASSET_DATABASE.lock() {
            Ok(mut asset_db) => {
                self.update_current_parameter_config(
                    &mut asset_db,
                    |asset_db, current_config_id, param_name, current_param_config, completer| {
                        let mut update_state = UpdateState::Unchanged;
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Название для редактора:");
                                    if ui.add(TextEdit::singleline(param_name).desired_width(f32::INFINITY)).changed() {
                                        update_state = UpdateState::Changed;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Имя для формул:");
                                    let old_name = current_param_config.bound_name.clone();
                                    if ui.add(TextEdit::singleline(&mut current_param_config.bound_name).desired_width(f32::INFINITY)).changed() {
                                        if let Ok(mut cache) = PARAMETER_CACHE.lock() {
                                            cache.flush_parameter_id(&old_name);
                                        }
                                        update_state = UpdateState::Changed;
                                    }
                                });
                                let duplicate_errors = current_param_config
                                    .check_duplicate_bound_name(current_config_id, asset_db);
                                if !duplicate_errors.is_empty() {
                                    ui.colored_label(egui::Color32::RED, duplicate_errors);
                                }
                                ui.horizontal(|ui| {
                                    ui.label("Название:");
                                    if ui.add(TextEdit::singleline(&mut current_param_config.name).desired_width(f32::INFINITY)).changed() {
                                        update_state = UpdateState::Changed;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Описание:");
                                    if ui.add(TextEdit::multiline(&mut current_param_config.description).desired_width(f32::INFINITY)).changed() {
                                        update_state = UpdateState::Changed;
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Изображение:");
                                    let full_width = ui.available_width();

                                    let response = ui.add_sized(
                                        [full_width, ui.spacing().interact_size.y],
                                        egui::Button::new(&current_param_config.sprite_name),
                                    );
                                    let popup_id = ui.make_persistent_id("выбор изображения параметра");
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

                                                for sprite in crate::graphics::SPRITE_ATLAS_DEF.sprites.keys() {
                                                    let sprite_name = sprite.as_str();

                                                    let ui = &mut uis[current_column];
                                                    ui.add_space(4f32);
                                                    let response = crate::app::editor_stage::image_widgets::atlas_sprite_button(
                                                        ui,
                                                        texture_id,
                                                        atlas_size,
                                                        sprite_name,
                                                        96f32,
                                                    );

                                                    if response.clicked() {
                                                        current_param_config.sprite_name.clear();
                                                        current_param_config.sprite_name += sprite_name;
                                                        update_state = UpdateState::Changed;
                                                        ui.memory_mut(|mem| mem.close_popup());
                                                    }

                                                    current_column = (current_column + 1) % COLUMNS_COUNT;
                                                }
                                            });
                                        },
                                    );
                                });
                                let entry = SPRITE_ATLAS_DEF
                                    .sprites
                                    .get(&current_param_config.sprite_name);
                                match entry {
                                    None => {}
                                    Some(sprite_data) => {
                                        let w = ui.available_width();
                                        let zoom = if sprite_data.size[0] == 0 {
                                            1f32
                                        } else {
                                            w / (sprite_data.size[0] as f32 * 16f32)
                                        };
                                        let old_pivot = current_param_config.sprite_pivot;
                                        sprite_pivot_editor(
                                            ui,
                                            texture_id,
                                            atlas_size,
                                            current_param_config,
                                            zoom,
                                        );
                                        if !old_pivot.eq(&current_param_config.sprite_pivot) {
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
                                                        &mut current_param_config.sprite_pivot[0],
                                                        0..=sprite_data.size[0] * 16 - 1,
                                                    )
                                                )
                                                .changed()
                                            {
                                                update_state = UpdateState::Changed;
                                            }
                                            if ui
                                                .add_sized(
                                                    [slider_width, ui.spacing().interact_size.y],
                                                    egui::Slider::new(
                                                        &mut current_param_config.sprite_pivot[1],
                                                        0..=sprite_data.size[1] * 16 - 1,
                                                    )
                                                )
                                                .changed()
                                            {
                                                update_state = UpdateState::Changed;
                                            }
                                        });
                                    }
                                }

                                ui.horizontal(|ui| {
                                    ui.label("Тип черты:");
                                    let full_width = ui.available_width();
                                    let is_constant = matches!(current_param_config.parameter_type, ParameterType::Constant);
                                    let response = ui.add_sized(
                                        [full_width, ui.spacing().interact_size.y],
                                        egui::Button::new(
                                            if is_constant { "Константа" } else { "Выражение" }
                                        ),
                                    );
                                    let popup_id = ui.make_persistent_id("выбор типа параметра");
                                    if response.clicked() {
                                        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                    }
                                    egui::popup_below_widget(
                                        ui,
                                        popup_id,
                                        &response,
                                        PopupCloseBehavior::CloseOnClickOutside,
                                        |ui| {
                                            if ui.button("Константа").clicked() {
                                                if !matches!(current_param_config.parameter_type, ParameterType::Constant) {
                                                    current_param_config.parameter_type = ParameterType::Constant;
                                                    update_state = UpdateState::Changed;
                                                }
                                                ui.memory_mut(|mem| mem.close_popup());
                                            }
                                            if ui.button("Выражение").clicked() {
                                                if !matches!(current_param_config.parameter_type, ParameterType::Expression(_)) {
                                                    current_param_config.parameter_type = ParameterType::Expression(String::new());
                                                    update_state = UpdateState::Changed;
                                                }
                                                ui.memory_mut(|mem| mem.close_popup());
                                            }
                                        },
                                    );
                                });

                                match &mut current_param_config.parameter_type {
                                    ParameterType::Constant => {}
                                    ParameterType::Expression(source) => {
                                        ui.horizontal(|ui| {
                                            ui.label("Выражение:");
                                        });
                                        let syntax = completer.syntax().clone();
                                        if completer.show_on_text_widget(
                                            ui,
                                            |ui| {
                                                CodeEditor::default()
                                                    .id_source("редактор выражений")
                                                    .with_rows(4)
                                                    .with_fontsize(14.0)
                                                    .with_theme(ColorTheme::GRUVBOX)
                                                    .with_syntax(syntax.clone())
                                                    .with_numlines(true)
                                                    .show(ui, source)
                                            }
                                        ).response.changed() {
                                            update_state = UpdateState::Changed;
                                        }

                                        if let Ok(mut cache) = PARAMETER_CACHE.lock() {
                                            current_param_config.compile_expression(&asset_db, &mut cache);
                                        }

                                        match current_param_config.compiled_expression() {
                                            CompiledExpressionParameterNode::None => {}
                                            CompiledExpressionParameterNode::Error { compile_error } => {
                                                ui.colored_label(egui::Color32::RED, compile_error);
                                            }
                                            CompiledExpressionParameterNode::Ok(_) => {
                                                ui.colored_label(egui::Color32::GREEN, "Выражение скомпилировано успешно");
                                            }
                                        }
                                    }
                                }
                            })
                        });

                        update_state
                    },
                );
            }
            _ => {}
        }
    }
}