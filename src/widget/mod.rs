use egui::{Pos2, Vec2, pos2};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct WidgetId(u64);

impl WidgetId {
    pub(crate) const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_z(&self) -> i32 {
        self.0 as i32
    }

    pub(crate) const fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DockArea {
    Free,
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

impl Default for DockArea {
    fn default() -> Self {
        DockArea::Free
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Widget {
    pub(crate) id: WidgetId,
    pub(crate) kind: WidgetKind,
    pub(crate) pos: Pos2,  // Top-left relative to canvas
    pub(crate) size: Vec2, // Desired size on canvas
    pub(crate) z: i32,     // draw order
    pub(crate) area: DockArea,
    pub(crate) props: WidgetProps,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub(crate) enum WidgetKind {
    MenuButton,
    Label,
    Button,
    ImageTextButton,
    Checkbox,
    TextEdit,
    Slider,
    ProgressBar,
    RadioGroup,
    Link,
    Hyperlink,
    SelectableLabel,
    ComboBox,
    Separator,
    CollapsingHeader,
    DatePicker,
    AngleSelector,
    Password,
    Tree,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WidgetProps {
    pub(crate) text: String,  // label/button/textedit placeholder
    pub(crate) checked: bool, // checkbox
    pub(crate) value: f32,    // slider/progress
    pub(crate) min: f32,
    pub(crate) max: f32,
    // lists (for radio/combobox)
    pub(crate) items: Vec<String>,
    pub(crate) selected: usize,
    // hyperlinks
    pub(crate) url: String,
    // date (stored as y/m/d to avoid chrono serde feature requirements)
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) icon: String,
}

impl Default for WidgetProps {
    fn default() -> Self {
        Self {
            text: "Label".into(),
            checked: false,
            value: 0.5,
            min: 0.0,
            max: 1.0,
            items: vec![],
            selected: 0,
            url: "https://example.com".into(),
            year: 2024,
            month: 1,
            day: 1,
            icon: "🖼️".into(),
        }
    }
}

pub(crate) fn snap_pos_with_grid(p: Pos2, grid: f32) -> Pos2 {
    pos2((p.x / grid).round() * grid, (p.y / grid).round() * grid)
}

pub(crate) fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl Widget {
    /// Emit a widget block at rect (origin + local pos)
    pub(crate) fn emit_widget(&self, out: &mut String, origin: &str) {
        let w = self;

        let pos = w.pos;
        let size = w.size;

        match w.kind {
            WidgetKind::MenuButton => {
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
        }
    }
}
