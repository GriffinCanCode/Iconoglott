//! Scene graph management

#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use super::shape::{Circle, Diamond, Edge, Ellipse, Image, Line, Node, Path, Polygon, Rect, Text};
use crate::CanvasSize;

/// A renderable element in the scene
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Element {
    Rect(Rect), Circle(Circle), Ellipse(Ellipse), Line(Line),
    Path(Path), Polygon(Polygon), Text(Text), Image(Image),
    Diamond(Diamond), Node(Node), Edge(Edge),
    Group(Vec<Element>, Option<String>),
    Graph(GraphContainer),
}

/// Container for graph elements with layout info
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GraphContainer {
    pub layout: String,
    pub direction: String,
    pub spacing: f32,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Default for GraphContainer {
    fn default() -> Self {
        Self { layout: "manual".into(), direction: "vertical".into(), spacing: 50.0, nodes: Vec::new(), edges: Vec::new() }
    }
}

impl GraphContainer {
    /// Compute edge endpoints based on node positions
    pub fn resolve_edges(&mut self) {
        use std::collections::HashMap;
        let node_map: HashMap<&str, &Node> = self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        
        for edge in &mut self.edges {
            if let (Some(from_node), Some(to_node)) = (node_map.get(edge.from_id.as_str()), node_map.get(edge.to_id.as_str())) {
                // Determine best anchor points based on relative positions
                let (from_side, to_side) = Self::best_anchors(from_node, to_node);
                edge.from_pt = from_node.anchor(from_side);
                edge.to_pt = to_node.anchor(to_side);
            }
        }
    }
    
    fn best_anchors(from: &Node, to: &Node) -> (&'static str, &'static str) {
        let dx = to.cx - from.cx;
        let dy = to.cy - from.cy;
        if dy.abs() > dx.abs() {
            if dy > 0.0 { ("bottom", "top") } else { ("top", "bottom") }
        } else {
            if dx > 0.0 { ("right", "left") } else { ("left", "right") }
        }
    }
    
    /// Apply auto-layout to nodes
    pub fn apply_layout(&mut self) {
        match self.layout.as_str() {
            "hierarchical" => self.layout_hierarchical(),
            "grid" => self.layout_grid(),
            _ => {} // manual - no changes
        }
    }
    
    fn layout_hierarchical(&mut self) {
        if self.nodes.is_empty() { return; }
        let is_vertical = self.direction != "horizontal";
        let spacing = self.spacing;
        
        // Simple hierarchical: place nodes in sequence
        let mut pos = spacing;
        for node in &mut self.nodes {
            if is_vertical {
                node.cy = pos;
                node.cx = spacing * 2.0;
                pos += node.h + spacing;
            } else {
                node.cx = pos;
                node.cy = spacing * 2.0;
                pos += node.w + spacing;
            }
        }
    }
    
    fn layout_grid(&mut self) {
        if self.nodes.is_empty() { return; }
        let cols = (self.nodes.len() as f32).sqrt().ceil() as usize;
        let spacing = self.spacing;
        
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let row = i / cols;
            let col = i % cols;
            node.cx = spacing + (col as f32) * (node.w + spacing) + node.w / 2.0;
            node.cy = spacing + (row as f32) * (node.h + spacing) + node.h / 2.0;
        }
    }
    
    pub fn to_svg(&self, arrow_prefix: &str) -> String {
        let mut svg = String::new();
        
        // Render edges first (behind nodes)
        for edge in &self.edges {
            svg.push_str(&edge.to_svg((&format!("{}-arrow-start", arrow_prefix), &format!("{}-arrow-end", arrow_prefix))));
        }
        
        // Render nodes
        for node in &self.nodes {
            svg.push_str(&node.to_svg());
        }
        
        format!("<g class=\"graph\">{}</g>", svg)
    }
    
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if self.nodes.is_empty() { return (0.0, 0.0, 0.0, 0.0); }
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for n in &self.nodes {
            let (x, y, w, h) = n.bounds();
            min_x = min_x.min(x); min_y = min_y.min(y); max_x = max_x.max(x + w); max_y = max_y.max(y + h);
        }
        (min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

impl Element {
    pub fn to_svg(&self) -> String {
        match self {
            Element::Rect(r) => r.to_svg(), Element::Circle(c) => c.to_svg(),
            Element::Ellipse(e) => e.to_svg(), Element::Line(l) => l.to_svg(),
            Element::Path(p) => p.to_svg(), Element::Polygon(p) => p.to_svg(),
            Element::Text(t) => t.to_svg(), Element::Image(i) => i.to_svg(),
            Element::Diamond(d) => d.to_svg(), Element::Node(n) => n.to_svg(),
            Element::Edge(e) => e.to_svg(("arrow-start", "arrow-end")),
            Element::Group(children, tf) => {
                let inner: String = children.iter().map(|e| e.to_svg()).collect();
                tf.as_ref().map_or_else(|| format!("<g>{}</g>", inner), |t| format!(r#"<g transform="{}">{}</g>"#, t, inner))
            }
            Element::Graph(g) => g.to_svg("graph"),
        }
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        match self {
            Element::Rect(r) => r.bounds(), Element::Circle(c) => c.bounds(),
            Element::Ellipse(e) => e.bounds(), Element::Line(l) => l.bounds(),
            Element::Path(p) => p.bounds(), Element::Polygon(p) => p.bounds(),
            Element::Text(t) => t.bounds(), Element::Image(i) => i.bounds(),
            Element::Diamond(d) => d.bounds(), Element::Node(n) => n.bounds(),
            Element::Edge(e) => e.bounds(), Element::Graph(g) => g.bounds(),
            Element::Group(children, _) => {
                if children.is_empty() { return (0.0, 0.0, 0.0, 0.0); }
                let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for c in children { let (x, y, w, h) = c.bounds(); min_x = min_x.min(x); min_y = min_y.min(y); max_x = max_x.max(x + w); max_y = max_y.max(y + h); }
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
    }
}

/// Gradient definition
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "GradientShape")]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Gradient {
    pub id: String, pub kind: String, pub from_color: String, pub to_color: String, pub angle: f32,
}

#[cfg(feature = "python")]
#[pymethods]
impl Gradient {
    #[new]
    #[pyo3(signature = (id, kind="linear".to_string(), from_color="#fff".to_string(), to_color="#000".to_string(), angle=90.0))]
    fn py_new(id: String, kind: String, from_color: String, to_color: String, angle: f32) -> Self { Self { id, kind, from_color, to_color, angle } }
}

impl Gradient {
    pub fn to_svg(&self) -> String {
        if self.kind == "radial" {
            format!(r#"<radialGradient id="{}"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="{}"/></radialGradient>"#, self.id, self.from_color, self.to_color)
        } else {
            let rad = (self.angle - 90.0).to_radians();
            format!(r#"<linearGradient id="{}" x1="0%" y1="0%" x2="{:.1}%" y2="{:.1}%"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="{}"/></linearGradient>"#,
                self.id, 50.0 + 50.0 * rad.cos(), 50.0 + 50.0 * rad.sin(), self.from_color, self.to_color)
        }
    }
}

/// Filter definition
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Filter {
    pub id: String, pub kind: String, pub dx: f32, pub dy: f32, pub blur: f32, pub color: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl Filter {
    #[new]
    #[pyo3(signature = (id, kind="shadow".to_string(), dx=0.0, dy=4.0, blur=8.0, color="#0004".to_string()))]
    fn py_new(id: String, kind: String, dx: f32, dy: f32, blur: f32, color: String) -> Self { Self { id, kind, dx, dy, blur, color } }
}

impl Filter {
    pub fn to_svg(&self) -> String {
        match self.kind.as_str() {
            "shadow" => format!(r#"<filter id="{}" x="-50%" y="-50%" width="200%" height="200%"><feDropShadow dx="{}" dy="{}" stdDeviation="{}" flood-color="{}"/></filter>"#, self.id, self.dx, self.dy, self.blur, self.color),
            "blur" => format!(r#"<filter id="{}"><feGaussianBlur stdDeviation="{}"/></filter>"#, self.id, self.blur),
            _ => String::new(),
        }
    }
}

/// Scene container using standardized sizes
#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Scene {
    pub size: CanvasSize,
    pub background: String,
    elements: Vec<Element>,
    gradients: Vec<Gradient>,
    filters: Vec<Filter>,
}

impl Default for Scene {
    fn default() -> Self {
        Self { size: CanvasSize::Medium, background: "#fff".into(), elements: Vec::new(), gradients: Vec::new(), filters: Vec::new() }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Scene {
    #[new]
    #[pyo3(signature = (size=CanvasSize::Medium, background="#fff".to_string()))]
    fn py_new(size: CanvasSize, background: String) -> Self {
        Self { size, background, elements: Vec::new(), gradients: Vec::new(), filters: Vec::new() }
    }
    #[getter] fn get_size(&self) -> CanvasSize { self.size }
    #[setter] fn set_size(&mut self, v: CanvasSize) { self.size = v; }
    #[getter] fn get_width(&self) -> u32 { self.width() }
    #[getter] fn get_height(&self) -> u32 { self.height() }
    #[getter] fn get_background(&self) -> String { self.background.clone() }
    #[setter] fn set_background(&mut self, v: String) { self.background = v; }
    fn add_rect(&mut self, rect: Rect) { self.elements.push(Element::Rect(rect)); }
    fn add_circle(&mut self, circle: Circle) { self.elements.push(Element::Circle(circle)); }
    fn add_ellipse(&mut self, ellipse: Ellipse) { self.elements.push(Element::Ellipse(ellipse)); }
    fn add_line(&mut self, line: Line) { self.elements.push(Element::Line(line)); }
    fn add_path(&mut self, path: Path) { self.elements.push(Element::Path(path)); }
    fn add_polygon(&mut self, polygon: Polygon) { self.elements.push(Element::Polygon(polygon)); }
    fn add_text(&mut self, text: Text) { self.elements.push(Element::Text(text)); }
    fn add_image(&mut self, image: Image) { self.elements.push(Element::Image(image)); }
    fn add_gradient(&mut self, gradient: Gradient) { self.gradients.push(gradient); }
    fn add_filter(&mut self, filter: Filter) { self.filters.push(filter); }
    fn clear(&mut self) { self.elements.clear(); self.gradients.clear(); self.filters.clear(); }
    fn count(&self) -> usize { self.elements.len() }
    fn to_svg(&self) -> String { self.render_svg() }
}

impl Scene {
    pub fn new(size: CanvasSize, background: String) -> Self {
        Self { size, background, elements: Vec::new(), gradients: Vec::new(), filters: Vec::new() }
    }
    
    #[inline] pub fn width(&self) -> u32 { self.size.pixels() }
    #[inline] pub fn height(&self) -> u32 { self.size.pixels() }
    #[inline] pub fn dimensions(&self) -> (u32, u32) { self.size.dimensions() }
    
    pub fn push(&mut self, el: Element) { self.elements.push(el); }
    #[inline] pub fn elements(&self) -> &[Element] { &self.elements }
    #[inline] pub fn elements_mut(&mut self) -> &mut Vec<Element> { &mut self.elements }
    #[inline] pub fn gradients(&self) -> &[Gradient] { &self.gradients }
    #[inline] pub fn filters(&self) -> &[Filter] { &self.filters }

    pub fn render_svg(&self) -> String {
        let (w, h) = self.dimensions();
        let mut svg = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#, w, h);
        svg.push_str(&format!(r#"<rect width="100%" height="100%" fill="{}"/>"#, self.background));
        
        // Check if we need arrow markers (for edges/graphs)
        let needs_markers = self.elements.iter().any(|e| matches!(e, Element::Edge(_) | Element::Graph(_)));
        
        if !self.gradients.is_empty() || !self.filters.is_empty() || needs_markers {
            svg.push_str("<defs>");
            for g in &self.gradients { svg.push_str(&g.to_svg()); }
            for f in &self.filters { svg.push_str(&f.to_svg()); }
            if needs_markers {
                svg.push_str(&super::shape::arrow_marker_defs("arrow", "#333"));
                svg.push_str(&super::shape::arrow_marker_defs("graph", "#333"));
            }
            svg.push_str("</defs>");
        }
        for el in &self.elements { svg.push_str(&el.to_svg()); }
        svg.push_str("</svg>");
        svg
    }
    pub fn to_json(&self) -> String { 
        let (w, h) = self.dimensions();
        serde_json::json!({"size": self.size.to_string(), "width": w, "height": h, "background": self.background, "element_count": self.elements.len()}).to_string() 
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_scene_new() { let s = Scene::new(CanvasSize::Large, "#fff".into()); assert_eq!(s.dimensions(), (96, 96)); }
    #[test] fn test_scene_svg() { let s = Scene::new(CanvasSize::Small, "#000".into()); assert!(s.render_svg().contains("</svg>")); assert!(s.render_svg().contains("48")); }
}
