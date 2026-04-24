//! A lightweight RAD GUI builder for `egui` written in Rust.

mod app;
mod highlight;
mod project;
mod widget;

use crate::app::RadBuilderApp;

use eframe::egui;

fn initial_inner_size() -> egui::Vec2 {
    // Mirror your defaults
    let project = project::Project::default();

    // Base: canvas
    let mut w = project.canvas_size.x;
    let mut h = project.canvas_size.y;

    // Right inspector (default width = 260)
    w += 260.0;

    // Left palette is open by default in RadBuilderApp::default()
    w += 220.0;

    // Small padding for menubar + side padding
    h += 40.0;
    w += 16.0;

    egui::vec2(w, h)
}

fn main() -> eframe::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    let size = initial_inner_size();

    // Set initial and min inner size before creating the window
    native_options.viewport = egui::ViewportBuilder::default()
        .with_inner_size(size)
        .with_min_inner_size(size) // optional: prevent starting smaller than the canvas
        .with_resizable(true);

    eframe::run_native(
        "egui RAD GUI Builder",
        native_options,
        Box::new(|_cc| Ok(Box::<RadBuilderApp>::default())),
    )
}
