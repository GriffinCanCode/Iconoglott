//! Shape primitives for the rendering engine

#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// RGBA color representation
#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ShapeStyle")]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Style {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub corner: f32,
    pub filter: Option<String>,
    /// Animation class name (references CSS animation)
    pub animation_class: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Style {
    #[new]
    #[pyo3(signature = (fill=None, stroke=None, stroke_width=1.0, opacity=1.0, corner=0.0, filter=None))]
    fn py_new(fill: Option<String>, stroke: Option<String>, stroke_width: f32, opacity: f32, corner: f32, filter: Option<String>) -> Self {
        Self { fill, stroke, stroke_width, opacity, corner, filter, animation_class: None }
    }
}

impl Style {
    pub fn with_fill(fill: &str) -> Self {
        Self { fill: Some(fill.into()), opacity: 1.0, stroke_width: 1.0, ..Default::default() }
    }
    
    pub fn with_animation_class(class: &str) -> Self {
        Self { animation_class: Some(class.into()), opacity: 1.0, stroke_width: 1.0, ..Default::default() }
    }
    
    pub fn to_svg_attrs(&self) -> String {
        let mut attrs = Vec::with_capacity(5);
        if let Some(ref fill) = self.fill { attrs.push(format!(r#"fill="{}""#, fill)); }
        if let Some(ref stroke) = self.stroke { attrs.push(format!(r#"stroke="{}" stroke-width="{}""#, stroke, self.stroke_width)); }
        if self.opacity < 1.0 { attrs.push(format!(r#"opacity="{}""#, self.opacity)); }
        if let Some(ref filter) = self.filter { attrs.push(format!(r#"filter="url(#{})""#, filter)); }
        if let Some(ref class) = self.animation_class { attrs.push(format!(r#"class="{}""#, class)); }
        if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) }
    }
    
    /// Generate style attribute with animation CSS
    pub fn to_style_attr(&self, anim_css: Option<&str>) -> String {
        match anim_css {
            Some(css) => format!(r#" style="{}""#, css),
            None => String::new(),
        }
    }
}

/// Rectangle primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
    pub fn bounds(&self) -> (f32, f32, f32, f32) { self.bounds_hint.unwrap_or_else(|| crate::path::parse_path_bounds(&self.d)) }
}

/// Polygon primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "TextShape")]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "ImageShape")]
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

/// Diamond primitive (rotated rect for flowcharts)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Diamond {
    pub cx: f32, pub cy: f32, pub w: f32, pub h: f32,
    pub style: Style, pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Diamond {
    #[new]
    #[pyo3(signature = (cx, cy, w, h, style=None, transform=None))]
    fn py_new(cx: f32, cy: f32, w: f32, h: f32, style: Option<Style>, transform: Option<String>) -> Self {
        Self { cx, cy, w, h, style: style.unwrap_or_default(), transform }
    }
}

impl Diamond {
    pub fn to_svg(&self) -> String {
        let pts = format!("{},{} {},{} {},{} {},{}",
            self.cx, self.cy - self.h / 2.0,
            self.cx + self.w / 2.0, self.cy,
            self.cx, self.cy + self.h / 2.0,
            self.cx - self.w / 2.0, self.cy);
        format!(r#"<polygon points="{}"{}{}/>"#, pts, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }
    pub fn bounds(&self) -> (f32, f32, f32, f32) { (self.cx - self.w / 2.0, self.cy - self.h / 2.0, self.w, self.h) }
}

/// Node for graph/flowchart (composite: shape + label)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export, rename = "GraphNodeShape")]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Node {
    pub id: String,
    pub shape: String,  // rect, circle, ellipse, diamond
    pub cx: f32, pub cy: f32, pub w: f32, pub h: f32,
    pub label: Option<String>,
    pub style: Style,
    pub label_style: Style,
    pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Node {
    #[new]
    #[pyo3(signature = (id, shape="rect".to_string(), cx=0.0, cy=0.0, w=80.0, h=40.0, label=None, style=None, transform=None))]
    fn py_new(id: String, shape: String, cx: f32, cy: f32, w: f32, h: f32, label: Option<String>, style: Option<Style>, transform: Option<String>) -> Self {
        Self { id, shape, cx, cy, w, h, label, style: style.unwrap_or_default(), label_style: Style::default(), transform }
    }
}

impl Node {
    pub fn to_svg(&self) -> String {
        let shape_svg = match self.shape.as_str() {
            "circle" => {
                let r = self.w.min(self.h) / 2.0;
                format!(r#"<circle cx="{}" cy="{}" r="{}"{}/>"#, self.cx, self.cy, r, self.style.to_svg_attrs())
            }
            "ellipse" => {
                format!(r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{}/>"#, self.cx, self.cy, self.w / 2.0, self.h / 2.0, self.style.to_svg_attrs())
            }
            "diamond" => {
                let pts = format!("{},{} {},{} {},{} {},{}",
                    self.cx, self.cy - self.h / 2.0,
                    self.cx + self.w / 2.0, self.cy,
                    self.cx, self.cy + self.h / 2.0,
                    self.cx - self.w / 2.0, self.cy);
                format!(r#"<polygon points="{}"{}/>"#, pts, self.style.to_svg_attrs())
            }
            _ => { // rect
                let x = self.cx - self.w / 2.0;
                let y = self.cy - self.h / 2.0;
                format!(r#"<rect x="{}" y="{}" width="{}" height="{}"{}/>"#, x, y, self.w, self.h, self.style.to_svg_attrs())
            }
        };
        
        let label_svg = self.label.as_ref().map_or(String::new(), |lbl| {
            let fill = self.label_style.fill.as_deref().unwrap_or("#000");
            format!(r#"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" fill="{}">{}</text>"#, 
                self.cx, self.cy, fill, html_escape(lbl))
        });
        
        format!(r#"<g id="node-{}"{}>{}{}</g>"#, html_escape(&self.id), transform_attr(&self.transform), shape_svg, label_svg)
    }
    
    pub fn bounds(&self) -> (f32, f32, f32, f32) { (self.cx - self.w / 2.0, self.cy - self.h / 2.0, self.w, self.h) }
    
    /// Get anchor point for edges (center of specified side)
    pub fn anchor(&self, side: &str) -> (f32, f32) {
        match side {
            "top" | "n" => (self.cx, self.cy - self.h / 2.0),
            "bottom" | "s" => (self.cx, self.cy + self.h / 2.0),
            "left" | "w" => (self.cx - self.w / 2.0, self.cy),
            "right" | "e" => (self.cx + self.w / 2.0, self.cy),
            _ => (self.cx, self.cy), // center
        }
    }
}

/// Edge style enumeration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum EdgeStyle { Straight, Curved, Orthogonal }

impl Default for EdgeStyle {
    fn default() -> Self { Self::Straight }
}

/// Arrow type enumeration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ArrowType { None, Forward, Backward, Both }

impl Default for ArrowType {
    fn default() -> Self { Self::Forward }
}

/// Edge/connector between nodes
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Edge {
    pub from_id: String,
    pub to_id: String,
    pub from_pt: (f32, f32),
    pub to_pt: (f32, f32),
    pub edge_style: String,
    pub arrow: String,
    pub label: Option<String>,
    pub style: Style,
}

#[cfg(feature = "python")]
#[pymethods]
impl Edge {
    #[new]
    #[pyo3(signature = (from_id, to_id, from_pt, to_pt, edge_style="straight".to_string(), arrow="forward".to_string(), label=None, style=None))]
    fn py_new(from_id: String, to_id: String, from_pt: (f32, f32), to_pt: (f32, f32), edge_style: String, arrow: String, label: Option<String>, style: Option<Style>) -> Self {
        let mut s = style.unwrap_or_default();
        if s.stroke.is_none() { s.stroke = Some("#333".into()); }
        if s.stroke_width == 0.0 { s.stroke_width = 2.0; }
        Self { from_id, to_id, from_pt, to_pt, edge_style, arrow, label, style: s }
    }
}

impl Edge {
    pub fn to_svg(&self, marker_ids: (&str, &str)) -> String {
        let (x1, y1) = self.from_pt;
        let (x2, y2) = self.to_pt;
        let stroke = self.style.stroke.as_deref().unwrap_or("#333");
        
        let path_d = match self.edge_style.as_str() {
            "curved" => {
                let mx = (x1 + x2) / 2.0;
                let my = (y1 + y2) / 2.0;
                let dx = (x2 - x1).abs();
                let dy = (y2 - y1).abs();
                let ctrl_offset = (dx.max(dy)) * 0.3;
                if (y2 - y1).abs() > (x2 - x1).abs() {
                    format!("M{},{} C{},{} {},{} {},{}", x1, y1, x1, my, x2, my, x2, y2)
                } else {
                    format!("M{},{} C{},{} {},{} {},{}", x1, y1, mx, y1 + ctrl_offset, mx, y2 - ctrl_offset, x2, y2)
                }
            }
            "orthogonal" => {
                let mx = (x1 + x2) / 2.0;
                format!("M{},{} L{},{} L{},{} L{},{}", x1, y1, mx, y1, mx, y2, x2, y2)
            }
            _ => format!("M{},{} L{},{}", x1, y1, x2, y2), // straight
        };
        
        let markers = match self.arrow.as_str() {
            "forward" => format!(r#" marker-end="url(#{})""#, marker_ids.1),
            "backward" => format!(r#" marker-start="url(#{})""#, marker_ids.0),
            "both" => format!(r#" marker-start="url(#{})" marker-end="url(#{})""#, marker_ids.0, marker_ids.1),
            _ => String::new(),
        };
        
        let label_svg = self.label.as_ref().map_or(String::new(), |lbl| {
            let mx = (x1 + x2) / 2.0;
            let my = (y1 + y2) / 2.0;
            format!(r##"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" font-size="12" fill="#666">{}</text>"##, mx, my - 8.0, html_escape(lbl))
        });
        
        format!(r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}"{}/>{}"#, 
            path_d, stroke, self.style.stroke_width, markers, label_svg)
    }
    
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let (x1, y1) = self.from_pt;
        let (x2, y2) = self.to_pt;
        (x1.min(x2), y1.min(y2), (x1 - x2).abs(), (y1 - y2).abs())
    }
}

/// Generate SVG defs for arrow markers
pub fn arrow_marker_defs(id_prefix: &str, color: &str) -> String {
    format!(
        r#"<marker id="{id_prefix}-arrow-start" markerWidth="10" markerHeight="7" refX="0" refY="3.5" orient="auto-start-reverse"><polygon points="10 0, 10 7, 0 3.5" fill="{color}"/></marker><marker id="{id_prefix}-arrow-end" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="{color}"/></marker>"#,
        id_prefix = id_prefix, color = color
    )
}

/// Symbol definition for reusable components (SVG <symbol>)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Symbol {
    pub id: String,
    pub viewbox: Option<(f32, f32, f32, f32)>,
    pub children: Vec<super::Element>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Symbol {
    #[new]
    #[pyo3(signature = (id, viewbox=None))]
    fn py_new(id: String, viewbox: Option<(f32, f32, f32, f32)>) -> Self {
        Self { id, viewbox, children: Vec::new() }
    }
    
    #[getter] fn get_id(&self) -> String { self.id.clone() }
    #[setter] fn set_id(&mut self, v: String) { self.id = v; }
    #[getter] fn get_viewbox(&self) -> Option<(f32, f32, f32, f32)> { self.viewbox }
    #[setter] fn set_viewbox(&mut self, v: Option<(f32, f32, f32, f32)>) { self.viewbox = v; }
    fn child_count(&self) -> usize { self.children.len() }
}

impl Symbol {
    pub fn to_svg_def(&self) -> String {
        let viewbox = self.viewbox.map_or(String::new(), |(x, y, w, h)| 
            format!(r#" viewBox="{} {} {} {}""#, x, y, w, h));
        let inner: String = self.children.iter().map(|e| e.to_svg()).collect();
        format!(r#"<symbol id="{}"{}>{}</symbol>"#, html_escape(&self.id), viewbox, inner)
    }
    
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        self.viewbox.unwrap_or_else(|| {
            if self.children.is_empty() { return (0.0, 0.0, 0.0, 0.0); }
            let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for c in &self.children {
                let (x, y, w, h) = c.bounds();
                min_x = min_x.min(x); min_y = min_y.min(y); max_x = max_x.max(x + w); max_y = max_y.max(y + h);
            }
            (min_x, min_y, max_x - min_x, max_y - min_y)
        })
    }
}

/// Use reference to instantiate a symbol (SVG <use>)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct Use {
    pub href: String,
    pub x: f32,
    pub y: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub style: Style,
    pub transform: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl Use {
    #[new]
    #[pyo3(signature = (href, x=0.0, y=0.0, width=None, height=None, style=None, transform=None))]
    fn py_new(href: String, x: f32, y: f32, width: Option<f32>, height: Option<f32>, style: Option<Style>, transform: Option<String>) -> Self {
        Self { href, x, y, width, height, style: style.unwrap_or_default(), transform }
    }
}

impl Use {
    pub fn to_svg(&self) -> String {
        let size = match (self.width, self.height) {
            (Some(w), Some(h)) => format!(r#" width="{}" height="{}""#, w, h),
            (Some(w), None) => format!(r#" width="{}""#, w),
            (None, Some(h)) => format!(r#" height="{}""#, h),
            _ => String::new(),
        };
        format!("<use href=\"#{}\" x=\"{}\" y=\"{}\"{}{}{}/>" , 
            html_escape(&self.href), self.x, self.y, size, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }
    
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width.unwrap_or(0.0), self.height.unwrap_or(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_rect_bounds() { assert_eq!(Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0, rx: 0.0, style: Style::default(), transform: None }.bounds(), (10.0, 20.0, 100.0, 50.0)); }
    #[test] fn test_circle_bounds() { assert_eq!(Circle { cx: 100.0, cy: 100.0, r: 50.0, style: Style::default(), transform: None }.bounds(), (50.0, 50.0, 100.0, 100.0)); }
}
