//! Shape primitives for the rendering engine

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// RGBA color representation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[pyclass]
pub struct Color {
    #[pyo3(get, set)]
    pub r: u8,
    #[pyo3(get, set)]
    pub g: u8,
    #[pyo3(get, set)]
    pub b: u8,
    #[pyo3(get, set)]
    pub a: f32,
}

#[pymethods]
impl Color {
    #[new]
    #[pyo3(signature = (r=0, g=0, b=0, a=1.0))]
    fn new(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[staticmethod]
    fn from_hex(hex: &str) -> PyResult<Self> {
        let hex = hex.trim_start_matches('#');
        let (r, g, b) = match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
                (r, g, b)
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                (r, g, b)
            }
            _ => (0, 0, 0),
        };
        Ok(Self { r, g, b, a: 1.0 })
    }

    fn to_css(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
    }
}

/// Style properties for shapes
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[pyclass]
pub struct Style {
    #[pyo3(get, set)]
    pub fill: Option<String>,
    #[pyo3(get, set)]
    pub stroke: Option<String>,
    #[pyo3(get, set)]
    pub stroke_width: f32,
    #[pyo3(get, set)]
    pub opacity: f32,
    #[pyo3(get, set)]
    pub corner: f32,
}

#[pymethods]
impl Style {
    #[new]
    #[pyo3(signature = (fill=None, stroke=None, stroke_width=1.0, opacity=1.0, corner=0.0))]
    fn new(fill: Option<String>, stroke: Option<String>, stroke_width: f32, opacity: f32, corner: f32) -> Self {
        Self { fill, stroke, stroke_width, opacity, corner }
    }
}

impl Style {
    pub fn to_svg_attrs(&self) -> String {
        let mut attrs = Vec::new();
        if let Some(ref fill) = self.fill {
            attrs.push(format!(r#"fill="{}""#, fill));
        }
        if let Some(ref stroke) = self.stroke {
            attrs.push(format!(r#"stroke="{}" stroke-width="{}""#, stroke, self.stroke_width));
        }
        if self.opacity < 1.0 {
            attrs.push(format!(r#"opacity="{}""#, self.opacity));
        }
        if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) }
    }
}

/// Rectangle primitive
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct Rect {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
    #[pyo3(get, set)]
    pub w: f32,
    #[pyo3(get, set)]
    pub h: f32,
    #[pyo3(get, set)]
    pub rx: f32,
    #[pyo3(get, set)]
    pub style: Style,
}

#[pymethods]
impl Rect {
    #[new]
    #[pyo3(signature = (x, y, w, h, rx=0.0, style=None))]
    fn new(x: f32, y: f32, w: f32, h: f32, rx: f32, style: Option<Style>) -> Self {
        Self { x, y, w, h, rx, style: style.unwrap_or_default() }
    }
}

impl Rect {
    pub fn to_svg(&self) -> String {
        let rx = if self.rx > 0.0 { format!(r#" rx="{}""#, self.rx) } else { String::new() };
        format!(r#"<rect x="{}" y="{}" width="{}" height="{}"{}{}/>"#,
            self.x, self.y, self.w, self.h, rx, self.style.to_svg_attrs())
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }
}

/// Circle primitive
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct Circle {
    #[pyo3(get, set)]
    pub cx: f32,
    #[pyo3(get, set)]
    pub cy: f32,
    #[pyo3(get, set)]
    pub r: f32,
    #[pyo3(get, set)]
    pub style: Style,
}

#[pymethods]
impl Circle {
    #[new]
    #[pyo3(signature = (cx, cy, r, style=None))]
    fn new(cx: f32, cy: f32, r: f32, style: Option<Style>) -> Self {
        Self { cx, cy, r, style: style.unwrap_or_default() }
    }
}

impl Circle {
    pub fn to_svg(&self) -> String {
        format!(r#"<circle cx="{}" cy="{}" r="{}"{}/>"#, self.cx, self.cy, self.r, self.style.to_svg_attrs())
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.cx - self.r, self.cy - self.r, self.r * 2.0, self.r * 2.0)
    }
}

/// Ellipse primitive
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct Ellipse {
    #[pyo3(get, set)]
    pub cx: f32,
    #[pyo3(get, set)]
    pub cy: f32,
    #[pyo3(get, set)]
    pub rx: f32,
    #[pyo3(get, set)]
    pub ry: f32,
    #[pyo3(get, set)]
    pub style: Style,
}

#[pymethods]
impl Ellipse {
    #[new]
    #[pyo3(signature = (cx, cy, rx, ry, style=None))]
    fn new(cx: f32, cy: f32, rx: f32, ry: f32, style: Option<Style>) -> Self {
        Self { cx, cy, rx, ry, style: style.unwrap_or_default() }
    }
}

impl Ellipse {
    pub fn to_svg(&self) -> String {
        format!(r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{}/>"#,
            self.cx, self.cy, self.rx, self.ry, self.style.to_svg_attrs())
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.cx - self.rx, self.cy - self.ry, self.rx * 2.0, self.ry * 2.0)
    }
}

/// Line primitive
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct Line {
    #[pyo3(get, set)]
    pub x1: f32,
    #[pyo3(get, set)]
    pub y1: f32,
    #[pyo3(get, set)]
    pub x2: f32,
    #[pyo3(get, set)]
    pub y2: f32,
    #[pyo3(get, set)]
    pub style: Style,
}

#[pymethods]
impl Line {
    #[new]
    #[pyo3(signature = (x1, y1, x2, y2, style=None))]
    fn new(x1: f32, y1: f32, x2: f32, y2: f32, style: Option<Style>) -> Self {
        let mut style = style.unwrap_or_default();
        if style.stroke.is_none() { style.stroke = Some("#000".into()); }
        Self { x1, y1, x2, y2, style }
    }
}

impl Line {
    pub fn to_svg(&self) -> String {
        let stroke = self.style.stroke.as_deref().unwrap_or("#000");
        format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>"#,
            self.x1, self.y1, self.x2, self.y2, stroke, self.style.stroke_width)
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let x = self.x1.min(self.x2);
        let y = self.y1.min(self.y2);
        (x, y, (self.x1 - self.x2).abs(), (self.y1 - self.y2).abs())
    }
}

/// Path primitive
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct Path {
    #[pyo3(get, set)]
    pub d: String,
    #[pyo3(get, set)]
    pub style: Style,
}

#[pymethods]
impl Path {
    #[new]
    #[pyo3(signature = (d, style=None))]
    fn new(d: String, style: Option<Style>) -> Self {
        Self { d, style: style.unwrap_or_default() }
    }
}

impl Path {
    pub fn to_svg(&self) -> String {
        format!(r#"<path d="{}"{}/>"#, self.d, self.style.to_svg_attrs())
    }
}

/// Polygon primitive
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct Polygon {
    #[pyo3(get, set)]
    pub points: Vec<(f32, f32)>,
    #[pyo3(get, set)]
    pub style: Style,
}

#[pymethods]
impl Polygon {
    #[new]
    #[pyo3(signature = (points, style=None))]
    fn new(points: Vec<(f32, f32)>, style: Option<Style>) -> Self {
        Self { points, style: style.unwrap_or_default() }
    }
}

impl Polygon {
    pub fn to_svg(&self) -> String {
        let pts: String = self.points.iter().map(|(x, y)| format!("{},{}", x, y)).collect::<Vec<_>>().join(" ");
        format!(r#"<polygon points="{}"{}/>"#, pts, self.style.to_svg_attrs())
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if self.points.is_empty() { return (0.0, 0.0, 0.0, 0.0); }
        let (mut min_x, mut min_y) = self.points[0];
        let (mut max_x, mut max_y) = self.points[0];
        for &(x, y) in &self.points[1..] {
            min_x = min_x.min(x); min_y = min_y.min(y);
            max_x = max_x.max(x); max_y = max_y.max(y);
        }
        (min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

/// Text primitive
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct Text {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
    #[pyo3(get, set)]
    pub content: String,
    #[pyo3(get, set)]
    pub font: String,
    #[pyo3(get, set)]
    pub size: f32,
    #[pyo3(get, set)]
    pub weight: String,
    #[pyo3(get, set)]
    pub anchor: String,
    #[pyo3(get, set)]
    pub style: Style,
}

#[pymethods]
impl Text {
    #[new]
    #[pyo3(signature = (x, y, content, font="system-ui".to_string(), size=16.0, weight="normal".to_string(), anchor="start".to_string(), style=None))]
    fn new(x: f32, y: f32, content: String, font: String, size: f32, weight: String, anchor: String, style: Option<Style>) -> Self {
        Self { x, y, content, font, size, weight, anchor, style: style.unwrap_or_default() }
    }
}

impl Text {
    pub fn to_svg(&self) -> String {
        let fill = self.style.fill.as_deref().unwrap_or("#000");
        format!(
            r#"<text x="{}" y="{}" font-family="{}" font-size="{}" font-weight="{}" text-anchor="{}" fill="{}">{}</text>"#,
            self.x, self.y, self.font, self.size, self.weight, self.anchor, fill, 
            html_escape(&self.content)
        )
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let w = self.content.len() as f32 * self.size * 0.6;
        let h = self.size * 1.2;
        (self.x, self.y - self.size, w, h)
    }
}

/// Image primitive
#[derive(Clone, Debug, Serialize, Deserialize)]
#[pyclass]
pub struct Image {
    #[pyo3(get, set)]
    pub x: f32,
    #[pyo3(get, set)]
    pub y: f32,
    #[pyo3(get, set)]
    pub w: f32,
    #[pyo3(get, set)]
    pub h: f32,
    #[pyo3(get, set)]
    pub href: String,
}

#[pymethods]
impl Image {
    #[new]
    #[pyo3(signature = (x, y, w, h, href))]
    fn new(x: f32, y: f32, w: f32, h: f32, href: String) -> Self {
        Self { x, y, w, h, href }
    }
}

impl Image {
    pub fn to_svg(&self) -> String {
        format!(r#"<image x="{}" y="{}" width="{}" height="{}" href="{}"/>"#,
            self.x, self.y, self.w, self.h, html_escape(&self.href))
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
