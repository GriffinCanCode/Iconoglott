//! Incremental scene diffing with stable element IDs
//!
//! Uses content-addressed hashing + ID-based reconciliation for O(n) diffing
//! with minimal SVG regeneration. Inspired by VDOM reconciliation algorithms.

use std::collections::HashMap;
use crate::id::{ContentHash, ElementId, ElementKind, Fnv1a, IdGen};
use crate::scene::{Element, Scene};
use crate::shape::Style;

/// Indexed element with stable identity and content hash
#[derive(Debug, Clone)]
pub struct IndexedElement {
    pub id: ElementId,
    pub hash: ContentHash,
    pub kind: ElementKind,
    pub index: usize,
}

impl IndexedElement {
    pub fn new(el: &Element, order: u64, index: usize) -> Self {
        let kind = element_kind(el);
        let id = compute_id(el, order, kind);
        let hash = ContentHash::from_svg(&el.to_svg());
        Self { id, hash, kind, index }
    }
}

/// Compute stable ID from element's key properties
fn compute_id(el: &Element, order: u64, kind: ElementKind) -> ElementId {
    let mut h = Fnv1a::default();
    
    // Key properties: position and identity-defining attributes
    match el {
        Element::Rect(r) => { h.write_f32(r.x); h.write_f32(r.y); }
        Element::Circle(c) => { h.write_f32(c.cx); h.write_f32(c.cy); }
        Element::Ellipse(e) => { h.write_f32(e.cx); h.write_f32(e.cy); }
        Element::Line(l) => { h.write_f32(l.x1); h.write_f32(l.y1); h.write_f32(l.x2); h.write_f32(l.y2); }
        Element::Path(p) => h.write_str(&p.d),
        Element::Polygon(p) => for (x, y) in &p.points { h.write_f32(*x); h.write_f32(*y); },
        Element::Text(t) => { h.write_f32(t.x); h.write_f32(t.y); h.write_str(&t.content); }
        Element::Image(i) => { h.write_str(&i.href); }
        Element::Group(_, tf) => if let Some(t) = tf { h.write_str(t); },
    }
    
    ElementId::with_key(order, kind.as_u8(), &h.finish().to_le_bytes())
}

/// Get element kind discriminant
#[inline]
pub fn element_kind(el: &Element) -> ElementKind {
    match el {
        Element::Rect(_) => ElementKind::Rect,
        Element::Circle(_) => ElementKind::Circle,
        Element::Ellipse(_) => ElementKind::Ellipse,
        Element::Line(_) => ElementKind::Line,
        Element::Path(_) => ElementKind::Path,
        Element::Polygon(_) => ElementKind::Polygon,
        Element::Text(_) => ElementKind::Text,
        Element::Image(_) => ElementKind::Image,
        Element::Group(_, _) => ElementKind::Group,
    }
}

/// Targeted diff operation for incremental updates
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOp {
    /// No changes
    None,
    /// Full scene redraw (canvas changed)
    FullRedraw,
    /// Add element at index with SVG
    Add { id: u64, idx: usize, svg: String },
    /// Remove element by ID
    Remove { id: u64, idx: usize },
    /// Update element attributes
    Update { id: u64, idx: usize, attrs: Vec<(String, String)>, svg: Option<String> },
    /// Move element from old index to new index
    Move { id: u64, from: usize, to: usize },
    /// Update defs (gradients/filters)
    UpdateDefs { svg: String },
}

/// Indexed scene for O(1) element lookup
#[derive(Debug, Default)]
pub struct IndexedScene {
    pub elements: Vec<IndexedElement>,
    id_map: HashMap<ElementId, usize>,
}

impl IndexedScene {
    pub fn from_scene(scene: &Scene) -> Self {
        let gen = IdGen::default();
        let elements: Vec<_> = scene.elements()
            .iter()
            .enumerate()
            .map(|(idx, el)| IndexedElement::new(el, gen.next(), idx))
            .collect();
        
        let id_map = elements.iter()
            .map(|e| (e.id, e.index))
            .collect();
        
        Self { elements, id_map }
    }

    #[inline]
    pub fn get(&self, id: &ElementId) -> Option<&IndexedElement> {
        self.id_map.get(id).map(|&idx| &self.elements[idx])
    }

    #[inline]
    pub fn len(&self) -> usize { self.elements.len() }
}

/// Diff result with operations
#[derive(Debug, Default)]
pub struct DiffResult {
    pub ops: Vec<DiffOp>,
    pub canvas_changed: bool,
}

impl DiffResult {
    pub fn full_redraw() -> Self {
        Self { ops: vec![DiffOp::FullRedraw], canvas_changed: true }
    }

    pub fn empty() -> Self { Self::default() }

    #[inline]
    pub fn is_empty(&self) -> bool { self.ops.is_empty() }

    #[inline]
    pub fn needs_full_redraw(&self) -> bool {
        self.canvas_changed || self.ops.iter().any(|o| matches!(o, DiffOp::FullRedraw))
    }
}

/// Diff two scenes using indexed reconciliation
pub fn diff(old: &Scene, new: &Scene) -> DiffResult {
    // Canvas change = full redraw
    if old.width != new.width || old.height != new.height || old.background != new.background {
        return DiffResult::full_redraw();
    }

    let old_els = old.elements();
    let new_els = new.elements();

    // Fast path: empty scenes
    if old_els.is_empty() && new_els.is_empty() {
        return DiffResult::empty();
    }

    // Index old scene for O(1) lookup
    let old_indexed = IndexedScene::from_scene(old);
    let gen = IdGen::default();
    
    let mut ops = Vec::new();
    let mut matched: Vec<bool> = vec![false; old_els.len()];

    // Pass 1: Match new elements to old
    for (new_idx, new_el) in new_els.iter().enumerate() {
        let new_kind = element_kind(new_el);
        let new_id = compute_id(new_el, gen.next(), new_kind);
        let new_hash = ContentHash::from_svg(&new_el.to_svg());

        if let Some(old_ie) = old_indexed.get(&new_id) {
            matched[old_ie.index] = true;
            
            // Same position, check content
            if old_ie.hash != new_hash {
                let attrs = diff_attrs(&old_els[old_ie.index], new_el);
                let svg = if attrs.len() > 3 { Some(new_el.to_svg()) } else { None };
                ops.push(DiffOp::Update { id: new_id.0, idx: new_idx, attrs, svg });
            }
            
            // Position change
            if old_ie.index != new_idx {
                ops.push(DiffOp::Move { id: new_id.0, from: old_ie.index, to: new_idx });
            }
        } else {
            // New element
            ops.push(DiffOp::Add { id: new_id.0, idx: new_idx, svg: new_el.to_svg() });
        }
    }

    // Pass 2: Remove unmatched old elements (reverse for stable indices)
    for (old_idx, &was_matched) in matched.iter().enumerate().rev() {
        if !was_matched {
            let old_el = &old_els[old_idx];
            let old_kind = element_kind(old_el);
            let old_id = compute_id(old_el, old_idx as u64, old_kind);
            ops.push(DiffOp::Remove { id: old_id.0, idx: old_idx });
        }
    }

    // Check defs changes
    let old_defs = build_defs_svg(old);
    let new_defs = build_defs_svg(new);
    if old_defs != new_defs {
        ops.push(DiffOp::UpdateDefs { svg: new_defs });
    }

    DiffResult { ops, canvas_changed: false }
}

fn build_defs_svg(scene: &Scene) -> String {
    let mut svg = String::new();
    for g in scene.gradients() { svg.push_str(&g.to_svg()); }
    for f in scene.filters() { svg.push_str(&f.to_svg()); }
    svg
}

/// Diff attributes between same-kind elements
fn diff_attrs(old: &Element, new: &Element) -> Vec<(String, String)> {
    let mut changes = Vec::new();

    match (old, new) {
        (Element::Rect(o), Element::Rect(n)) => {
            if o.x != n.x { changes.push(("x".into(), n.x.to_string())); }
            if o.y != n.y { changes.push(("y".into(), n.y.to_string())); }
            if o.w != n.w { changes.push(("width".into(), n.w.to_string())); }
            if o.h != n.h { changes.push(("height".into(), n.h.to_string())); }
            if o.rx != n.rx { changes.push(("rx".into(), n.rx.to_string())); }
            diff_style(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Circle(o), Element::Circle(n)) => {
            if o.cx != n.cx { changes.push(("cx".into(), n.cx.to_string())); }
            if o.cy != n.cy { changes.push(("cy".into(), n.cy.to_string())); }
            if o.r != n.r { changes.push(("r".into(), n.r.to_string())); }
            diff_style(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Ellipse(o), Element::Ellipse(n)) => {
            if o.cx != n.cx { changes.push(("cx".into(), n.cx.to_string())); }
            if o.cy != n.cy { changes.push(("cy".into(), n.cy.to_string())); }
            if o.rx != n.rx { changes.push(("rx".into(), n.rx.to_string())); }
            if o.ry != n.ry { changes.push(("ry".into(), n.ry.to_string())); }
            diff_style(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Line(o), Element::Line(n)) => {
            if o.x1 != n.x1 { changes.push(("x1".into(), n.x1.to_string())); }
            if o.y1 != n.y1 { changes.push(("y1".into(), n.y1.to_string())); }
            if o.x2 != n.x2 { changes.push(("x2".into(), n.x2.to_string())); }
            if o.y2 != n.y2 { changes.push(("y2".into(), n.y2.to_string())); }
            diff_style(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Path(o), Element::Path(n)) => {
            if o.d != n.d { changes.push(("d".into(), n.d.clone())); }
            diff_style(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Text(o), Element::Text(n)) => {
            if o.x != n.x { changes.push(("x".into(), n.x.to_string())); }
            if o.y != n.y { changes.push(("y".into(), n.y.to_string())); }
            if o.content != n.content { changes.push(("textContent".into(), n.content.clone())); }
            if o.font != n.font { changes.push(("font-family".into(), n.font.clone())); }
            if o.size != n.size { changes.push(("font-size".into(), n.size.to_string())); }
            if o.weight != n.weight { changes.push(("font-weight".into(), n.weight.clone())); }
            if o.anchor != n.anchor { changes.push(("text-anchor".into(), n.anchor.clone())); }
            diff_style(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Image(o), Element::Image(n)) => {
            if o.x != n.x { changes.push(("x".into(), n.x.to_string())); }
            if o.y != n.y { changes.push(("y".into(), n.y.to_string())); }
            if o.w != n.w { changes.push(("width".into(), n.w.to_string())); }
            if o.h != n.h { changes.push(("height".into(), n.h.to_string())); }
            if o.href != n.href { changes.push(("href".into(), n.href.clone())); }
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        (Element::Polygon(o), Element::Polygon(n)) => {
            if o.points != n.points {
                let pts: String = n.points.iter()
                    .map(|(x, y)| format!("{},{}", x, y))
                    .collect::<Vec<_>>()
                    .join(" ");
                changes.push(("points".into(), pts));
            }
            diff_style(&o.style, &n.style, &mut changes);
            diff_transform(&o.transform, &n.transform, &mut changes);
        }
        _ => {}
    }

    changes
}

fn diff_style(old: &Style, new: &Style, out: &mut Vec<(String, String)>) {
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

#[inline]
fn diff_transform(old: &Option<String>, new: &Option<String>, out: &mut Vec<(String, String)>) {
    if old != new {
        out.push(("transform".into(), new.clone().unwrap_or_default()));
    }
}

// Backwards compatibility aliases
pub type Patch = DiffOp;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{Circle, Ellipse, Image, Line, Path, Polygon, Rect, Style, Text};

    fn make_scene(w: u32, h: u32, bg: &str) -> Scene {
        Scene::new_internal(w, h, bg.to_string())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // DiffResult tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_diff_result_empty() {
        let r = DiffResult::empty();
        assert!(r.is_empty());
        assert!(!r.needs_full_redraw());
    }

    #[test]
    fn test_diff_result_full_redraw() {
        let r = DiffResult::full_redraw();
        assert!(r.needs_full_redraw());
        assert!(r.canvas_changed);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Scene diff tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_identical_scenes() {
        let s1 = make_scene(800, 600, "#fff");
        let s2 = make_scene(800, 600, "#fff");
        assert!(diff(&s1, &s2).is_empty());
    }

    #[test]
    fn test_canvas_width_change() {
        let s1 = make_scene(800, 600, "#fff");
        let s2 = make_scene(1024, 600, "#fff");
        assert!(diff(&s1, &s2).needs_full_redraw());
    }

    #[test]
    fn test_canvas_height_change() {
        let s1 = make_scene(800, 600, "#fff");
        let s2 = make_scene(800, 768, "#fff");
        assert!(diff(&s1, &s2).needs_full_redraw());
    }

    #[test]
    fn test_canvas_background_change() {
        let s1 = make_scene(800, 600, "#fff");
        let s2 = make_scene(800, 600, "#000");
        assert!(diff(&s1, &s2).needs_full_redraw());
    }

    #[test]
    fn test_empty_scenes_no_diff() {
        let s1 = make_scene(100, 100, "#fff");
        let s2 = make_scene(100, 100, "#fff");
        let result = diff(&s1, &s2);
        assert!(result.is_empty());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // IndexedScene tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_indexed_scene_empty() {
        let scene = make_scene(800, 600, "#fff");
        let indexed = IndexedScene::from_scene(&scene);
        assert_eq!(indexed.len(), 0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ElementKind tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_element_kind_rect() {
        let el = Element::Rect(make_rect(0.0, 0.0, 100.0, 50.0, 0.0, None, None));
        assert_eq!(element_kind(&el), ElementKind::Rect);
    }

    #[test]
    fn test_element_kind_circle() {
        let el = Element::Circle(make_circle(50.0, 50.0, 25.0, None, None));
        assert_eq!(element_kind(&el), ElementKind::Circle);
    }

    #[test]
    fn test_element_kind_ellipse() {
        let el = Element::Ellipse(Ellipse { cx: 50.0, cy: 50.0, rx: 30.0, ry: 20.0, style: Style::default(), transform: None });
        assert_eq!(element_kind(&el), ElementKind::Ellipse);
    }

    #[test]
    fn test_element_kind_line() {
        let mut s = Style::default();
        s.stroke = Some("#000".into());
        let el = Element::Line(Line { x1: 0.0, y1: 0.0, x2: 100.0, y2: 100.0, style: s, transform: None });
        assert_eq!(element_kind(&el), ElementKind::Line);
    }

    #[test]
    fn test_element_kind_path() {
        let el = Element::Path(Path { d: "M 0 0".into(), style: Style::default(), transform: None, bounds_hint: None });
        assert_eq!(element_kind(&el), ElementKind::Path);
    }

    #[test]
    fn test_element_kind_polygon() {
        let el = Element::Polygon(Polygon { points: vec![(0.0, 0.0), (100.0, 0.0), (50.0, 100.0)], style: Style::default(), transform: None });
        assert_eq!(element_kind(&el), ElementKind::Polygon);
    }

    #[test]
    fn test_element_kind_text() {
        let el = Element::Text(make_text(10.0, 20.0, "Test", "Arial", 16.0, "normal", "start"));
        assert_eq!(element_kind(&el), ElementKind::Text);
    }

    #[test]
    fn test_element_kind_image() {
        let el = Element::Image(Image { x: 0.0, y: 0.0, w: 100.0, h: 100.0, href: "img.png".into(), transform: None });
        assert_eq!(element_kind(&el), ElementKind::Image);
    }

    #[test]
    fn test_element_kind_group() {
        let el = Element::Group(vec![], None);
        assert_eq!(element_kind(&el), ElementKind::Group);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // DiffOp tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_diff_op_none_eq() {
        assert_eq!(DiffOp::None, DiffOp::None);
    }

    #[test]
    fn test_diff_op_full_redraw_eq() {
        assert_eq!(DiffOp::FullRedraw, DiffOp::FullRedraw);
    }

    #[test]
    fn test_diff_op_add() {
        let op = DiffOp::Add { id: 1, idx: 0, svg: "<rect/>".into() };
        assert!(matches!(op, DiffOp::Add { id: 1, idx: 0, .. }));
    }

    #[test]
    fn test_diff_op_remove() {
        let op = DiffOp::Remove { id: 2, idx: 5 };
        assert!(matches!(op, DiffOp::Remove { id: 2, idx: 5 }));
    }

    #[test]
    fn test_diff_op_update() {
        let op = DiffOp::Update { id: 3, idx: 1, attrs: vec![("x".into(), "10".into())], svg: None };
        assert!(matches!(op, DiffOp::Update { id: 3, .. }));
    }

    #[test]
    fn test_diff_op_move() {
        let op = DiffOp::Move { id: 4, from: 0, to: 5 };
        assert!(matches!(op, DiffOp::Move { from: 0, to: 5, .. }));
    }

    #[test]
    fn test_diff_op_update_defs() {
        let op = DiffOp::UpdateDefs { svg: "<defs/>".into() };
        assert!(matches!(op, DiffOp::UpdateDefs { .. }));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // diff_attrs tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_diff_attrs_rect_position() {
        let s = Style::default();
        let r1 = Element::Rect(Rect { x: 0.0, y: 0.0, w: 100.0, h: 50.0, rx: 0.0, style: s.clone(), transform: None });
        let r2 = Element::Rect(Rect { x: 10.0, y: 0.0, w: 100.0, h: 50.0, rx: 0.0, style: s, transform: None });
        let changes = diff_attrs(&r1, &r2);
        assert!(changes.iter().any(|(k, v)| k == "x" && v == "10"));
    }

    #[test]
    fn test_diff_attrs_rect_size() {
        let s = Style::default();
        let r1 = Element::Rect(Rect { x: 0.0, y: 0.0, w: 100.0, h: 50.0, rx: 0.0, style: s.clone(), transform: None });
        let r2 = Element::Rect(Rect { x: 0.0, y: 0.0, w: 200.0, h: 100.0, rx: 0.0, style: s, transform: None });
        let changes = diff_attrs(&r1, &r2);
        assert!(changes.len() >= 2);
    }

    #[test]
    fn test_diff_attrs_circle() {
        let s = Style::default();
        let c1 = Element::Circle(Circle { cx: 50.0, cy: 50.0, r: 25.0, style: s.clone(), transform: None });
        let c2 = Element::Circle(Circle { cx: 60.0, cy: 50.0, r: 30.0, style: s, transform: None });
        let changes = diff_attrs(&c1, &c2);
        assert!(changes.iter().any(|(k, _)| k == "cx"));
        assert!(changes.iter().any(|(k, _)| k == "r"));
    }

    fn make_text(x: f32, y: f32, content: &str, font: &str, size: f32, weight: &str, anchor: &str) -> Text {
        Text { x, y, content: content.to_string(), font: font.to_string(), size, weight: weight.to_string(), anchor: anchor.to_string(), style: Style::default(), transform: None }
    }

    fn make_style(fill: Option<&str>, stroke: Option<&str>, sw: f32, op: f32, corner: f32, filter: Option<&str>) -> Style {
        Style { fill: fill.map(String::from), stroke: stroke.map(String::from), stroke_width: sw, opacity: op, corner, filter: filter.map(String::from) }
    }

    fn make_rect(x: f32, y: f32, w: f32, h: f32, rx: f32, style: Option<Style>, transform: Option<String>) -> Rect {
        Rect { x, y, w, h, rx, style: style.unwrap_or_default(), transform }
    }

    fn make_circle(cx: f32, cy: f32, r: f32, style: Option<Style>, transform: Option<String>) -> Circle {
        Circle { cx, cy, r, style: style.unwrap_or_default(), transform }
    }

    #[test]
    fn test_diff_attrs_text_content() {
        let t1 = Element::Text(make_text(0.0, 0.0, "Hello", "Arial", 16.0, "normal", "start"));
        let t2 = Element::Text(make_text(0.0, 0.0, "World", "Arial", 16.0, "normal", "start"));
        let changes = diff_attrs(&t1, &t2);
        assert!(changes.iter().any(|(k, v)| k == "textContent" && v == "World"));
    }

    #[test]
    fn test_diff_attrs_style_fill() {
        let s1 = make_style(Some("#f00"), None, 1.0, 1.0, 0.0, None);
        let s2 = make_style(Some("#00f"), None, 1.0, 1.0, 0.0, None);
        let r1 = Element::Rect(Rect { x: 0.0, y: 0.0, w: 100.0, h: 50.0, rx: 0.0, style: s1, transform: None });
        let r2 = Element::Rect(Rect { x: 0.0, y: 0.0, w: 100.0, h: 50.0, rx: 0.0, style: s2, transform: None });
        let changes = diff_attrs(&r1, &r2);
        assert!(changes.iter().any(|(k, _)| k == "fill"));
    }

    #[test]
    fn test_diff_attrs_transform() {
        let s = Style::default();
        let r1 = Element::Rect(Rect { x: 0.0, y: 0.0, w: 50.0, h: 50.0, rx: 0.0, style: s.clone(), transform: None });
        let r2 = Element::Rect(Rect { x: 0.0, y: 0.0, w: 50.0, h: 50.0, rx: 0.0, style: s, transform: Some("rotate(45)".into()) });
        let changes = diff_attrs(&r1, &r2);
        assert!(changes.iter().any(|(k, _)| k == "transform"));
    }

    #[test]
    fn test_diff_attrs_mismatched_types() {
        let r = Element::Rect(make_rect(0.0, 0.0, 100.0, 50.0, 0.0, None, None));
        let c = Element::Circle(make_circle(50.0, 50.0, 25.0, None, None));
        let changes = diff_attrs(&r, &c);
        assert!(changes.is_empty()); // Mismatched types return empty
    }

    // ─────────────────────────────────────────────────────────────────────────
    // diff_style tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_diff_style_fill() {
        let s1 = make_style(Some("#f00"), None, 1.0, 1.0, 0.0, None);
        let s2 = make_style(Some("#00f"), None, 1.0, 1.0, 0.0, None);
        let mut out = Vec::new();
        diff_style(&s1, &s2, &mut out);
        assert!(out.iter().any(|(k, v)| k == "fill" && v == "#00f"));
    }

    #[test]
    fn test_diff_style_stroke() {
        let s1 = make_style(None, Some("#000"), 1.0, 1.0, 0.0, None);
        let s2 = make_style(None, Some("#fff"), 2.0, 1.0, 0.0, None);
        let mut out = Vec::new();
        diff_style(&s1, &s2, &mut out);
        assert!(out.iter().any(|(k, _)| k == "stroke"));
        assert!(out.iter().any(|(k, _)| k == "stroke-width"));
    }

    #[test]
    fn test_diff_style_opacity() {
        let s1 = make_style(None, None, 1.0, 1.0, 0.0, None);
        let s2 = make_style(None, None, 1.0, 0.5, 0.0, None);
        let mut out = Vec::new();
        diff_style(&s1, &s2, &mut out);
        assert!(out.iter().any(|(k, _)| k == "opacity"));
    }

    #[test]
    fn test_diff_style_filter() {
        let s1 = make_style(None, None, 1.0, 1.0, 0.0, None);
        let s2 = make_style(None, None, 1.0, 1.0, 0.0, Some("blur1"));
        let mut out = Vec::new();
        diff_style(&s1, &s2, &mut out);
        assert!(out.iter().any(|(k, v)| k == "filter" && v.contains("blur1")));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // IndexedElement tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_indexed_element_new() {
        let r = Element::Rect(make_rect(10.0, 20.0, 100.0, 50.0, 0.0, None, None));
        let ie = IndexedElement::new(&r, 0, 0);
        assert_eq!(ie.kind, ElementKind::Rect);
        assert_eq!(ie.index, 0);
    }

    #[test]
    fn test_indexed_element_same_content_same_hash() {
        let r1 = Element::Rect(make_rect(10.0, 20.0, 100.0, 50.0, 0.0, None, None));
        let r2 = Element::Rect(make_rect(10.0, 20.0, 100.0, 50.0, 0.0, None, None));
        let ie1 = IndexedElement::new(&r1, 0, 0);
        let ie2 = IndexedElement::new(&r2, 0, 0);
        assert_eq!(ie1.hash, ie2.hash);
    }

    #[test]
    fn test_indexed_element_different_content_different_hash() {
        let r1 = Element::Rect(make_rect(10.0, 20.0, 100.0, 50.0, 0.0, None, None));
        let r2 = Element::Rect(make_rect(20.0, 20.0, 100.0, 50.0, 0.0, None, None));
        let ie1 = IndexedElement::new(&r1, 0, 0);
        let ie2 = IndexedElement::new(&r2, 0, 0);
        assert_ne!(ie1.hash, ie2.hash);
    }
}
