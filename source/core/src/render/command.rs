//! Command pattern for undoable scene mutations
//!
//! Wraps scene operations in reversible commands for undo/redo.
//! Leverages diffing primitives for efficient change tracking.

use crate::hash::ElementId;
use crate::scene::{Element, Filter, Gradient, Scene, Style, Symbol};

/// Reversible scene mutation command
#[derive(Debug, Clone)]
pub enum SceneCommand {
    /// Add element at index
    AddElement { element: Element, index: usize },
    /// Remove element by ID (stores removed element for undo)
    RemoveElement { id: ElementId, index: usize, element: Element },
    /// Modify element style
    ModifyStyle { id: ElementId, index: usize, old: Style, new: Style },
    /// Move element position (translate)
    MoveElement { id: ElementId, index: usize, dx: f32, dy: f32 },
    /// Replace element entirely
    ReplaceElement { id: ElementId, index: usize, old: Element, new: Element },
    /// Transform element (rotate/scale/skew)
    Transform { id: ElementId, index: usize, old: Option<String>, new: Option<String> },
    /// Add gradient definition
    AddGradient { gradient: Gradient },
    /// Remove gradient by id
    RemoveGradient { id: String, gradient: Gradient },
    /// Add filter definition
    AddFilter { filter: Filter },
    /// Remove filter by id
    RemoveFilter { id: String, filter: Filter },
    /// Add symbol definition
    AddSymbol { symbol: Symbol },
    /// Remove symbol by id
    RemoveSymbol { id: String, symbol: Symbol },
    /// Change canvas background
    SetBackground { old: String, new: String },
    /// Batch multiple commands (for compound operations)
    Batch(Vec<SceneCommand>),
}

impl SceneCommand {
    /// Apply command to scene, mutating it
    pub fn apply(&self, scene: &mut Scene) {
        match self {
            Self::AddElement { element, index } => {
                let els = scene.elements_mut();
                if *index >= els.len() { els.push(element.clone()); }
                else { els.insert(*index, element.clone()); }
            }
            Self::RemoveElement { index, .. } => {
                let els = scene.elements_mut();
                if *index < els.len() { els.remove(*index); }
            }
            Self::ModifyStyle { index, new, .. } => {
                if let Some(el) = scene.elements_mut().get_mut(*index) {
                    apply_style(el, new.clone());
                }
            }
            Self::MoveElement { index, dx, dy, .. } => {
                if let Some(el) = scene.elements_mut().get_mut(*index) {
                    translate_element(el, *dx, *dy);
                }
            }
            Self::ReplaceElement { index, new, .. } => {
                if let Some(el) = scene.elements_mut().get_mut(*index) {
                    *el = new.clone();
                }
            }
            Self::Transform { index, new, .. } => {
                if let Some(el) = scene.elements_mut().get_mut(*index) {
                    set_transform(el, new.clone());
                }
            }
            Self::AddGradient { gradient } => scene.push_gradient(gradient.clone()),
            Self::RemoveGradient { id, .. } => scene.remove_gradient(id),
            Self::AddFilter { filter } => scene.push_filter(filter.clone()),
            Self::RemoveFilter { id, .. } => scene.remove_filter(id),
            Self::AddSymbol { symbol } => scene.push_symbol(symbol.clone()),
            Self::RemoveSymbol { id, .. } => scene.remove_symbol(id),
            Self::SetBackground { new, .. } => scene.background = new.clone(),
            Self::Batch(cmds) => cmds.iter().for_each(|c| c.apply(scene)),
        }
    }

    /// Unapply command (reverse/undo)
    pub fn unapply(&self, scene: &mut Scene) {
        match self {
            Self::AddElement { index, .. } => {
                let els = scene.elements_mut();
                if *index < els.len() { els.remove(*index); }
            }
            Self::RemoveElement { element, index, .. } => {
                let els = scene.elements_mut();
                if *index >= els.len() { els.push(element.clone()); }
                else { els.insert(*index, element.clone()); }
            }
            Self::ModifyStyle { index, old, .. } => {
                if let Some(el) = scene.elements_mut().get_mut(*index) {
                    apply_style(el, old.clone());
                }
            }
            Self::MoveElement { index, dx, dy, .. } => {
                if let Some(el) = scene.elements_mut().get_mut(*index) {
                    translate_element(el, -*dx, -*dy);
                }
            }
            Self::ReplaceElement { index, old, .. } => {
                if let Some(el) = scene.elements_mut().get_mut(*index) {
                    *el = old.clone();
                }
            }
            Self::Transform { index, old, .. } => {
                if let Some(el) = scene.elements_mut().get_mut(*index) {
                    set_transform(el, old.clone());
                }
            }
            Self::AddGradient { gradient } => scene.remove_gradient(&gradient.id),
            Self::RemoveGradient { gradient, .. } => scene.push_gradient(gradient.clone()),
            Self::AddFilter { filter } => scene.remove_filter(&filter.id),
            Self::RemoveFilter { filter, .. } => scene.push_filter(filter.clone()),
            Self::AddSymbol { symbol } => scene.remove_symbol(&symbol.id),
            Self::RemoveSymbol { symbol, .. } => scene.push_symbol(symbol.clone()),
            Self::SetBackground { old, .. } => scene.background = old.clone(),
            Self::Batch(cmds) => cmds.iter().rev().for_each(|c| c.unapply(scene)),
        }
    }

    /// Invert command (create undo command)
    pub fn invert(&self) -> Self {
        match self {
            Self::AddElement { element, index } => Self::RemoveElement {
                id: ElementId::new(*index as u64, 0),
                index: *index,
                element: element.clone(),
            },
            Self::RemoveElement { index, element, .. } => Self::AddElement {
                element: element.clone(),
                index: *index,
            },
            Self::ModifyStyle { id, index, old, new } => Self::ModifyStyle {
                id: *id,
                index: *index,
                old: new.clone(),
                new: old.clone(),
            },
            Self::MoveElement { id, index, dx, dy } => Self::MoveElement {
                id: *id,
                index: *index,
                dx: -*dx,
                dy: -*dy,
            },
            Self::ReplaceElement { id, index, old, new } => Self::ReplaceElement {
                id: *id,
                index: *index,
                old: new.clone(),
                new: old.clone(),
            },
            Self::Transform { id, index, old, new } => Self::Transform {
                id: *id,
                index: *index,
                old: new.clone(),
                new: old.clone(),
            },
            Self::AddGradient { gradient } => Self::RemoveGradient {
                id: gradient.id.clone(),
                gradient: gradient.clone(),
            },
            Self::RemoveGradient { gradient, .. } => Self::AddGradient {
                gradient: gradient.clone(),
            },
            Self::AddFilter { filter } => Self::RemoveFilter {
                id: filter.id.clone(),
                filter: filter.clone(),
            },
            Self::RemoveFilter { filter, .. } => Self::AddFilter {
                filter: filter.clone(),
            },
            Self::AddSymbol { symbol } => Self::RemoveSymbol {
                id: symbol.id.clone(),
                symbol: symbol.clone(),
            },
            Self::RemoveSymbol { symbol, .. } => Self::AddSymbol {
                symbol: symbol.clone(),
            },
            Self::SetBackground { old, new } => Self::SetBackground {
                old: new.clone(),
                new: old.clone(),
            },
            Self::Batch(cmds) => Self::Batch(cmds.iter().rev().map(|c| c.invert()).collect()),
        }
    }
}

/// Undo/redo history manager
#[derive(Debug, Default)]
pub struct CommandHistory {
    undos: Vec<SceneCommand>,
    redos: Vec<SceneCommand>,
    max_size: usize,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self { undos: Vec::with_capacity(max_size), redos: Vec::new(), max_size }
    }

    /// Execute command and push to history
    pub fn execute(&mut self, cmd: SceneCommand, scene: &mut Scene) {
        cmd.apply(scene);
        self.undos.push(cmd);
        self.redos.clear(); // Clear redo stack on new action
        if self.undos.len() > self.max_size {
            self.undos.remove(0);
        }
    }

    /// Undo last command
    pub fn undo(&mut self, scene: &mut Scene) -> bool {
        if let Some(cmd) = self.undos.pop() {
            cmd.unapply(scene);
            self.redos.push(cmd);
            true
        } else { false }
    }

    /// Redo last undone command
    pub fn redo(&mut self, scene: &mut Scene) -> bool {
        if let Some(cmd) = self.redos.pop() {
            cmd.apply(scene);
            self.undos.push(cmd);
            true
        } else { false }
    }

    #[inline] pub fn can_undo(&self) -> bool { !self.undos.is_empty() }
    #[inline] pub fn can_redo(&self) -> bool { !self.redos.is_empty() }
    #[inline] pub fn undo_count(&self) -> usize { self.undos.len() }
    #[inline] pub fn redo_count(&self) -> usize { self.redos.len() }
    
    pub fn clear(&mut self) {
        self.undos.clear();
        self.redos.clear();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Element mutation helpers
// ─────────────────────────────────────────────────────────────────────────────

fn apply_style(el: &mut Element, style: Style) {
    match el {
        Element::Rect(r) => r.style = style,
        Element::Circle(c) => c.style = style,
        Element::Ellipse(e) => e.style = style,
        Element::Line(l) => l.style = style,
        Element::Path(p) => p.style = style,
        Element::Polygon(p) => p.style = style,
        Element::Text(t) => t.style = style,
        Element::Diamond(d) => d.style = style,
        Element::Node(n) => n.style = style,
        Element::Edge(e) => e.style = style,
        Element::Use(u) => u.style = style,
        _ => {}
    }
}

fn translate_element(el: &mut Element, dx: f32, dy: f32) {
    match el {
        Element::Rect(r) => { r.x += dx; r.y += dy; }
        Element::Circle(c) => { c.cx += dx; c.cy += dy; }
        Element::Ellipse(e) => { e.cx += dx; e.cy += dy; }
        Element::Line(l) => { l.x1 += dx; l.y1 += dy; l.x2 += dx; l.y2 += dy; }
        Element::Text(t) => { t.x += dx; t.y += dy; }
        Element::Image(i) => { i.x += dx; i.y += dy; }
        Element::Diamond(d) => { d.cx += dx; d.cy += dy; }
        Element::Node(n) => { n.cx += dx; n.cy += dy; }
        Element::Use(u) => { u.x += dx; u.y += dy; }
        Element::Polygon(p) => {
            for pt in &mut p.points { pt.0 += dx; pt.1 += dy; }
        }
        _ => {}
    }
}

fn set_transform(el: &mut Element, tf: Option<String>) {
    match el {
        Element::Rect(r) => r.transform = tf,
        Element::Circle(c) => c.transform = tf,
        Element::Ellipse(e) => e.transform = tf,
        Element::Line(l) => l.transform = tf,
        Element::Path(p) => p.transform = tf,
        Element::Polygon(p) => p.transform = tf,
        Element::Text(t) => t.transform = tf,
        Element::Image(i) => i.transform = tf,
        Element::Diamond(d) => d.transform = tf,
        Element::Node(n) => n.transform = tf,
        Element::Use(u) => u.transform = tf,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Circle, Rect};
    use crate::CanvasSize;

    fn test_scene() -> Scene { Scene::new(CanvasSize::Medium, "#fff".into()) }

    #[test]
    fn test_add_remove_roundtrip() {
        let mut scene = test_scene();
        let rect = Element::Rect(Rect {
            x: 10.0, y: 10.0, w: 50.0, h: 50.0, rx: 0.0,
            style: Style::default(), transform: None,
        });
        let cmd = SceneCommand::AddElement { element: rect.clone(), index: 0 };
        
        cmd.apply(&mut scene);
        assert_eq!(scene.elements().len(), 1);
        
        cmd.unapply(&mut scene);
        assert_eq!(scene.elements().len(), 0);
    }

    #[test]
    fn test_modify_style() {
        let mut scene = test_scene();
        let rect = Element::Rect(Rect {
            x: 10.0, y: 10.0, w: 50.0, h: 50.0, rx: 0.0,
            style: Style::with_fill("#red"), transform: None,
        });
        scene.push(rect);
        
        let new_style = Style::with_fill("#blue");
        let cmd = SceneCommand::ModifyStyle {
            id: ElementId::new(0, 0),
            index: 0,
            old: Style::with_fill("#red"),
            new: new_style.clone(),
        };
        
        cmd.apply(&mut scene);
        if let Element::Rect(r) = &scene.elements()[0] {
            assert_eq!(r.style.fill, Some("#blue".into()));
        }
        
        cmd.unapply(&mut scene);
        if let Element::Rect(r) = &scene.elements()[0] {
            assert_eq!(r.style.fill, Some("#red".into()));
        }
    }

    #[test]
    fn test_move_element() {
        let mut scene = test_scene();
        scene.push(Element::Circle(Circle {
            cx: 50.0, cy: 50.0, r: 25.0,
            style: Style::default(), transform: None,
        }));
        
        let cmd = SceneCommand::MoveElement {
            id: ElementId::new(0, 1),
            index: 0,
            dx: 10.0,
            dy: 20.0,
        };
        
        cmd.apply(&mut scene);
        if let Element::Circle(c) = &scene.elements()[0] {
            assert_eq!((c.cx, c.cy), (60.0, 70.0));
        }
        
        cmd.unapply(&mut scene);
        if let Element::Circle(c) = &scene.elements()[0] {
            assert_eq!((c.cx, c.cy), (50.0, 50.0));
        }
    }

    #[test]
    fn test_history_undo_redo() {
        let mut scene = test_scene();
        let mut history = CommandHistory::new(100);
        
        let rect = Element::Rect(Rect {
            x: 0.0, y: 0.0, w: 100.0, h: 100.0, rx: 0.0,
            style: Style::default(), transform: None,
        });
        
        history.execute(SceneCommand::AddElement { element: rect, index: 0 }, &mut scene);
        assert_eq!(scene.elements().len(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
        
        history.undo(&mut scene);
        assert_eq!(scene.elements().len(), 0);
        assert!(!history.can_undo());
        assert!(history.can_redo());
        
        history.redo(&mut scene);
        assert_eq!(scene.elements().len(), 1);
    }

    #[test]
    fn test_batch_command() {
        let mut scene = test_scene();
        let rect = Element::Rect(Rect {
            x: 0.0, y: 0.0, w: 50.0, h: 50.0, rx: 0.0,
            style: Style::default(), transform: None,
        });
        let circle = Element::Circle(Circle {
            cx: 100.0, cy: 100.0, r: 25.0,
            style: Style::default(), transform: None,
        });
        
        let batch = SceneCommand::Batch(vec![
            SceneCommand::AddElement { element: rect, index: 0 },
            SceneCommand::AddElement { element: circle, index: 1 },
        ]);
        
        batch.apply(&mut scene);
        assert_eq!(scene.elements().len(), 2);
        
        batch.unapply(&mut scene);
        assert_eq!(scene.elements().len(), 0);
    }

    #[test]
    fn test_invert() {
        let cmd = SceneCommand::SetBackground { old: "#fff".into(), new: "#000".into() };
        let inv = cmd.invert();
        if let SceneCommand::SetBackground { old, new } = inv {
            assert_eq!(old, "#000");
            assert_eq!(new, "#fff");
        } else { panic!("Expected SetBackground"); }
    }
}

