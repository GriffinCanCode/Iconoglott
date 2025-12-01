//! Scene graph management

#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use super::shape::{Circle, Ellipse, Image, Line, Path, Polygon, Rect, Text};

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
#[cfg_attr(feature = "python", pyclass)]
pub struct Gradient {
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub id: String,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub kind: String,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub from_color: String,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub to_color: String,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub angle: f32,
}

#[cfg(feature = "python")]
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
#[cfg_attr(feature = "python", pyclass)]
pub struct Filter {
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub id: String,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub kind: String,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub dx: f32,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub dy: f32,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub blur: f32,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub color: String,
}

#[cfg(feature = "python")]
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
#[cfg_attr(feature = "python", pyclass)]
pub struct Scene {
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub width: u32,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub height: u32,
    #[cfg_attr(feature = "python", pyo3(get, set))]
    pub background: String,
    elements: Vec<Element>,
    gradients: Vec<Gradient>,
    filters: Vec<Filter>,
}

#[cfg(feature = "python")]
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

    fn to_svg(&self) -> String { self.render_svg() }
}

impl Scene {
    /// Internal constructor (not exposed to Python)
    pub fn new_internal(width: u32, height: u32, background: String) -> Self {
        Self { width, height, background, elements: Vec::new(), gradients: Vec::new(), filters: Vec::new() }
    }

    /// Add element (Rust API)
    pub fn push(&mut self, el: Element) { self.elements.push(el); }

    /// Access elements slice for diffing
    #[inline]
    pub fn elements(&self) -> &[Element] { &self.elements }

    /// Mutable elements access
    #[inline]
    pub fn elements_mut(&mut self) -> &mut Vec<Element> { &mut self.elements }

    /// Access gradients slice for diffing
    #[inline]
    pub fn gradients(&self) -> &[Gradient] { &self.gradients }

    /// Access filters slice for diffing
    #[inline]
    pub fn filters(&self) -> &[Filter] { &self.filters }

    /// Render to SVG string
    pub fn render_svg(&self) -> String {
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

    pub fn to_json(&self) -> String {
        serde_json::json!({
            "width": self.width,
            "height": self.height,
            "background": self.background,
            "element_count": self.elements.len()
        }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::shape::Style;

    fn make_style(fill: &str) -> Style {
        Style { fill: Some(fill.into()), ..Style::default() }
    }

    fn make_rect(x: f32, y: f32, w: f32, h: f32, style: Option<Style>) -> Rect {
        Rect { x, y, w, h, rx: 0.0, style: style.unwrap_or_default(), transform: None }
    }

    #[test]
    fn test_scene_new() {
        let s = Scene::new_internal(800, 600, "#fff".into());
        assert_eq!(s.width, 800);
        assert_eq!(s.height, 600);
        assert_eq!(s.background, "#fff");
    }

    #[test]
    fn test_scene_empty_svg() {
        let s = Scene::new_internal(100, 100, "#000".into());
        let svg = s.render_svg();
        assert!(svg.contains(r#"xmlns="http://www.w3.org/2000/svg""#));
        assert!(svg.contains(r#"width="100""#));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_scene_to_json() {
        let s = Scene::new_internal(800, 600, "#1a1a2e".into());
        let json = s.to_json();
        assert!(json.contains(r#""width":800"#));
        assert!(json.contains(r#""height":600"#));
    }

    #[test]
    fn test_element_rect_svg() {
        let r = make_rect(10.0, 20.0, 100.0, 50.0, Some(make_style("#f00")));
        let el = Element::Rect(r);
        let svg = el.to_svg();
        assert!(svg.contains("<rect"));
        assert!(svg.contains("fill=\"#f00\""));
    }

    #[test]
    fn test_element_group_bounds() {
        let style = make_style("#fff");
        let r1 = make_rect(0.0, 0.0, 50.0, 50.0, Some(style.clone()));
        let r2 = make_rect(100.0, 100.0, 50.0, 50.0, Some(style));
        let group = Element::Group(vec![Element::Rect(r1), Element::Rect(r2)], None);
        let (x, y, w, h) = group.bounds();
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);
        assert!((w - 150.0).abs() < 0.001);
        assert!((h - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_element_group_empty_bounds() {
        let group = Element::Group(vec![], None);
        assert_eq!(group.bounds(), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_gradient_linear() {
        let g = Gradient { id: "grad1".into(), kind: "linear".into(), from_color: "#f00".into(), to_color: "#00f".into(), angle: 90.0 };
        let svg = g.to_svg();
        assert!(svg.contains("<linearGradient id=\"grad1\""));
        assert!(svg.contains("stop-color=\"#f00\""));
    }

    #[test]
    fn test_gradient_radial() {
        let g = Gradient { id: "grad2".into(), kind: "radial".into(), from_color: "#fff".into(), to_color: "#000".into(), angle: 0.0 };
        let svg = g.to_svg();
        assert!(svg.contains(r#"<radialGradient id="grad2""#));
    }

    #[test]
    fn test_filter_shadow() {
        let f = Filter { id: "shadow1".into(), kind: "shadow".into(), dx: 2.0, dy: 4.0, blur: 8.0, color: "#0008".into() };
        let svg = f.to_svg();
        assert!(svg.contains(r#"<filter id="shadow1""#));
        assert!(svg.contains(r#"<feDropShadow"#));
    }

    #[test]
    fn test_filter_blur() {
        let f = Filter { id: "blur1".into(), kind: "blur".into(), dx: 0.0, dy: 0.0, blur: 5.0, color: "#000".into() };
        let svg = f.to_svg();
        assert!(svg.contains(r#"<feGaussianBlur"#));
    }

    #[test]
    fn test_filter_unknown_type() {
        let f = Filter { id: "unk".into(), kind: "glow".into(), dx: 0.0, dy: 0.0, blur: 5.0, color: "#fff".into() };
        assert_eq!(f.to_svg(), "");
    }
}
