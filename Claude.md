# Claude.md - Project Analysis & Improvement Roadmap

## Project Overview

**egui-rad-builder** is a Rapid Application Development (RAD) GUI builder tool for the egui immediate-mode GUI framework. It allows developers to visually design user interfaces through drag-and-drop, then generates production-ready Rust code for egui-based applications.

**Current Version:** 0.1.10
**License:** MIT
**Status:** Active early development

---

## Design Inspiration & Reference Architecture

### Mobius-ECS Analysis ([saturn77/mobius-ecs](https://github.com/saturn77/mobius-ecs))

Mobius-ECS is a visual UI designer for Rust/egui built on Entity Component System principles. Key design elements to consider:

#### Architecture Patterns Worth Adopting

| Pattern | Mobius Approach | Opportunity for egui-rad-builder |
|---------|-----------------|----------------------------------|
| **ECS Foundation** | Uses `bevy_ecs` for modularity | Consider lightweight ECS for widget management, enabling dynamic component composition |
| **Signals/Slots** | `egui_mobius` for thread-safe communication | Add event-driven widget communication for preview interactivity |
| **Two-Tier Structure** | Designer app + Framework library | Separate core widget library from builder app for reusability |
| **Docking Integration** | `egui_dock` (0.17.0) as template layer | Already using panels; could adopt egui_dock for modular window architecture |

#### Notable Features to Consider

1. **Visual Alignment Tools** - Horizontal/vertical alignment with distribution controls and grid snapping (priority from Issue #15)
2. **Hot-Reload Support** - Instant UI updates during development
3. **Template System** - Declarative UI structure definitions
4. **Project Export** - Generates complete Cargo projects with dependencies (already implemented here)

#### Code Generation Approach (Mobius)
- Uses `syntect` for syntax highlighting in generated code preview
- Produces production-ready egui code from visual designs
- Integrates signals/slots patterns into generated code

### egui Demo Widget Gallery Analysis ([egui.rs](https://www.egui.rs))

The official egui demo showcases best practices for widget organization and UX patterns:

#### Widget Organization Patterns

**Consistent Configuration**:
- Builder pattern with chainable methods (`.with_date_button()`, `.on_hover_text()`)
- State fields for: enabled, visible, opacity, interactivity
- Hover text documentation on all interactive elements

**Grid-Based Layout**:
- Two-column grid: labels/docs on left, widgets on right
- Consistent spacing and striping for visual hierarchy
- `ui.scope_builder()` for grouped widget state management

**Progressive Disclosure**:
- CollapsingHeader for expandable sections
- Feature-gated optional components
- Conditional widget inclusion without layout disruption

#### Widget Categories (Official egui)

| Category | Widgets |
|----------|---------|
| **Text & Display** | Label, Hyperlink, Separator |
| **Input** | TextEdit (with hint text), Button, Link |
| **Selection** | Checkbox, RadioButton, SelectableLabel, ComboBox |
| **Numeric** | Slider (with suffix), DragValue (with speed config) |
| **Feedback** | ProgressBar (animated), Spinner |
| **Visual** | Color picker, Image, Image+Button combo |
| **Structure** | CollapsingHeader, Grid (configurable striping/spacing) |

#### UX Best Practices from egui Demo

1. **Scope Isolation** - Wrap widget groups with shared opacity/interactivity states
2. **Conditional Features** - Feature-gated components (e.g., chrono-dependent DatePicker)
3. **Snapshot Testing** - Multiple pixel densities and theme combinations
4. **Hover Documentation** - Consistent `.on_hover_text()` across all interactive elements

---

## Recent Changes (2025-12-27)

### Phase 5: Panel Tabs and UX Improvements
Implemented tabbed panel interface for better workspace organization:

**Right Panel Tabs:**
- Added "Inspector" and "Code" tabs to the right panel
- Click tabs to switch between widget properties and generated code
- Cleaner UI - no more cramped Inspector + Code in one scroll

**egui_dock Research (Deferred):**
- Researched `egui_dock` v0.18 for full docking system
- Compatible with egui 0.33, but requires significant refactoring
- Current tabbed approach provides 80% of UX benefit with 20% of complexity
- Full docking integration reserved for future if user demand exists

### Phase 6 Feature: Live Preview Mode
- **Preview mode toggle** - F5 or View menu to switch between Edit and Preview modes
- **Toolbar indicator** - Color-coded Edit/Preview button shows current mode
- **Selection handles hidden** - In preview mode, no selection boxes or resize handles
- **Widget interaction** - Interact with widgets (click buttons, type in text fields) in preview mode
- **Shortcuts updated** - Added F5 to palette shortcuts list

### Phase 4 Implementation (Code Generation)
- **Syntax highlighting** - Added `syntect` crate for Rust code highlighting in code preview
- **Highlighter module** - New `src/highlight.rs` with `Highlighter` struct and `layout_job()` method
- **Auto-generate toggle** - Settings > Code Generation > "Auto-generate code" option
- **View menu option** - Toggle syntax highlighting on/off for performance
- **Code generation formats** - `CodeGenFormat` enum with Single File, Separate Files, UI Only modes
- **Separate files mode** - Shows Cargo.toml + main.rs with clear file headers
- **UI Only mode** - Extracts just the UI function for embedding in existing apps
- **Include comments** - Toggle to add explanatory comments to generated code
- **Improved formatting** - Better indentation and structure in generated code
- **16 unit tests** - Added 3 highlighter tests (16 total)

### Phase 2 & 3 Implementation
- **Multi-select support** - Changed selection from `Option<WidgetId>` to `Vec<WidgetId>` for multi-widget operations
- **Shift+click selection** - Toggle selection by holding Shift while clicking widgets
- **Native file dialogs** - Added `rfd` crate for New/Open/Save/Save As file operations
- **Alignment menu** - Full Align menu with:
  - Horizontal alignment (left, center, right)
  - Vertical alignment (top, middle, bottom)
  - Distribute evenly (horizontal, vertical)
  - Match sizes (width, height)
- **Status messages** - Auto-clearing status bar for user feedback
- **Edit menu** - Added Delete, Duplicate, Copy, Paste, Select All, Deselect All
- **Selection count** - Shows number of selected widgets in menu bar
- **Code quality** - Fixed all Clippy warnings

### Infrastructure
- **WidgetId derives** - Added `PartialOrd` and `Ord` for comparison operations
- **DockArea derives** - Simplified with `#[derive(Default)]` attribute
- **Helper methods** - Added selection helpers with `#[allow(dead_code)]` for future use

---

## Previous Changes (2025-12-26)

### New Widgets Added (15 new types, 34 total)
- **TextArea** - Multi-line text editing
- **DragValue** - Compact numeric input with drag-to-adjust
- **Spinner** - Loading/progress indicator
- **ColorPicker** - RGBA color selection with picker UI
- **Code** - Code editor with monospace font and syntax styling
- **Heading** - Large heading text
- **Small** - Small text for secondary content
- **Monospace** - Monospace text for code/values
- **Image** - Image placeholder with URI (generates egui::Image code)
- **Placeholder** - Colored rectangle for layout mockups
- **Group** - Container with title, border, and horizontal/vertical layout
- **ScrollBox** - Scrollable content area (egui::ScrollArea)
- **TabBar** - Tab selection bar with configurable tabs
- **Columns** - Multi-column layout container (1-10 columns)
- **Window** - Floating window with title bar (egui::Window)

### Keyboard Shortcuts Implemented
| Shortcut | Action |
|----------|--------|
| `Arrow Keys` | Nudge selected widget by grid size |
| `Delete` / `Backspace` | Delete selected widget |
| `Ctrl+C` | Copy selected widget |
| `Ctrl+V` | Paste widget |
| `Ctrl+D` | Duplicate selected widget |
| `]` | Bring widget to front (z-order) |
| `[` | Send widget to back (z-order) |
| `Ctrl+G` | Generate code |
| `F5` | Toggle Preview/Edit mode |

### UX Improvements
- Widget clipboard for copy/paste operations
- Arrow key nudging respects grid size
- Z-order controls for widget layering
- Tooltip property available for all widgets
- Group containers support horizontal/vertical layout toggle
- Scrollable palette with collapsible widget categories (Basic, Input, Display, Containers, Advanced)

---

## Architecture Analysis

### Codebase Structure

```
src/
├── main.rs        (47 lines)   - Entry point, window initialization
├── app.rs         (1668 lines) - Core application logic (needs splitting)
├── project.rs     (26 lines)   - Project data model
└── widget/
    └── mod.rs     (119 lines)  - Widget types and utilities
```

**Total:** ~1,860 lines of Rust

### Design Pattern

The codebase follows an MVC-style architecture:
- **Model:** `Project` and `Widget` types define the data
- **View:** GUI rendering in `preview_panels_ui()`, `draw_widget()`, palette/inspector UI
- **Controller:** Event handling and state management in `RadBuilderApp`

### Supported Widgets (34 types)

**Basic:** Label, Heading, Small, Monospace, Button, ImageTextButton, Checkbox, Link, Hyperlink, SelectableLabel, Separator

**Input:** TextEdit, TextArea, Password, Slider, DragValue, ComboBox, RadioGroup, DatePicker, AngleSelector, ColorPicker

**Display:** Image, Placeholder, Spinner, ProgressBar

**Containers:** Group (horizontal/vertical layout), ScrollBox, Columns (1-10), TabBar, Window

**Advanced:** MenuButton, CollapsingHeader, Tree, Code

---

## Strengths

1. **Clean separation of concerns** - Well-organized module structure
2. **Type safety** - Extensive use of Rust enums for widget types and dock areas
3. **Full serialization support** - JSON import/export works reliably
4. **Sophisticated code generation** - Produces complete, compilable Rust/egui applications
5. **Grid snapping** - Configurable 1-64px grid alignment
6. **Docking system** - Widgets can be placed in 6 areas (Free, Top, Bottom, Left, Right, Center)
7. **Interactive preview** - Live widget manipulation with selection handles

---

## Identified Issues & Improvement Opportunities

### Critical Priority

#### 1. Large Monolithic File (`app.rs`)
**Problem:** At 1,668 lines, `app.rs` handles too many concerns:
- Application state
- Widget spawning
- Canvas rendering
- Widget drawing
- Inspector UI
- Palette UI
- Menu bar
- Code generation (544 lines alone!)

**Solution:** Split into focused modules:
```
src/
├── app/
│   ├── mod.rs           - RadBuilderApp struct and main update loop
│   ├── canvas.rs        - preview_panels_ui(), draw_widget(), draw_grid()
│   ├── palette.rs       - palette_ui(), palette_item()
│   ├── inspector.rs     - inspector_ui()
│   ├── menubar.rs       - top_bar()
│   └── codegen.rs       - generate_code() and all code emission logic
```

#### 2. No Automated Tests
**Problem:** Zero test coverage. The project relies entirely on manual testing.

**Solution:** Add test suites:
- Unit tests for `snap_pos_with_grid()`, `escape()`, widget defaults
- Integration tests for code generation (parse generated code, verify it compiles)
- Property-based tests for serialization roundtrips

#### 3. No CI/CD Pipeline
**Problem:** Only FUNDING.yml exists in `.github/workflows/`. No automated builds or tests.

**Solution:** Add GitHub Actions workflow:
```yaml
- cargo fmt --check
- cargo clippy
- cargo test
- cargo build --release
```

### High Priority

#### 4. Duplicated Widget Size Constants
**Problem:** Widget default sizes are defined twice:
- In `spawn_widget()` (lines 101-263)
- In ghost preview in `preview_panels_ui()` (lines 412-432)

**Solution:** Create a `WidgetKind::default_size()` method:
```rust
impl WidgetKind {
    pub fn default_size(&self) -> Vec2 {
        match self {
            WidgetKind::Label => vec2(140.0, 24.0),
            WidgetKind::Button => vec2(160.0, 32.0),
            // ...
        }
    }
}
```

#### 5. No Undo/Redo Support
**Problem:** Users cannot undo accidental deletions or modifications.

**Solution:** Implement command pattern with history stack:
```rust
enum Command {
    AddWidget(Widget),
    DeleteWidget(WidgetId),
    MoveWidget(WidgetId, Pos2, Pos2),
    ResizeWidget(WidgetId, Vec2, Vec2),
    ModifyProps(WidgetId, WidgetProps, WidgetProps),
}

struct History {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
}
```

#### 6. No Native File Save/Load
**Problem:** Users must copy JSON from editor and paste it back in. No native file dialogs.

**Solution:** Add `rfd` (Rust File Dialog) crate for native save/load:
```rust
// File menu additions:
- Save Project (Ctrl+S)
- Save Project As...
- Open Project (Ctrl+O)
- Recent Projects submenu
```

#### 7. Missing Error Handling
**Problem:** JSON parsing failures are silently ignored:
```rust
// Current (line 1032):
if let Ok(p) = serde_json::from_str::<Project>(&self.generated) {
    self.project = p;
}
// User gets no feedback on failure
```

**Solution:** Add error state and display:
```rust
error_message: Option<String>,
// Display in UI when present
```

### Medium Priority

#### 8. Widget Registry/Plugin System (Inspired by Mobius-ECS)
**Problem:** Adding new widgets requires modifying 4+ places in `app.rs`.

**Solution:** Create a widget registry following Mobius-ECS's component-based approach:
```rust
/// Widget factory trait - each widget type implements this
trait WidgetFactory: Send + Sync {
    fn kind(&self) -> WidgetKind;
    fn display_name(&self) -> &str;
    fn category(&self) -> WidgetCategory;  // Basic, Input, Display, Containers, Advanced
    fn default_size(&self) -> Vec2;
    fn default_props(&self) -> WidgetProps;
    fn draw(&self, ui: &mut Ui, widget: &mut Widget);
    fn draw_inspector(&self, ui: &mut Ui, widget: &mut Widget);
    fn emit_code(&self, widget: &Widget, origin: &str) -> String;
    fn palette_icon(&self) -> &str;  // Emoji or icon for palette
}

/// Central registry - inspired by ECS entity management
struct WidgetRegistry {
    factories: HashMap<WidgetKind, Arc<dyn WidgetFactory>>,
    categories: HashMap<WidgetCategory, Vec<WidgetKind>>,
}

impl WidgetRegistry {
    pub fn register(&mut self, factory: impl WidgetFactory + 'static) { ... }
    pub fn spawn(&self, kind: WidgetKind, pos: Pos2) -> Widget { ... }
    pub fn draw(&self, ui: &mut Ui, widget: &mut Widget) { ... }
}
```

**Benefits (following Mobius patterns):**
- Single point of widget definition (DRY)
- Easy to add new widgets without touching core code
- Enables future plugin/extension system
- Categories auto-populate palette sections

#### 9. Keyboard Shortcuts ✅ IMPLEMENTED
All basic shortcuts now available:
- `Delete` - Delete selected widget
- `Ctrl+C/V` - Copy/paste widget
- `Ctrl+D` - Duplicate widget
- `Ctrl+G` - Generate code
- `Arrow keys` - Nudge selected widget
- `] / [` - Z-order controls

**Still needed:**
- `Ctrl+Z/Y` - Undo/redo
- `Ctrl+S` - Save project

#### 10. Widget Alignment Tools
**Problem:** No way to align multiple widgets.

**Solution:** Add alignment toolbar when multiple widgets selected:
- Align left/center/right
- Align top/middle/bottom
- Distribute horizontally/vertically
- Match widths/heights

#### 11. Z-Order Controls ✅ IMPLEMENTED
Now available via keyboard:
- `]` - Bring to Front
- `[` - Send to Back

### Lower Priority (Future Enhancements)

#### 12. Multi-Page/Screen Support (from TODO)
Allow designing multiple screens/views that can be navigated between.

#### 13. Theming Support (from TODO)
- Font family, size, weight options
- Color customization per widget
- Dark/light theme preview
- Export theme as separate struct

#### 14. Additional Widgets (from TODO + egui demo + Mobius analysis)
**Added:** TextArea, DragValue, Spinner, ColorPicker, Code, Heading, Image, Placeholder, Group, ScrollBox, Small, Monospace, TabBar, Columns, Window

**Also added:** Tooltip property for all widgets

**Still needed (prioritized from egui demo analysis):**

| Priority | Widget | Source | Notes |
|----------|--------|--------|-------|
| High | Table/Grid | egui_extras | Configurable striping, spacing (egui demo pattern) |
| High | Plot/Chart | egui_plot | Common visualization need |
| Medium | Toolbar | Mobius | ControlsPanel-style button row |
| Medium | Statusbar | Mobius | EventLoggerPanel inspiration |
| Medium | Right-click context menu | egui | Native egui support |
| Medium | Image+Button combo | egui demo | Icon buttons with text |
| Low | NodeGraph | egui_node_graph2 | From README TODO |
| Low | TreeView | egui_ltreeview | From README TODO |

**Widget Inspector Improvements (from egui demo):**
- Add suffix display for Slider (e.g., "°" for angles)
- Add speed config for DragValue
- Add animated ProgressBar option
- Add hint text for TextEdit (placeholder text)

#### 15. Live Preview Mode ✅ IMPLEMENTED
Toggle between edit mode (current) and preview mode (interact with widgets without selection handles).
- F5 keyboard shortcut or View > Preview Mode menu item
- Toolbar shows Edit/Preview indicator with toggle button

#### 16. Code Generation Improvements
- Generate idiomatic Rust (not string concatenation)
- Option to generate separate files (state.rs, ui.rs, main.rs)
- Generate event handlers as closures
- Add comments explaining generated code
- **Real-time code generation** while placing components (from Issue #15)
- Syntax highlighting in code preview

#### 17. Project Templates
Starter templates:
- Settings dialog
- Login form
- Dashboard layout
- Wizard/multi-step form

#### 18. Improved Palette UX
**Problem:** With 34+ widgets, the palette can overflow the available space and is hard to navigate.

**Solution:**
- Wrap palette contents in a ScrollArea
- Make widget categories collapsible (CollapsingHeader for each section)
- Consider search/filter functionality for quick widget finding

---

## Community Feedback (GitHub Issue #15)

Discussion with saturn77 (mobius-designer) identified shared priorities and integration opportunities:

### Top Priorities (saturn77)
1. **Alignment Features** - Horizontal and vertical alignment with spacing controls
2. **Core UI Elements** - Buttons, radio boxes, checkboxes, and plotting elements ✅ (mostly implemented)
3. **Real-time Code Generation** - Live code generation while placing components with syntax highlighting

### Secondary Requirements
- Project scaffolding and skeleton generation
- UI element styling capabilities
- Syntax highlighting as "a significant stylistic effect"

### Integration Architecture (saturn77)
Leverage `egui_dock` as an application template layer where generated RAD windows integrate into a larger framework, promoting modular design patterns.

### Desired Features (timschmidt)
- **Ingest existing Rust/egui code** - Parse and edit existing UI code
- **Group selection and distribution** - Align, distribute, match sizes
- **Multi-page/multi-screen support** - Design multiple views with navigation
- **Modal support** - Dialog windows and popups ✅ (Window widget added)
- **egui_dock integration** - Modular window-based architecture

---

## Suggested Roadmap

### Phase 1: Foundation (Code Quality) ✅ COMPLETE
1. ~~Split `app.rs` into modules~~ (Partial: widget module expanded with WidgetKind methods)
2. ~~Extract duplicated widget size constants~~ ✅ `WidgetKind::default_size()` + `default_props()`
3. ~~Add basic unit tests~~ ✅ 13 tests covering core functions
4. ~~Set up GitHub Actions CI~~ ✅ check, fmt, clippy, test, build jobs
5. ~~Implement WidgetCategory enum~~ ✅ Mobius-inspired category system with `WidgetKind::category()`, `display_name()`, `all()`

### Phase 2: Core UX Improvements *(High Priority from Issue #15)* ✅ MOSTLY COMPLETE
1. ~~Add keyboard shortcuts~~ ✅
2. ~~Add widget copy/paste~~ ✅
3. ~~Z-order controls~~ ✅
4. ~~Improved palette (scrollable, collapsible categories)~~ ✅
5. Implement undo/redo (Command pattern) - *Deferred*
6. ~~Add native file save/load (`rfd` crate)~~ ✅ File menu with New/Open/Save/Save As
7. ~~Add error handling with user feedback~~ ✅ Status message display
8. **NEW:** Add `.on_hover_text()` tooltips throughout UI (egui best practice) ✅ Partial - menu items have tooltips

### Phase 3: Alignment & Selection *(Top Priority from Issue #15 + Mobius)* ✅ MOSTLY COMPLETE
1. ~~Multi-select widgets (Shift+click)~~ ✅ Shift+click toggle selection
2. ~~Alignment tools (left/center/right, top/middle/bottom)~~ ✅ Align menu with all options
3. ~~Distribution tools (horizontal/vertical spacing)~~ ✅ Distribute evenly across selected widgets
4. ~~Match sizes (width/height)~~ ✅ Match width/height to first selected
5. Group/ungroup widgets - *Deferred*
6. ~~Grid snapping with visual guides~~ ✅ Grid display with configurable size

### Phase 4: Code Generation *(Priority from Issue #15 + Mobius)* ✅ MOSTLY COMPLETE
1. ~~Real-time code generation while placing components~~ ✅ Auto-generate toggle in Settings
2. ~~Syntax highlighting in code preview (`syntect` - same as Mobius)~~ ✅ Toggle in View menu
3. ~~Generate idiomatic Rust code~~ ✅ Better formatting, proper indentation
4. ~~Option to generate separate files~~ ✅ CodeGenFormat enum (Single File, Separate Files, UI Only)
5. ~~Project scaffolding/skeleton generation~~ ✅ Cargo.toml generation in Separate Files mode
6. **NEW:** Consider signals/slots pattern in generated code (Mobius `egui_mobius`) - *Planned*

### Phase 5: Architecture Evolution *(Inspired by Mobius-ECS)* - PARTIAL
1. **Two-tier separation:** Core library (`egui-rad-widgets`) + Builder app
2. ✅ Panel tabs for Inspector/Code switching *(simpler alternative to egui_dock)*
3. `egui_dock` full docking system *(researched, deferred - see Recent Changes)*
4. Optional ECS-based widget management for complex projects
5. Template system for declarative UI definitions
6. Hot-reload support for live development

**Status:** Tabbed panel interface implemented. Full egui_dock integration researched but deferred in favor of simpler tab solution.

### Phase 6: Advanced Features
1. Multi-page/screen support with navigation
2. Plot/Chart widget (egui_plot integration)
3. Ingest and edit existing Rust/egui code
4. Theming and styling options (following egui demo patterns)
5. ✅ Live preview mode (toggle edit handles on/off)

### Phase 7: Polish & Ecosystem
1. Project templates (settings dialog, login form, dashboard)
2. Documentation and tutorials
3. Performance optimization
4. Accessibility improvements
5. **NEW:** Community widget contributions (via WidgetFactory trait)

---

## Feature Comparison: egui-rad-builder vs Mobius-ECS

| Feature | egui-rad-builder | Mobius-ECS | Notes |
|---------|------------------|------------|-------|
| **Core Framework** | Pure egui/eframe | bevy_ecs + egui | Mobius uses ECS for modularity |
| **Widget Count** | 34 types | ~6 panel types | We have more widgets; Mobius focuses on architecture |
| **Code Generation** | ✅ Complete apps | ✅ Complete apps | Both generate production-ready code |
| **Visual Alignment** | ✅ Align, distribute, match sizes | ✅ Full alignment tools | Feature parity achieved |
| **Docking System** | Panel-based | egui_dock | Consider egui_dock integration |
| **Syntax Highlighting** | ✅ syntect | ✅ syntect | Feature parity achieved |
| **Hot Reload** | ❌ | ✅ | Future consideration |
| **Live Preview** | ✅ F5 toggle | ❌ | Test widgets without handles |
| **Signals/Slots** | ❌ | ✅ egui_mobius | Event communication pattern |
| **Template System** | ❌ | ✅ Declarative | Future phase |
| **Project Structure** | Single app | Designer + Framework | Consider separation |
| **Serialization** | ✅ JSON | ✅ | Both support project save/load |

### Key Takeaways

**Strengths of egui-rad-builder:**
- More comprehensive widget library (34 vs ~6)
- Simpler architecture (easier to understand/contribute)
- Already has working code generation

**Areas to Learn from Mobius:**
- Visual alignment tools (top priority)
- ECS-based widget registry pattern
- Syntax highlighting with `syntect`
- Two-tier architecture (library + app separation)
- `egui_dock` for professional docking UI

**Areas to Learn from egui Demo:**
- Consistent `.on_hover_text()` documentation
- Grid-based inspector layout
- Scope isolation for grouped widgets
- Progressive disclosure with CollapsingHeader

---

## Technical Debt Notes

1. **Rust Edition 2024** in Cargo.toml appears non-standard (should be 2021)
2. `edit_mode` is stored in egui's temp data instead of app state (line 726-729)
3. Some `WidgetKind` arms in inspector use catch-all `_ => {}` that should be exhaustive
4. Generated code uses `from_id_source()` which is deprecated in favor of `from_id_salt()`
5. Tree widget parsing is duplicated between `draw_widget()` and `generate_code()`
6. **NEW:** Consider adding `egui_dock` dependency for modular window management
7. **NEW:** Consider `syntect` for syntax highlighting in code output

---

## Development Tips

### Building
```bash
cargo build
cargo run
```

### Testing Generated Code
```bash
# Generate code in the tool, save to test_app/src/main.rs
cd test_app
cargo run
```

### Useful Commands
```bash
cargo fmt          # Format code
cargo clippy       # Lint
cargo doc --open   # Generate docs
```

---

## Reference Links

- **Mobius-ECS**: https://github.com/saturn77/mobius-ecs
- **egui Demo**: https://www.egui.rs
- **egui Widget Gallery Source**: https://github.com/emilk/egui/blob/main/crates/egui_demo_lib/src/demo/widget_gallery.rs
- **egui Documentation**: https://docs.rs/egui/latest/egui/
- **egui_dock**: https://github.com/Adanos020/egui_dock
- **syntect**: https://github.com/trishume/syntect

---

*Last updated: 2025-12-27*
*Analysis performed by Claude with design insights from mobius-ecs and egui demo*
