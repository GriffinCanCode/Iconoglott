//! Scene diffing engine with content-addressed hashing
//!
//! Uses type-stratified matching and per-field change detection
//! to produce minimal DOM patch operations.

use std::collections::HashMap;
use crate::scene::{Element, Scene};
use crate::shape::Style;

/// Hash of element content for O(1) equality checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(u64);

impl ContentHash {
    fn new(data: &[u8]) -> Self {
        // FNV-1a hash - fast and good distribution
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in data {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Self(hash)
    }

    pub fn from_element(el: &Element) -> Self {
        Self::new(el.to_svg().as_bytes())
    }
}

/// Granular patch operation for minimal updates
#[derive(Debug, Clone, PartialEq)]
pub enum Patch {
    /// No changes needed
    None,
    /// Full scene redraw (canvas changed)
    FullRedraw,
    /// Element operations
    Add { idx: usize, svg: String },
    Remove { idx: usize },
    /// Update specific attributes on element
    Update { idx: usize, attrs: Vec<(String, String)> },
    /// Element moved position
    Move { from: usize, to: usize },
    /// Batch reorder for efficiency
    Reorder { order: Vec<usize> },
    /// Gradient/filter def changes
    UpdateDefs { svg: String },
}

/// Result of scene comparison
#[derive(Debug, Default)]
pub struct DiffResult {
    pub patches: Vec<Patch>,
    pub canvas_changed: bool,
}

impl DiffResult {
    pub fn full_redraw() -> Self {
        Self { patches: vec![Patch::FullRedraw], canvas_changed: true }
    }

    pub fn empty() -> Self {
        Self { patches: vec![], canvas_changed: false }
    }

    #[inline]
    pub fn is_empty(&self) -> bool { self.patches.is_empty() }

    #[inline]
    pub fn needs_full_redraw(&self) -> bool {
        self.canvas_changed || self.patches.iter().any(|p| matches!(p, Patch::FullRedraw))
    }
}

/// Diff two scenes and produce minimal patches
pub fn diff(old: &Scene, new: &Scene) -> DiffResult {
    // Canvas change = full redraw
    if old.width != new.width || old.height != new.height || old.background != new.background {
        return DiffResult::full_redraw();
    }

    let old_els = old.elements();
    let new_els = new.elements();

    // Fast path: identical element counts with same hashes
    if old_els.len() == new_els.len() {
        let patches = diff_matched_elements(old_els, new_els);
        if patches.is_empty() {
            return DiffResult::empty();
        }
        return DiffResult { patches, canvas_changed: false };
    }

    // General case: element count differs
    diff_general(old_els, new_els)
}

/// Diff when element counts match (most common case during editing)
fn diff_matched_elements(old: &[Element], new: &[Element]) -> Vec<Patch> {
    old.iter()
        .zip(new.iter())
        .enumerate()
        .filter_map(|(idx, (o, n))| diff_element(idx, o, n))
        .collect()
}

/// Compare two elements at same position
fn diff_element(idx: usize, old: &Element, new: &Element) -> Option<Patch> {
    // Type mismatch = replace
    if element_type(old) != element_type(new) {
        return Some(Patch::Remove { idx });
    }

    // Content hash check for fast equality
    if ContentHash::from_element(old) == ContentHash::from_element(new) {
        return None;
    }

    // Compute attribute-level diff
    let attrs = diff_attrs(old, new);
    if attrs.is_empty() {
        None
    } else {
        Some(Patch::Update { idx, attrs })
    }
}

/// Get discriminant type name for element
fn element_type(el: &Element) -> &'static str {
    match el {
        Element::Rect(_) => "rect",
        Element::Circle(_) => "circle",
        Element::Ellipse(_) => "ellipse",
        Element::Line(_) => "line",
        Element::Path(_) => "path",
        Element::Polygon(_) => "polygon",
        Element::Text(_) => "text",
        Element::Image(_) => "image",
        Element::Group(_, _) => "group",
    }
}

/// Compute changed attributes between same-type elements
fn diff_attrs(old: &Element, new: &Element) -> Vec<(String, String)> {
    let mut changes = Vec::new();

    match (old, new) {
        (Element::Rect(o), Element::Rect(n)) => {
            if o.x != n.x { changes.push(("x".into(), n.x.to_string())); }
            if o.y != n.y { changes.push(("y".into(), n.y.to_string())); }
            if o.w != n.w { changes.push(("width".into(), n.w.to_string())); }
            if o.h != n.h { changes.push(("height".into(), n.h.to_string())); }
            if o.rx != n.rx { changes.push(("rx".into(), n.rx.to_string())); }
            diff_style_attrs(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Circle(o), Element::Circle(n)) => {
            if o.cx != n.cx { changes.push(("cx".into(), n.cx.to_string())); }
            if o.cy != n.cy { changes.push(("cy".into(), n.cy.to_string())); }
            if o.r != n.r { changes.push(("r".into(), n.r.to_string())); }
            diff_style_attrs(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Ellipse(o), Element::Ellipse(n)) => {
            if o.cx != n.cx { changes.push(("cx".into(), n.cx.to_string())); }
            if o.cy != n.cy { changes.push(("cy".into(), n.cy.to_string())); }
            if o.rx != n.rx { changes.push(("rx".into(), n.rx.to_string())); }
            if o.ry != n.ry { changes.push(("ry".into(), n.ry.to_string())); }
            diff_style_attrs(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Line(o), Element::Line(n)) => {
            if o.x1 != n.x1 { changes.push(("x1".into(), n.x1.to_string())); }
            if o.y1 != n.y1 { changes.push(("y1".into(), n.y1.to_string())); }
            if o.x2 != n.x2 { changes.push(("x2".into(), n.x2.to_string())); }
            if o.y2 != n.y2 { changes.push(("y2".into(), n.y2.to_string())); }
            diff_style_attrs(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Path(o), Element::Path(n)) => {
            if o.d != n.d { changes.push(("d".into(), n.d.clone())); }
            diff_style_attrs(&o.style, &n.style, &mut changes);
        }
        (Element::Text(o), Element::Text(n)) => {
            if o.x != n.x { changes.push(("x".into(), n.x.to_string())); }
            if o.y != n.y { changes.push(("y".into(), n.y.to_string())); }
            if o.content != n.content { changes.push(("textContent".into(), n.content.clone())); }
            if o.font != n.font { changes.push(("font-family".into(), n.font.clone())); }
            if o.size != n.size { changes.push(("font-size".into(), n.size.to_string())); }
            if o.weight != n.weight { changes.push(("font-weight".into(), n.weight.clone())); }
            if o.anchor != n.anchor { changes.push(("text-anchor".into(), n.anchor.clone())); }
            diff_style_attrs(&o.style, &n.style, &mut changes);
        }
        (Element::Image(o), Element::Image(n)) => {
            if o.x != n.x { changes.push(("x".into(), n.x.to_string())); }
            if o.y != n.y { changes.push(("y".into(), n.y.to_string())); }
            if o.w != n.w { changes.push(("width".into(), n.w.to_string())); }
            if o.h != n.h { changes.push(("height".into(), n.h.to_string())); }
            if o.href != n.href { changes.push(("href".into(), n.href.clone())); }
        }
        (Element::Polygon(o), Element::Polygon(n)) => {
            if o.points != n.points {
                let pts: String = n.points.iter()
                    .map(|(x, y)| format!("{},{}", x, y))
                    .collect::<Vec<_>>()
                    .join(" ");
                changes.push(("points".into(), pts));
            }
            diff_style_attrs(&o.style, &n.style, &mut changes);
        }
        _ => {} // Groups handled recursively
    }

    changes
}

/// Diff style properties
fn diff_style_attrs(old: &Style, new: &Style, out: &mut Vec<(String, String)>) {
    if old.fill != new.fill {
        out.push(("fill".into(), new.fill.clone().unwrap_or_default()));
    }
    if old.stroke != new.stroke {
        out.push(("stroke".into(), new.stroke.clone().unwrap_or_default()));
    }
    if old.stroke_width != new.stroke_width {
        out.push(("stroke-width".into(), new.stroke_width.to_string()));
    }
    if old.opacity != new.opacity {
        out.push(("opacity".into(), new.opacity.to_string()));
    }
    if old.filter != new.filter {
        let val = new.filter.as_ref().map(|f| format!("url(#{})", f)).unwrap_or_default();
        out.push(("filter".into(), val));
    }
}

/// Diff transform attribute
#[inline]
fn diff_transform(old: &Option<String>, new: &Option<String>, out: &mut Vec<(String, String)>) {
    if old != new {
        out.push(("transform".into(), new.clone().unwrap_or_default()));
    }
}

/// General diff for when element counts differ
fn diff_general(old: &[Element], new: &[Element]) -> DiffResult {
    let old_hashes: Vec<_> = old.iter().map(ContentHash::from_element).collect();
    let new_hashes: Vec<_> = new.iter().map(ContentHash::from_element).collect();

    // Build hash -> indices map for old elements
    let mut old_map: HashMap<ContentHash, Vec<usize>> = HashMap::new();
    for (i, h) in old_hashes.iter().enumerate() {
        old_map.entry(*h).or_default().push(i);
    }

    let mut patches = Vec::new();
    let mut matched_old: Vec<bool> = vec![false; old.len()];

    // Pass 1: Match new elements to old by hash
    for (new_idx, new_hash) in new_hashes.iter().enumerate() {
        if let Some(old_indices) = old_map.get_mut(new_hash) {
            if let Some(old_idx) = old_indices.pop() {
                matched_old[old_idx] = true;
                // Check if position changed
                if old_idx != new_idx {
                    patches.push(Patch::Move { from: old_idx, to: new_idx });
                }
            } else {
                // Hash existed but all instances used - add new
                patches.push(Patch::Add { idx: new_idx, svg: new[new_idx].to_svg() });
            }
        } else {
            // No match - new element
            patches.push(Patch::Add { idx: new_idx, svg: new[new_idx].to_svg() });
        }
    }

    // Pass 2: Remove unmatched old elements (reverse order for stable indices)
    for (old_idx, matched) in matched_old.iter().enumerate().rev() {
        if !matched {
            patches.push(Patch::Remove { idx: old_idx });
        }
    }

    DiffResult { patches, canvas_changed: false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{Rect, Circle, Style};

    fn make_scene(w: u32, h: u32, bg: &str) -> Scene {
        Scene::new_internal(w, h, bg.to_string())
    }

    #[test]
    fn test_identical_scenes() {
        let s1 = make_scene(800, 600, "#fff");
        let s2 = make_scene(800, 600, "#fff");
        let result = diff(&s1, &s2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_canvas_change_triggers_full_redraw() {
        let s1 = make_scene(800, 600, "#fff");
        let s2 = make_scene(800, 600, "#000");
        let result = diff(&s1, &s2);
        assert!(result.needs_full_redraw());
    }

    #[test]
    fn test_content_hash_consistency() {
        let style = Style::default();
        let r1 = Element::Rect(Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0, rx: 0.0, style: style.clone(), transform: None });
        let r2 = Element::Rect(Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0, rx: 0.0, style, transform: None });
        assert_eq!(ContentHash::from_element(&r1), ContentHash::from_element(&r2));
    }

    #[test]
    fn test_property_change_detection() {
        let style = Style::default();
        let old = Element::Circle(Circle { cx: 50.0, cy: 50.0, r: 25.0, style: style.clone(), transform: None });
        let new = Element::Circle(Circle { cx: 60.0, cy: 50.0, r: 25.0, style, transform: None });
        
        let attrs = diff_attrs(&old, &new);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0], ("cx".into(), "60".into()));
    }
}

