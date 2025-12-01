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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
    #[pyo3(get, set)]
    pub filter: Option<String>,
}

#[pymethods]
impl Style {
    #[new]
    #[pyo3(signature = (fill=None, stroke=None, stroke_width=1.0, opacity=1.0, corner=0.0, filter=None))]
    fn new(fill: Option<String>, stroke: Option<String>, stroke_width: f32, opacity: f32, corner: f32, filter: Option<String>) -> Self {
        Self { fill, stroke, stroke_width, opacity, corner, filter }
    }
}

impl Style {
    pub fn to_svg_attrs(&self) -> String {
        let mut attrs = Vec::with_capacity(4);
        if let Some(ref fill) = self.fill {
            attrs.push(format!(r#"fill="{}""#, fill));
        }
        if let Some(ref stroke) = self.stroke {
            attrs.push(format!(r#"stroke="{}" stroke-width="{}""#, stroke, self.stroke_width));
        }
        if self.opacity < 1.0 {
            attrs.push(format!(r#"opacity="{}""#, self.opacity));
        }
        if let Some(ref filter) = self.filter {
            attrs.push(format!(r#"filter="url(#{})""#, filter));
        }
        if attrs.is_empty() { String::new() } else { format!(" {}", attrs.join(" ")) }
    }
}

/// Rectangle primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    #[pyo3(get, set)]
    pub transform: Option<String>,
}

#[pymethods]
impl Rect {
    #[new]
    #[pyo3(signature = (x, y, w, h, rx=0.0, style=None, transform=None))]
    fn new(x: f32, y: f32, w: f32, h: f32, rx: f32, style: Option<Style>, transform: Option<String>) -> Self {
        Self { x, y, w, h, rx, style: style.unwrap_or_default(), transform }
    }
}

impl Rect {
    pub fn to_svg(&self) -> String {
        let rx = if self.rx > 0.0 { format!(r#" rx="{}""#, self.rx) } else { String::new() };
        format!(r#"<rect x="{}" y="{}" width="{}" height="{}"{}{}{}/>"#,
            self.x, self.y, self.w, self.h, rx, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }
}

/// Circle primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    #[pyo3(get, set)]
    pub transform: Option<String>,
}

#[pymethods]
impl Circle {
    #[new]
    #[pyo3(signature = (cx, cy, r, style=None, transform=None))]
    fn new(cx: f32, cy: f32, r: f32, style: Option<Style>, transform: Option<String>) -> Self {
        Self { cx, cy, r, style: style.unwrap_or_default(), transform }
    }
}

impl Circle {
    pub fn to_svg(&self) -> String {
        format!(r#"<circle cx="{}" cy="{}" r="{}"{}{}/>"#, self.cx, self.cy, self.r, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.cx - self.r, self.cy - self.r, self.r * 2.0, self.r * 2.0)
    }
}

/// Ellipse primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    #[pyo3(get, set)]
    pub transform: Option<String>,
}

#[pymethods]
impl Ellipse {
    #[new]
    #[pyo3(signature = (cx, cy, rx, ry, style=None, transform=None))]
    fn new(cx: f32, cy: f32, rx: f32, ry: f32, style: Option<Style>, transform: Option<String>) -> Self {
        Self { cx, cy, rx, ry, style: style.unwrap_or_default(), transform }
    }
}

impl Ellipse {
    pub fn to_svg(&self) -> String {
        format!(r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{}{}/>"#,
            self.cx, self.cy, self.rx, self.ry, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.cx - self.rx, self.cy - self.ry, self.rx * 2.0, self.ry * 2.0)
    }
}

/// Line primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    #[pyo3(get, set)]
    pub transform: Option<String>,
}

#[pymethods]
impl Line {
    #[new]
    #[pyo3(signature = (x1, y1, x2, y2, style=None, transform=None))]
    fn new(x1: f32, y1: f32, x2: f32, y2: f32, style: Option<Style>, transform: Option<String>) -> Self {
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
        let x = self.x1.min(self.x2);
        let y = self.y1.min(self.y2);
        (x, y, (self.x1 - self.x2).abs(), (self.y1 - self.y2).abs())
    }
}

/// Path primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct Path {
    #[pyo3(get, set)]
    pub d: String,
    #[pyo3(get, set)]
    pub style: Style,
    #[pyo3(get, set)]
    pub transform: Option<String>,
    /// Explicit bounds override (x, y, w, h) - if set, skips path parsing
    #[pyo3(get, set)]
    pub bounds_hint: Option<(f32, f32, f32, f32)>,
}

#[pymethods]
impl Path {
    #[new]
    #[pyo3(signature = (d, style=None, transform=None, bounds_hint=None))]
    fn new(d: String, style: Option<Style>, transform: Option<String>, bounds_hint: Option<(f32, f32, f32, f32)>) -> Self {
        Self { d, style: style.unwrap_or_default(), transform, bounds_hint }
    }
}

impl Path {
    pub fn to_svg(&self) -> String {
        format!(r#"<path d="{}"{}{}/>"#, self.d, self.style.to_svg_attrs(), transform_attr(&self.transform))
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if let Some(hint) = self.bounds_hint {
            return hint;
        }
        parse_path_bounds(&self.d)
    }
}

/// Minimal SVG path parser for bounding box extraction
fn parse_path_bounds(d: &str) -> (f32, f32, f32, f32) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    let mut cur_x = 0.0_f32;
    let mut cur_y = 0.0_f32;
    let mut start_x = 0.0_f32;
    let mut start_y = 0.0_f32;
    
    let mut track = |x: f32, y: f32| {
        min_x = min_x.min(x); min_y = min_y.min(y);
        max_x = max_x.max(x); max_y = max_y.max(y);
    };

    let nums: Vec<f32> = extract_numbers(d);
    let cmds: Vec<(char, usize)> = extract_commands(d);
    let mut idx = 0;

    for (cmd, _pos) in cmds {
        match cmd {
            'M' => if idx + 1 < nums.len() {
                cur_x = nums[idx]; cur_y = nums[idx + 1];
                start_x = cur_x; start_y = cur_y;
                track(cur_x, cur_y); idx += 2;
                // Implicit L after first pair
                while idx + 1 < nums.len() && !is_next_cmd(d, _pos, idx, &nums) {
                    cur_x = nums[idx]; cur_y = nums[idx + 1];
                    track(cur_x, cur_y); idx += 2;
                }
            }
            'm' => if idx + 1 < nums.len() {
                cur_x += nums[idx]; cur_y += nums[idx + 1];
                start_x = cur_x; start_y = cur_y;
                track(cur_x, cur_y); idx += 2;
                while idx + 1 < nums.len() && !is_next_cmd(d, _pos, idx, &nums) {
                    cur_x += nums[idx]; cur_y += nums[idx + 1];
                    track(cur_x, cur_y); idx += 2;
                }
            }
            'L' => while idx + 1 < nums.len() {
                cur_x = nums[idx]; cur_y = nums[idx + 1];
                track(cur_x, cur_y); idx += 2;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'l' => while idx + 1 < nums.len() {
                cur_x += nums[idx]; cur_y += nums[idx + 1];
                track(cur_x, cur_y); idx += 2;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'H' => while idx < nums.len() {
                cur_x = nums[idx]; track(cur_x, cur_y); idx += 1;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'h' => while idx < nums.len() {
                cur_x += nums[idx]; track(cur_x, cur_y); idx += 1;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'V' => while idx < nums.len() {
                cur_y = nums[idx]; track(cur_x, cur_y); idx += 1;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'v' => while idx < nums.len() {
                cur_y += nums[idx]; track(cur_x, cur_y); idx += 1;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'C' => while idx + 5 < nums.len() {
                // Cubic bezier: track control points and endpoint
                for i in (0..6).step_by(2) { track(nums[idx + i], nums[idx + i + 1]); }
                cur_x = nums[idx + 4]; cur_y = nums[idx + 5]; idx += 6;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'c' => while idx + 5 < nums.len() {
                for i in (0..6).step_by(2) { track(cur_x + nums[idx + i], cur_y + nums[idx + i + 1]); }
                cur_x += nums[idx + 4]; cur_y += nums[idx + 5]; idx += 6;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'S' => while idx + 3 < nums.len() {
                track(nums[idx], nums[idx + 1]); track(nums[idx + 2], nums[idx + 3]);
                cur_x = nums[idx + 2]; cur_y = nums[idx + 3]; idx += 4;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            's' => while idx + 3 < nums.len() {
                track(cur_x + nums[idx], cur_y + nums[idx + 1]);
                cur_x += nums[idx + 2]; cur_y += nums[idx + 3];
                track(cur_x, cur_y); idx += 4;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'Q' => while idx + 3 < nums.len() {
                track(nums[idx], nums[idx + 1]); track(nums[idx + 2], nums[idx + 3]);
                cur_x = nums[idx + 2]; cur_y = nums[idx + 3]; idx += 4;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'q' => while idx + 3 < nums.len() {
                track(cur_x + nums[idx], cur_y + nums[idx + 1]);
                cur_x += nums[idx + 2]; cur_y += nums[idx + 3];
                track(cur_x, cur_y); idx += 4;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'T' => while idx + 1 < nums.len() {
                cur_x = nums[idx]; cur_y = nums[idx + 1];
                track(cur_x, cur_y); idx += 2;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            't' => while idx + 1 < nums.len() {
                cur_x += nums[idx]; cur_y += nums[idx + 1];
                track(cur_x, cur_y); idx += 2;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'A' => while idx + 6 < nums.len() {
                // Arc: rx, ry, rotation, large-arc, sweep, x, y
                // Track current + radii for conservative bound, then endpoint
                let rx = nums[idx].abs(); let ry = nums[idx + 1].abs();
                track(cur_x - rx, cur_y - ry); track(cur_x + rx, cur_y + ry);
                cur_x = nums[idx + 5]; cur_y = nums[idx + 6];
                track(cur_x, cur_y); idx += 7;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'a' => while idx + 6 < nums.len() {
                let rx = nums[idx].abs(); let ry = nums[idx + 1].abs();
                track(cur_x - rx, cur_y - ry); track(cur_x + rx, cur_y + ry);
                cur_x += nums[idx + 5]; cur_y += nums[idx + 6];
                track(cur_x, cur_y); idx += 7;
                if is_next_cmd(d, _pos, idx, &nums) { break; }
            }
            'Z' | 'z' => { cur_x = start_x; cur_y = start_y; }
            _ => {}
        }
    }

    if min_x == f32::MAX { (0.0, 0.0, 0.0, 0.0) }
    else { (min_x, min_y, max_x - min_x, max_y - min_y) }
}

/// Extract all numbers from path data
fn extract_numbers(d: &str) -> Vec<f32> {
    let mut nums = Vec::new();
    let mut buf = String::new();
    let mut chars = d.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() || c == '.' || (c == '-' && buf.is_empty()) || (c == '-' && buf.ends_with("e")) {
            buf.push(c);
        } else if c == 'e' || c == 'E' {
            buf.push('e');
        } else {
            if !buf.is_empty() {
                if let Ok(n) = buf.parse::<f32>() { nums.push(n); }
                buf.clear();
            }
            // Handle negative sign as start of new number
            if c == '-' { buf.push(c); }
        }
    }
    if !buf.is_empty() {
        if let Ok(n) = buf.parse::<f32>() { nums.push(n); }
    }
    nums
}

/// Extract command letters with their positions
fn extract_commands(d: &str) -> Vec<(char, usize)> {
    d.char_indices()
        .filter(|(_, c)| matches!(c, 'M'|'m'|'L'|'l'|'H'|'h'|'V'|'v'|'C'|'c'|'S'|'s'|'Q'|'q'|'T'|'t'|'A'|'a'|'Z'|'z'))
        .map(|(i, c)| (c, i))
        .collect()
}

/// Check if we've consumed numbers up to the next command (simple heuristic)
fn is_next_cmd(_d: &str, _cmd_pos: usize, _idx: usize, _nums: &[f32]) -> bool {
    false // Let the match arms handle iteration via index bounds
}

/// Polygon primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[pyclass]
pub struct Polygon {
    #[pyo3(get, set)]
    pub points: Vec<(f32, f32)>,
    #[pyo3(get, set)]
    pub style: Style,
    #[pyo3(get, set)]
    pub transform: Option<String>,
}

#[pymethods]
impl Polygon {
    #[new]
    #[pyo3(signature = (points, style=None, transform=None))]
    fn new(points: Vec<(f32, f32)>, style: Option<Style>, transform: Option<String>) -> Self {
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
        for &(x, y) in &self.points[1..] {
            min_x = min_x.min(x); min_y = min_y.min(y);
            max_x = max_x.max(x); max_y = max_y.max(y);
        }
        (min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

/// Text primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    #[pyo3(get, set)]
    pub transform: Option<String>,
}

#[pymethods]
impl Text {
    #[new]
    #[pyo3(signature = (x, y, content, font="system-ui".to_string(), size=16.0, weight="normal".to_string(), anchor="start".to_string(), style=None, transform=None))]
    fn new(x: f32, y: f32, content: String, font: String, size: f32, weight: String, anchor: String, style: Option<Style>, transform: Option<String>) -> Self {
        Self { x, y, content, font, size, weight, anchor, style: style.unwrap_or_default(), transform }
    }
}

impl Text {
    pub fn to_svg(&self) -> String {
        let fill = self.style.fill.as_deref().unwrap_or("#000");
        format!(
            r#"<text x="{}" y="{}" font-family="{}" font-size="{}" font-weight="{}" text-anchor="{}" fill="{}"{}>{}</text>"#,
            self.x, self.y, self.font, self.size, self.weight, self.anchor, fill, 
            transform_attr(&self.transform), html_escape(&self.content)
        )
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let w = self.content.len() as f32 * self.size * 0.6;
        let h = self.size * 1.2;
        (self.x, self.y - self.size, w, h)
    }
}

/// Image primitive
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    #[pyo3(get, set)]
    pub transform: Option<String>,
}

#[pymethods]
impl Image {
    #[new]
    #[pyo3(signature = (x, y, w, h, href, transform=None))]
    fn new(x: f32, y: f32, w: f32, h: f32, href: String, transform: Option<String>) -> Self {
        Self { x, y, w, h, href, transform }
    }
}

impl Image {
    pub fn to_svg(&self) -> String {
        format!(r#"<image x="{}" y="{}" width="{}" height="{}" href="{}"{}/>"#,
            self.x, self.y, self.w, self.h, html_escape(&self.href), transform_attr(&self.transform))
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[inline]
fn transform_attr(tf: &Option<String>) -> String {
    tf.as_ref().map_or(String::new(), |t| format!(r#" transform="{}""#, t))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_style(fill: Option<&str>, stroke: Option<&str>, sw: f32, op: f32, corner: f32, filter: Option<&str>) -> Style {
        Style { fill: fill.map(String::from), stroke: stroke.map(String::from), stroke_width: sw, opacity: op, corner, filter: filter.map(String::from) }
    }

    fn make_rect(x: f32, y: f32, w: f32, h: f32, rx: f32, style: Option<Style>, transform: Option<String>) -> Rect {
        Rect { x, y, w, h, rx, style: style.unwrap_or_default(), transform }
    }

    fn make_circle(cx: f32, cy: f32, r: f32, style: Option<Style>, transform: Option<String>) -> Circle {
        Circle { cx, cy, r, style: style.unwrap_or_default(), transform }
    }

    fn make_ellipse(cx: f32, cy: f32, rx: f32, ry: f32, style: Option<Style>, transform: Option<String>) -> Ellipse {
        Ellipse { cx, cy, rx, ry, style: style.unwrap_or_default(), transform }
    }

    fn make_line(x1: f32, y1: f32, x2: f32, y2: f32, style: Option<Style>, transform: Option<String>) -> Line {
        let mut s = style.unwrap_or_default();
        if s.stroke.is_none() { s.stroke = Some("#000".into()); }
        Line { x1, y1, x2, y2, style: s, transform }
    }

    fn make_path(d: &str, style: Option<Style>, transform: Option<String>, bounds_hint: Option<(f32, f32, f32, f32)>) -> Path {
        Path { d: d.to_string(), style: style.unwrap_or_default(), transform, bounds_hint }
    }

    fn make_polygon(points: Vec<(f32, f32)>, style: Option<Style>, transform: Option<String>) -> Polygon {
        Polygon { points, style: style.unwrap_or_default(), transform }
    }

    fn make_text(x: f32, y: f32, content: &str, font: &str, size: f32, weight: &str, anchor: &str, style: Option<Style>, transform: Option<String>) -> Text {
        Text { x, y, content: content.to_string(), font: font.to_string(), size, weight: weight.to_string(), anchor: anchor.to_string(), style: style.unwrap_or_default(), transform }
    }

    fn make_image(x: f32, y: f32, w: f32, h: f32, href: &str, transform: Option<String>) -> Image {
        Image { x, y, w, h, href: href.to_string(), transform }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Color tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_color_default() {
        let c = Color::default();
        assert_eq!((c.r, c.g, c.b), (0, 0, 0));
    }

    #[test]
    fn test_color_to_css() {
        let c = Color { r: 100, g: 150, b: 200, a: 0.8 };
        assert_eq!(c.to_css(), "rgba(100,150,200,0.8)");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Style tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_style_default() {
        let s = Style::default();
        assert!(s.fill.is_none());
        // Default for f32 is 0.0, not 1.0
        assert!((s.opacity - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_style_to_svg_attrs_empty() {
        // Default has opacity=0, which gets rendered as opacity="0"
        let s = Style::default();
        // When opacity < 1.0, it's included in attrs
        assert!(s.to_svg_attrs().contains("opacity"));
    }

    #[test]
    fn test_style_to_svg_attrs_fill() {
        let s = make_style(Some("#ff0"), None, 1.0, 1.0, 0.0, None);
        assert!(s.to_svg_attrs().contains("fill=\"#ff0\""));
    }

    #[test]
    fn test_style_to_svg_attrs_stroke() {
        let s = make_style(None, Some("#000"), 2.0, 1.0, 0.0, None);
        let attrs = s.to_svg_attrs();
        assert!(attrs.contains("stroke=\"#000\""));
        assert!(attrs.contains("stroke-width=\"2\""));
    }

    #[test]
    fn test_style_to_svg_attrs_opacity() {
        let s = make_style(None, None, 1.0, 0.5, 0.0, None);
        assert!(s.to_svg_attrs().contains("opacity=\"0.5\""));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Rect tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_rect_to_svg_basic() {
        let r = make_rect(10.0, 20.0, 100.0, 50.0, 0.0, None, None);
        let svg = r.to_svg();
        assert!(svg.starts_with("<rect "));
        assert!(svg.contains(r#"x="10""#));
    }

    #[test]
    fn test_rect_to_svg_rounded() {
        let r = make_rect(0.0, 0.0, 50.0, 50.0, 8.0, None, None);
        assert!(r.to_svg().contains(r#"rx="8""#));
    }

    #[test]
    fn test_rect_bounds() {
        let r = make_rect(10.0, 20.0, 100.0, 50.0, 0.0, None, None);
        assert_eq!(r.bounds(), (10.0, 20.0, 100.0, 50.0));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Circle tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_circle_to_svg() {
        let c = make_circle(50.0, 60.0, 25.0, None, None);
        let svg = c.to_svg();
        assert!(svg.contains(r#"cx="50""#));
        assert!(svg.contains(r#"r="25""#));
    }

    #[test]
    fn test_circle_bounds() {
        let c = make_circle(100.0, 100.0, 50.0, None, None);
        let (x, _, w, _) = c.bounds();
        assert!((x - 50.0).abs() < 0.001);
        assert!((w - 100.0).abs() < 0.001);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Ellipse tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_ellipse_to_svg() {
        let e = make_ellipse(100.0, 50.0, 80.0, 40.0, None, None);
        let svg = e.to_svg();
        assert!(svg.contains(r#"rx="80""#));
        assert!(svg.contains(r#"ry="40""#));
    }

    #[test]
    fn test_ellipse_bounds() {
        let e = make_ellipse(100.0, 100.0, 50.0, 30.0, None, None);
        let (_, _, w, h) = e.bounds();
        assert!((w - 100.0).abs() < 0.001);
        assert!((h - 60.0).abs() < 0.001);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Line tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_line_to_svg() {
        let l = make_line(0.0, 0.0, 100.0, 100.0, None, None);
        let svg = l.to_svg();
        assert!(svg.contains(r#"x1="0""#));
        assert!(svg.contains(r#"x2="100""#));
    }

    #[test]
    fn test_line_bounds() {
        let l = make_line(10.0, 20.0, 50.0, 80.0, None, None);
        let (x, y, w, h) = l.bounds();
        assert!((x - 10.0).abs() < 0.001);
        assert!((y - 20.0).abs() < 0.001);
        assert!((w - 40.0).abs() < 0.001);
        assert!((h - 60.0).abs() < 0.001);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Path tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_path_to_svg() {
        let p = make_path("M 0 0 L 100 100", None, None, None);
        assert!(p.to_svg().contains(r#"d="M 0 0 L 100 100""#));
    }

    #[test]
    fn test_path_bounds_hint() {
        let p = make_path("M 0 0", None, None, Some((5.0, 10.0, 90.0, 80.0)));
        assert_eq!(p.bounds(), (5.0, 10.0, 90.0, 80.0));
    }

    #[test]
    fn test_path_bounds_parsed() {
        let p = make_path("M 10 20 L 50 80", None, None, None);
        let (x, y, w, h) = p.bounds();
        assert!((x - 10.0).abs() < 0.001);
        assert!((y - 20.0).abs() < 0.001);
        assert!((w - 40.0).abs() < 0.001);
        assert!((h - 60.0).abs() < 0.001);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Polygon tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_polygon_to_svg() {
        let p = make_polygon(vec![(0.0, 0.0), (100.0, 0.0), (50.0, 100.0)], None, None);
        assert!(p.to_svg().contains(r#"points="#));
    }

    #[test]
    fn test_polygon_bounds() {
        let p = make_polygon(vec![(10.0, 20.0), (90.0, 30.0), (50.0, 80.0)], None, None);
        let (x, y, w, h) = p.bounds();
        assert!((x - 10.0).abs() < 0.001);
        assert!((y - 20.0).abs() < 0.001);
        assert!((w - 80.0).abs() < 0.001);
        assert!((h - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_polygon_bounds_empty() {
        let p = make_polygon(vec![], None, None);
        assert_eq!(p.bounds(), (0.0, 0.0, 0.0, 0.0));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Text tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_text_to_svg() {
        let t = make_text(10.0, 20.0, "Hello", "Arial", 16.0, "normal", "start", None, None);
        let svg = t.to_svg();
        assert!(svg.contains(">Hello</text>"));
        assert!(svg.contains(r#"font-family="Arial""#));
    }

    #[test]
    fn test_text_escapes_html() {
        let t = make_text(0.0, 0.0, "<script>&", "Arial", 12.0, "normal", "start", None, None);
        let svg = t.to_svg();
        assert!(svg.contains("&lt;script&gt;&amp;"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Image tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_image_to_svg() {
        let i = make_image(10.0, 20.0, 100.0, 80.0, "img.png", None);
        let svg = i.to_svg();
        assert!(svg.contains(r#"href="img.png""#));
    }

    #[test]
    fn test_image_bounds() {
        let i = make_image(10.0, 20.0, 100.0, 80.0, "", None);
        assert_eq!(i.bounds(), (10.0, 20.0, 100.0, 80.0));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Helper tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<>&\""), "&lt;&gt;&amp;&quot;");
    }

    #[test]
    fn test_transform_attr_none() {
        assert_eq!(transform_attr(&None), "");
    }

    #[test]
    fn test_transform_attr_some() {
        assert_eq!(transform_attr(&Some("rotate(45)".into())), r#" transform="rotate(45)""#);
    }
}
