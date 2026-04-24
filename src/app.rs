use crate::{
    highlight::Highlighter,
    project::Project,
    widget::{self, DockArea, Widget, WidgetId, WidgetKind, escape, snap_pos_with_grid},
};
use chrono::{Datelike, NaiveDate};
use egui::{Color32, CornerRadius, Id, Pos2, Rect, Sense, Stroke, UiBuilder, pos2, vec2};
use egui_extras::DatePickerButton;
use std::path::PathBuf;

/// Code generation output format
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CodeGenFormat {
    /// Single file with all code
    #[default]
    SingleFile,
    /// Separate files: main.rs, state.rs, ui.rs
    SeparateFiles,
    /// Just the UI function (for embedding)
    UiOnly,
}

impl CodeGenFormat {
    pub const fn display_name(&self) -> &'static str {
        match self {
            CodeGenFormat::SingleFile => "Single File",
            CodeGenFormat::SeparateFiles => "Separate Files",
            CodeGenFormat::UiOnly => "UI Function Only",
        }
    }
}

pub(crate) struct RadBuilderApp {
    palette_open: bool,
    project: Project,
    /// Currently selected widgets (supports multi-select)
    selected: Vec<WidgetId>,
    next_id: u64,
    // Drag state for spawning from palette
    spawning: Option<WidgetKind>,
    // Cached generated code
    generated: String,
    // Settings
    grid_size: f32,
    show_grid: bool,
    live_top: Option<Rect>,
    live_bottom: Option<Rect>,
    live_left: Option<Rect>,
    live_right: Option<Rect>,
    live_center: Option<Rect>,
    // Clipboard for copy/paste
    clipboard: Option<Widget>,
    /// Current project file path (for Save)
    current_file: Option<PathBuf>,
    /// Error/status message to display
    status_message: Option<(String, std::time::Instant)>,
    /// Drag selection box (start position when dragging to select)
    #[allow(dead_code)]
    drag_select_start: Option<Pos2>,
    /// Syntax highlighter for code preview
    highlighter: Highlighter,
    /// Whether to show syntax highlighting (can be toggled for performance)
    syntax_highlighting: bool,
    /// Auto-generate code on widget changes
    auto_generate: bool,
    /// Code generation output format
    codegen_format: CodeGenFormat,
    /// Add comments to generated code
    codegen_comments: bool,
    /// Preview mode: interact with widgets without selection handles
    preview_mode: bool,
    /// Active tab in the right panel (0 = Inspector, 1 = Code Output)
    right_panel_tab: usize,
}

impl Default for RadBuilderApp {
    fn default() -> Self {
        Self {
            palette_open: true,
            project: Project::default(),
            selected: Vec::new(),
            next_id: 1,
            spawning: None,
            generated: String::new(),
            grid_size: 1.0,
            show_grid: false,
            live_top: None,
            live_bottom: None,
            live_left: None,
            live_right: None,
            live_center: None,
            clipboard: None,
            current_file: None,
            status_message: None,
            drag_select_start: None,
            highlighter: Highlighter::new(),
            syntax_highlighting: true,
            auto_generate: false,
            codegen_format: CodeGenFormat::default(),
            codegen_comments: true,
            preview_mode: false,
            right_panel_tab: 0,
        }
    }
}

impl RadBuilderApp {
    fn area_at(&self, pos: Pos2) -> DockArea {
        if let Some(r) = self.live_top
            && r.contains(pos)
        {
            return DockArea::Top;
        }
        if let Some(r) = self.live_bottom
            && r.contains(pos)
        {
            return DockArea::Bottom;
        }
        if let Some(r) = self.live_left
            && r.contains(pos)
        {
            return DockArea::Left;
        }
        if let Some(r) = self.live_right
            && r.contains(pos)
        {
            return DockArea::Right;
        }
        if let Some(r) = self.live_center
            && r.contains(pos)
        {
            return DockArea::Center;
        }
        DockArea::Free
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

        // Use centralized default_size and default_props from WidgetKind
        let size = kind.default_size();
        let props = kind.default_props();

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
        self.selected = vec![id];
    }

    /// Returns the first selected widget for editing (inspector uses this)
    fn selected_mut(&mut self) -> Option<&mut Widget> {
        let id = *self.selected.first()?;
        self.project.widgets.iter_mut().find(|w| w.id == id)
    }

    /// Check if a widget is selected
    #[allow(dead_code)]
    fn is_selected(&self, id: WidgetId) -> bool {
        self.selected.contains(&id)
    }

    /// Select a single widget (clears other selections)
    #[allow(dead_code)]
    fn select_single(&mut self, id: WidgetId) {
        self.selected = vec![id];
    }

    /// Toggle selection of a widget (for Shift+click multi-select)
    #[allow(dead_code)]
    fn toggle_selection(&mut self, id: WidgetId) {
        if let Some(pos) = self.selected.iter().position(|&x| x == id) {
            self.selected.remove(pos);
        } else {
            self.selected.push(id);
        }
    }

    /// Add widget to selection (for drag box select)
    #[allow(dead_code)]
    fn add_to_selection(&mut self, id: WidgetId) {
        if !self.selected.contains(&id) {
            self.selected.push(id);
        }
    }

    /// Clear all selections
    #[allow(dead_code)]
    fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// Save project to file
    fn save_project(&mut self, path: PathBuf) {
        match serde_json::to_string_pretty(&self.project) {
            Ok(json) => match std::fs::write(&path, &json) {
                Ok(_) => {
                    self.current_file = Some(path.clone());
                    self.set_status(format!("Saved to {}", path.display()));
                }
                Err(e) => self.set_status(format!("Save failed: {}", e)),
            },
            Err(e) => self.set_status(format!("Serialization failed: {}", e)),
        }
    }

    /// Load project from file
    fn load_project(&mut self, path: PathBuf) {
        match std::fs::read_to_string(&path) {
            Ok(json) => {
                match serde_json::from_str::<Project>(&json) {
                    Ok(project) => {
                        // Find max widget id to continue numbering
                        let max_id = project.widgets.iter().map(|w| w.id).max();
                        if let Some(id) = max_id {
                            self.next_id = id.as_z() as u64 + 1;
                        }
                        self.project = project;
                        self.selected.clear();
                        self.current_file = Some(path.clone());
                        self.set_status(format!("Loaded {}", path.display()));
                    }
                    Err(e) => self.set_status(format!("Parse failed: {}", e)),
                }
            }
            Err(e) => self.set_status(format!("Load failed: {}", e)),
        }
    }

    /// Set a status message that will auto-clear after a few seconds
    fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, std::time::Instant::now()));
    }

    /// Get widgets in selection rect (for drag-box selection)
    #[allow(dead_code)]
    fn widgets_in_rect(&self, rect: Rect, area_origin: Pos2) -> Vec<WidgetId> {
        self.project
            .widgets
            .iter()
            .filter(|w| {
                let widget_rect = Rect::from_min_size(area_origin + w.pos.to_vec2(), w.size);
                rect.intersects(widget_rect)
            })
            .map(|w| w.id)
            .collect()
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
            if let Some(kind) = self.spawning {
                if let Some(mouse) = ui.ctx().pointer_interact_pos() {
                    // Use centralized default_size from WidgetKind
                    let ghost_size = kind.default_size();
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
                self.selected.clear();
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
        selected: &mut Vec<WidgetId>,
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
                WidgetKind::TextArea => {
                    let mut buf = w.props.text.clone();
                    let resp = egui::TextEdit::multiline(&mut buf)
                        .desired_width(w.size.x)
                        .desired_rows(5);
                    ui.add_sized(w.size, resp);
                    w.props.text = buf;
                }
                WidgetKind::DragValue => {
                    let mut v = w.props.value;
                    ui.horizontal(|ui| {
                        ui.label(&w.props.text);
                        ui.add(egui::DragValue::new(&mut v).range(w.props.min..=w.props.max));
                    });
                    w.props.value = v;
                }
                WidgetKind::Spinner => {
                    ui.add(egui::Spinner::new());
                }
                WidgetKind::ColorPicker => {
                    let mut color = Color32::from_rgba_unmultiplied(
                        w.props.color[0],
                        w.props.color[1],
                        w.props.color[2],
                        w.props.color[3],
                    );
                    ui.horizontal(|ui| {
                        ui.label(&w.props.text);
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut color,
                            egui::color_picker::Alpha::OnlyBlend,
                        );
                    });
                    w.props.color = [color.r(), color.g(), color.b(), color.a()];
                }
                WidgetKind::Code => {
                    let mut buf = w.props.text.clone();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut buf)
                                    .code_editor()
                                    .desired_width(w.size.x)
                                    .desired_rows(8),
                            );
                        });
                    w.props.text = buf;
                }
                WidgetKind::Heading => {
                    ui.heading(&w.props.text);
                }
                WidgetKind::Small => {
                    ui.small(&w.props.text);
                }
                WidgetKind::Monospace => {
                    ui.monospace(&w.props.text);
                }
                WidgetKind::Image => {
                    // Show placeholder with image info
                    let color = Color32::from_rgba_unmultiplied(80, 80, 80, 200);
                    egui::Frame::NONE
                        .fill(color)
                        .stroke(Stroke::new(1.0, Color32::GRAY))
                        .show(ui, |ui| {
                            ui.set_min_size(w.size);
                            ui.centered_and_justified(|ui| {
                                ui.label(format!(
                                    "🖼 {}\n{}x{}",
                                    w.props.text, w.size.x as i32, w.size.y as i32
                                ));
                            });
                        });
                }
                WidgetKind::Placeholder => {
                    let color = Color32::from_rgba_unmultiplied(
                        w.props.color[0],
                        w.props.color[1],
                        w.props.color[2],
                        w.props.color[3],
                    );
                    egui::Frame::NONE
                        .fill(color)
                        .stroke(Stroke::new(1.0, Color32::GRAY))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.set_min_size(w.size);
                            ui.centered_and_justified(|ui| {
                                ui.label(&w.props.text);
                            });
                        });
                }
                WidgetKind::Group => {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_min_size(w.size - vec2(12.0, 12.0));
                        let add_contents = |ui: &mut egui::Ui| {
                            if !w.props.text.is_empty() {
                                ui.strong(&w.props.text);
                                ui.separator();
                            }
                            ui.label("(group contents)");
                        };
                        if w.props.horizontal {
                            ui.horizontal(add_contents);
                        } else {
                            ui.vertical(add_contents);
                        }
                    });
                }
                WidgetKind::ScrollBox => {
                    egui::Frame::NONE
                        .stroke(Stroke::new(1.0, Color32::GRAY))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            egui::ScrollArea::both()
                                .max_width(w.size.x - 4.0)
                                .max_height(w.size.y - 4.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.label(&w.props.text);
                                });
                        });
                }
                WidgetKind::TabBar => {
                    ui.horizontal(|ui| {
                        for (i, item) in w.props.items.iter().enumerate() {
                            let selected = i == w.props.selected;
                            if ui.selectable_label(selected, item).clicked() {
                                w.props.selected = i;
                            }
                        }
                    });
                }
                WidgetKind::Columns => {
                    let cols = w.props.columns.max(1);
                    egui::Frame::NONE
                        .stroke(Stroke::new(1.0, Color32::GRAY))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.columns(cols, |columns| {
                                for (i, col) in columns.iter_mut().enumerate() {
                                    col.label(format!("Col {}", i + 1));
                                    col.label(&w.props.text);
                                }
                            });
                        });
                }
                WidgetKind::Window => {
                    egui::Frame::window(ui.style()).show(ui, |ui| {
                        ui.set_min_size(w.size - vec2(16.0, 16.0));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.strong(&w.props.text);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.small("✕");
                                    },
                                );
                            });
                            ui.separator();
                            ui.label("(window contents)");
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
        let is_selected = selected.contains(&w.id);
        let stroke = if is_selected {
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
                // Check if Shift is held for multi-select
                let shift_held = ui.ctx().input(|i| i.modifiers.shift);
                if shift_held {
                    // Toggle selection
                    if let Some(pos) = selected.iter().position(|&x| x == w.id) {
                        selected.remove(pos);
                    } else {
                        selected.push(w.id);
                    }
                } else {
                    // Single select
                    selected.clear();
                    selected.push(w.id);
                }
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
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::CollapsingHeader::new("Basic")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.palette_item(ui, "Label", WidgetKind::Label);
                        self.palette_item(ui, "Button", WidgetKind::Button);
                        self.palette_item(ui, "Image + Text Button", WidgetKind::ImageTextButton);
                        self.palette_item(ui, "Checkbox", WidgetKind::Checkbox);
                        self.palette_item(ui, "Link", WidgetKind::Link);
                        self.palette_item(ui, "Hyperlink", WidgetKind::Hyperlink);
                        self.palette_item(ui, "Selectable Label", WidgetKind::SelectableLabel);
                        self.palette_item(ui, "Separator", WidgetKind::Separator);
                    });

                egui::CollapsingHeader::new("Input")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.palette_item(ui, "TextEdit", WidgetKind::TextEdit);
                        self.palette_item(ui, "Text Area", WidgetKind::TextArea);
                        self.palette_item(ui, "Password", WidgetKind::Password);
                        self.palette_item(ui, "Slider", WidgetKind::Slider);
                        self.palette_item(ui, "Drag Value", WidgetKind::DragValue);
                        self.palette_item(ui, "Combo Box", WidgetKind::ComboBox);
                        self.palette_item(ui, "Radio Group", WidgetKind::RadioGroup);
                        self.palette_item(ui, "Date Picker", WidgetKind::DatePicker);
                        self.palette_item(ui, "Angle Selector", WidgetKind::AngleSelector);
                        self.palette_item(ui, "Color Picker", WidgetKind::ColorPicker);
                    });

                egui::CollapsingHeader::new("Display")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.palette_item(ui, "Heading", WidgetKind::Heading);
                        self.palette_item(ui, "Small", WidgetKind::Small);
                        self.palette_item(ui, "Monospace", WidgetKind::Monospace);
                        self.palette_item(ui, "ProgressBar", WidgetKind::ProgressBar);
                        self.palette_item(ui, "Spinner", WidgetKind::Spinner);
                        self.palette_item(ui, "Image", WidgetKind::Image);
                        self.palette_item(ui, "Placeholder", WidgetKind::Placeholder);
                    });

                egui::CollapsingHeader::new("Containers")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.palette_item(ui, "Group", WidgetKind::Group);
                        self.palette_item(ui, "Scroll Box", WidgetKind::ScrollBox);
                        self.palette_item(ui, "Columns", WidgetKind::Columns);
                        self.palette_item(ui, "Tab Bar", WidgetKind::TabBar);
                        self.palette_item(ui, "Window", WidgetKind::Window);
                        self.palette_item(ui, "Collapsing Header", WidgetKind::CollapsingHeader);
                    });

                egui::CollapsingHeader::new("Advanced")
                    .default_open(false)
                    .show(ui, |ui| {
                        self.palette_item(ui, "Menu Button", WidgetKind::MenuButton);
                        self.palette_item(ui, "Tree", WidgetKind::Tree);
                        self.palette_item(ui, "Code Editor", WidgetKind::Code);
                    });

                ui.add_space(8.0);
                ui.separator();
                egui::CollapsingHeader::new("Shortcuts")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.small("Arrows: nudge widget");
                        ui.small("Delete: remove");
                        ui.small("Ctrl+C/V: copy/paste");
                        ui.small("Ctrl+D: duplicate");
                        ui.small("] / [: z-order");
                        ui.small("Ctrl+G: generate");
                        ui.small("F5: toggle preview");
                    });
            });
    }

    fn palette_item(&mut self, ui: &mut egui::Ui, label: &str, kind: WidgetKind) {
        let r = ui.add(egui::Button::new(label).sense(Sense::drag()));
        if r.drag_started() || r.clicked() {
            self.spawning = Some(kind);
        }
    }

    fn inspector_ui(&mut self, ui: &mut egui::Ui) {
        let grid = self.grid_size; // read before mutably borrowing self
        ui.heading("Inspector");
        ui.separator();
        if let Some(w) = self.selected_mut() {
            ui.label(format!("ID: {:?}", w.id));
            ui.add_space(6.0);
            match w.kind {
                WidgetKind::Label
                | WidgetKind::Heading
                | WidgetKind::Small
                | WidgetKind::Monospace
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
                | WidgetKind::DatePicker
                | WidgetKind::DragValue
                | WidgetKind::ColorPicker
                | WidgetKind::Placeholder
                | WidgetKind::Group
                | WidgetKind::Window
                | WidgetKind::Columns => {
                    ui.label("Text");
                    ui.text_edit_singleline(&mut w.props.text);
                }
                WidgetKind::ProgressBar
                | WidgetKind::RadioGroup
                | WidgetKind::ComboBox
                | WidgetKind::Tree
                | WidgetKind::Separator
                | WidgetKind::Spinner
                | WidgetKind::TabBar => {}
                WidgetKind::MenuButton => {
                    ui.label("Text");
                    ui.text_edit_singleline(&mut w.props.text);
                }
                WidgetKind::TextArea | WidgetKind::Code | WidgetKind::ScrollBox => {
                    ui.label("Content");
                    ui.add(
                        egui::TextEdit::multiline(&mut w.props.text)
                            .desired_rows(6)
                            .desired_width(f32::INFINITY),
                    );
                }
                WidgetKind::Image => {
                    ui.label("Filename");
                    ui.text_edit_singleline(&mut w.props.text);
                    ui.label("URI");
                    ui.text_edit_singleline(&mut w.props.url);
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
                | WidgetKind::MenuButton
                | WidgetKind::TabBar => {
                    ui.label(match w.kind {
                        WidgetKind::Tree => "Nodes (indent with spaces; 2 spaces per level)",
                        WidgetKind::TabBar => "Tabs (one per line)",
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
                WidgetKind::DragValue => {
                    ui.add(
                        egui::Slider::new(&mut w.props.value, w.props.min..=w.props.max)
                            .text("value"),
                    );
                    ui.add(egui::Slider::new(&mut w.props.min, -1000.0..=w.props.max).text("min"));
                    ui.add(egui::Slider::new(&mut w.props.max, w.props.min..=1000.0).text("max"));
                }
                WidgetKind::ColorPicker | WidgetKind::Placeholder => {
                    let mut color = Color32::from_rgba_unmultiplied(
                        w.props.color[0],
                        w.props.color[1],
                        w.props.color[2],
                        w.props.color[3],
                    );
                    ui.horizontal(|ui| {
                        ui.label("Color");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut color,
                            egui::color_picker::Alpha::OnlyBlend,
                        );
                    });
                    w.props.color = [color.r(), color.g(), color.b(), color.a()];
                }
                WidgetKind::Group => {
                    ui.checkbox(&mut w.props.horizontal, "horizontal layout");
                }
                WidgetKind::Columns => {
                    ui.horizontal(|ui| {
                        ui.label("Columns");
                        ui.add(egui::DragValue::new(&mut w.props.columns).range(1..=10));
                    });
                }
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
            ui.label("Position / Size");
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(egui::DragValue::new(&mut w.pos.x));
                ui.label("y");
                ui.add(egui::DragValue::new(&mut w.pos.y));
            });
            ui.horizontal(|ui| {
                ui.label("w");
                ui.add(egui::DragValue::new(&mut w.size.x).range(16.0..=2000.0));
                ui.label("h");
                ui.add(egui::DragValue::new(&mut w.size.y).range(12.0..=2000.0));
            });

            ui.separator();
            ui.label("Tooltip (optional)");
            ui.text_edit_singleline(&mut w.props.tooltip);

            ui.add_space(6.0);
            if ui.button("Delete").clicked() {
                let id = w.id; // capture
                self.project.widgets.retain(|w| w.id != id);
                self.selected.clear();
            }
        } else {
            ui.weak("No selection");
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        // Show status message if recent
        if let Some((msg, time)) = &self.status_message {
            if time.elapsed().as_secs() < 3 {
                ui.horizontal(|ui| {
                    ui.label(msg);
                });
            } else {
                self.status_message = None;
            }
        }

        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui
                    .button("New Project")
                    .on_hover_text("Create a new empty project")
                    .clicked()
                {
                    self.project = Project::default();
                    self.selected.clear();
                    self.current_file = None;
                    self.set_status("New project created".into());
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                if ui
                    .button("Open...")
                    .on_hover_text("Open a project file (Ctrl+O)")
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("RAD Project", &["json", "rad"])
                        .pick_file()
                    {
                        self.load_project(path);
                    }
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui
                    .button("Save")
                    .on_hover_text("Save project (Ctrl+S)")
                    .clicked()
                {
                    if let Some(path) = self.current_file.clone() {
                        self.save_project(path);
                    } else if let Some(path) = rfd::FileDialog::new()
                        .add_filter("RAD Project", &["json", "rad"])
                        .set_file_name("project.json")
                        .save_file()
                    {
                        self.save_project(path);
                    }
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui
                    .button("Save As...")
                    .on_hover_text("Save project to a new file")
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("RAD Project", &["json", "rad"])
                        .set_file_name("project.json")
                        .save_file()
                    {
                        self.save_project(path);
                    }
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                if ui
                    .button("Generate Code")
                    .on_hover_text("Generate Rust code (Ctrl+G)")
                    .clicked()
                {
                    self.generated = self.generate_code();
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui
                    .button("Export JSON")
                    .on_hover_text("Export project as JSON to the editor")
                    .clicked()
                {
                    if let Ok(s) = serde_json::to_string_pretty(&self.project) {
                        self.generated = s;
                    }
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui
                    .button("Import JSON")
                    .on_hover_text("Import project from the editor below")
                    .clicked()
                {
                    if let Ok(p) = serde_json::from_str::<Project>(&self.generated) {
                        self.project = p;
                        self.selected.clear();
                    }
                    ui.close_kind(egui::UiKind::Menu);
                }
            });

            ui.menu_button("Edit", |ui| {
                let has_selection = !self.selected.is_empty();
                let _multi_selected = self.selected.len() > 1;

                ui.add_enabled_ui(has_selection, |ui| {
                    if ui
                        .button("Delete")
                        .on_hover_text("Delete selected (Del)")
                        .clicked()
                    {
                        let to_delete: Vec<_> = self.selected.clone();
                        self.project.widgets.retain(|w| !to_delete.contains(&w.id));
                        self.selected.clear();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui
                        .button("Duplicate")
                        .on_hover_text("Duplicate selected (Ctrl+D)")
                        .clicked()
                    {
                        // Handled in keyboard shortcuts
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui
                        .button("Copy")
                        .on_hover_text("Copy selected (Ctrl+C)")
                        .clicked()
                    {
                        if let Some(&sel_id) = self.selected.first()
                            && let Some(w) = self.project.widgets.iter().find(|w| w.id == sel_id)
                        {
                            self.clipboard = Some(w.clone());
                        }
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });
                if ui
                    .add_enabled(self.clipboard.is_some(), egui::Button::new("Paste"))
                    .on_hover_text("Paste from clipboard (Ctrl+V)")
                    .clicked()
                {
                    // Handled in keyboard shortcuts
                    ui.close_kind(egui::UiKind::Menu);
                }
                ui.separator();
                if ui
                    .button("Select All")
                    .on_hover_text("Select all widgets")
                    .clicked()
                {
                    self.selected = self.project.widgets.iter().map(|w| w.id).collect();
                    ui.close_kind(egui::UiKind::Menu);
                }
                if ui
                    .add_enabled(has_selection, egui::Button::new("Deselect All"))
                    .on_hover_text("Clear selection")
                    .clicked()
                {
                    self.selected.clear();
                    ui.close_kind(egui::UiKind::Menu);
                }
            });

            // Alignment menu (only enabled with multi-select)
            ui.menu_button("Align", |ui| {
                let multi_selected = self.selected.len() > 1;
                ui.add_enabled_ui(multi_selected, |ui| {
                    ui.label("Horizontal:");
                    if ui
                        .button("⬅ Left")
                        .on_hover_text("Align left edges")
                        .clicked()
                    {
                        self.align_left();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui
                        .button("⬌ Center")
                        .on_hover_text("Align centers horizontally")
                        .clicked()
                    {
                        self.align_center_h();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui
                        .button("➡ Right")
                        .on_hover_text("Align right edges")
                        .clicked()
                    {
                        self.align_right();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    ui.label("Vertical:");
                    if ui
                        .button("⬆ Top")
                        .on_hover_text("Align top edges")
                        .clicked()
                    {
                        self.align_top();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui
                        .button("⬍ Middle")
                        .on_hover_text("Align centers vertically")
                        .clicked()
                    {
                        self.align_center_v();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui
                        .button("⬇ Bottom")
                        .on_hover_text("Align bottom edges")
                        .clicked()
                    {
                        self.align_bottom();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    ui.label("Distribute:");
                    if ui
                        .button("↔ Horizontal")
                        .on_hover_text("Distribute evenly horizontally")
                        .clicked()
                    {
                        self.distribute_horizontal();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui
                        .button("↕ Vertical")
                        .on_hover_text("Distribute evenly vertically")
                        .clicked()
                    {
                        self.distribute_vertical();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    ui.separator();
                    ui.label("Size:");
                    if ui
                        .button("Match Width")
                        .on_hover_text("Make all same width")
                        .clicked()
                    {
                        self.match_width();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                    if ui
                        .button("Match Height")
                        .on_hover_text("Make all same height")
                        .clicked()
                    {
                        self.match_height();
                        ui.close_kind(egui::UiKind::Menu);
                    }
                });
                if !multi_selected {
                    ui.label("Select 2+ widgets to align");
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.palette_open, "Show Palette");
                ui.checkbox(&mut self.show_grid, "Show Grid");
                ui.checkbox(&mut self.syntax_highlighting, "Syntax Highlighting")
                    .on_hover_text("Enable syntax highlighting in code output");
                ui.separator();
                ui.checkbox(&mut self.preview_mode, "Preview Mode (F5)")
                    .on_hover_text("Toggle preview mode: interact with widgets without selection handles");
            });

            ui.menu_button("Settings", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Grid Size");
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
                ui.separator();
                ui.strong("Code Generation");
                ui.add_space(4.0);
                ui.checkbox(&mut self.auto_generate, "Auto-generate code")
                    .on_hover_text("Automatically regenerate code when widgets change");
                ui.checkbox(&mut self.codegen_comments, "Include comments")
                    .on_hover_text("Add explanatory comments to generated code");
                ui.horizontal(|ui| {
                    ui.label("Output format:");
                    egui::ComboBox::from_id_salt("codegen_format")
                        .selected_text(self.codegen_format.display_name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.codegen_format,
                                CodeGenFormat::SingleFile,
                                "Single File",
                            );
                            ui.selectable_value(
                                &mut self.codegen_format,
                                CodeGenFormat::SeparateFiles,
                                "Separate Files",
                            );
                            ui.selectable_value(
                                &mut self.codegen_format,
                                CodeGenFormat::UiOnly,
                                "UI Function Only",
                            );
                        });
                });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Generate Code").on_hover_text("Ctrl+G").clicked() {
                    self.generated = self.generate_code();
                }
                // Preview/Edit mode toggle button
                ui.separator();
                let mode_label = if self.preview_mode { "Preview" } else { "Edit" };
                let mode_color = if self.preview_mode {
                    Color32::from_rgb(100, 180, 100) // Green for preview
                } else {
                    Color32::from_rgb(100, 160, 255) // Blue for edit
                };
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new(mode_label).color(mode_color))
                            .min_size(vec2(60.0, 0.0)),
                    )
                    .on_hover_text("Toggle Preview/Edit mode (F5)")
                    .clicked()
                {
                    self.preview_mode = !self.preview_mode;
                }
                // Show selection count
                if !self.selected.is_empty() {
                    ui.separator();
                    ui.label(format!("{} selected", self.selected.len()));
                }
                ui.separator();
                ui.strong("egui RAD GUI Builder");
            });
        });
    }

    // Alignment functions
    fn align_left(&mut self) {
        if self.selected.len() < 2 {
            return;
        }
        let min_x = self
            .selected
            .iter()
            .filter_map(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| w.pos.x)
            .fold(f32::INFINITY, f32::min);
        for id in &self.selected {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.pos.x = min_x;
            }
        }
    }

    fn align_right(&mut self) {
        if self.selected.len() < 2 {
            return;
        }
        let max_right = self
            .selected
            .iter()
            .filter_map(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| w.pos.x + w.size.x)
            .fold(f32::NEG_INFINITY, f32::max);
        for id in &self.selected {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.pos.x = max_right - w.size.x;
            }
        }
    }

    fn align_center_h(&mut self) {
        if self.selected.len() < 2 {
            return;
        }
        let centers: Vec<f32> = self
            .selected
            .iter()
            .filter_map(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| w.pos.x + w.size.x / 2.0)
            .collect();
        let avg_center = centers.iter().sum::<f32>() / centers.len() as f32;
        for id in &self.selected {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.pos.x = avg_center - w.size.x / 2.0;
            }
        }
    }

    fn align_top(&mut self) {
        if self.selected.len() < 2 {
            return;
        }
        let min_y = self
            .selected
            .iter()
            .filter_map(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| w.pos.y)
            .fold(f32::INFINITY, f32::min);
        for id in &self.selected {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.pos.y = min_y;
            }
        }
    }

    fn align_bottom(&mut self) {
        if self.selected.len() < 2 {
            return;
        }
        let max_bottom = self
            .selected
            .iter()
            .filter_map(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| w.pos.y + w.size.y)
            .fold(f32::NEG_INFINITY, f32::max);
        for id in &self.selected {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.pos.y = max_bottom - w.size.y;
            }
        }
    }

    fn align_center_v(&mut self) {
        if self.selected.len() < 2 {
            return;
        }
        let centers: Vec<f32> = self
            .selected
            .iter()
            .filter_map(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| w.pos.y + w.size.y / 2.0)
            .collect();
        let avg_center = centers.iter().sum::<f32>() / centers.len() as f32;
        for id in &self.selected {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.pos.y = avg_center - w.size.y / 2.0;
            }
        }
    }

    fn distribute_horizontal(&mut self) {
        if self.selected.len() < 3 {
            return;
        }
        let mut widgets: Vec<_> = self
            .selected
            .iter()
            .filter_map(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| (w.id, w.pos.x, w.size.x))
            .collect();
        widgets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let first_left = widgets.first().map(|w| w.1).unwrap_or(0.0);
        let last_right = widgets.last().map(|w| w.1 + w.2).unwrap_or(0.0);
        let total_width: f32 = widgets.iter().map(|w| w.2).sum();
        let spacing = (last_right - first_left - total_width) / (widgets.len() - 1) as f32;

        let mut x = first_left;
        for (id, _, width) in &widgets {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.pos.x = x;
            }
            x += width + spacing;
        }
    }

    fn distribute_vertical(&mut self) {
        if self.selected.len() < 3 {
            return;
        }
        let mut widgets: Vec<_> = self
            .selected
            .iter()
            .filter_map(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| (w.id, w.pos.y, w.size.y))
            .collect();
        widgets.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let first_top = widgets.first().map(|w| w.1).unwrap_or(0.0);
        let last_bottom = widgets.last().map(|w| w.1 + w.2).unwrap_or(0.0);
        let total_height: f32 = widgets.iter().map(|w| w.2).sum();
        let spacing = (last_bottom - first_top - total_height) / (widgets.len() - 1) as f32;

        let mut y = first_top;
        for (id, _, height) in &widgets {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.pos.y = y;
            }
            y += height + spacing;
        }
    }

    fn match_width(&mut self) {
        if self.selected.len() < 2 {
            return;
        }
        // Use width of first selected widget
        let target_width = self
            .selected
            .first()
            .and_then(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| w.size.x)
            .unwrap_or(100.0);
        for id in &self.selected {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.size.x = target_width;
            }
        }
    }

    fn match_height(&mut self) {
        if self.selected.len() < 2 {
            return;
        }
        // Use height of first selected widget
        let target_height = self
            .selected
            .first()
            .and_then(|id| self.project.widgets.iter().find(|w| w.id == *id))
            .map(|w| w.size.y)
            .unwrap_or(30.0);
        for id in &self.selected {
            if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *id) {
                w.size.y = target_height;
            }
        }
    }

    fn generated_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Generated Output");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.syntax_highlighting, "Syntax Highlighting")
                    .on_hover_text(
                        "Toggle syntax highlighting (may affect performance with large code)",
                    );
            });
        });
        ui.label("Rust code (or JSON export) will appear here. Copy-paste into your app.");

        // A scrollable viewport for the generated text:
        egui::ScrollArea::vertical()
            .id_salt("generated_output_scroll")
            .max_height(280.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.syntax_highlighting && !self.generated.is_empty() {
                    // Display with syntax highlighting (read-only view)
                    let job = self.highlighter.layout_job(&self.generated);
                    ui.add(egui::Label::new(job).selectable(true));
                } else {
                    // Plain text editor (editable)
                    let editor = egui::TextEdit::multiline(&mut self.generated)
                        .code_editor()
                        .lock_focus(true)
                        .desired_rows(18)
                        .desired_width(f32::INFINITY);
                    ui.add(editor);
                }
            });
    }

    fn generate_code(&self) -> String {
        match self.codegen_format {
            CodeGenFormat::SingleFile => self.generate_single_file(),
            CodeGenFormat::SeparateFiles => self.generate_separate_files(),
            CodeGenFormat::UiOnly => self.generate_ui_only(),
        }
    }

    /// Generate all code in a single file
    fn generate_single_file(&self) -> String {
        use DockArea::*;
        let mut out = String::new();

        // Header comment
        if self.codegen_comments {
            out.push_str("// =============================================================================\n");
            out.push_str("// Generated by egui RAD GUI Builder\n");
            out.push_str("// https://github.com/timschmidt/egui-rad-builder\n");
            out.push_str("// =============================================================================\n\n");
        } else {
            out.push_str("// --- generated by egui RAD GUI Builder ---\n");
        }

        out.push_str("use eframe::egui;\n");
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
        out.push_str(
            "    enable_top: bool, enable_bottom: bool, enable_left: bool, enable_right: bool,\n",
        );
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
                WidgetKind::TextArea => out.push_str(&format!("    textarea_{}: String,\n", w.id)),
                WidgetKind::DragValue => out.push_str(&format!("    drag_{}: f32,\n", w.id)),
                WidgetKind::ColorPicker => {
                    out.push_str(&format!("    color_{}: egui::Color32,\n", w.id))
                }
                WidgetKind::Code => out.push_str(&format!("    code_{}: String,\n", w.id)),
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
                WidgetKind::TextArea => {
                    out.push_str(&format!(
                        "            textarea_{}: \"{}\".to_owned(),\n",
                        w.id,
                        widget::escape(&w.props.text)
                    ));
                }
                WidgetKind::DragValue => {
                    out.push_str(&format!(
                        "            drag_{}: {:.3},\n",
                        w.id, w.props.value
                    ));
                }
                WidgetKind::ColorPicker => {
                    out.push_str(&format!(
                        "            color_{}: egui::Color32::from_rgba_unmultiplied({}, {}, {}, {}),\n",
                        w.id, w.props.color[0], w.props.color[1], w.props.color[2], w.props.color[3]
                    ));
                }
                WidgetKind::Code => {
                    out.push_str(&format!(
                        "            code_{}: \"{}\".to_owned(),\n",
                        w.id,
                        widget::escape(&w.props.text)
                    ));
                }
                _ => {}
            }
        }
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        // helper macro to emit a widget block at rect (origin + local pos)
        let emit_widget = |w: &Widget, out: &mut String, origin: &str| {
            let pos = w.pos;
            let size = w.size;
            match w.kind {
				WidgetKind::MenuButton=>{
					let items_code = if w.props.items.is_empty() {
						"\"Item\".to_string()".to_owned()
					} else {
						w.props.items.iter().map(|s| format!("\"{}\".to_string()", escape(s))).collect::<Vec<_>>().join(", ")
					};
					out.push_str(&format!(
						"    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{\n",
						x=w.pos.x, y=w.pos.y, w=w.size.x, h=w.size.y
					));
					out.push_str(&format!("        let items = vec![{items}];\n", items=items_code));
					out.push_str(&format!(
						"        ui.menu_button(\"{}\", |ui| {{\n", escape(&w.props.text)
					));
					out.push_str(&format!(
						"            for (i, it) in items.iter().enumerate() {{ if ui.button(it).clicked() {{ state.sel_{id} = i; ui.close_kind(egui::UiKind::Menu); }} }}\n",
						id = w.id
					));
					out.push_str("        });\n");
					out.push_str("    });\n");
				}
                WidgetKind::Label => out.push_str(&format!(
                    "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.label(\"{}\"); }});\n",
                    pos.x,pos.y,size.x,size.y,escape(&w.props.text)
                )),
                WidgetKind::Small => out.push_str(&format!(
                    "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.small(\"{}\"); }});\n",
                    pos.x,pos.y,size.x,size.y,escape(&w.props.text)
                )),
                WidgetKind::Monospace => out.push_str(&format!(
                    "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.monospace(\"{}\"); }});\n",
                    pos.x,pos.y,size.x,size.y,escape(&w.props.text)
                )),
                WidgetKind::Button => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::Button::new(\"{}\")); }});\n",
                        pos.x, pos.y, size.x, size.y, size.x, size.y, escape(&w.props.text)
                    ));
                }
                WidgetKind::ImageTextButton => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
							{origin} + egui::vec2({x:.1},{y:.1}), \
							egui::vec2({w:.1},{h:.1}))), |ui| {{ \
							ui.add_sized(egui::vec2({w:.1},{h:.1}), \
								egui::Button::new(format!(\"{{}}  {{}}\", \"{icon}\", \"{text}\")) \
							); \
						}});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        icon = escape(&w.props.icon),
                        text = escape(&w.props.text),
                    ));
                }
                WidgetKind::Checkbox => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.checkbox(&mut state.checked_{}, \"{}\"); }});\n",
                        pos.x, pos.y, size.x, size.y, w.id, escape(&w.props.text)
                    ));
                }
                WidgetKind::TextEdit => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::TextEdit::singleline(&mut state.text_{}).hint_text(\"{}\")); }});\n",
                        pos.x, pos.y, size.x, size.y, size.x, size.y, w.id, escape(&w.props.text)
                    ));
                }
                WidgetKind::Slider => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::Slider::new(&mut state.value_{}, {:.3}..={:.3}).text(\"{}\")); }});\n",
                        pos.x, pos.y, size.x, size.y, size.x, size.y, w.id, w.props.min, w.props.max, escape(&w.props.text)
                    ));
                }
                WidgetKind::ProgressBar => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.add_sized(egui::vec2({:.1},{:.1}), egui::ProgressBar::new(state.progress_{}).show_percentage()); }});\n",
                        pos.x, pos.y, size.x, size.y, size.x, size.y, w.id
                    ));
                }
                WidgetKind::RadioGroup => {
                    let items_code = if w.props.items.is_empty() {
                        "\"Item\".to_string()".to_owned()
                    } else {
                        w.props
                            .items
                            .iter()
                            .map(|s| format!("\"{}\".to_string()", escape(s)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{\n",
                        pos.x, pos.y, size.x, size.y
                    ));
                    out.push_str(&format!("        let items = vec![{}];\n", items_code));
                    out.push_str(&format!(
                        "        for (i, it) in items.iter().enumerate() {{ if ui.add(egui::RadioButton::new(state.sel_{} == i, it)).clicked() {{ state.sel_{} = i; }} }}\n",
                        w.id, w.id
                    ));
                    out.push_str("    });\n");
                }
                WidgetKind::Link => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.link(\"{}\"); }});\n",
                        pos.x, pos.y, size.x, size.y, escape(&w.props.text)
                    ));
                }
                WidgetKind::Hyperlink => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.hyperlink_to(\"{}\", \"{}\"); }});\n",
                        pos.x, pos.y, size.x, size.y, escape(&w.props.text), escape(&w.props.url)
                    ));
                }
                WidgetKind::SelectableLabel => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ if ui.add(egui::Button::selectable(state.sel_{}, \"{}\")).clicked() {{ state.sel_{} = !state.sel_{}; }} }});\n",
                        pos.x, pos.y, size.x, size.y, w.id, escape(&w.props.text), w.id, w.id
                    ));
                }
                WidgetKind::ComboBox => {
                    let items_code = if w.props.items.is_empty() {
                        "\"Item\".to_string()".to_owned()
                    } else {
                        w.props
                            .items
                            .iter()
                            .map(|s| format!("\"{}\".to_string()", escape(s)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };

                    out.push_str(&format!(
						"    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{\n",
						x = pos.x, y = pos.y, w = size.x, h = size.y
					));
                    out.push_str(&format!(
                        "        let items = vec![{items}];\n",
                        items = items_code
                    ));
                    out.push_str(&format!(
                        "        egui::ComboBox::from_id_source({id})\n",
                        id = w.id
                    ));
                    out.push_str(&format!("            .width({:.1})\n", size.x));
                    out.push_str(&format!(
						"            .selected_text(items.get(state.sel_{id}).cloned().unwrap_or_else(|| \"\".to_string()))\n",
						id = w.id
					));
                    out.push_str("            .show_ui(ui, |ui| {\n");
                    out.push_str(&format!(
						"                for (i, it) in items.iter().enumerate() {{ ui.selectable_value(&mut state.sel_{id}, i, it.clone()); }}\n",
						id = w.id
					));
                    out.push_str("            });\n");
                    out.push_str("    });\n");
                }
                WidgetKind::Separator => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.separator(); }});\n",
                        pos.x, pos.y, size.x, size.y
                    ));
                }
                WidgetKind::CollapsingHeader => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ egui::CollapsingHeader::new(\"{}\").default_open(state.open_{}).show(ui, |ui| {{ ui.label(\"… place your inner content here …\"); }}); }});\n",
                        pos.x, pos.y, size.x, size.y, escape(&w.props.text), w.id
                    ));
                }
                WidgetKind::DatePicker => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size({origin} + egui::vec2({:.1},{:.1}), egui::vec2({:.1},{:.1}))), |ui| {{ ui.horizontal(|ui| {{ ui.label(\"{}\"); ui.add(DatePickerButton::new(&mut state.date_{})); }}); }});\n",
                        pos.x, pos.y, size.x, size.y, escape(&w.props.text), w.id
                    ));
                }
                WidgetKind::Password => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
							{origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
							ui.add_sized(egui::vec2({w:.1},{h:.1}), \
								egui::TextEdit::singleline(&mut state.pass_{id}).password(true).hint_text(\"password\") \
							); \
						}});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        id = w.id,
                    ));
                }
                WidgetKind::AngleSelector => {
                    out.push_str(&format!(
						"    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
							{origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
							ui.add_sized(egui::vec2({w:.1},{h:.1}), \
								egui::Slider::new(&mut state.angle_{id}, {min:.3}..={max:.3}).suffix(\"°\").text(\"{label}\") \
							); \
						}});\n",
						x=pos.x,y=pos.y,w=size.x,h=size.y,id=w.id,
						min=w.props.min, max=w.props.max, label=escape(&w.props.text)
					));
                }
                WidgetKind::Tree => {
                    // Helpers live only in the generator (not emitted), so we can use any Rust we want here:
                    #[derive(Clone)]
                    struct Node {
                        label: String,
                        children: Vec<Node>,
                    }

                    fn parse_nodes(lines: &[String]) -> Vec<Node> {
                        let items: Vec<(usize, String)> = lines
                            .iter()
                            .map(|s| {
                                let indent = s.chars().take_while(|c| *c == ' ').count() / 2;
                                (indent, s.trim().to_string())
                            })
                            .filter(|(_, s)| !s.is_empty())
                            .collect();

                        fn build<I: Iterator<Item = (usize, String)>>(
                            it: &mut std::iter::Peekable<I>,
                            level: usize,
                        ) -> Vec<Node> {
                            let mut out = Vec::new();
                            while let Some((ind, _)) = it.peek().cloned() {
                                if ind < level {
                                    break;
                                }
                                if ind > level {
                                    break;
                                }
                                let (_, label) = it.next().unwrap();
                                let children = build(it, level + 1);
                                out.push(Node { label, children });
                            }
                            out
                        }

                        let mut it = items.into_iter().peekable();
                        build(&mut it, 0)
                    }

                    fn nodes_to_literal(nodes: &[Node]) -> String {
                        fn one(n: &Node) -> String {
                            let kids = if n.children.is_empty() {
                                "vec![]".to_string()
                            } else {
                                format!(
                                    "vec![{}]",
                                    n.children.iter().map(one).collect::<Vec<_>>().join(", ")
                                )
                            };
                            format!(
                                "GenTreeNode {{ label: \"{}\".to_string(), children: {} }}",
                                crate::widget::escape(&n.label),
                                kids
                            )
                        }
                        format!(
                            "vec![{}]",
                            nodes.iter().map(one).collect::<Vec<_>>().join(", ")
                        )
                    }

                    let items = if w.props.items.is_empty() {
                        vec!["Root".into(), "  Child".into()]
                    } else {
                        w.props.items.clone()
                    };

                    let nodes_literal = {
                        let nodes = parse_nodes(&items);
                        nodes_to_literal(&nodes)
                    };

                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
							{origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
							let nodes: Vec<GenTreeNode> = {nodes}; \
							egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {{ \
								gen_show_tree(ui, &nodes); \
							}}); \
						}});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        nodes = nodes_literal,
                    ));
                }
                WidgetKind::TextArea => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            ui.add_sized(egui::vec2({w:.1},{h:.1}), \
                                egui::TextEdit::multiline(&mut state.textarea_{id}).desired_rows(5) \
                            ); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        id = w.id,
                    ));
                }
                WidgetKind::DragValue => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            ui.horizontal(|ui| {{ \
                                ui.label(\"{label}\"); \
                                ui.add(egui::DragValue::new(&mut state.drag_{id}).range({min:.3}..={max:.3})); \
                            }}); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        id = w.id,
                        label = escape(&w.props.text),
                        min = w.props.min,
                        max = w.props.max,
                    ));
                }
                WidgetKind::Spinner => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            ui.add(egui::Spinner::new()); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                    ));
                }
                WidgetKind::ColorPicker => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            ui.horizontal(|ui| {{ \
                                ui.label(\"{label}\"); \
                                egui::color_picker::color_edit_button_srgba(ui, &mut state.color_{id}, egui::color_picker::Alpha::OnlyBlend); \
                            }}); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        id = w.id,
                        label = escape(&w.props.text),
                    ));
                }
                WidgetKind::Code => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            egui::ScrollArea::vertical().auto_shrink([false,false]).show(ui, |ui| {{ \
                                ui.add(egui::TextEdit::multiline(&mut state.code_{id}).code_editor().desired_width({w:.1}).desired_rows(8)); \
                            }}); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        id = w.id,
                    ));
                }
                WidgetKind::Heading => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            ui.heading(\"{text}\"); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        text = escape(&w.props.text),
                    ));
                }
                WidgetKind::Image => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            ui.add(egui::Image::new(\"{uri}\").fit_to_exact_size(egui::vec2({w:.1},{h:.1}))); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        uri = escape(&w.props.url),
                    ));
                }
                WidgetKind::Placeholder => {
                    let c = w.props.color;
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied({r},{g},{b},{a})).corner_radius(4.0).show(ui, |ui| {{ \
                                ui.set_min_size(egui::vec2({w:.1},{h:.1})); \
                                ui.centered_and_justified(|ui| ui.label(\"{text}\")); \
                            }}); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        r = c[0], g = c[1], b = c[2], a = c[3],
                        text = escape(&w.props.text),
                    ));
                }
                WidgetKind::Group => {
                    let title_code = if w.props.text.is_empty() {
                        String::new()
                    } else {
                        format!("ui.strong(\"{}\"); ui.separator(); ", escape(&w.props.text))
                    };
                    let layout_fn = if w.props.horizontal { "horizontal" } else { "vertical" };
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            egui::Frame::group(ui.style()).show(ui, |ui| {{ \
                                ui.set_min_size(egui::vec2({iw:.1},{ih:.1})); \
                                ui.{layout_fn}(|ui| {{ {title}/* group contents */ }}); \
                            }}); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        iw = size.x - 12.0,
                        ih = size.y - 12.0,
                        title = title_code,
                        layout_fn = layout_fn,
                    ));
                }
                WidgetKind::ScrollBox => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            egui::ScrollArea::both().max_width({sw:.1}).max_height({sh:.1}).auto_shrink([false,false]).show(ui, |ui| {{ \
                                ui.label(\"{text}\"); \
                            }}); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        sw = size.x - 4.0,
                        sh = size.y - 4.0,
                        text = escape(&w.props.text),
                    ));
                }
                WidgetKind::TabBar => {
                    let tabs_code: String = w.props.items.iter().enumerate().map(|(i, tab)| {
                        format!("ui.selectable_value(&mut state.tab_{id}, {i}, \"{tab}\"); ",
                            id = w.id, i = i, tab = escape(tab))
                    }).collect();
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            ui.horizontal(|ui| {{ {tabs} }}); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        tabs = tabs_code,
                    ));
                }
                WidgetKind::Columns => {
                    out.push_str(&format!(
                        "    ui.scope_builder(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(\
                            {origin} + egui::vec2({x:.1},{y:.1}), egui::vec2({w:.1},{h:.1}))), |ui| {{ \
                            ui.columns({cols}, |columns| {{ \
                                for col in columns.iter_mut() {{ col.label(\"{text}\"); }} \
                            }}); \
                        }});\n",
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        cols = w.props.columns.max(1),
                        text = escape(&w.props.text),
                    ));
                }
                WidgetKind::Window => {
                    let title = escape(&w.props.text);
                    out.push_str(&format!(
                        "    egui::Window::new(\"{title}\").default_pos({origin} + egui::vec2({x:.1},{y:.1})).default_size(egui::vec2({w:.1},{h:.1})).open(&mut state.window_{id}_open).show(ctx, |ui| {{ \
                            /* window contents */ \
                        }});\n",
                        title = title,
                        x = pos.x,
                        y = pos.y,
                        w = size.x,
                        h = size.y,
                        id = w.id,
                    ));
                }
            }
        };

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
            emit_widget(w, &mut out, "ui.min_rect().min");
        }
        out.push_str("            });\n");
        out.push_str("    }\n");

        // BOTTOM
        out.push_str("    if state.enable_bottom {\n");
        out.push_str("        egui::TopBottomPanel::bottom(\"gen_bottom\")\n");
        out.push_str("            .resizable(true)\n");
        out.push_str("            .show(ctx, |ui| {\n");
        for w in bottom {
            emit_widget(w, &mut out, "ui.min_rect().min");
        }
        out.push_str("            });\n");
        out.push_str("    }\n");

        // LEFT
        out.push_str("    if state.enable_left {\n");
        out.push_str("        egui::SidePanel::left(\"gen_left\")\n");
        out.push_str("            .resizable(true)\n");
        out.push_str("            .show(ctx, |ui| {\n");
        for w in left {
            emit_widget(w, &mut out, "ui.min_rect().min");
        }
        out.push_str("            });\n");
        out.push_str("    }\n");

        // RIGHT
        out.push_str("    if state.enable_right {\n");
        out.push_str("        egui::SidePanel::right(\"gen_right\")\n");
        out.push_str("            .resizable(true)\n");
        out.push_str("            .show(ctx, |ui| {\n");
        for w in right {
            emit_widget(w, &mut out, "ui.min_rect().min");
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
            emit_widget(w, &mut out, "canvas.min");
        }
        for w in free {
            emit_widget(w, &mut out, "canvas.min");
        }
        out.push_str("    });\n");

        out.push_str("}\n\n");

        // ---------- Example eframe app (updated to call generated_ui with ctx) ----------
        if self.codegen_comments {
            out.push_str("// =============================================================================\n");
            out.push_str("// Application entry point\n");
            out.push_str("// =============================================================================\n\n");
        }

        out.push_str(
            "pub struct GeneratedApp {\n\
			     state: GeneratedState,\n\
			 }\n\n\
			 impl Default for GeneratedApp {\n\
			     fn default() -> Self {\n\
			         Self { state: Default::default() }\n\
			     }\n\
			 }\n\n\
			 impl eframe::App for GeneratedApp {\n\
			     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {\n\
			         generated_ui(ctx, &mut self.state);\n\
			     }\n\
			 }\n\n\
			 fn main() -> eframe::Result<()> {\n\
			     let native_options = eframe::NativeOptions::default();\n\
			     eframe::run_native(\n\
			         \"Generated UI\",\n\
			         native_options,\n\
			         Box::new(|_cc| Ok(Box::new(GeneratedApp::default()))),\n\
			     )\n\
			 }\n",
        );

        out
    }

    /// Generate code split into separate conceptual files (shown with file headers)
    fn generate_separate_files(&self) -> String {
        let single = self.generate_single_file();

        // For now, show the code with clear section headers
        // A future enhancement could actually save separate files
        let mut out = String::new();

        out.push_str(
            "// =============================================================================\n",
        );
        out.push_str("// FILE: Cargo.toml\n");
        out.push_str(
            "// =============================================================================\n",
        );
        out.push_str("[package]\n");
        out.push_str("name = \"generated-ui\"\n");
        out.push_str("version = \"0.1.0\"\n");
        out.push_str("edition = \"2021\"\n\n");
        out.push_str("[dependencies]\n");
        out.push_str("eframe = \"0.33\"\n");
        out.push_str("egui = \"0.33\"\n");
        out.push_str("egui_extras = { version = \"0.33\", features = [\"chrono\"] }\n");
        out.push_str("chrono = \"0.4\"\n\n");

        out.push_str(
            "// =============================================================================\n",
        );
        out.push_str("// FILE: src/main.rs\n");
        out.push_str(
            "// =============================================================================\n",
        );
        out.push_str(&single);

        out
    }

    /// Generate only the UI function (for embedding in existing code)
    fn generate_ui_only(&self) -> String {
        let mut out = String::new();

        if self.codegen_comments {
            out.push_str("// UI function generated by egui RAD GUI Builder\n");
            out.push_str("// Embed this in your existing application\n\n");
        }

        // We need to include the state struct since UI references it
        out.push_str("// Required state struct for the UI\n");

        // Generate just the state struct and the UI function
        // We'll extract parts from generate_single_file
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
                     for n in nodes {\n\
                         if n.children.is_empty() { ui.label(&n.label); }\n\
                         else { ui.collapsing(&n.label, |ui| gen_show_tree(ui, &n.children)); }\n\
                     }\n\
                 }\n\n",
            );
        }

        out.push_str("struct GeneratedState {\n");
        out.push_str(
            "    enable_top: bool, enable_bottom: bool, enable_left: bool, enable_right: bool,\n",
        );
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
                WidgetKind::DatePicker => {
                    out.push_str(&format!("    date_{}: chrono::NaiveDate,\n", w.id))
                }
                WidgetKind::Password => out.push_str(&format!("    pass_{}: String,\n", w.id)),
                WidgetKind::AngleSelector => out.push_str(&format!("    angle_{}: f32,\n", w.id)),
                WidgetKind::TextArea => out.push_str(&format!("    textarea_{}: String,\n", w.id)),
                WidgetKind::DragValue => out.push_str(&format!("    drag_{}: f32,\n", w.id)),
                WidgetKind::ColorPicker => {
                    out.push_str(&format!("    color_{}: egui::Color32,\n", w.id))
                }
                WidgetKind::Code => out.push_str(&format!("    code_{}: String,\n", w.id)),
                _ => {}
            }
        }
        out.push_str("}\n\n");

        out.push_str("// Call this function from your eframe::App::update method:\n");
        out.push_str("// generated_ui(ctx, &mut self.state);\n\n");

        // Extract just the generated_ui function from single file output
        let single = self.generate_single_file();
        if let Some(start) = single.find("fn generated_ui(") {
            // Find the end of the function (look for the closing brace followed by app struct)
            if let Some(end) = single[start..].find("\npub struct GeneratedApp") {
                out.push_str(&single[start..start + end]);
            } else {
                // Fallback: include from generated_ui to end
                out.push_str(&single[start..]);
            }
        }

        out
    }
}

impl eframe::App for RadBuilderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard shortcuts - check input first, then apply changes
        let (
            delete_pressed,
            duplicate_pressed,
            generate_pressed,
            copy_pressed,
            paste_pressed,
            arrow_up,
            arrow_down,
            arrow_left,
            arrow_right,
            bring_front,
            send_back,
            toggle_preview,
        ) = ctx.input(|i| {
            let del = i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace);
            let dup = i.modifiers.command && i.key_pressed(egui::Key::D);
            let gencode = i.modifiers.command && i.key_pressed(egui::Key::G);
            let copy = i.modifiers.command && i.key_pressed(egui::Key::C);
            let paste = i.modifiers.command && i.key_pressed(egui::Key::V);
            // Arrow keys for nudging
            let up = i.key_pressed(egui::Key::ArrowUp);
            let down = i.key_pressed(egui::Key::ArrowDown);
            let left = i.key_pressed(egui::Key::ArrowLeft);
            let right = i.key_pressed(egui::Key::ArrowRight);
            // Z-order: ] = bring to front, [ = send to back
            let front = i.key_pressed(egui::Key::CloseBracket);
            let back = i.key_pressed(egui::Key::OpenBracket);
            // F5: Toggle preview mode
            let preview = i.key_pressed(egui::Key::F5);
            (
                del, dup, gencode, copy, paste, up, down, left, right, front, back, preview,
            )
        });

        // F5: Toggle preview mode
        if toggle_preview {
            self.preview_mode = !self.preview_mode;
        }

        // Delete selected widgets
        if delete_pressed && !self.selected.is_empty() {
            let to_delete: Vec<_> = self.selected.clone();
            self.project.widgets.retain(|w| !to_delete.contains(&w.id));
            self.selected.clear();
        }

        // Arrow keys: Nudge all selected widgets
        if !self.selected.is_empty() && (arrow_up || arrow_down || arrow_left || arrow_right) {
            let nudge = self.grid_size.max(1.0);
            let selected_ids: Vec<_> = self.selected.clone();
            for sel_id in selected_ids {
                if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == sel_id) {
                    if arrow_up {
                        w.pos.y -= nudge;
                    }
                    if arrow_down {
                        w.pos.y += nudge;
                    }
                    if arrow_left {
                        w.pos.x -= nudge;
                    }
                    if arrow_right {
                        w.pos.x += nudge;
                    }
                    // Clamp position
                    w.pos.x = w.pos.x.max(0.0);
                    w.pos.y = w.pos.y.max(0.0);
                }
            }
        }

        // Z-order controls (apply to all selected)
        if bring_front && !self.selected.is_empty() {
            let max_z = self.project.widgets.iter().map(|w| w.z).max().unwrap_or(0);
            let selected_ids: Vec<_> = self.selected.clone();
            for (i, sel_id) in selected_ids.iter().enumerate() {
                if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *sel_id) {
                    w.z = max_z + 1 + i as i32;
                }
            }
        }
        if send_back && !self.selected.is_empty() {
            let min_z = self.project.widgets.iter().map(|w| w.z).min().unwrap_or(0);
            let selected_ids: Vec<_> = self.selected.clone();
            for (i, sel_id) in selected_ids.iter().enumerate() {
                if let Some(w) = self.project.widgets.iter_mut().find(|w| w.id == *sel_id) {
                    w.z = min_z - 1 - i as i32;
                }
            }
        }

        // Ctrl+C: Copy first selected widget
        if copy_pressed
            && let Some(&sel_id) = self.selected.first()
            && let Some(w) = self.project.widgets.iter().find(|w| w.id == sel_id)
        {
            self.clipboard = Some(w.clone());
        }

        // Ctrl+V: Paste widget from clipboard
        if paste_pressed && let Some(w) = self.clipboard.clone() {
            let new_id = WidgetId::new(self.next_id);
            self.next_id += 1;
            let mut pasted = w;
            pasted.id = new_id;
            pasted.z = new_id.as_z();
            pasted.pos.x += 20.0;
            pasted.pos.y += 20.0;
            self.project.widgets.push(pasted);
            self.selected = vec![new_id];
        }

        // Ctrl+D: Duplicate all selected widgets
        if duplicate_pressed && !self.selected.is_empty() {
            let selected_ids: Vec<_> = self.selected.clone();
            let mut new_ids = Vec::new();
            for sel_id in selected_ids {
                if let Some(w) = self
                    .project
                    .widgets
                    .iter()
                    .find(|w| w.id == sel_id)
                    .cloned()
                {
                    let new_id = WidgetId::new(self.next_id);
                    self.next_id += 1;
                    let mut dup = w;
                    dup.id = new_id;
                    dup.z = new_id.as_z();
                    dup.pos.x += 20.0;
                    dup.pos.y += 20.0;
                    self.project.widgets.push(dup);
                    new_ids.push(new_id);
                }
            }
            self.selected = new_ids;
        }

        // Ctrl+G: Generate code
        if generate_pressed {
            self.generated = self.generate_code();
        }

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
                // Tab bar for right panel
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.right_panel_tab == 0, "Inspector")
                        .clicked()
                    {
                        self.right_panel_tab = 0;
                    }
                    if ui
                        .selectable_label(self.right_panel_tab == 1, "Code")
                        .clicked()
                    {
                        self.right_panel_tab = 1;
                    }
                });
                ui.separator();

                match self.right_panel_tab {
                    0 => self.inspector_ui(ui),
                    1 => self.generated_panel(ui),
                    _ => {}
                }
            });

        // Set edit mode for widget rendering (inverse of preview mode)
        ctx.data_mut(|d| d.insert_temp(Id::new("edit_mode"), !self.preview_mode));

        self.preview_panels_ui(ctx);

        // Auto-generate code if enabled and widgets exist
        if self.auto_generate && !self.project.widgets.is_empty() {
            self.generated = self.generate_code();
        }

        if self.spawning.is_some() {
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        }
    }
}
