/*
Данный функционал позаимствован (с некоторыми изменениями) из крейта egui_code_editor
ввиду более старой версии egui и слегка других нужд. Огромное спасибо Роману Чумаку за этот
замечательный крейт ☺
*/

use std::collections::BTreeSet;
use egui::{
    show_tooltip_for,
    Color32,
    Event,
    Id,
    Modifiers,
    Sense,
    Stroke,
    TextBuffer,
    TextWrapMode,
    scroll_area::ScrollBarVisibility,
    text::LayoutJob,
    text_edit::TextEditOutput,
    text_selection::text_cursor_state::ccursor_previous_word
};
use trie::Trie;
use crate::game_config::parameters::ParameterOperator;

pub mod trie;


#[derive(Default, Debug, Clone, PartialEq)]
/// Постановщик кода с попапом поверх редактора текста
pub struct Completer {
    prefix: String,
    cursor: usize,
    ignore_cursor: Option<usize>,
    words: Trie,
    variant_id: usize,
    completions: BTreeSet<String>,
    pub text_edit_id: Option<Id>,
}

impl Completer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_words(&mut self) {
        self.words.clear();
    }

    pub fn add_word(&mut self, word: &str) {
        self.words.push(word);
    }

    pub fn show_on_text_widget(
        &mut self,
        ui: &mut egui::Ui,
        mut widget: impl FnMut(&mut egui::Ui) -> TextEditOutput,
    ) -> TextEditOutput {
        self.handle_input(ui.ctx());
        let fontsize = ui.text_style_height(&egui::TextStyle::Monospace);
        let mut output = widget(ui);
        self.show(fontsize, &mut output);
        output
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        if self.prefix.is_empty() {
            return;
        }

        if let Some(cursor) = self.ignore_cursor && cursor == self.cursor {
            return;
        }

        let completions = self.words.find_completions(&self.prefix);
        self.completions = BTreeSet::from_iter(completions.into_iter());
        if self.completions.is_empty() {
            return;
        }
        let last = self.completions.len().saturating_sub(1);
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape)) {
            self.ignore_cursor = Some(self.cursor);
            if let Some(id) = self.text_edit_id {
                ctx.memory_mut(|m| {
                    m.request_focus(id);
                });
            }
        } else {
            ctx.input_mut(|i| {
                if i.consume_key(Modifiers::NONE, egui::Key::ArrowDown) {
                    self.variant_id = if self.variant_id == last {
                        0
                    } else {
                        self.variant_id.saturating_add(1).min(last)
                    };
                } else if i.consume_key(Modifiers::NONE, egui::Key::ArrowUp) {
                    self.variant_id = if self.variant_id == 0 {
                        last
                    } else {
                        self.variant_id.saturating_sub(1)
                    };
                } else if i.consume_key(Modifiers::NONE, egui::Key::Tab) {
                    let completion = self
                        .completions
                        .iter()
                        .nth(self.variant_id)
                        .map(String::from)
                        .unwrap_or_default();
                    i.events.push(Event::Paste(completion));
                }
            });
        }
    }

    fn show(
        &mut self,
        fontsize: f32,
        editor_output: &mut TextEditOutput,
    ) {
        self.text_edit_id = editor_output
            .response
            .has_focus()
            .then_some(editor_output.response.id)
            .or(self.text_edit_id);

        let ctx = editor_output.response.ctx.clone();
        if !editor_output.response.has_focus() {
            return;
        }

        let galley = &editor_output.galley;

        // Auto-Completer
        let cursor_range = editor_output.state.cursor.char_range();
        if let Some(range) = cursor_range {
            let mut cursor = range.primary;
            cursor.index = cursor.index.min(galley.job.text.chars().count());
            let cursor_pos_in_galley = galley.pos_from_ccursor(cursor);
            let cursor_rect = cursor_pos_in_galley
                .translate(editor_output.response.rect.left_top().to_vec2());
            let word_start = ccursor_previous_word(galley.text(), cursor);
            if self.cursor != cursor.index {
                self.cursor = cursor.index;
                self.prefix.clear();
                // self.completions.clear();
                self.ignore_cursor = None;
                self.variant_id = 0;
            }

            if self.ignore_cursor.is_some_and(|c| c == self.cursor) {
                editor_output.response.request_focus();
                return;
            } else {
                self.ignore_cursor = None;
            }
            let next_char_allows = galley
                .chars()
                .nth(cursor.index)
                .is_none_or(|c| !(c.is_alphanumeric() ||
                    c == '_' ||
                    c == '{' ||
                    c == '[' ||
                    ParameterOperator::is_valid_prefix_for_completion(c)));
            let next_char_allows = next_char_allows || range.secondary.index > range.primary.index;

            self.prefix = if next_char_allows {
                let prefix = galley
                    .text()
                    .char_range(word_start.index..cursor.index)
                    .to_string();
                if let Some((_, tail)) = prefix.rsplit_once(|c: char| {
                    !(c.is_alphanumeric() ||
                        c == '_' ||
                        c == '{' ||
                        c == '[' ||
                        ParameterOperator::is_valid_prefix_for_completion(c)
                    )
                }) {
                    tail.to_string()
                } else {
                    prefix
                }
            } else {
                String::new()
            };
            if !(self.prefix.is_empty() || self.completions.is_empty()) {
                show_tooltip_for(
                    &ctx,
                    editor_output.response.layer_id,
                    Id::new("Completer"),
                    &cursor_rect,
                    |ui| {
                        ui.response().sense = Sense::empty();
                        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                        let height = (fontsize
                            + ui.style().visuals.widgets.hovered.bg_stroke.width * 2.0
                            + ui.style().spacing.button_padding.y * 2.0
                            + ui.style().spacing.item_spacing.y)
                            * self.completions.len().min(10) as f32
                            - ui.style().spacing.item_spacing.y;
                        ui.set_height(height);

                        egui::ScrollArea::vertical()
                            .auto_shrink([true, true])
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                            .show(ui, |ui| {
                                for (i, completion) in self.completions.iter().enumerate() {
                                    let word = format!("{}{completion}", &self.prefix);
                                    let fmt = format_token(fontsize, &word);
                                    let colored_text = LayoutJob::single_section(word, fmt);
                                    let selected = i == self.variant_id;

                                    let hovered_style = &ui.style().visuals.widgets.hovered;

                                    let button = ui.add(
                                        egui::Button::new(colored_text)
                                            .sense(Sense::empty())
                                            .frame(true)
                                            .fill(hovered_style.bg_fill)
                                            .stroke(if selected {
                                                Stroke::new(
                                                    hovered_style.bg_stroke.width,
                                                    hovered_style.bg_stroke.color,
                                                )
                                            } else {
                                                Stroke::NONE
                                            }),
                                    );
                                    if selected {
                                        button.scroll_to_me(None);
                                    }
                                }
                            });
                    }
                );
            }
        }
    }
}

fn format_token(fontsize: f32, word: &str) -> egui::text::TextFormat {
    let font_id = egui::FontId::monospace(fontsize);

    let color = if word.starts_with("[") {
        Color32::from_rgb(255, 222, 128)
    } else if word.starts_with("{") {
        Color32::from_rgb(222, 255, 128)
    } else {
        Color32::LIGHT_GRAY
    };

    egui::text::TextFormat::simple(font_id, color)
}