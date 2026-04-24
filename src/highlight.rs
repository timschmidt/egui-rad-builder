//! Syntax highlighting for generated code using syntect.

use egui::Color32;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Cached syntax highlighting resources.
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            theme_name: "base16-ocean.dark".to_string(),
        }
    }

    /// Highlight Rust code and return a list of (text, color) spans.
    pub fn highlight_rust(&self, code: &str) -> Vec<(String, Color32)> {
        let syntax = self
            .syntax_set
            .find_syntax_by_extension("rs")
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = self
            .theme_set
            .themes
            .get(&self.theme_name)
            .unwrap_or_else(|| {
                self.theme_set
                    .themes
                    .values()
                    .next()
                    .expect("No themes available")
            });

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();

        for line in LinesWithEndings::from(code) {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(ranges) => {
                    for (style, text) in ranges {
                        let color = style_to_color32(style);
                        result.push((text.to_string(), color));
                    }
                }
                Err(_) => {
                    // Fallback to plain text on error
                    result.push((line.to_string(), Color32::LIGHT_GRAY));
                }
            }
        }

        result
    }

    /// Render highlighted code as a LayoutJob for egui.
    pub fn layout_job(&self, code: &str) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        for (text, color) in self.highlight_rust(code) {
            job.append(
                &text,
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::monospace(12.0),
                    color,
                    ..Default::default()
                },
            );
        }

        job
    }
}

/// Convert syntect Style to egui Color32.
fn style_to_color32(style: Style) -> Color32 {
    Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}

/// Simple code viewer with syntax highlighting (read-only).
#[allow(dead_code)]
pub fn code_viewer(ui: &mut egui::Ui, highlighter: &Highlighter, code: &str) {
    let job = highlighter.layout_job(code);

    egui::ScrollArea::vertical()
        .id_salt("highlighted_code_scroll")
        .max_height(280.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Use a Label with the layout job for syntax-highlighted display
            ui.add(egui::Label::new(job).selectable(true));
        });
}

/// Code editor with syntax highlighting (editable).
/// Returns true if the code was modified.
#[allow(dead_code)]
pub fn code_editor_highlighted(
    ui: &mut egui::Ui,
    _highlighter: &Highlighter,
    code: &mut String,
) -> bool {
    let mut changed = false;

    egui::ScrollArea::vertical()
        .id_salt("code_editor_scroll")
        .max_height(280.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // For editing, we use a regular TextEdit with code_editor styling
            // Syntax highlighting on edit is expensive, so we show it read-only
            // The user can toggle between edit and view modes
            let response = ui.add(
                egui::TextEdit::multiline(code)
                    .code_editor()
                    .desired_rows(18)
                    .desired_width(f32::INFINITY),
            );
            changed = response.changed();
        });

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_creation() {
        let highlighter = Highlighter::new();
        // Should not panic
        let _ = highlighter.highlight_rust("fn main() {}");
    }

    #[test]
    fn test_highlight_rust_basic() {
        let highlighter = Highlighter::new();
        let spans = highlighter.highlight_rust("fn main() {\n    println!(\"Hello\");\n}\n");
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_layout_job() {
        let highlighter = Highlighter::new();
        let job = highlighter.layout_job("let x = 42;");
        assert!(!job.text.is_empty());
    }
}
