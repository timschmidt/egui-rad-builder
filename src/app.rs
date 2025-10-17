use crate::{
    project::Project,
    widget::{
        self, DockArea, Widget, WidgetId, WidgetKind, WidgetProps, escape, snap_pos_with_grid,
    },
};
use chrono::{Datelike, NaiveDate};
use copypasta::ClipboardProvider;
use egui::{pos2, vec2, Button, Color32, CornerRadius, Id, Modal, Pos2, Rect, Sense, Stroke, Ui, UiBuilder};
use egui_extras::{syntax_highlighting::CodeTheme, DatePickerButton};
use serde::{Deserialize, Serialize};

fn bool_true() -> bool { true }

#[derive(Serialize, Deserialize)]
pub(crate) struct RadBuilderApp {
    #[serde(skip, default = "bool_true")]
    /// Is the widget picker open
    palette_open: bool,
    project: Project,
    #[serde(skip, default)]
    selected: Option<WidgetId>,
    next_id: u64,
    /// Drag state for spawning from palette
    #[serde(skip, default)]
    spawning: Option<WidgetKind>,
    /// Cached generated code
    #[serde(skip, default)]
    generated: String,
    // Settings
    grid_size: f32,
    show_grid: bool,
    live_top: Option<Rect>,
    live_bottom: Option<Rect>,
    live_left: Option<Rect>,
    live_right: Option<Rect>,
    live_center: Option<Rect>,

    #[serde(skip, default)]
    /// The open modal if any
    open_modal: Option<OpenModal>,
}

pub(crate) enum OpenModal {
    /// json import modal is open
    Import,
    /// json export modal is open
    Export,
}

impl Default for RadBuilderApp {
    fn default() -> Self {
        Self {
            palette_open: true,
            project: Project::default(),
            selected: None,
            next_id: 1,
            spawning: None,
            generated: String::new(),

            grid_size: 14.0,
            show_grid: false,
            live_top: None,
            live_bottom: None,
            live_left: None,
            live_right: None,
            live_center: None,

            open_modal: None,
        }
    }
}

impl RadBuilderApp {
    fn area_at(&self, pos: Pos2) -> DockArea {
        if let Some(r) = self.live_top && r.contains(pos) {
            DockArea::Top
        } else if let Some(r) = self.live_bottom && r.contains(pos) {
            DockArea::Bottom
        } else if let Some(r) = self.live_left && r.contains(pos) {
            DockArea::Left
        } else if let Some(r) = self.live_right && r.contains(pos) {
            DockArea::Right
        } else if let Some(r) = self.live_center && r.contains(pos) {
            DockArea::Center
        } else {
            DockArea::Free
        }
    }

    fn origin_for_area(&self, area: DockArea) -> Option<Pos2> {
        match area {
            DockArea::Top => self.live_top.map(|r| r.min),
            DockArea::Bottom => self.live_bottom.map(|r| r.min),
            DockArea::Left => self.live_left.map(|r| r.min),
            DockArea::Right => self.live_right.map(|r| r.min),
            DockArea::Center => self.live_center.map(|r| r.min),
            DockArea::Free => self.live_center.map(|r| r.min), // place Free inside center canvas
        }
    }

    fn spawn_widget(
        &mut self,
        kind: WidgetKind,
        at_global: Pos2,
        area: DockArea,
        area_origin: Pos2,
    ) {
        let id = WidgetId::new(self.next_id);
        self.next_id += 1;

        let (size, props) = match kind {
            WidgetKind::MenuButton => {
                let mut p = WidgetProps {
                    text: "Menu".into(),
                    ..Default::default()
                };
                p.items = vec!["First".into(), "Second".into(), "Third".into()];
                p.selected = 0;
                (vec2(180.0, 28.0), p)
            }
            WidgetKind::Label => (
                vec2(140.0, 24.0),
                WidgetProps {
                    text: "Label".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Button => (
                vec2(160.0, 32.0),
                WidgetProps {
                    text: "Button".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::ImageTextButton => (
                vec2(200.0, 36.0),
                WidgetProps {
                    text: "Button".into(),
                    icon: "🖼️".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Checkbox => (
                vec2(160.0, 28.0),
                WidgetProps {
                    text: "Checkbox".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::TextEdit => (
                vec2(220.0, 36.0),
                WidgetProps {
                    text: "Type here".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Slider => (
                vec2(220.0, 24.0),
                WidgetProps {
                    text: "Value".into(),
                    min: 0.0,
                    max: 100.0,
                    value: 42.0,
                    checked: false,
                    ..Default::default()
                },
            ),
            WidgetKind::ProgressBar => (
                vec2(220.0, 20.0),
                WidgetProps {
                    text: "".into(),
                    value: 0.25,
                    min: 0.0,
                    max: 1.0,
                    checked: false,
                    ..Default::default()
                },
            ),
            WidgetKind::RadioGroup => {
                let mut p = WidgetProps {
                    text: "Radio Group".into(),
                    ..Default::default()
                };
                p.items = vec!["Option A".into(), "Option B".into(), "Option C".into()];
                p.selected = 0;
                (vec2(200.0, 80.0), p)
            }
            WidgetKind::Link => (
                vec2(160.0, 20.0),
                WidgetProps {
                    text: "Link text".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Hyperlink => (
                vec2(200.0, 20.0),
                WidgetProps {
                    text: "Open website".into(),
                    url: "https://example.com".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::SelectableLabel => (
                vec2(180.0, 24.0),
                WidgetProps {
                    text: "Selectable".into(),
                    checked: false,
                    ..Default::default()
                },
            ),
            WidgetKind::ComboBox => {
                let mut p = WidgetProps {
                    text: "Choose one".into(),
                    ..Default::default()
                };
                p.items = vec!["Red".into(), "Green".into(), "Blue".into()];
                p.selected = 0;
                (vec2(220.0, 28.0), p)
            }
            WidgetKind::Separator => (vec2(220.0, 8.0), WidgetProps::default()),
            WidgetKind::CollapsingHeader => (
                vec2(260.0, 80.0),
                WidgetProps {
                    text: "Section".into(),
                    checked: true, // default open
                    ..Default::default()
                },
            ),
            WidgetKind::DatePicker => (
                vec2(200.0, 28.0),
                WidgetProps {
                    text: "Pick a date".into(),
                    year: 2025,
                    month: 1,
                    day: 1,
                    ..Default::default()
                },
            ),
            WidgetKind::AngleSelector => (
                vec2(220.0, 28.0),
                WidgetProps {
                    text: "Angle (deg)".into(),
                    min: 0.0,
                    max: 360.0,
                    value: 45.0,
                    ..Default::default()
                },
            ),
            WidgetKind::Password => (
                vec2(220.0, 36.0),
                WidgetProps {
                    text: "password".into(),
                    ..Default::default()
                },
            ),
            WidgetKind::Tree => {
                let mut p = WidgetProps {
                    text: "Tree".into(),
                    ..Default::default()
                };
                // Indentation (two spaces = one level) to define hierarchy:
                p.items = vec![
                    "Animals".into(),
                    "  Mammals".into(),
                    "    Dogs".into(),
                    "    Cats".into(),
                    "  Birds".into(),
                    "Plants".into(),
                    "  Trees".into(),
                    "  Flowers".into(),
                ];
                (vec2(260.0, 200.0), p)
            }
        };

        let vecpos = at_global - area_origin - size * 0.5; // local to area
        let pos = self.snap_pos(pos2(vecpos.x, vecpos.y));
        let w = Widget {
            id,
            kind,
            pos,
            size,
            z: id.as_z(),
            area,
            props,
        };
        self.project.widgets.push(w);
        self.selected = Some(id);
    }

    /// Self::selected_mut(self.selected, &mut self.project.widgets)
    fn selected_mut(selected: Option<WidgetId>, widgets: &mut Vec<Widget>) -> Option<&mut Widget> {
        let id = selected?;
        widgets.iter_mut().find(|w| w.id == id)
    }

    fn preview_panels_ui(&mut self, ctx: &egui::Context) {
        use DockArea::*;

        // Optional: stable visual order
        self.project.widgets.sort_by_key(|w| w.z);

        // Reset live rects each frame
        self.live_top = None;
        self.live_bottom = None;
        self.live_left = None;
        self.live_right = None;
        self.live_center = None;

        // -------- 1) Bucket INDICES (not &mut) by area in a read-only pass --------
        let mut top_idx = Vec::new();
        let mut bottom_idx = Vec::new();
        let mut left_idx = Vec::new();
        let mut right_idx = Vec::new();
        let mut center_idx = Vec::new();
        let mut free_idx = Vec::new();

        for (i, w) in self.project.widgets.iter().enumerate() {
            match w.area {
                Top => top_idx.push(i),
                Bottom => bottom_idx.push(i),
                Left => left_idx.push(i),
                Right => right_idx.push(i),
                Center => center_idx.push(i),
                Free => free_idx.push(i),
            }
        }

        // Top
        if self.project.panel_top_enabled {
            egui::TopBottomPanel::top("rb_top")
                .resizable(true)
                .show(ctx, |ui| {
                    let panel_rect = ui.clip_rect();
                    self.live_top = Some(panel_rect);
                    if self.show_grid {
                        self.draw_grid(ui, panel_rect);
                    }
                    for &i in &top_idx {
                        let w = &mut self.project.widgets[i];
                        Self::draw_widget(ui, panel_rect, self.grid_size, &mut self.selected, w);
                    }
                });
        }

        // Bottom
        if self.project.panel_bottom_enabled {
            egui::TopBottomPanel::bottom("rb_bottom")
                .resizable(true)
                .show(ctx, |ui| {
                    let panel_rect = ui.clip_rect();
                    self.live_bottom = Some(panel_rect);
                    if self.show_grid {
                        self.draw_grid(ui, panel_rect);
                    }
                    for &i in &bottom_idx {
                        let w = &mut self.project.widgets[i];
                        Self::draw_widget(ui, panel_rect, self.grid_size, &mut self.selected, w);
                    }
                });
        }

        // Left
        if self.project.panel_left_enabled {
            egui::SidePanel::left("rb_left")
                .resizable(true)
                .show(ctx, |ui| {
                    let panel_rect = ui.clip_rect();
                    self.live_left = Some(panel_rect);
                    if self.show_grid {
                        self.draw_grid(ui, panel_rect);
                    }
                    for &i in &left_idx {
                        let w = &mut self.project.widgets[i];
                        Self::draw_widget(ui, panel_rect, self.grid_size, &mut self.selected, w);
                    }
                });
        }

        // Right
        if self.project.panel_right_enabled {
            egui::SidePanel::right("rb_right")
                .resizable(true)
                .show(ctx, |ui| {
                    let panel_rect = ui.clip_rect();
                    self.live_right = Some(panel_rect);
                    if self.show_grid {
                        self.draw_grid(ui, panel_rect);
                    }
                    for &i in &right_idx {
                        let w = &mut self.project.widgets[i];
                        Self::draw_widget(ui, panel_rect, self.grid_size, &mut self.selected, w);
                    }
                });
        }

        // Center (design canvas)
        egui::CentralPanel::default().show(ctx, |ui| {
            // Fixed canvas to mirror generated app
            let canvas = egui::Rect::from_min_size(ui.min_rect().min, self.project.canvas_size);
            self.live_center = Some(canvas);

            let (resp, _) = ui.allocate_painter(canvas.size(), egui::Sense::hover());
            let painter_rect = egui::Rect::from_min_size(canvas.min, canvas.size());

            if self.show_grid {
                self.draw_grid(ui, painter_rect);
            }

            // Draw Center + Free widgets inside the center canvas
            for &i in &center_idx {
                let w = &mut self.project.widgets[i];
                Self::draw_widget(ui, painter_rect, self.grid_size, &mut self.selected, w);
            }
            for &i in &free_idx {
                let w = &mut self.project.widgets[i];
                Self::draw_widget(ui, painter_rect, self.grid_size, &mut self.selected, w);
            }

            // --- Drag ghost + drop ---
            if let Some(kind) = self.spawning.clone() {
                if let Some(mouse) = ui.ctx().pointer_interact_pos() {
                    let ghost_size = match kind {
                        WidgetKind::MenuButton => vec2(180.0, 28.0),
                        WidgetKind::Label => vec2(140.0, 24.0),
                        WidgetKind::Button => vec2(160.0, 32.0),
                        WidgetKind::ImageTextButton => vec2(200.0, 36.0),
                        WidgetKind::Checkbox => vec2(160.0, 28.0),
                        WidgetKind::TextEdit => vec2(220.0, 36.0),
                        WidgetKind::Slider => vec2(220.0, 24.0),
                        WidgetKind::ProgressBar => vec2(220.0, 20.0),
                        WidgetKind::RadioGroup => vec2(200.0, 80.0),
                        WidgetKind::Link => vec2(160.0, 20.0),
                        WidgetKind::Hyperlink => vec2(200.0, 20.0),
                        WidgetKind::SelectableLabel => vec2(180.0, 24.0),
                        WidgetKind::ComboBox => vec2(220.0, 28.0),
                        WidgetKind::Separator => vec2(220.0, 8.0),
                        WidgetKind::CollapsingHeader => vec2(260.0, 80.0),
                        WidgetKind::DatePicker => vec2(200.0, 28.0),
                        WidgetKind::AngleSelector => vec2(220.0, 28.0),
                        WidgetKind::Password => vec2(220.0, 36.0),
                        WidgetKind::Tree => vec2(260.0, 200.0),
                    };
                    let ghost = egui::Rect::from_center_size(mouse, ghost_size);
                    let layer = egui::LayerId::new(egui::Order::Tooltip, Id::new("ghost"));
                    let painter = ui.ctx().layer_painter(layer);
                    painter.rect_filled(ghost, 4.0, Color32::from_gray(40));
                    painter.rect_stroke(
                        ghost,
                        CornerRadius::same(4),
                        Stroke::new(1.0, Color32::LIGHT_BLUE),
                        egui::StrokeKind::Outside,
                    );

                    // highlight target panel
                    let area = self.area_at(mouse);
                    if let Some(hilite) = match area {
                        DockArea::Top => self.live_top,
                        DockArea::Bottom => self.live_bottom,
                        DockArea::Left => self.live_left,
                        DockArea::Right => self.live_right,
                        DockArea::Center | DockArea::Free => self.live_center,
                    } {
                        painter.rect_stroke(
                            hilite,
                            CornerRadius::same(6),
                            Stroke::new(2.0, Color32::LIGHT_BLUE),
                            egui::StrokeKind::Outside,
                        );
                    }
                }

                if ui.input(|i| i.pointer.any_released()) {
                    if let Some(pos) = ui.ctx().pointer_interact_pos() {
                        let area = self.area_at(pos);
                        if let Some(origin) = self.origin_for_area(area) {
                            self.spawn_widget(kind, pos, area, origin);
                        }
                    }
                    self.spawning = None;
                }
            }

            if resp.clicked() {
                self.selected = None;
            }
        });
    }

    fn draw_grid(&self, ui: &mut egui::Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        let g = self.grid_size;
        let cols = (rect.width() / g) as i32;
        let rows = (rect.height() / g) as i32;
        for c in 0..=cols {
            let x = rect.left() + c as f32 * g;
            painter.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(1.0, Color32::from_gray(40)),
            );
        }
        for r in 0..=rows {
            let y = rect.top() + r as f32 * g;
            painter.line_segment(
                [pos2(rect.left(), y), pos2(rect.right(), y)],
                Stroke::new(1.0, Color32::from_gray(40)),
            );
        }
    }

    fn draw_widget(
        ui: &mut egui::Ui,
        canvas_rect: Rect,
        grid: f32,
        selected: &mut Option<WidgetId>,
        w: &mut Widget,
    ) {
        let rect = Rect::from_min_size(canvas_rect.min + w.pos.to_vec2(), w.size);
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            match w.kind {
                WidgetKind::MenuButton => {
                    let items = if w.props.items.is_empty() {
                        vec!["Item".into()]
                    } else {
                        w.props.items.clone()
                    };
                    let mut sel = w.props.selected.min(items.len() - 1);
                    ui.menu_button(&w.props.text, |ui| {
                        for (i, it) in items.iter().enumerate() {
                            if ui.button(it).clicked() {
                                sel = i;
                                ui.close_kind(egui::UiKind::Menu);
                            }
                        }
                    });
                    w.props.selected = sel;
                }
                WidgetKind::Label => {
                    ui.vertical_centered(|ui| {
                        ui.label(&w.props.text);
                    });
                }
                WidgetKind::Button => {
                    ui.add_sized(w.size, egui::Button::new(&w.props.text));
                }
                WidgetKind::ImageTextButton => {
                    // We keep it simple: icon + text as the button label.
                    // Users can change `icon` to any emoji / short string.
                    let label = format!("{}  {}", w.props.icon, w.props.text);
                    ui.add_sized(w.size, egui::Button::new(label));
                }
                WidgetKind::Checkbox => {
                    let mut checked = w.props.checked;
                    ui.add_sized(w.size, egui::Checkbox::new(&mut checked, &w.props.text));
                    w.props.checked = checked;
                }
                WidgetKind::TextEdit => {
                    let mut buf = w.props.text.clone();
                    let resp = egui::TextEdit::singleline(&mut buf).hint_text("text");
                    ui.add_sized(w.size, resp);
                    w.props.text = buf;
                }
                WidgetKind::Slider => {
                    let mut v = w.props.value;
                    let slider =
                        egui::Slider::new(&mut v, w.props.min..=w.props.max).text(&w.props.text);
                    ui.add_sized(w.size, slider);
                    w.props.value = v;
                }
                WidgetKind::ProgressBar => {
                    let bar =
                        egui::ProgressBar::new(w.props.value.clamp(0.0, 1.0)).show_percentage();
                    ui.add_sized(w.size, bar);
                }
                WidgetKind::RadioGroup => {
                    let mut sel = w.props.selected.min(w.props.items.len().saturating_sub(1));
                    ui.vertical(|ui| {
                        for (i, it) in w.props.items.iter().enumerate() {
                            if ui.add(egui::RadioButton::new(sel == i, it)).clicked() {
                                sel = i;
                            }
                        }
                    });
                    w.props.selected = sel;
                }
                WidgetKind::Link => {
                    let _ = ui.link(&w.props.text);
                }
                WidgetKind::Hyperlink => {
                    ui.hyperlink_to(&w.props.text, &w.props.url);
                }
                WidgetKind::SelectableLabel => {
                    let mut on = w.props.checked;
                    if ui
                        .add(egui::Button::selectable(on, &w.props.text))
                        .clicked()
                    {
                        on = !on;
                    }
                    w.props.checked = on;
                }
                WidgetKind::ComboBox => {
                    let items = if w.props.items.is_empty() {
                        vec!["Item".into()]
                    } else {
                        w.props.items.clone()
                    };
                    let mut sel = w.props.selected.min(items.len() - 1);
                    egui::ComboBox::from_id_salt(w.id)
                        .width(w.size.x)
                        .selected_text(items[sel].clone())
                        .show_ui(ui, |ui| {
                            for (i, it) in items.iter().enumerate() {
                                ui.selectable_value(&mut sel, i, it.clone());
                            }
                        });
                    w.props.selected = sel;
                }
                WidgetKind::Separator => {
                    ui.separator();
                }
                WidgetKind::CollapsingHeader => {
                    egui::CollapsingHeader::new(&w.props.text)
                        .default_open(w.props.checked)
                        .show(ui, |ui| {
                            ui.label("… place your inner content here …");
                        });
                }
                WidgetKind::DatePicker => {
                    let mut date = NaiveDate::from_ymd_opt(
                        w.props.year,
                        w.props.month.clamp(1, 12),
                        w.props.day.clamp(1, 28), // simple clamp
                    )
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
                    ui.horizontal(|ui| {
                        ui.label(&w.props.text);
                        ui.add(DatePickerButton::new(&mut date));
                    });
                    w.props.year = date.year();
                    w.props.month = date.month();
                    w.props.day = date.day();
                }
                WidgetKind::AngleSelector => {
                    // Angle editor as slider in degrees
                    let mut v = w.props.value.clamp(w.props.min, w.props.max);
                    let slider = egui::Slider::new(&mut v, w.props.min..=w.props.max)
                        .suffix("°")
                        .text(&w.props.text);
                    ui.add_sized(w.size, slider);
                    w.props.value = v;
                }
                WidgetKind::Password => {
                    let mut buf = w.props.text.clone();
                    let resp = egui::TextEdit::singleline(&mut buf)
                        .password(true)
                        .hint_text("password");
                    ui.add_sized(w.size, resp);
                    w.props.text = buf;
                }
                WidgetKind::Tree => {
                    // Parse items (two leading spaces per level) into nodes:
                    #[derive(Clone)]
                    struct Node {
                        label: String,
                        children: Vec<Node>,
                    }

                    fn parse_nodes(lines: &[String]) -> Vec<Node> {
                        // (indent, label)
                        let mut items: Vec<(usize, String)> = lines
                            .iter()
                            .map(|s| {
                                let indent = s.chars().take_while(|c| *c == ' ').count() / 2;
                                (indent, s.trim().to_string())
                            })
                            .collect();
                        // Remove empties
                        items.retain(|(_, s)| !s.is_empty());

                        fn build<I: Iterator<Item = (usize, String)>>(
                            iter: &mut std::iter::Peekable<I>,
                            level: usize,
                        ) -> Vec<Node> {
                            let mut out = Vec::new();
                            while let Some((ind, _)) = iter.peek().cloned() {
                                if ind < level {
                                    break;
                                }
                                if ind > level {
                                    // child of previous; let outer loop handle
                                    break;
                                }
                                // ind == level
                                let (_, label) = iter.next().unwrap();
                                // gather children (ind + 1)
                                let children = build(iter, level + 1);
                                out.push(Node { label, children });
                            }
                            out
                        }

                        let mut it = items.into_iter().peekable();
                        build(&mut it, 0)
                    }

                    fn show_nodes(ui: &mut egui::Ui, nodes: &[Node]) {
                        for n in nodes {
                            if n.children.is_empty() {
                                ui.label(&n.label);
                            } else {
                                ui.collapsing(&n.label, |ui| {
                                    show_nodes(ui, &n.children);
                                });
                            }
                        }
                    }

                    let lines = if w.props.items.is_empty() {
                        vec!["Root".into(), "  Child".into()]
                    } else {
                        w.props.items.clone()
                    };
                    let nodes = parse_nodes(&lines);

                    // Constrain content to the widget rect:
                    egui::Frame::NONE.show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                show_nodes(ui, &nodes);
                            });
                    });
                }
            }
        });
        let is_edit_mode = ui
            .ctx()
            .data(|d| d.get_temp::<bool>(Id::new("edit_mode")))
            .unwrap_or(true);
        let painter = ui.painter();
        let stroke = if *selected == Some(w.id) {
            Stroke::new(2.0, Color32::LIGHT_BLUE)
        } else {
            Stroke::new(1.0, Color32::from_gray(90))
        };
        painter.rect_stroke(
            rect,
            CornerRadius::same(6),
            stroke,
            egui::StrokeKind::Outside,
        );
        if is_edit_mode {
            let pad = 6.0;
            let expanded = rect.expand(pad);
            let top = Rect::from_min_max(expanded.min, pos2(expanded.max.x, rect.min.y));
            let bottom = Rect::from_min_max(pos2(expanded.min.x, rect.max.y), expanded.max);
            let left = Rect::from_min_max(
                pos2(expanded.min.x, rect.min.y),
                pos2(rect.min.x, rect.max.y),
            );
            let right = Rect::from_min_max(
                pos2(rect.max.x, rect.min.y),
                pos2(expanded.max.x, rect.max.y),
            );

            let mut any_clicked = false;
            let mut drag_delta = egui::Vec2::ZERO;
            for (i, edge) in [top, right, bottom, left].into_iter().enumerate() {
                let id = ui.make_persistent_id(("edge", w.id, i as u8));
                let resp = ui.interact(edge, id, Sense::click_and_drag());
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                }
                if resp.clicked() {
                    any_clicked = true;
                }
                if resp.dragged() {
                    drag_delta += resp.drag_delta();
                }
            }
            if any_clicked {
                *selected = Some(w.id);
            }
            if drag_delta != egui::Vec2::ZERO {
                w.pos += drag_delta;
                w.pos = snap_pos_with_grid(w.pos, grid);
                let maxx = (canvas_rect.width() - w.size.x).max(0.0);
                let maxy = (canvas_rect.height() - w.size.y).max(0.0);
                w.pos.x = w.pos.x.clamp(0.0, maxx);
                w.pos.y = w.pos.y.clamp(0.0, maxy);
            }

            // resize handle unchanged, plus clamp
            let handle = {
                let hs = 12.0;
                Rect::from_min_size(expanded.max - vec2(hs, hs), vec2(hs, hs))
            };
            let rid = ui.make_persistent_id(("resize", w.id));
            let rresp = ui.interact(handle, rid, Sense::click_and_drag());
            if rresp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
            }
            if rresp.dragged() {
                let delta = rresp.drag_delta();
                w.size += delta;
                w.size.x = w.size.x.max(20.0).min(canvas_rect.width());
                w.size.y = w.size.y.max(16.0).min(canvas_rect.height());
            }
            ui.painter()
                .rect_filled(handle, 2.0, Color32::from_rgb(100, 160, 255));
        }
    }

    fn snap_pos(&self, p: Pos2) -> Pos2 {
        pos2(
            (p.x / self.grid_size).round() * self.grid_size,
            (p.y / self.grid_size).round() * self.grid_size,
        )
    }

    fn palette_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Palette");
        ui.separator();
        ui.label("Drag any control onto the canvas");
        ui.add_space(8.0);

        self.palette_item(ui, "Menu Button", WidgetKind::MenuButton);
        self.palette_item(ui, "Label", WidgetKind::Label);
        self.palette_item(ui, "Button", WidgetKind::Button);
        self.palette_item(ui, "Image + Text Button", WidgetKind::ImageTextButton);
        self.palette_item(ui, "Checkbox", WidgetKind::Checkbox);
        self.palette_item(ui, "TextEdit", WidgetKind::TextEdit);
        self.palette_item(ui, "Slider", WidgetKind::Slider);
        self.palette_item(ui, "ProgressBar", WidgetKind::ProgressBar);
        self.palette_item(ui, "Radio Group", WidgetKind::RadioGroup);
        self.palette_item(ui, "Link", WidgetKind::Link);
        self.palette_item(ui, "Hyperlink", WidgetKind::Hyperlink);
        self.palette_item(ui, "Selectable Label", WidgetKind::SelectableLabel);
        self.palette_item(ui, "Combo Box", WidgetKind::ComboBox);
        self.palette_item(ui, "Separator", WidgetKind::Separator);
        self.palette_item(ui, "Collapsing Header", WidgetKind::CollapsingHeader);
        self.palette_item(ui, "Date Picker", WidgetKind::DatePicker);
        self.palette_item(ui, "Angle Selector", WidgetKind::AngleSelector);
        self.palette_item(ui, "Password", WidgetKind::Password);
        self.palette_item(ui, "Tree", WidgetKind::Tree);

        ui.separator();
        ui.label("Tips:");
        ui.small(
            "• Click frame around control to select it\n• Drag to move, drag the corner to resize\n• Snap-to-grid can be changed in Settings",
        );
    }

    fn palette_item(&mut self, ui: &mut egui::Ui, label: &str, kind: WidgetKind) {
        let r = ui.add(egui::Button::new(label).sense(Sense::drag()));
        if r.drag_started() || r.clicked() {
            self.spawning = Some(kind);
        }
    }

    fn inspector_ui(&mut self, ui: &mut egui::Ui) {
        let grid = self.grid_size; // read before mutably borrowing self
        let mut duplicate = None;

        ui.heading("Inspector");
        ui.separator();
        if let Some(w) = Self::selected_mut(self.selected, &mut self.project.widgets) {
            ui.label(format!("ID: {:?}", w.id));
            ui.add_space(6.0);
            match w.kind {
                WidgetKind::Label
                // todo add click type(eg. click, release) and onclick function
                | WidgetKind::Button
                | WidgetKind::ImageTextButton
                | WidgetKind::TextEdit
                | WidgetKind::Checkbox
                | WidgetKind::Slider
                | WidgetKind::Link
                | WidgetKind::Hyperlink
                | WidgetKind::SelectableLabel
                | WidgetKind::CollapsingHeader
                | WidgetKind::Password
                | WidgetKind::AngleSelector
                | WidgetKind::DatePicker => {
                    ui.label("Text");
                    ui.text_edit_singleline(&mut w.props.text);
                }
                WidgetKind::ProgressBar
                | WidgetKind::RadioGroup
                | WidgetKind::ComboBox
                | WidgetKind::Tree
                | WidgetKind::Separator => {}
                WidgetKind::MenuButton => {
                    ui.label("Text");
                    ui.text_edit_singleline(&mut w.props.text);
                }
            }
            match w.kind {
                WidgetKind::ImageTextButton => {
                    ui.label("Icon / Emoji");
                    ui.text_edit_singleline(&mut w.props.icon);
                }
                WidgetKind::Checkbox => {
                    ui.checkbox(&mut w.props.checked, "checked");
                }
                WidgetKind::Slider => {
                    ui.add(
                        egui::Slider::new(&mut w.props.value, w.props.min..=w.props.max)
                            .text("value"),
                    );
                    ui.add(egui::Slider::new(&mut w.props.min, -1000.0..=w.props.max).text("min"));
                    ui.add(egui::Slider::new(&mut w.props.max, w.props.min..=1000.0).text("max"));
                }
                WidgetKind::ProgressBar => {
                    ui.add(egui::Slider::new(&mut w.props.value, 0.0..=1.0).text("progress"));
                }
                WidgetKind::Hyperlink => {
                    ui.label("URL");
                    ui.text_edit_singleline(&mut w.props.url);
                }
                WidgetKind::RadioGroup
                | WidgetKind::ComboBox
                | WidgetKind::Tree
                | WidgetKind::MenuButton => {
                    ui.label(match w.kind {
                        WidgetKind::Tree => "Nodes (indent with spaces; 2 spaces per level)",
                        _ => "Items (one per line)",
                    });
                    let mut buf = w.props.items.join("\n");
                    if ui
                        .add(
                            egui::TextEdit::multiline(&mut buf)
                                .desired_rows(8)
                                .desired_width(f32::INFINITY),
                        )
                        .changed()
                    {
                        w.props.items = buf.lines().map(|s| s.to_string()).collect();
                        if w.props.selected >= w.props.items.len() {
                            w.props.selected = w.props.items.len().saturating_sub(1);
                        }
                    }
                    if !matches!(w.kind, WidgetKind::Tree) && !w.props.items.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("Selected index");
                            ui.add(
                                egui::DragValue::new(&mut w.props.selected)
                                    .range(0..=w.props.items.len().saturating_sub(1)),
                            );
                        });
                    }
                }
                WidgetKind::CollapsingHeader => {
                    ui.checkbox(&mut w.props.checked, "open by default");
                }
                WidgetKind::DatePicker => {
                    ui.horizontal(|ui| {
                        ui.label("Year");
                        ui.add(egui::DragValue::new(&mut w.props.year));
                        ui.label("Month");
                        ui.add(egui::DragValue::new(&mut w.props.month).range(1..=12));
                        ui.label("Day");
                        ui.add(egui::DragValue::new(&mut w.props.day).range(1..=31));
                    });
                }
                WidgetKind::AngleSelector => {
                    ui.add(
                        egui::Slider::new(&mut w.props.value, w.props.min..=w.props.max)
                            .text("value (deg)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut w.props.min, -1080.0..=w.props.max)
                            .text("min (deg)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut w.props.max, w.props.min..=1080.0).text("max (deg)"),
                    );
                }
                WidgetKind::Password => { /* no extra props */ }
                _ => {}
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Area");
                let mut area = w.area;
                egui::ComboBox::from_id_salt(("area", w.id))
                    .selected_text(format!("{:?}", area))
                    .show_ui(ui, |ui| {
                        for a in [
                            DockArea::Free,
                            DockArea::Top,
                            DockArea::Bottom,
                            DockArea::Left,
                            DockArea::Right,
                            DockArea::Center,
                        ] {
                            ui.selectable_value(&mut area, a, format!("{:?}", a));
                        }
                    });
                if area != w.area {
                    w.area = area;
                    // reset pos within new area (keeps roughly same coords snapped)
                    w.pos = snap_pos_with_grid(w.pos, grid);
                }
            });
            ui.add_space(2.0);
            ui.label("Position");
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::DragValue::new(&mut w.pos.x).range(0..=self.project.canvas_size.x.ceil() as u32));
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut w.pos.y).range(0..=self.project.canvas_size.y.ceil() as u32));
            });
            ui.label("Size");
            ui.horizontal(|ui| {
                ui.label("W:");
                ui.add(egui::DragValue::new(&mut w.size.x).range(16.0..=2000.0));
                ui.label("H:");
                ui.add(egui::DragValue::new(&mut w.size.y).range(12.0..=2000.0));
            });

            ui.add_space(6.0);
            // Duplicate button
            if ui.button("Duplicate").on_hover_cursor(egui::CursorIcon::Copy).clicked() {
                let mut new_w = w.clone();
                new_w.id = WidgetId::new(self.next_id);
                // move above
                new_w.z += 1;
                // move down to the right
                new_w.pos.x += 26.0;
                new_w.pos.y += 26.0;
                // add to widgets
                duplicate = Some(new_w);

                self.selected = None;
            }

            ui.add_space(6.0);
            // Delete button
            if ui.add(Button::new("Delete").fill(Color32::from_rgb(210, 0, 0))).clicked() {
                let id = w.id; // capture
                self.project.widgets.retain(|w| w.id != id);
                self.selected = None;
            }
        } else {
            ui.weak("No selection");
        }

        if let Some(new_w) = duplicate {
            self.selected = Some(new_w.id);

            // add to widgets
            self.project.widgets.push(new_w);
            self.next_id += 1;
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Generate Code").clicked() {
                    self.generate_code();
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Export JSON").clicked() {
                    self.open_modal = Some(OpenModal::Export);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Import JSON").clicked() {
                    self.open_modal = Some(OpenModal::Import);
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui.button("Clear Project").clicked() {
                    self.project = Project::default();
                    self.selected = None;
                    ui.close_kind(egui::UiKind::Menu);
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.palette_open, "Show Palette");
            });
            ui.menu_button("Settings", |ui| {
                ui.checkbox(&mut self.show_grid, "Show grid");
                ui.horizontal(|ui| {
                    ui.label("Grid");
                    ui.add(egui::DragValue::new(&mut self.grid_size).range(1.0..=64.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Canvas size");
                    ui.add(egui::DragValue::new(&mut self.project.canvas_size.x));
                    ui.add(egui::DragValue::new(&mut self.project.canvas_size.y));
                });
                ui.separator();
                ui.strong("Panels");
                ui.add_space(4.0);
                ui.checkbox(&mut self.project.panel_top_enabled, "Top");
                ui.checkbox(&mut self.project.panel_bottom_enabled, "Bottom");
                ui.checkbox(&mut self.project.panel_left_enabled, "Left");
                ui.checkbox(&mut self.project.panel_right_enabled, "Right");
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Generate Code").clicked() {
                    self.generate_code();
                }
                ui.separator();
                ui.strong("egui RAD GUI Builder");
            });
        });
    }

    fn modals(&mut self, ctx: &egui::Context) {
        let Some(ref open_modal) = self.open_modal else { return; };

        fn default_modal(id: Id, ctx: &egui::Context) -> Modal {
            let view_rect_height = ctx.content_rect().height().abs();
            // 5% + 10px padding on top and bottom
            let width = view_rect_height * 0.9 - 10.0;
            Modal {
                area: Modal::default_area(id.into()).default_size(vec2(550.0, width)),
                backdrop_color: Color32::from_black_alpha(100),
                frame: None,
            }
        }

        let mut close_modal = false;
        let modal = match open_modal {
            OpenModal::Import => default_modal("Import JSON modal".into(), ctx).show(ctx, |ui| {
                    // todo add text box for alt input

                    // todo add icon
                    if ui.button("Paste JSON").clicked() {
                        let mut ctx = copypasta::ClipboardContext::new().unwrap();
                        if let Ok(paste_str) = ctx.get_contents()
                        && let Ok(p) = serde_json::from_str::<Project>(&paste_str) {
                            self.project = p;
                            self.selected = None;
                            // update the next_id to be correct
                            self.next_id = self.project.widgets.iter().map(|w| w.id.value()).max()
                                .map(|id| id + 1).unwrap_or(0);

                            self.generated.clear();
                        }
                        close_modal = true;
                    }
                }),
            OpenModal::Export => default_modal("Export JSON modal".into(), ctx).show(ctx, |ui| {
                    if let Ok(ex_json) = serde_json::to_string_pretty(&self.project) {
                        // todo add icon
                        if ui.button("Copy Exported").clicked() {
                            ui.ctx().copy_text(ex_json.clone());
                        }

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                egui_extras::syntax_highlighting::code_view_ui(
                                    ui,
                                    &CodeTheme::from_style(ui.style()),
                                    &ex_json,
                                    "json"
                                );
                            });
                    } else { ui.label("Failed to serialize project to json"); }
                }),
        };

        if close_modal || modal.should_close() {
            self.open_modal = None;
        }
    }

    fn generated_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Generated Output");
        ui.label("Rust code. Copy-paste into your app.");

        // todo add icon
        if ui.button("Copy code").clicked() {
            ui.ctx().copy_text(self.generated.clone());
        }

        // A scrollable viewport for the generated text:
        egui::ScrollArea::vertical()
            .id_salt("generated_output_scroll")
            .max_height(280.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui_extras::syntax_highlighting::code_view_ui(ui, &CodeTheme::from_style(ui.style()), &self.generated, "rs");
            });
    }

    fn generate_code(&mut self) {
        use DockArea::*;

        self.generated.clear();
        let mut out = &mut self.generated;
        out.push_str("// --- generated by egui RAD GUI Builder ---\n");
        out.push_str("use eframe::egui;\n");
        // todo only add if used by code
        out.push_str("use egui_extras::DatePickerButton;\n");
        out.push_str("use chrono::NaiveDate;\n\n");

        let has_tree = self
            .project
            .widgets
            .iter()
            .any(|w| matches!(w.kind, WidgetKind::Tree));
        if has_tree {
            out.push_str(
                "#[derive(Clone)]\n\
				 struct GenTreeNode { label: String, children: Vec<GenTreeNode> }\n\
				 \n\
				 fn gen_show_tree(ui: &mut egui::Ui, nodes: &[GenTreeNode]) {\n\
				 \tfor n in nodes {\n\
				 \t\tif n.children.is_empty() { ui.label(&n.label); }\n\
				 \t\telse { ui.collapsing(&n.label, |ui| gen_show_tree(ui, &n.children)); }\n\
				 \t}\n\
				 }\n\n",
            );
        }

        out.push_str("struct GeneratedState {\n");
        out.push_str(&format!(
            "    enable_top: bool, enable_bottom: bool, enable_left: bool, enable_right: bool,\n"
        ));
        for w in &self.project.widgets {
            match w.kind {
                WidgetKind::TextEdit => out.push_str(&format!("    text_{}: String,\n", w.id)),
                WidgetKind::Checkbox => out.push_str(&format!("    checked_{}: bool,\n", w.id)),
                WidgetKind::Slider => out.push_str(&format!("    value_{}: f32,\n", w.id)),
                WidgetKind::ProgressBar => out.push_str(&format!("    progress_{}: f32,\n", w.id)),
                WidgetKind::SelectableLabel => out.push_str(&format!("    sel_{}: bool,\n", w.id)),
                WidgetKind::RadioGroup | WidgetKind::ComboBox | WidgetKind::MenuButton => {
                    out.push_str(&format!("    sel_{}: usize,\n", w.id))
                }
                WidgetKind::CollapsingHeader => {
                    out.push_str(&format!("    open_{}: bool,\n", w.id))
                }
                WidgetKind::DatePicker => out.push_str(&format!("    date_{}: NaiveDate,\n", w.id)),
                WidgetKind::Password => out.push_str(&format!("    pass_{}: String,\n", w.id)),
                WidgetKind::AngleSelector => out.push_str(&format!("    angle_{}: f32,\n", w.id)),
                _ => {}
            }
        }
        out.push_str("}\n\n");

        out.push_str("impl Default for GeneratedState {\n");
        out.push_str("    fn default() -> Self {\n");
        out.push_str("        Self {\n");
        out.push_str(&format!(
            "            enable_top: {}, enable_bottom: {}, enable_left: {}, enable_right: {},\n",
            if self.project.panel_top_enabled {
                "true"
            } else {
                "false"
            },
            if self.project.panel_bottom_enabled {
                "true"
            } else {
                "false"
            },
            if self.project.panel_left_enabled {
                "true"
            } else {
                "false"
            },
            if self.project.panel_right_enabled {
                "true"
            } else {
                "false"
            },
        ));

        for w in &self.project.widgets {
            match w.kind {
                WidgetKind::TextEdit => {
                    out.push_str(&format!(
                        "            text_{}: \"{}\".to_owned(),\n",
                        w.id,
                        widget::escape(&w.props.text)
                    ));
                }
                WidgetKind::Checkbox => {
                    out.push_str(&format!(
                        "            checked_{}: {},\n",
                        w.id,
                        if w.props.checked { "true" } else { "false" }
                    ));
                }
                WidgetKind::Slider => {
                    out.push_str(&format!(
                        "            value_{}: {:.3},\n",
                        w.id, w.props.value
                    ));
                }
                WidgetKind::ProgressBar => {
                    let p = w.props.value.clamp(0.0, 1.0);
                    out.push_str(&format!("            progress_{}: {:.3},\n", w.id, p));
                }
                WidgetKind::SelectableLabel => {
                    out.push_str(&format!(
                        "            sel_{}: {},\n",
                        w.id,
                        if w.props.checked { "true" } else { "false" }
                    ));
                }
                WidgetKind::RadioGroup | WidgetKind::ComboBox | WidgetKind::MenuButton => {
                    let sel = if w.props.items.is_empty() {
                        0
                    } else {
                        w.props.selected.min(w.props.items.len() - 1)
                    };
                    out.push_str(&format!("            sel_{}: {},\n", w.id, sel));
                }
                WidgetKind::CollapsingHeader => {
                    out.push_str(&format!(
                        "            open_{}: {},\n",
                        w.id,
                        if w.props.checked { "true" } else { "false" }
                    ));
                }
                WidgetKind::DatePicker => {
                    let y = w.props.year;
                    let m = w.props.month.clamp(1, 12);
                    let d = w.props.day.clamp(1, 28);
                    out.push_str(&format!(
                        "            date_{}: NaiveDate::from_ymd_opt({}, {}, {}).unwrap(),\n",
                        w.id, y, m, d
                    ));
                }
                WidgetKind::Password => {
                    out.push_str(&format!(
                        "            pass_{}: \"{}\".to_owned(),\n",
                        w.id,
                        widget::escape(&w.props.text)
                    ));
                }
                WidgetKind::AngleSelector => {
                    out.push_str(&format!(
                        "            angle_{}: {:.3},\n",
                        w.id, w.props.value
                    ));
                }
                _ => {}
            }
        }
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        let mut top = Vec::new();
        let mut bottom = Vec::new();
        let mut left = Vec::new();
        let mut right = Vec::new();
        let mut center = Vec::new();
        let mut free = Vec::new();
        for w in &self.project.widgets {
            match w.area {
                Top => top.push(w),
                Bottom => bottom.push(w),
                Left => left.push(w),
                Right => right.push(w),
                Center => center.push(w),
                Free => free.push(w),
            }
        }

        out.push_str("fn generated_ui(ctx: &egui::Context, state: &mut GeneratedState) {\n");

        // TOP
        out.push_str("    if state.enable_top {\n");
        out.push_str("        egui::TopBottomPanel::top(\"gen_top\")\n");
        out.push_str("            .resizable(true)\n");
        out.push_str("            .show(ctx, |ui| {\n");
        for w in top {
            w.emit_widget(&mut out, "ui.min_rect().min");
        }
        out.push_str("            });\n");
        out.push_str("    }\n");

        // BOTTOM
        out.push_str("    if state.enable_bottom {\n");
        out.push_str("        egui::TopBottomPanel::bottom(\"gen_bottom\")\n");
        out.push_str("            .resizable(true)\n");
        out.push_str("            .show(ctx, |ui| {\n");
        for w in bottom {
            w.emit_widget(&mut out, "ui.min_rect().min");
        }
        out.push_str("            });\n");
        out.push_str("    }\n");

        // LEFT
        out.push_str("    if state.enable_left {\n");
        out.push_str("        egui::SidePanel::left(\"gen_left\")\n");
        out.push_str("            .resizable(true)\n");
        out.push_str("            .show(ctx, |ui| {\n");
        for w in left {
            w.emit_widget(&mut out, "ui.min_rect().min");
        }
        out.push_str("            });\n");
        out.push_str("    }\n");

        // RIGHT
        out.push_str("    if state.enable_right {\n");
        out.push_str("        egui::SidePanel::right(\"gen_right\")\n");
        out.push_str("            .resizable(true)\n");
        out.push_str("            .show(ctx, |ui| {\n");
        for w in right {
            w.emit_widget(&mut out, "ui.min_rect().min");
        }
        out.push_str("            });\n");
        out.push_str("    }\n");

        // CENTER (+ FREE): use CentralPanel; widgets are placed absolutely within it.
        out.push_str("    egui::CentralPanel::default().show(ctx, |ui| {\n");
        // fixed logical canvas (keeps your designed size)
        out.push_str(&format!(
			"        let canvas = egui::Rect::from_min_size(ui.min_rect().min, egui::vec2({:.1}, {:.1}));\n",
			self.project.canvas_size.x, self.project.canvas_size.y
		));
        out.push_str("        let _ = ui.allocate_painter(canvas.size(), egui::Sense::hover());\n");
        for w in center {
            w.emit_widget(&mut out, "canvas.min");
        }
        for w in free {
            w.emit_widget(&mut out, "canvas.min");
        }
        out.push_str("    });\n");

        out.push_str("}\n\n");

        // ---------- Example eframe app (updated to call generated_ui with ctx) ----------
        out.push_str(
			"pub struct GeneratedApp { state: GeneratedState }\n\
			 impl Default for GeneratedApp { fn default() -> Self { Self { state: Default::default() } } }\n\
			 impl eframe::App for GeneratedApp {\n\
			 \tfn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {\n\
			 \t\tgenerated_ui(ctx, &mut self.state);\n\
			 \t}\n\
			 }\n\n\
			 fn main() -> eframe::Result<()> {\n\
			 \tlet native_options = eframe::NativeOptions::default();\n\
			 \teframe::run_native(\"Generated UI\", native_options, Box::new(|_cc| Ok(Box::new(GeneratedApp::default()))))\n\
			 }\n",
		);
    }
}

impl eframe::App for RadBuilderApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| self.top_bar(ui));
        if self.palette_open {
            egui::SidePanel::left("palette")
                .resizable(true)
                .show(ctx, |ui| {
                    self.palette_ui(ui);
                });
        }
        egui::SidePanel::right("inspector")
            .default_width(260.0)
            .show(ctx, |ui| {
                self.inspector_ui(ui);
                ui.separator();
                self.generated_panel(ui);
            });

        self.preview_panels_ui(ctx);

        if self.spawning.is_some() {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        }

        // show modals
        self.modals(ctx);
    }
}
