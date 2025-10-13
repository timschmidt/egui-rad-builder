use egui::{Vec2, vec2};
use serde::{Deserialize, Serialize};
use crate::widget::Widget;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Project {
    pub(crate) widgets: Vec<Widget>,
    pub(crate) canvas_size: Vec2,
	pub(crate) panel_top_h: f32,
    pub(crate) panel_bottom_h: f32,
    pub(crate) panel_left_w: f32,
    pub(crate) panel_right_w: f32,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            canvas_size: vec2(1200.0, 800.0),
            panel_top_h: 80.0,
            panel_bottom_h: 80.0,
            panel_left_w: 220.0,
            panel_right_w: 260.0,
        }
    }
}
