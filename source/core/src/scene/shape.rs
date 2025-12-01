//! Shape primitives for the rendering engine

#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// RGBA color representation
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

#[cfg(feature = "python")]
#[pymethods]
impl Color {
    #[new]
    #[pyo3(signature = (r=0, g=0, b=0, a=1.0))]
    fn py_new(r: u8, g: u8, b: u8, a: f32) -> Self { Self { r, g, b, a } }

    #[staticmethod]
    fn from_hex(hex: &str) -> PyResult<Self> { Ok(Self::parse_hex(hex)) }
    fn to_css(&self) -> String { self.css() }
}

impl Color {
    pub fn parse_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let (r, g, b) = match hex.len() {
            3 => (
                u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0),
                u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0),
                u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0),
            ),
            6 => (
                u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
                u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
                u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
            ),
            _ => (0, 0, 0),
        };
        Self { r, g, b, a: 1.0 }
    }
    pub fn css(&self) -> String { format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a) }
}

/// Style properties for shapes
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Style {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub corner: f32,
    pub filter: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Style {
    #[new]
    #[pyo3(signature = (fill=None, stroke=None, stroke_width=1.0, opacity=1.0, corner=0.0, filter=None))]
    fn py_new(fill: Option<String>, stroke: Option<String>, stroke_width: f32, opacity: f32, corner: f32, filter: Option<String>) -> Self {
        Self { fill, stroke, stroke_width, opacity, corner, filter }
    }
}

impl Style {
    pub fn with_fill(fill: &str) -> Self {
        Self { fill: Some(fill.into()), opacity: 1.0, stroke_width: 1.0, ..Default::default() }
    }
    pub fn to_svg_attrs(&self) -> String {
        let mut attrs = Vec::with_capacity(4);
        if let Some(ref fill) = self.fill { attrs.push(format!(r#"fill="{}""#, fill)); }
        if let Some(ref stroke) = self.stroke { attrs.push(format!(r#"stroke="{}" stroke-width="{}""#, stroke, self.stroke_width)); }
        if self.opacity < 1.0 { attrs.push(format!(r#"opacity="{}""#, self.opacity)); }
        if let Some(ref filter) = self.filter { attrs.push(format!(r#"filter="url(#{})""#, filter)); }
        if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) }
    }
}

/// Rectangle primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Rect {
    pub x: f32, pub y: f32, pub w: f32, pub h: f32, pub rx: f32,
    pub style: Style, pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Rect {
    #[new]
    #[pyo3(signature = (x, y, w, h, rx=0.0, style=None, transform=None))]
    fn py_new(x: f32, y: f32, w: f32, h: f32, rx: f32, style: Option<Style>, transform: Option<String>) -> Self {
        Self { x, y, w, h, rx, style: style.unwrap_or_default(), transform }
    }
}

impl Rect {
    pub fn to_svg(&self) -> String {
        let rx = if self.rx > 0.0 { format!(r#" rx="{}""#, self.rx) } else { String::new() };
        format!(r#"<rect x="{}" y="{}" width="{}" height="{}"{}{}{}/>"#,
            self.x, self.y, self.w, self.h, rx, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) { (self.x, self.y, self.w, self.h) }
}

/// Circle primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Circle {
    pub cx: f32, pub cy: f32, pub r: f32,
    pub style: Style, pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Circle {
    #[new]
    #[pyo3(signature = (cx, cy, r, style=None, transform=None))]
    fn py_new(cx: f32, cy: f32, r: f32, style: Option<Style>, transform: Option<String>) -> Self {
        Self { cx, cy, r, style: style.unwrap_or_default(), transform }
    }
}

impl Circle {
    pub fn to_svg(&self) -> String {
        format!(r#"<circle cx="{}" cy="{}" r="{}"{}{}/>"#, self.cx, self.cy, self.r, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) { (self.cx - self.r, self.cy - self.r, self.r * 2.0, self.r * 2.0) }
}

/// Ellipse primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Ellipse {
    pub cx: f32, pub cy: f32, pub rx: f32, pub ry: f32,
    pub style: Style, pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Ellipse {
    #[new]
    #[pyo3(signature = (cx, cy, rx, ry, style=None, transform=None))]
    fn py_new(cx: f32, cy: f32, rx: f32, ry: f32, style: Option<Style>, transform: Option<String>) -> Self {
        Self { cx, cy, rx, ry, style: style.unwrap_or_default(), transform }
    }
}

impl Ellipse {
    pub fn to_svg(&self) -> String {
        format!(r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{}{}/>"#, self.cx, self.cy, self.rx, self.ry, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) { (self.cx - self.rx, self.cy - self.ry, self.rx * 2.0, self.ry * 2.0) }
}

/// Line primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Line {
    pub x1: f32, pub y1: f32, pub x2: f32, pub y2: f32,
    pub style: Style, pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Line {
    #[new]
    #[pyo3(signature = (x1, y1, x2, y2, style=None, transform=None))]
    fn py_new(x1: f32, y1: f32, x2: f32, y2: f32, style: Option<Style>, transform: Option<String>) -> Self {
        let mut style = style.unwrap_or_default();
        if style.stroke.is_none() { style.stroke = Some("#000".into()); }
        Self { x1, y1, x2, y2, style, transform }
    }
}

impl Line {
    pub fn to_svg(&self) -> String {
        let stroke = self.style.stroke.as_deref().unwrap_or("#000");
        format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"{}/>"#,
            self.x1, self.y1, self.x2, self.y2, stroke, self.style.stroke_width, transform_attr(&self.transform))
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x1.min(self.x2), self.y1.min(self.y2), (self.x1 - self.x2).abs(), (self.y1 - self.y2).abs())
    }
}

/// Path primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Path {
    pub d: String, pub style: Style, pub transform: Option<String>,
    pub bounds_hint: Option<(f32, f32, f32, f32)>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Path {
    #[new]
    #[pyo3(signature = (d, style=None, transform=None, bounds_hint=None))]
    fn py_new(d: String, style: Option<Style>, transform: Option<String>, bounds_hint: Option<(f32, f32, f32, f32)>) -> Self {
        Self { d, style: style.unwrap_or_default(), transform, bounds_hint }
    }
}

impl Path {
    pub fn to_svg(&self) -> String {
        format!(r#"<path d="{}"{}{}/>"#, self.d, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) { self.bounds_hint.unwrap_or_else(|| parse_path_bounds(&self.d)) }
}

fn parse_path_bounds(d: &str) -> (f32, f32, f32, f32) {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let (mut cur_x, mut cur_y, mut start_x, mut start_y) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    let mut track = |x: f32, y: f32| { min_x = min_x.min(x); min_y = min_y.min(y); max_x = max_x.max(x); max_y = max_y.max(y); };
    let nums: Vec<f32> = extract_numbers(d);
    let cmds: Vec<char> = d.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let mut idx = 0;
    for cmd in cmds {
        match cmd {
            'M' if idx + 1 < nums.len() => { cur_x = nums[idx]; cur_y = nums[idx + 1]; start_x = cur_x; start_y = cur_y; track(cur_x, cur_y); idx += 2; }
            'm' if idx + 1 < nums.len() => { cur_x += nums[idx]; cur_y += nums[idx + 1]; start_x = cur_x; start_y = cur_y; track(cur_x, cur_y); idx += 2; }
            'L' if idx + 1 < nums.len() => { cur_x = nums[idx]; cur_y = nums[idx + 1]; track(cur_x, cur_y); idx += 2; }
            'l' if idx + 1 < nums.len() => { cur_x += nums[idx]; cur_y += nums[idx + 1]; track(cur_x, cur_y); idx += 2; }
            'H' if idx < nums.len() => { cur_x = nums[idx]; track(cur_x, cur_y); idx += 1; }
            'h' if idx < nums.len() => { cur_x += nums[idx]; track(cur_x, cur_y); idx += 1; }
            'V' if idx < nums.len() => { cur_y = nums[idx]; track(cur_x, cur_y); idx += 1; }
            'v' if idx < nums.len() => { cur_y += nums[idx]; track(cur_x, cur_y); idx += 1; }
            'C' if idx + 5 < nums.len() => { for i in (0..6).step_by(2) { track(nums[idx + i], nums[idx + i + 1]); } cur_x = nums[idx + 4]; cur_y = nums[idx + 5]; idx += 6; }
            'c' if idx + 5 < nums.len() => { for i in (0..6).step_by(2) { track(cur_x + nums[idx + i], cur_y + nums[idx + i + 1]); } cur_x += nums[idx + 4]; cur_y += nums[idx + 5]; idx += 6; }
            'Z' | 'z' => { cur_x = start_x; cur_y = start_y; }
            _ => {}
        }
    }
    if min_x == f32::MAX { (0.0, 0.0, 0.0, 0.0) } else { (min_x, min_y, max_x - min_x, max_y - min_y) }
}

fn extract_numbers(d: &str) -> Vec<f32> {
    let mut nums = Vec::new();
    let mut buf = String::new();
    for c in d.chars() {
        if c.is_ascii_digit() || c == '.' || (c == '-' && buf.is_empty()) || (c == '-' && buf.ends_with('e')) { buf.push(c); }
        else if c == 'e' || c == 'E' { buf.push('e'); }
        else { if !buf.is_empty() { if let Ok(n) = buf.parse::<f32>() { nums.push(n); } buf.clear(); } if c == '-' { buf.push(c); } }
    }
    if !buf.is_empty() { if let Ok(n) = buf.parse::<f32>() { nums.push(n); } }
    nums
}

/// Polygon primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Polygon {
    pub points: Vec<(f32, f32)>, pub style: Style, pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Polygon {
    #[new]
    #[pyo3(signature = (points, style=None, transform=None))]
    fn py_new(points: Vec<(f32, f32)>, style: Option<Style>, transform: Option<String>) -> Self {
        Self { points, style: style.unwrap_or_default(), transform }
    }
}

impl Polygon {
    pub fn to_svg(&self) -> String {
        let pts: String = self.points.iter().map(|(x, y)| format!("{},{}", x, y)).collect::<Vec<_>>().join(" ");
        format!(r#"<polygon points="{}"{}{}/>"#, pts, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if self.points.is_empty() { return (0.0, 0.0, 0.0, 0.0); }
        let (mut min_x, mut min_y) = self.points[0];
        let (mut max_x, mut max_y) = self.points[0];
        for &(x, y) in &self.points[1..] { min_x = min_x.min(x); min_y = min_y.min(y); max_x = max_x.max(x); max_y = max_y.max(y); }
        (min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

/// Text primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Text {
    pub x: f32, pub y: f32, pub content: String, pub font: String, pub size: f32,
    pub weight: String, pub anchor: String, pub style: Style, pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Text {
    #[new]
    #[pyo3(signature = (x, y, content, font="system-ui".to_string(), size=16.0, weight="normal".to_string(), anchor="start".to_string(), style=None, transform=None))]
    fn py_new(x: f32, y: f32, content: String, font: String, size: f32, weight: String, anchor: String, style: Option<Style>, transform: Option<String>) -> Self {
        Self { x, y, content, font, size, weight, anchor, style: style.unwrap_or_default(), transform }
    }
}

impl Text {
    pub fn to_svg(&self) -> String {
        let fill = self.style.fill.as_deref().unwrap_or("#000");
        format!(r#"<text x="{}" y="{}" font-family="{}" font-size="{}" font-weight="{}" text-anchor="{}" fill="{}"{}>{}</text>"#,
            self.x, self.y, self.font, self.size, self.weight, self.anchor, fill, transform_attr(&self.transform), html_escape(&self.content))
    }
    
    /// Compute bounding box using font metrics
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let metrics = crate::font::measure_text(&self.content, &self.font, self.size);
        let x = match self.anchor.as_str() {
            "middle" => self.x - metrics.width / 2.0,
            "end" => self.x - metrics.width,
            _ => self.x,
        };
        (x, self.y - metrics.ascender, metrics.width, metrics.height)
    }
    
    /// Get detailed text metrics
    pub fn metrics(&self) -> crate::font::TextMetrics {
        crate::font::measure_text(&self.content, &self.font, self.size)
    }
}

/// Image primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Image {
    pub x: f32, pub y: f32, pub w: f32, pub h: f32, pub href: String, pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Image {
    #[new]
    #[pyo3(signature = (x, y, w, h, href, transform=None))]
    fn py_new(x: f32, y: f32, w: f32, h: f32, href: String, transform: Option<String>) -> Self {
        Self { x, y, w, h, href, transform }
    }
}

impl Image {
    pub fn to_svg(&self) -> String {
        format!(r#"<image x="{}" y="{}" width="{}" height="{}" href="{}"{}/>"#, self.x, self.y, self.w, self.h, html_escape(&self.href), transform_attr(&self.transform))
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) { (self.x, self.y, self.w, self.h) }
}

fn html_escape(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;") }
#[inline] fn transform_attr(tf: &Option<String>) -> String { tf.as_ref().map_or(String::new(), |t| format!(r#" transform="{}""#, t)) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_rect_bounds() { assert_eq!(Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0, rx: 0.0, style: Style::default(), transform: None }.bounds(), (10.0, 20.0, 100.0, 50.0)); }
    #[test] fn test_circle_bounds() { assert_eq!(Circle { cx: 100.0, cy: 100.0, r: 50.0, style: Style::default(), transform: None }.bounds(), (50.0, 50.0, 100.0, 100.0)); }
}
