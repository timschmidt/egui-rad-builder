//! Undo and redo history tracking for the RAD builder.
//!
//! Stores project snapshots so actions can be undone and redone cleanly without
//! complex inverse command state machines.

use crate::project::Project;
use crate::widget::WidgetId;

/// A snapshot of the builder state captured for undo and redo operations.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HistorySnapshot {
    pub(crate) project: Project,
    pub(crate) selected: Vec<WidgetId>,
    pub(crate) next_id: u64,
}

impl HistorySnapshot {
    pub(crate) fn new(project: Project, selected: Vec<WidgetId>, next_id: u64) -> Self {
        Self {
            project,
            selected,
            next_id,
        }
    }
}

/// History manager holding undo and redo stacks with configurable capacity.
#[derive(Clone, Debug)]
pub(crate) struct History {
    undo_stack: Vec<HistorySnapshot>,
    redo_stack: Vec<HistorySnapshot>,
    max_depth: usize,
    /// Pending snapshot captured at the start of an interactive action (drag, resize, slider, etc.)
    pending_snapshot: Option<HistorySnapshot>,
}

impl History {
    /// Creates a new `History` instance with the given maximum stack depth.
    pub(crate) fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: max_depth.max(1),
            pending_snapshot: None,
        }
    }

    /// Returns `true` if there are actions that can be undone.
    pub(crate) fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there are actions that can be redone.
    pub(crate) fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Pushes a discrete snapshot onto the undo stack.
    /// Clears any redo history and discards any pending interactive snapshot.
    pub(crate) fn push(&mut self, snapshot: HistorySnapshot) {
        self.pending_snapshot = None;
        if let Some(top) = self.undo_stack.last()
            && top.project == snapshot.project
        {
            return;
        }
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Records the initial snapshot of an ongoing interactive operation if one is not already tracked.
    pub(crate) fn set_pending_if_none(&mut self, snapshot: HistorySnapshot) {
        if self.pending_snapshot.is_none() {
            self.pending_snapshot = Some(snapshot);
        }
    }

    /// Commits the pending interactive snapshot if the project actually changed.
    pub(crate) fn commit_pending_if_changed(&mut self, current_project: &Project) {
        if let Some(pending) = self.pending_snapshot.take()
            && &pending.project != current_project
        {
            self.push(pending);
        }
    }

    /// Returns whether an interactive action is currently pending.
    #[allow(dead_code)]
    pub(crate) fn has_pending(&self) -> bool {
        self.pending_snapshot.is_some()
    }

    /// Discards any pending interactive snapshot without committing it.
    #[allow(dead_code)]
    pub(crate) fn cancel_pending(&mut self) {
        self.pending_snapshot = None;
    }

    /// Undoes the last operation, moving the current state to the redo stack
    /// and returning the prior snapshot to be restored.
    pub(crate) fn undo(&mut self, current: HistorySnapshot) -> Option<HistorySnapshot> {
        self.pending_snapshot = None;
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current);
            Some(prev)
        } else {
            None
        }
    }

    /// Redoes the last undone operation, moving the current state to the undo stack
    /// and returning the next snapshot to be restored.
    pub(crate) fn redo(&mut self, current: HistorySnapshot) -> Option<HistorySnapshot> {
        self.pending_snapshot = None;
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current);
            Some(next)
        } else {
            None
        }
    }

    /// Clears all undo and redo history.
    pub(crate) fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.pending_snapshot = None;
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{DockArea, Widget, WidgetKind, WidgetProps};
    use egui::{pos2, vec2};

    fn make_test_widget(id: u64, x: f32, y: f32) -> Widget {
        Widget {
            id: WidgetId::new(id),
            kind: WidgetKind::Button,
            pos: pos2(x, y),
            size: vec2(100.0, 30.0),
            z: id as i32,
            area: DockArea::Free,
            props: WidgetProps::default(),
        }
    }

    fn make_test_snapshot(widget_count: usize, x_offset: f32) -> HistorySnapshot {
        let mut p = Project::default();
        for i in 1..=widget_count {
            p.widgets
                .push(make_test_widget(i as u64, x_offset * i as f32, 10.0));
        }
        let sel = if widget_count > 0 {
            vec![WidgetId::new(1)]
        } else {
            Vec::new()
        };
        HistorySnapshot::new(p, sel, (widget_count + 1) as u64)
    }

    #[test]
    fn test_history_empty_state() {
        let history = History::default();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_push_undo_redo_cycle() {
        let mut history = History::default();
        let s0 = make_test_snapshot(0, 0.0);
        let s1 = make_test_snapshot(1, 10.0);
        let s2 = make_test_snapshot(2, 20.0);

        // Transition from s0 -> s1: push s0
        history.push(s0.clone());
        assert!(history.can_undo());
        assert!(!history.can_redo());

        // Transition from s1 -> s2: push s1
        history.push(s1.clone());
        assert!(history.can_undo());
        assert!(!history.can_redo());

        // Undo from s2: current is s2, restores s1
        let restored = history.undo(s2.clone()).expect("Should undo s1");
        assert_eq!(restored, s1);
        assert!(history.can_undo());
        assert!(history.can_redo());

        // Undo from s1: current is s1, restores s0
        let restored = history.undo(s1.clone()).expect("Should undo s0");
        assert_eq!(restored, s0);
        assert!(!history.can_undo());
        assert!(history.can_redo());

        // Redo from s0: current is s0, restores s1
        let restored = history.redo(s0.clone()).expect("Should redo s1");
        assert_eq!(restored, s1);
        assert!(history.can_undo());
        assert!(history.can_redo());

        // Redo from s1: current is s1, restores s2
        let restored = history.redo(s1.clone()).expect("Should redo s2");
        assert_eq!(restored, s2);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_push_clears_redo() {
        let mut history = History::default();
        let s0 = make_test_snapshot(0, 0.0);
        let s1 = make_test_snapshot(1, 10.0);
        let s2 = make_test_snapshot(2, 20.0);
        let s3 = make_test_snapshot(3, 30.0);

        history.push(s0.clone());
        history.push(s1.clone());

        let _ = history.undo(s2);
        assert!(history.can_redo());

        // Doing a new action pushes s3 and must clear the redo stack
        history.push(s3);
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_deduplicate_identical_project() {
        let mut history = History::default();
        let s1 = make_test_snapshot(1, 10.0);

        history.push(s1.clone());
        assert_eq!(history.undo_stack.len(), 1);

        // Pushing identical snapshot should be a no-op
        history.push(s1);
        assert_eq!(history.undo_stack.len(), 1);
    }

    #[test]
    fn test_history_max_depth() {
        let mut history = History::new(3);
        let s0 = make_test_snapshot(0, 0.0);
        let s1 = make_test_snapshot(1, 10.0);
        let s2 = make_test_snapshot(2, 20.0);
        let s3 = make_test_snapshot(3, 30.0);
        let s4 = make_test_snapshot(4, 40.0);

        // Transition from s0 -> s1 (push s0)
        history.push(s0);
        // Transition from s1 -> s2 (push s1)
        history.push(s1.clone());
        // Transition from s2 -> s3 (push s2)
        history.push(s2.clone());
        // Transition from s3 -> s4 (push s3)
        history.push(s3.clone());

        // Stack size should be capped at 3: [s1, s2, s3] (s0 was dropped)
        assert_eq!(history.undo_stack.len(), 3);

        // Undo from s4 restores s3
        let restored = history.undo(s4).unwrap();
        assert_eq!(restored, s3);

        // Undo from s3 restores s2
        let restored = history.undo(restored).unwrap();
        assert_eq!(restored, s2);

        // Undo from s2 restores s1
        let restored = history.undo(restored).unwrap();
        assert_eq!(restored, s1);

        // Stack should now be empty (s0 was dropped due to max_depth)
        assert!(!history.can_undo());
    }

    #[test]
    fn test_history_clear() {
        let mut history = History::default();
        let s1 = make_test_snapshot(1, 10.0);
        history.push(s1);
        assert!(history.can_undo());

        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_interactive_commit() {
        let mut history = History::default();
        let s0 = make_test_snapshot(1, 10.0);
        let s1 = make_test_snapshot(1, 50.0);

        // Start interaction
        history.set_pending_if_none(s0.clone());
        // Subsequent calls while dragging don't overwrite initial state
        history.set_pending_if_none(s1.clone());
        assert!(history.has_pending());

        // If project changed, commit pending snapshot
        history.commit_pending_if_changed(&s1.project);
        assert!(!history.has_pending());
        assert!(history.can_undo());

        let restored = history.undo(s1).unwrap();
        assert_eq!(restored, s0);
    }

    #[test]
    fn test_history_interactive_unchanged_discard() {
        let mut history = History::default();
        let s0 = make_test_snapshot(1, 10.0);

        // Start interaction but release without changes
        history.set_pending_if_none(s0.clone());
        history.commit_pending_if_changed(&s0.project);

        assert!(!history.has_pending());
        assert!(!history.can_undo());
    }
}
