//! Split View Support
//!
//! Provides horizontal and vertical split views for the terminal interface,
//! allowing multiple sessions to be viewed simultaneously.

use egui::{self, Rect, Pos2, Color32, Stroke, CursorIcon, StrokeKind};
use std::collections::HashMap;

/// Split orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal, // Side by side
    Vertical,   // Top and bottom
}

/// A pane in a split view
#[derive(Debug, Clone)]
pub struct Pane {
    /// Unique pane ID
    pub id: usize,
    /// Session ID displayed in this pane (if any)
    pub session_id: Option<usize>,
    /// Pane title
    pub title: String,
    /// Is this pane focused?
    pub focused: bool,
}

impl Pane {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            session_id: None,
            title: format!("Pane {}", id),
            focused: false,
        }
    }
    
    pub fn with_session(mut self, session_id: usize) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// A split node in the tree
#[derive(Debug, Clone)]
pub enum SplitNode {
    /// Leaf node - contains a pane
    Leaf(Pane),
    /// Split node - contains two children
    Split {
        orientation: SplitOrientation,
        /// Split ratio (0.0 to 1.0)
        ratio: f32,
        /// First child (left or top)
        first: Box<SplitNode>,
        /// Second child (right or bottom)
        second: Box<SplitNode>,
    },
}

impl SplitNode {
    /// Create a new leaf node
    pub fn leaf(pane: Pane) -> Self {
        Self::Leaf(pane)
    }
    
    /// Create a horizontal split
    pub fn horizontal(first: SplitNode, second: SplitNode, ratio: f32) -> Self {
        Self::Split {
            orientation: SplitOrientation::Horizontal,
            ratio: ratio.clamp(0.1, 0.9),
            first: Box::new(first),
            second: Box::new(second),
        }
    }
    
    /// Create a vertical split
    pub fn vertical(first: SplitNode, second: SplitNode, ratio: f32) -> Self {
        Self::Split {
            orientation: SplitOrientation::Vertical,
            ratio: ratio.clamp(0.1, 0.9),
            first: Box::new(first),
            second: Box::new(second),
        }
    }
    
    /// Count total panes
    pub fn pane_count(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Split { first, second, .. } => first.pane_count() + second.pane_count(),
        }
    }
    
    /// Get all panes
    pub fn get_panes(&self) -> Vec<&Pane> {
        match self {
            Self::Leaf(pane) => vec![pane],
            Self::Split { first, second, .. } => {
                let mut panes = first.get_panes();
                panes.extend(second.get_panes());
                panes
            }
        }
    }
    
    /// Get mutable reference to all panes
    pub fn get_panes_mut(&mut self) -> Vec<&mut Pane> {
        match self {
            Self::Leaf(pane) => vec![pane],
            Self::Split { first, second, .. } => {
                let mut panes = first.get_panes_mut();
                panes.extend(second.get_panes_mut());
                panes
            }
        }
    }
    
    /// Find pane by ID
    pub fn find_pane(&self, id: usize) -> Option<&Pane> {
        match self {
            Self::Leaf(pane) if pane.id == id => Some(pane),
            Self::Leaf(_) => None,
            Self::Split { first, second, .. } => {
                first.find_pane(id).or_else(|| second.find_pane(id))
            }
        }
    }
    
    /// Find pane by ID (mutable)
    pub fn find_pane_mut(&mut self, id: usize) -> Option<&mut Pane> {
        match self {
            Self::Leaf(pane) if pane.id == id => Some(pane),
            Self::Leaf(_) => None,
            Self::Split { first, second, .. } => {
                if let Some(pane) = first.find_pane_mut(id) {
                    Some(pane)
                } else {
                    second.find_pane_mut(id)
                }
            }
        }
    }
    
    /// Get focused pane
    pub fn get_focused(&self) -> Option<&Pane> {
        match self {
            Self::Leaf(pane) if pane.focused => Some(pane),
            Self::Leaf(_) => None,
            Self::Split { first, second, .. } => {
                first.get_focused().or_else(|| second.get_focused())
            }
        }
    }
    
    /// Set focus to pane
    pub fn set_focus(&mut self, id: usize) {
        for pane in self.get_panes_mut() {
            pane.focused = pane.id == id;
        }
    }
}

/// Split view state
#[derive(Debug)]
pub struct SplitView {
    /// Root node of the split tree
    pub root: SplitNode,
    /// Next pane ID to assign
    next_pane_id: usize,
    /// Minimum pane size
    pub min_pane_size: f32,
    /// Splitter width
    pub splitter_width: f32,
    /// Currently dragging splitter
    dragging_splitter: Option<SplitterId>,
}

#[derive(Debug, Clone, Copy)]
struct SplitterId {
    path: [u8; 8], // Path through tree (0 = first, 1 = second)
    depth: usize,
}

impl Default for SplitView {
    fn default() -> Self {
        Self::new()
    }
}

impl SplitView {
    /// Create new split view with single pane
    pub fn new() -> Self {
        let pane = Pane::new(0);
        Self {
            root: SplitNode::leaf(pane),
            next_pane_id: 1,
            min_pane_size: 100.0,
            splitter_width: 4.0,
            dragging_splitter: None,
        }
    }
    
    /// Get next pane ID
    fn allocate_pane_id(&mut self) -> usize {
        let id = self.next_pane_id;
        self.next_pane_id += 1;
        id
    }
    
    /// Split a pane horizontally
    pub fn split_horizontal(&mut self, pane_id: usize) -> Option<usize> {
        self.split_pane(pane_id, SplitOrientation::Horizontal)
    }
    
    /// Split a pane vertically
    pub fn split_vertical(&mut self, pane_id: usize) -> Option<usize> {
        self.split_pane(pane_id, SplitOrientation::Vertical)
    }
    
    fn split_pane(&mut self, pane_id: usize, orientation: SplitOrientation) -> Option<usize> {
        let new_id = self.allocate_pane_id();
        
        fn split_recursive(
            node: &mut SplitNode,
            target_id: usize,
            orientation: SplitOrientation,
            new_pane: Pane,
        ) -> bool {
            match node {
                SplitNode::Leaf(pane) if pane.id == target_id => {
                    // Found the pane to split
                    let old_pane = std::mem::replace(pane, Pane::new(0)); // Placeholder
                    let new_node = match orientation {
                        SplitOrientation::Horizontal => {
                            SplitNode::horizontal(
                                SplitNode::leaf(old_pane),
                                SplitNode::leaf(new_pane),
                                0.5,
                            )
                        }
                        SplitOrientation::Vertical => {
                            SplitNode::vertical(
                                SplitNode::leaf(old_pane),
                                SplitNode::leaf(new_pane),
                                0.5,
                            )
                        }
                    };
                    *node = new_node;
                    true
                }
                SplitNode::Leaf(_) => false,
                SplitNode::Split { first, second, .. } => {
                    split_recursive(first, target_id, orientation, new_pane.clone())
                        || split_recursive(second, target_id, orientation, new_pane)
                }
            }
        }
        
        let new_pane = Pane::new(new_id);
        if split_recursive(&mut self.root, pane_id, orientation, new_pane) {
            Some(new_id)
        } else {
            None
        }
    }
    
    /// Close a pane
    pub fn close_pane(&mut self, pane_id: usize) -> bool {
        if self.root.pane_count() <= 1 {
            return false; // Can't close the last pane
        }
        
        fn close_recursive(node: &mut SplitNode, target_id: usize) -> Option<SplitNode> {
            match node {
                SplitNode::Leaf(pane) if pane.id == target_id => {
                    None // This node should be removed
                }
                SplitNode::Leaf(_) => Some(node.clone()),
                SplitNode::Split { first, second, .. } => {
                    let first_result = close_recursive(first, target_id);
                    let second_result = close_recursive(second, target_id);
                    
                    match (first_result, second_result) {
                        (None, Some(remaining)) => Some(remaining),
                        (Some(remaining), None) => Some(remaining),
                        (Some(f), Some(s)) => {
                            if let SplitNode::Split { orientation, ratio, .. } = node {
                                Some(SplitNode::Split {
                                    orientation: *orientation,
                                    ratio: *ratio,
                                    first: Box::new(f),
                                    second: Box::new(s),
                                })
                            } else {
                                unreachable!()
                            }
                        }
                        (None, None) => None,
                    }
                }
            }
        }
        
        if let Some(new_root) = close_recursive(&mut self.root, pane_id) {
            self.root = new_root;
            true
        } else {
            false
        }
    }
    
    /// Get focused pane
    pub fn focused_pane(&self) -> Option<&Pane> {
        self.root.get_focused()
    }
    
    /// Set focus to pane
    pub fn focus(&mut self, pane_id: usize) {
        self.root.set_focus(pane_id);
    }
    
    /// Focus next pane (cycle)
    pub fn focus_next(&mut self) {
        let panes = self.root.get_panes();
        if let Some(focused) = panes.iter().position(|p| p.focused) {
            let next = (focused + 1) % panes.len();
            self.focus(panes[next].id);
        } else if let Some(first) = panes.first() {
            self.focus(first.id);
        }
    }
    
    /// Focus previous pane (cycle)
    pub fn focus_prev(&mut self) {
        let panes = self.root.get_panes();
        if let Some(focused) = panes.iter().position(|p| p.focused) {
            let prev = if focused == 0 { panes.len() - 1 } else { focused - 1 };
            self.focus(panes[prev].id);
        } else if let Some(first) = panes.first() {
            self.focus(first.id);
        }
    }
    
    /// Render the split view
    /// 
    /// The `render_pane` closure is called for each pane with its rect
    pub fn show<F>(&mut self, ui: &mut egui::Ui, rect: Rect, mut render_pane: F)
    where
        F: FnMut(&mut egui::Ui, &Pane, Rect),
    {
        self.render_node(ui, &mut self.root.clone(), rect, &mut render_pane, &mut vec![]);
        
        // Update root after rendering (for drag operations)
        // This is a workaround since we can't easily borrow mutably during iteration
    }
    
    fn render_node<F>(
        &mut self,
        ui: &mut egui::Ui,
        node: &SplitNode,
        rect: Rect,
        render_pane: &mut F,
        path: &mut Vec<u8>,
    )
    where
        F: FnMut(&mut egui::Ui, &Pane, Rect),
    {
        match node {
            SplitNode::Leaf(pane) => {
                // Render pane background
                let stroke = if pane.focused {
                    Stroke::new(2.0, Color32::from_rgb(100, 150, 255))
                } else {
                    Stroke::new(1.0, Color32::from_gray(60))
                };
                
                ui.painter().rect_stroke(rect, 0.0, stroke, StrokeKind::Outside);
                
                // Call render callback
                render_pane(ui, pane, rect.shrink(2.0));
            }
            SplitNode::Split { orientation, ratio, first, second } => {
                let (first_rect, splitter_rect, second_rect) = match orientation {
                    SplitOrientation::Horizontal => {
                        let split_x = rect.left() + rect.width() * ratio;
                        let half_splitter = self.splitter_width / 2.0;
                        (
                            Rect::from_min_max(
                                rect.min,
                                Pos2::new(split_x - half_splitter, rect.max.y),
                            ),
                            Rect::from_min_max(
                                Pos2::new(split_x - half_splitter, rect.min.y),
                                Pos2::new(split_x + half_splitter, rect.max.y),
                            ),
                            Rect::from_min_max(
                                Pos2::new(split_x + half_splitter, rect.min.y),
                                rect.max,
                            ),
                        )
                    }
                    SplitOrientation::Vertical => {
                        let split_y = rect.top() + rect.height() * ratio;
                        let half_splitter = self.splitter_width / 2.0;
                        (
                            Rect::from_min_max(
                                rect.min,
                                Pos2::new(rect.max.x, split_y - half_splitter),
                            ),
                            Rect::from_min_max(
                                Pos2::new(rect.min.x, split_y - half_splitter),
                                Pos2::new(rect.max.x, split_y + half_splitter),
                            ),
                            Rect::from_min_max(
                                Pos2::new(rect.min.x, split_y + half_splitter),
                                rect.max,
                            ),
                        )
                    }
                };
                
                // Render splitter
                let splitter_response = ui.allocate_rect(splitter_rect, egui::Sense::drag());
                
                let splitter_color = if splitter_response.hovered() || splitter_response.dragged() {
                    Color32::from_rgb(100, 150, 255)
                } else {
                    Color32::from_gray(80)
                };
                
                ui.painter().rect_filled(splitter_rect, 0.0, splitter_color);
                
                // Handle drag
                if splitter_response.dragged() {
                    let delta = splitter_response.drag_delta();
                    let new_ratio = match orientation {
                        SplitOrientation::Horizontal => {
                            (ratio + delta.x / rect.width()).clamp(0.1, 0.9)
                        }
                        SplitOrientation::Vertical => {
                            (ratio + delta.y / rect.height()).clamp(0.1, 0.9)
                        }
                    };
                    
                    // Update ratio in the actual root
                    self.update_ratio_at_path(path, new_ratio);
                }
                
                // Set cursor
                if splitter_response.hovered() || splitter_response.dragged() {
                    ui.ctx().set_cursor_icon(match orientation {
                        SplitOrientation::Horizontal => CursorIcon::ResizeHorizontal,
                        SplitOrientation::Vertical => CursorIcon::ResizeVertical,
                    });
                }
                
                // Render children
                path.push(0);
                self.render_node(ui, first, first_rect, render_pane, path);
                path.pop();
                
                path.push(1);
                self.render_node(ui, second, second_rect, render_pane, path);
                path.pop();
            }
        }
    }
    
    fn update_ratio_at_path(&mut self, path: &[u8], new_ratio: f32) {
        fn update_recursive(node: &mut SplitNode, path: &[u8], new_ratio: f32) {
            if path.is_empty() {
                if let SplitNode::Split { ratio, .. } = node {
                    *ratio = new_ratio;
                }
                return;
            }
            
            if let SplitNode::Split { first, second, .. } = node {
                match path[0] {
                    0 => update_recursive(first, &path[1..], new_ratio),
                    1 => update_recursive(second, &path[1..], new_ratio),
                    _ => {}
                }
            }
        }
        
        update_recursive(&mut self.root, path, new_ratio);
    }
    
    /// Assign session to pane
    pub fn assign_session(&mut self, pane_id: usize, session_id: usize) {
        if let Some(pane) = self.root.find_pane_mut(pane_id) {
            pane.session_id = Some(session_id);
        }
    }
    
    /// Get all panes with their session IDs
    pub fn pane_sessions(&self) -> HashMap<usize, Option<usize>> {
        let mut map = HashMap::new();
        for pane in self.root.get_panes() {
            map.insert(pane.id, pane.session_id);
        }
        map
    }
}

/// Preset split layouts
pub mod layouts {
    use super::*;
    
    /// Single pane (default)
    pub fn single() -> SplitView {
        SplitView::new()
    }
    
    /// Two panes side by side (50/50)
    pub fn two_horizontal() -> SplitView {
        let mut view = SplitView::new();
        view.split_horizontal(0);
        view
    }
    
    /// Two panes stacked (50/50)
    pub fn two_vertical() -> SplitView {
        let mut view = SplitView::new();
        view.split_vertical(0);
        view
    }
    
    /// Three panes: one large on left, two stacked on right
    pub fn one_two() -> SplitView {
        let mut view = SplitView::new();
        view.split_horizontal(0); // Creates pane 1 on right
        view.split_vertical(1);   // Splits right pane
        view
    }
    
    /// Four panes in grid (2x2)
    pub fn grid_2x2() -> SplitView {
        let mut view = SplitView::new();
        view.split_horizontal(0);  // Creates pane 1 on right
        view.split_vertical(0);    // Splits left
        view.split_vertical(1);    // Splits right
        view
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_split_view_creation() {
        let view = SplitView::new();
        assert_eq!(view.root.pane_count(), 1);
    }
    
    #[test]
    fn test_horizontal_split() {
        let mut view = SplitView::new();
        let new_id = view.split_horizontal(0);
        assert!(new_id.is_some());
        assert_eq!(view.root.pane_count(), 2);
    }
    
    #[test]
    fn test_vertical_split() {
        let mut view = SplitView::new();
        let new_id = view.split_vertical(0);
        assert!(new_id.is_some());
        assert_eq!(view.root.pane_count(), 2);
    }
    
    #[test]
    fn test_close_pane() {
        let mut view = SplitView::new();
        view.split_horizontal(0);
        assert_eq!(view.root.pane_count(), 2);
        
        view.close_pane(1);
        assert_eq!(view.root.pane_count(), 1);
    }
    
    #[test]
    fn test_focus_cycle() {
        let mut view = SplitView::new();
        view.split_horizontal(0);
        view.split_horizontal(1);
        
        view.focus(0);
        assert_eq!(view.focused_pane().unwrap().id, 0);
        
        view.focus_next();
        assert_eq!(view.focused_pane().unwrap().id, 1);
        
        view.focus_next();
        assert_eq!(view.focused_pane().unwrap().id, 2);
        
        view.focus_next();
        assert_eq!(view.focused_pane().unwrap().id, 0);
    }
    
    #[test]
    fn test_layouts() {
        let single = layouts::single();
        assert_eq!(single.root.pane_count(), 1);
        
        let two_h = layouts::two_horizontal();
        assert_eq!(two_h.root.pane_count(), 2);
        
        let grid = layouts::grid_2x2();
        assert_eq!(grid.root.pane_count(), 4);
    }
}

