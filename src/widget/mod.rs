use egui::{Pos2, Vec2, pos2};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub(crate) struct WidgetId(u64);

impl WidgetId {
    pub(crate) const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_z(&self) -> i32 {
        self.0 as i32
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) enum DockArea {
    #[default]
    Free,
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

/// Widget categories for palette organization (Mobius-ECS inspired)
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WidgetCategory {
    Basic,
    Input,
    Display,
    Containers,
    Advanced,
}

#[allow(dead_code)]
impl WidgetCategory {
    /// Returns all categories in display order
    pub const fn all() -> &'static [WidgetCategory] {
        &[
            WidgetCategory::Basic,
            WidgetCategory::Input,
            WidgetCategory::Display,
            WidgetCategory::Containers,
            WidgetCategory::Advanced,
        ]
    }

    /// Returns the display name for this category
    pub const fn display_name(&self) -> &'static str {
        match self {
            WidgetCategory::Basic => "Basic",
            WidgetCategory::Input => "Input",
            WidgetCategory::Display => "Display",
            WidgetCategory::Containers => "Containers",
            WidgetCategory::Advanced => "Advanced",
        }
    }

    /// Returns whether this category should be open by default in the palette
    pub const fn default_open(&self) -> bool {
        !matches!(self, WidgetCategory::Advanced)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub(crate) enum WidgetKind {
    MenuButton,
    Label,
    Heading,
    Small,
    Monospace,
    Button,
    ImageTextButton,
    Checkbox,
    TextEdit,
    TextArea,
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
    DragValue,
    Spinner,
    ColorPicker,
    Code,
    Image,
    Placeholder,
    Group,
    ScrollBox,
    TabBar,
    Columns,
    Window,
}

impl WidgetKind {
    /// Returns the category this widget belongs to (for palette organization)
    #[allow(dead_code)]
    pub const fn category(&self) -> WidgetCategory {
        match self {
            // Basic: simple display and interaction elements
            WidgetKind::Label
            | WidgetKind::Button
            | WidgetKind::ImageTextButton
            | WidgetKind::Checkbox
            | WidgetKind::Link
            | WidgetKind::Hyperlink
            | WidgetKind::SelectableLabel
            | WidgetKind::Separator => WidgetCategory::Basic,

            // Input: user data entry widgets
            WidgetKind::TextEdit
            | WidgetKind::TextArea
            | WidgetKind::Password
            | WidgetKind::Slider
            | WidgetKind::DragValue
            | WidgetKind::ComboBox
            | WidgetKind::RadioGroup
            | WidgetKind::DatePicker
            | WidgetKind::AngleSelector
            | WidgetKind::ColorPicker => WidgetCategory::Input,

            // Display: output and feedback widgets
            WidgetKind::Heading
            | WidgetKind::Small
            | WidgetKind::Monospace
            | WidgetKind::ProgressBar
            | WidgetKind::Spinner
            | WidgetKind::Image
            | WidgetKind::Placeholder => WidgetCategory::Display,

            // Containers: layout and grouping widgets
            WidgetKind::Group
            | WidgetKind::ScrollBox
            | WidgetKind::Columns
            | WidgetKind::TabBar
            | WidgetKind::Window
            | WidgetKind::CollapsingHeader => WidgetCategory::Containers,

            // Advanced: complex or specialized widgets
            WidgetKind::MenuButton | WidgetKind::Tree | WidgetKind::Code => {
                WidgetCategory::Advanced
            }
        }
    }

    /// Returns the display name for this widget kind (used in palette)
    #[allow(dead_code)]
    pub const fn display_name(&self) -> &'static str {
        match self {
            WidgetKind::MenuButton => "Menu Button",
            WidgetKind::Label => "Label",
            WidgetKind::Heading => "Heading",
            WidgetKind::Small => "Small",
            WidgetKind::Monospace => "Monospace",
            WidgetKind::Button => "Button",
            WidgetKind::ImageTextButton => "Image + Text Button",
            WidgetKind::Checkbox => "Checkbox",
            WidgetKind::TextEdit => "TextEdit",
            WidgetKind::TextArea => "Text Area",
            WidgetKind::Slider => "Slider",
            WidgetKind::ProgressBar => "ProgressBar",
            WidgetKind::RadioGroup => "Radio Group",
            WidgetKind::Link => "Link",
            WidgetKind::Hyperlink => "Hyperlink",
            WidgetKind::SelectableLabel => "Selectable Label",
            WidgetKind::ComboBox => "Combo Box",
            WidgetKind::Separator => "Separator",
            WidgetKind::CollapsingHeader => "Collapsing Header",
            WidgetKind::DatePicker => "Date Picker",
            WidgetKind::AngleSelector => "Angle Selector",
            WidgetKind::Password => "Password",
            WidgetKind::Tree => "Tree",
            WidgetKind::DragValue => "Drag Value",
            WidgetKind::Spinner => "Spinner",
            WidgetKind::ColorPicker => "Color Picker",
            WidgetKind::Code => "Code Editor",
            WidgetKind::Image => "Image",
            WidgetKind::Placeholder => "Placeholder",
            WidgetKind::Group => "Group",
            WidgetKind::ScrollBox => "Scroll Box",
            WidgetKind::TabBar => "Tab Bar",
            WidgetKind::Columns => "Columns",
            WidgetKind::Window => "Window",
        }
    }

    /// Returns all widget kinds in a given category
    #[allow(dead_code)]
    pub fn widgets_in_category(category: WidgetCategory) -> Vec<WidgetKind> {
        Self::all()
            .iter()
            .filter(|k| k.category() == category)
            .copied()
            .collect()
    }

    /// Returns all widget kinds
    #[allow(dead_code)]
    pub const fn all() -> &'static [WidgetKind] {
        &[
            WidgetKind::Label,
            WidgetKind::Button,
            WidgetKind::ImageTextButton,
            WidgetKind::Checkbox,
            WidgetKind::Link,
            WidgetKind::Hyperlink,
            WidgetKind::SelectableLabel,
            WidgetKind::Separator,
            WidgetKind::TextEdit,
            WidgetKind::TextArea,
            WidgetKind::Password,
            WidgetKind::Slider,
            WidgetKind::DragValue,
            WidgetKind::ComboBox,
            WidgetKind::RadioGroup,
            WidgetKind::DatePicker,
            WidgetKind::AngleSelector,
            WidgetKind::ColorPicker,
            WidgetKind::Heading,
            WidgetKind::Small,
            WidgetKind::Monospace,
            WidgetKind::ProgressBar,
            WidgetKind::Spinner,
            WidgetKind::Image,
            WidgetKind::Placeholder,
            WidgetKind::Group,
            WidgetKind::ScrollBox,
            WidgetKind::Columns,
            WidgetKind::TabBar,
            WidgetKind::Window,
            WidgetKind::CollapsingHeader,
            WidgetKind::MenuButton,
            WidgetKind::Tree,
            WidgetKind::Code,
        ]
    }

    /// Returns the default size for a widget of this kind.
    /// Centralized to avoid duplication between spawn_widget and ghost preview.
    pub fn default_size(&self) -> egui::Vec2 {
        use egui::vec2;
        match self {
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
            WidgetKind::TextArea => vec2(280.0, 120.0),
            WidgetKind::DragValue => vec2(180.0, 24.0),
            WidgetKind::Spinner => vec2(32.0, 32.0),
            WidgetKind::ColorPicker => vec2(200.0, 28.0),
            WidgetKind::Code => vec2(300.0, 150.0),
            WidgetKind::Heading => vec2(200.0, 32.0),
            WidgetKind::Small => vec2(120.0, 20.0),
            WidgetKind::Monospace => vec2(140.0, 20.0),
            WidgetKind::Image => vec2(150.0, 150.0),
            WidgetKind::Placeholder => vec2(200.0, 100.0),
            WidgetKind::Group => vec2(250.0, 150.0),
            WidgetKind::ScrollBox => vec2(200.0, 150.0),
            WidgetKind::TabBar => vec2(300.0, 32.0),
            WidgetKind::Columns => vec2(300.0, 120.0),
            WidgetKind::Window => vec2(280.0, 180.0),
        }
    }

    /// Returns the default properties for a widget of this kind.
    pub fn default_props(&self) -> WidgetProps {
        match self {
            WidgetKind::MenuButton => {
                let mut p = WidgetProps {
                    text: "Menu".into(),
                    ..Default::default()
                };
                p.items = vec!["First".into(), "Second".into(), "Third".into()];
                p.selected = 0;
                p
            }
            WidgetKind::Label => WidgetProps {
                text: "Label".into(),
                ..Default::default()
            },
            WidgetKind::Button => WidgetProps {
                text: "Button".into(),
                ..Default::default()
            },
            WidgetKind::ImageTextButton => WidgetProps {
                text: "Button".into(),
                icon: "🖼️".into(),
                ..Default::default()
            },
            WidgetKind::Checkbox => WidgetProps {
                text: "Checkbox".into(),
                ..Default::default()
            },
            WidgetKind::TextEdit => WidgetProps {
                text: "Type here".into(),
                ..Default::default()
            },
            WidgetKind::Slider => WidgetProps {
                text: "Value".into(),
                min: 0.0,
                max: 100.0,
                value: 42.0,
                checked: false,
                ..Default::default()
            },
            WidgetKind::ProgressBar => WidgetProps {
                text: "".into(),
                value: 0.25,
                min: 0.0,
                max: 1.0,
                checked: false,
                ..Default::default()
            },
            WidgetKind::RadioGroup => {
                let mut p = WidgetProps {
                    text: "Radio Group".into(),
                    ..Default::default()
                };
                p.items = vec!["Option A".into(), "Option B".into(), "Option C".into()];
                p.selected = 0;
                p
            }
            WidgetKind::Link => WidgetProps {
                text: "Link text".into(),
                ..Default::default()
            },
            WidgetKind::Hyperlink => WidgetProps {
                text: "Open website".into(),
                url: "https://example.com".into(),
                ..Default::default()
            },
            WidgetKind::SelectableLabel => WidgetProps {
                text: "Selectable".into(),
                checked: false,
                ..Default::default()
            },
            WidgetKind::ComboBox => {
                let mut p = WidgetProps {
                    text: "Choose one".into(),
                    ..Default::default()
                };
                p.items = vec!["Red".into(), "Green".into(), "Blue".into()];
                p.selected = 0;
                p
            }
            WidgetKind::Separator => WidgetProps::default(),
            WidgetKind::CollapsingHeader => WidgetProps {
                text: "Section".into(),
                checked: true, // default open
                ..Default::default()
            },
            WidgetKind::DatePicker => WidgetProps {
                text: "Pick a date".into(),
                year: 2025,
                month: 1,
                day: 1,
                ..Default::default()
            },
            WidgetKind::AngleSelector => WidgetProps {
                text: "Angle (deg)".into(),
                min: 0.0,
                max: 360.0,
                value: 45.0,
                ..Default::default()
            },
            WidgetKind::Password => WidgetProps {
                text: "password".into(),
                ..Default::default()
            },
            WidgetKind::Tree => {
                let mut p = WidgetProps {
                    text: "Tree".into(),
                    ..Default::default()
                };
                // Indentation (two spaces = one level) to define hierarchy
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
                p
            }
            WidgetKind::TextArea => WidgetProps {
                text: "Multi-line\ntext here".into(),
                ..Default::default()
            },
            WidgetKind::DragValue => WidgetProps {
                text: "Value".into(),
                value: 42.0,
                min: 0.0,
                max: 100.0,
                ..Default::default()
            },
            WidgetKind::Spinner => WidgetProps::default(),
            WidgetKind::ColorPicker => WidgetProps {
                text: "Color".into(),
                color: [100, 149, 237, 255],
                ..Default::default()
            },
            WidgetKind::Code => WidgetProps {
                text: "fn main() {\n    println!(\"Hello\");\n}".into(),
                ..Default::default()
            },
            WidgetKind::Heading => WidgetProps {
                text: "Heading".into(),
                ..Default::default()
            },
            WidgetKind::Small => WidgetProps {
                text: "Small text".into(),
                ..Default::default()
            },
            WidgetKind::Monospace => WidgetProps {
                text: "code_value".into(),
                ..Default::default()
            },
            WidgetKind::Image => WidgetProps {
                text: "image.png".into(),
                url: "file://image.png".into(),
                ..Default::default()
            },
            WidgetKind::Placeholder => WidgetProps {
                text: "Placeholder".into(),
                color: [128, 128, 128, 128],
                ..Default::default()
            },
            WidgetKind::Group => WidgetProps {
                text: "Group".into(),
                ..Default::default()
            },
            WidgetKind::ScrollBox => WidgetProps {
                text: "Scroll content here...".into(),
                ..Default::default()
            },
            WidgetKind::TabBar => WidgetProps {
                items: vec!["Tab 1".into(), "Tab 2".into(), "Tab 3".into()],
                selected: 0,
                ..Default::default()
            },
            WidgetKind::Columns => WidgetProps {
                text: "Column content".into(),
                columns: 2,
                ..Default::default()
            },
            WidgetKind::Window => WidgetProps {
                text: "Window Title".into(),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct WidgetProps {
    pub(crate) text: String,  // label/button/textedit placeholder
    pub(crate) checked: bool, // checkbox
    pub(crate) value: f32,    // slider/progress/dragvalue
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
    // color (rgba 0-255)
    pub(crate) color: [u8; 4],
    // optional tooltip text
    pub(crate) tooltip: String,
    // layout direction (for Group)
    pub(crate) horizontal: bool,
    // enabled state
    pub(crate) enabled: bool,
    // column count (for Columns widget)
    pub(crate) columns: usize,
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
            color: [100, 149, 237, 255], // cornflower blue
            tooltip: String::new(),
            horizontal: false,
            enabled: true,
            columns: 2,
        }
    }
}

pub(crate) fn snap_pos_with_grid(p: Pos2, grid: f32) -> Pos2 {
    pos2((p.x / grid).round() * grid, (p.y / grid).round() * grid)
}

pub(crate) fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn test_snap_pos_with_grid() {
        // Test snapping to grid of 10
        assert_eq!(snap_pos_with_grid(pos2(5.0, 5.0), 10.0), pos2(10.0, 10.0));
        assert_eq!(snap_pos_with_grid(pos2(4.9, 4.9), 10.0), pos2(0.0, 0.0));
        assert_eq!(snap_pos_with_grid(pos2(15.0, 25.0), 10.0), pos2(20.0, 30.0));

        // Test snapping to grid of 1 (no-op for integers)
        assert_eq!(snap_pos_with_grid(pos2(5.4, 3.6), 1.0), pos2(5.0, 4.0));

        // Test snapping to grid of 8
        assert_eq!(snap_pos_with_grid(pos2(12.0, 20.0), 8.0), pos2(16.0, 24.0));
    }

    #[test]
    fn test_escape() {
        // Test basic strings
        assert_eq!(escape("hello"), "hello");

        // Test backslash escaping
        assert_eq!(escape("path\\to\\file"), "path\\\\to\\\\file");

        // Test quote escaping
        assert_eq!(escape("say \"hello\""), "say \\\"hello\\\"");

        // Test combined
        assert_eq!(escape("c:\\path\\\"file\""), "c:\\\\path\\\\\\\"file\\\"");
    }

    #[test]
    fn test_widget_kind_default_size() {
        // All widget kinds should return positive dimensions
        let kinds = [
            WidgetKind::Label,
            WidgetKind::Button,
            WidgetKind::Checkbox,
            WidgetKind::TextEdit,
            WidgetKind::Slider,
            WidgetKind::ProgressBar,
            WidgetKind::RadioGroup,
            WidgetKind::ComboBox,
            WidgetKind::Separator,
            WidgetKind::Spinner,
            WidgetKind::ColorPicker,
            WidgetKind::Group,
            WidgetKind::Window,
        ];

        for kind in kinds {
            let size = kind.default_size();
            assert!(size.x > 0.0, "{:?} should have positive width", kind);
            assert!(size.y > 0.0, "{:?} should have positive height", kind);
        }
    }

    #[test]
    fn test_widget_kind_default_props() {
        // Test that default props are reasonable
        let label_props = WidgetKind::Label.default_props();
        assert_eq!(label_props.text, "Label");

        let button_props = WidgetKind::Button.default_props();
        assert_eq!(button_props.text, "Button");

        let slider_props = WidgetKind::Slider.default_props();
        assert!(slider_props.min < slider_props.max);
        assert!(slider_props.value >= slider_props.min);
        assert!(slider_props.value <= slider_props.max);

        let combobox_props = WidgetKind::ComboBox.default_props();
        assert!(!combobox_props.items.is_empty());
        assert!(combobox_props.selected < combobox_props.items.len());
    }

    #[test]
    fn test_widget_props_default() {
        let props = WidgetProps::default();
        assert!(!props.text.is_empty());
        assert!(props.min <= props.max);
        assert_eq!(props.columns, 2);
        assert!(props.enabled);
    }

    #[test]
    fn test_dock_area_default() {
        let area = DockArea::default();
        assert_eq!(area, DockArea::Free);
    }

    #[test]
    fn test_widget_id_display() {
        let id = WidgetId::new(42);
        assert_eq!(format!("{}", id), "42");
    }

    #[test]
    fn test_widget_id_z_order() {
        let id = WidgetId::new(100);
        assert_eq!(id.as_z(), 100);
    }

    #[test]
    fn test_widget_category_all() {
        // All categories should be present
        let categories = WidgetCategory::all();
        assert_eq!(categories.len(), 5);
        assert!(categories.contains(&WidgetCategory::Basic));
        assert!(categories.contains(&WidgetCategory::Input));
        assert!(categories.contains(&WidgetCategory::Display));
        assert!(categories.contains(&WidgetCategory::Containers));
        assert!(categories.contains(&WidgetCategory::Advanced));
    }

    #[test]
    fn test_widget_kind_categories() {
        // Test a few widgets are in correct categories
        assert_eq!(WidgetKind::Label.category(), WidgetCategory::Basic);
        assert_eq!(WidgetKind::Button.category(), WidgetCategory::Basic);
        assert_eq!(WidgetKind::TextEdit.category(), WidgetCategory::Input);
        assert_eq!(WidgetKind::Slider.category(), WidgetCategory::Input);
        assert_eq!(WidgetKind::ProgressBar.category(), WidgetCategory::Display);
        assert_eq!(WidgetKind::Group.category(), WidgetCategory::Containers);
        assert_eq!(WidgetKind::Tree.category(), WidgetCategory::Advanced);
    }

    #[test]
    fn test_widget_kind_all_have_categories() {
        // All widgets should have a valid category
        for kind in WidgetKind::all() {
            let _category = kind.category(); // Should not panic
            let _name = kind.display_name(); // Should not panic
        }
    }

    #[test]
    fn test_widget_kind_display_names() {
        assert_eq!(WidgetKind::Label.display_name(), "Label");
        assert_eq!(
            WidgetKind::ImageTextButton.display_name(),
            "Image + Text Button"
        );
        assert_eq!(
            WidgetKind::CollapsingHeader.display_name(),
            "Collapsing Header"
        );
    }

    #[test]
    fn test_widgets_in_category() {
        let basic_widgets = WidgetKind::widgets_in_category(WidgetCategory::Basic);
        assert!(basic_widgets.contains(&WidgetKind::Label));
        assert!(basic_widgets.contains(&WidgetKind::Button));
        assert!(!basic_widgets.contains(&WidgetKind::TextEdit)); // Input, not Basic

        let input_widgets = WidgetKind::widgets_in_category(WidgetCategory::Input);
        assert!(input_widgets.contains(&WidgetKind::TextEdit));
        assert!(input_widgets.contains(&WidgetKind::Slider));
    }
}
