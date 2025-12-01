//! Scene graph management

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shape::{Circle, Ellipse, Image, Line, Path, Polygon, Rect, Text};

/// A renderable element in the scene
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Rect(Rect),
    Circle(Circle),
    Ellipse(Ellipse),
    Line(Line),
    Path(Path),
    Polygon(Polygon),
    Text(Text),
    Image(Image),
    Group(Vec<Element>, Option<String>),
}

impl Element {
    pub fn to_svg(&self) -> String {
        match self {
            Element::Rect(r) => r.to_svg(),
            Element::Circle(c) => c.to_svg(),
            Element::Ellipse(e) => e.to_svg(),
            Element::Line(l) => l.to_svg(),
            Element::Path(p) => p.to_svg(),
            Element::Polygon(p) => p.to_svg(),
            Element::Text(t) => t.to_svg(),
            Element::Image(i) => i.to_svg(),
            Element::Group(children, transform) => {
                let inner: String = children.iter().map(|e| e.to_svg()).collect();
                match transform {
                    Some(tf) => format!(r#"<g transform="{}">{}</g>"#, tf, inner),
                    None => format!("<g>{}</g>", inner),
                }
            }
        }
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        match self {
            Element::Rect(r) => r.bounds(),
            Element::Circle(c) => c.bounds(),
            Element::Ellipse(e) => e.bounds(),
            Element::Line(l) => l.bounds(),
            Element::Path(p) => p.bounds(),
            Element::Polygon(p) => p.bounds(),
            Element::Text(t) => t.bounds(),
            Element::Image(i) => i.bounds(),
            Element::Group(children, _) => {
                if children.is_empty() { return (0.0, 0.0, 0.0, 0.0); }
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;
                let mut max_x = f32::MIN;
                let mut max_y = f32::MIN;
                for c in children {
                    let (x, y, w, h) = c.bounds();
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x + w);
                    max_y = max_y.max(y + h);
                }
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
    }
}

/// Gradient definition
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct Gradient {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub kind: String,
    #[pyo3(get, set)]
    pub from_color: String,
    #[pyo3(get, set)]
    pub to_color: String,
    #[pyo3(get, set)]
    pub angle: f32,
}

#[pymethods]
impl Gradient {
    #[new]
    #[pyo3(signature = (id, kind="linear".to_string(), from_color="#fff".to_string(), to_color="#000".to_string(), angle=90.0))]
    fn new(id: String, kind: String, from_color: String, to_color: String, angle: f32) -> Self {
        Self { id, kind, from_color, to_color, angle }
    }
}

impl Gradient {
    pub fn to_svg(&self) -> String {
        if self.kind == "radial" {
            format!(
                r#"<radialGradient id="{}"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="{}"/></radialGradient>"#,
                self.id, self.from_color, self.to_color
            )
        } else {
            let rad = (self.angle - 90.0).to_radians();
            let x2 = 50.0 + 50.0 * rad.cos();
            let y2 = 50.0 + 50.0 * rad.sin();
            format!(
                r#"<linearGradient id="{}" x1="0%" y1="0%" x2="{:.1}%" y2="{:.1}%"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="{}"/></linearGradient>"#,
                self.id, x2, y2, self.from_color, self.to_color
            )
        }
    }
}

/// Filter definition
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct Filter {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub kind: String,
    #[pyo3(get, set)]
    pub dx: f32,
    #[pyo3(get, set)]
    pub dy: f32,
    #[pyo3(get, set)]
    pub blur: f32,
    #[pyo3(get, set)]
    pub color: String,
}

#[pymethods]
impl Filter {
    #[new]
    #[pyo3(signature = (id, kind="shadow".to_string(), dx=0.0, dy=4.0, blur=8.0, color="#0004".to_string()))]
    fn new(id: String, kind: String, dx: f32, dy: f32, blur: f32, color: String) -> Self {
        Self { id, kind, dx, dy, blur, color }
    }
}

impl Filter {
    pub fn to_svg(&self) -> String {
        match self.kind.as_str() {
            "shadow" => format!(
                r#"<filter id="{}" x="-50%" y="-50%" width="200%" height="200%"><feDropShadow dx="{}" dy="{}" stdDeviation="{}" flood-color="{}"/></filter>"#,
                self.id, self.dx, self.dy, self.blur, self.color
            ),
            "blur" => format!(
                r#"<filter id="{}"><feGaussianBlur stdDeviation="{}"/></filter>"#,
                self.id, self.blur
            ),
            _ => String::new(),
        }
    }
}

/// Scene container
#[derive(Clone, Debug, Default)]
#[pyclass]
pub struct Scene {
    #[pyo3(get, set)]
    pub width: u32,
    #[pyo3(get, set)]
    pub height: u32,
    #[pyo3(get, set)]
    pub background: String,
    elements: Vec<Element>,
    gradients: Vec<Gradient>,
    filters: Vec<Filter>,
}

#[pymethods]
impl Scene {
    #[new]
    #[pyo3(signature = (width=800, height=600, background="#fff".to_string()))]
    fn new(width: u32, height: u32, background: String) -> Self {
        Self { width, height, background, elements: Vec::new(), gradients: Vec::new(), filters: Vec::new() }
    }

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

    fn clear(&mut self) {
        self.elements.clear();
        self.gradients.clear();
        self.filters.clear();
    }

    fn count(&self) -> usize { self.elements.len() }

    fn to_svg(&self) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
            self.width, self.height
        );
        
        svg.push_str(&format!(r#"<rect width="100%" height="100%" fill="{}"/>"#, self.background));
        
        if !self.gradients.is_empty() || !self.filters.is_empty() {
            svg.push_str("<defs>");
            for g in &self.gradients { svg.push_str(&g.to_svg()); }
            for f in &self.filters { svg.push_str(&f.to_svg()); }
            svg.push_str("</defs>");
        }
        
        for el in &self.elements { svg.push_str(&el.to_svg()); }
        
        svg.push_str("</svg>");
        svg
    }
}

impl Scene {
    /// Internal constructor for testing (not exposed to Python)
    pub fn new_internal(width: u32, height: u32, background: String) -> Self {
        Self { width, height, background, elements: Vec::new(), gradients: Vec::new(), filters: Vec::new() }
    }

    /// Access elements slice for diffing
    #[inline]
    pub fn elements(&self) -> &[Element] { &self.elements }

    /// Access gradients slice for diffing
    #[inline]
    pub fn gradients(&self) -> &[Gradient] { &self.gradients }

    /// Access filters slice for diffing
    #[inline]
    pub fn filters(&self) -> &[Filter] { &self.filters }

    pub fn to_json(&self) -> String {
        serde_json::json!({
            "width": self.width,
            "height": self.height,
            "background": self.background,
            "element_count": self.elements.len()
        }).to_string()
    }
}
