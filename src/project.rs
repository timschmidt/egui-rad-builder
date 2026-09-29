use crate::widget::Widget;
use egui::{Vec2, vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Project {
    pub(crate) widgets: Vec<Widget>,
    pub(crate) canvas_size: Vec2,
    pub(crate) panel_top_enabled: bool,
    pub(crate) panel_bottom_enabled: bool,
    pub(crate) panel_left_enabled: bool,
    pub(crate) panel_right_enabled: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            canvas_size: vec2(700.0, 600.0),
            panel_top_enabled: false,
            panel_bottom_enabled: false,
            panel_left_enabled: false,
            panel_right_enabled: false,
        }
    }
}
