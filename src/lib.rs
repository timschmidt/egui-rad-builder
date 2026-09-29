//! Library support for the `egui-rad-builder` application.
//!
//! The crate is primarily a desktop RAD builder for designing [`egui`] user
//! interfaces and generating Rust boilerplate for [`eframe`] applications.
//! This library target exposes the small reusable surface that is useful in
//! generated-code previews, palette metadata, and documentation builds.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Builder application state and code-generation settings.
pub mod app;
/// Syntax highlighting helpers used by generated-code previews.
pub mod highlight;
mod history;
mod project;
/// Widget metadata, defaults, and palette categorization.
pub mod widget;

/// Returns the default design-canvas size used by new projects.
pub fn default_canvas_size() -> egui::Vec2 {
    project::Project::default().canvas_size
}
