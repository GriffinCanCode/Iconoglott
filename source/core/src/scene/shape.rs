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
    pub fn bounds(&self) -> (f32, f32, f32, f32) { self.bounds_hint.unwrap_or_else(|| parse_path_bounds(&self.d)) }
}

fn parse_path_bounds(d: &str) -> (f32, f32, f32, f32) {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let (mut cur_x, mut cur_y, mut start_x, mut start_y) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    let (mut last_ctrl_x, mut last_ctrl_y) = (0.0_f32, 0.0_f32); // for S/T commands
    let mut last_cmd = ' ';
    
    let mut track = |x: f32, y: f32| { min_x = min_x.min(x); min_y = min_y.min(y); max_x = max_x.max(x); max_y = max_y.max(y); };
    let nums: Vec<f32> = extract_numbers(d);
    let cmds: Vec<char> = d.chars().filter(|c| matches!(c, 'M'|'m'|'L'|'l'|'H'|'h'|'V'|'v'|'C'|'c'|'S'|'s'|'Q'|'q'|'T'|'t'|'A'|'a'|'Z'|'z')).collect();
    let mut idx = 0;
    
    for cmd in cmds {
        match cmd {
            'M' if idx + 1 < nums.len() => { cur_x = nums[idx]; cur_y = nums[idx + 1]; start_x = cur_x; start_y = cur_y; track(cur_x, cur_y); idx += 2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'm' if idx + 1 < nums.len() => { cur_x += nums[idx]; cur_y += nums[idx + 1]; start_x = cur_x; start_y = cur_y; track(cur_x, cur_y); idx += 2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'L' if idx + 1 < nums.len() => { cur_x = nums[idx]; cur_y = nums[idx + 1]; track(cur_x, cur_y); idx += 2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'l' if idx + 1 < nums.len() => { cur_x += nums[idx]; cur_y += nums[idx + 1]; track(cur_x, cur_y); idx += 2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'H' if idx < nums.len() => { cur_x = nums[idx]; track(cur_x, cur_y); idx += 1; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'h' if idx < nums.len() => { cur_x += nums[idx]; track(cur_x, cur_y); idx += 1; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'V' if idx < nums.len() => { cur_y = nums[idx]; track(cur_x, cur_y); idx += 1; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'v' if idx < nums.len() => { cur_y += nums[idx]; track(cur_x, cur_y); idx += 1; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            'C' if idx + 5 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1, x2, y2, x3, y3) = (nums[idx], nums[idx+1], nums[idx+2], nums[idx+3], nums[idx+4], nums[idx+5]);
                cubic_bezier_bounds(x0, y0, x1, y1, x2, y2, x3, y3, &mut track);
                cur_x = x3; cur_y = y3; last_ctrl_x = x2; last_ctrl_y = y2; idx += 6;
            }
            'c' if idx + 5 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1, x2, y2, x3, y3) = (cur_x + nums[idx], cur_y + nums[idx+1], cur_x + nums[idx+2], cur_y + nums[idx+3], cur_x + nums[idx+4], cur_y + nums[idx+5]);
                cubic_bezier_bounds(x0, y0, x1, y1, x2, y2, x3, y3, &mut track);
                last_ctrl_x = x2; last_ctrl_y = y2; cur_x = x3; cur_y = y3; idx += 6;
            }
            'S' if idx + 3 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1) = if matches!(last_cmd, 'C'|'c'|'S'|'s') { (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y) } else { (cur_x, cur_y) };
                let (x2, y2, x3, y3) = (nums[idx], nums[idx+1], nums[idx+2], nums[idx+3]);
                cubic_bezier_bounds(x0, y0, x1, y1, x2, y2, x3, y3, &mut track);
                last_ctrl_x = x2; last_ctrl_y = y2; cur_x = x3; cur_y = y3; idx += 4;
            }
            's' if idx + 3 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1) = if matches!(last_cmd, 'C'|'c'|'S'|'s') { (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y) } else { (cur_x, cur_y) };
                let (x2, y2, x3, y3) = (cur_x + nums[idx], cur_y + nums[idx+1], cur_x + nums[idx+2], cur_y + nums[idx+3]);
                cubic_bezier_bounds(x0, y0, x1, y1, x2, y2, x3, y3, &mut track);
                last_ctrl_x = x2; last_ctrl_y = y2; cur_x = x3; cur_y = y3; idx += 4;
            }
            'Q' if idx + 3 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1, x2, y2) = (nums[idx], nums[idx+1], nums[idx+2], nums[idx+3]);
                quadratic_bezier_bounds(x0, y0, x1, y1, x2, y2, &mut track);
                last_ctrl_x = x1; last_ctrl_y = y1; cur_x = x2; cur_y = y2; idx += 4;
            }
            'q' if idx + 3 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1, x2, y2) = (cur_x + nums[idx], cur_y + nums[idx+1], cur_x + nums[idx+2], cur_y + nums[idx+3]);
                quadratic_bezier_bounds(x0, y0, x1, y1, x2, y2, &mut track);
                last_ctrl_x = x1; last_ctrl_y = y1; cur_x = x2; cur_y = y2; idx += 4;
            }
            'T' if idx + 1 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1) = if matches!(last_cmd, 'Q'|'q'|'T'|'t') { (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y) } else { (cur_x, cur_y) };
                let (x2, y2) = (nums[idx], nums[idx+1]);
                quadratic_bezier_bounds(x0, y0, x1, y1, x2, y2, &mut track);
                last_ctrl_x = x1; last_ctrl_y = y1; cur_x = x2; cur_y = y2; idx += 2;
            }
            't' if idx + 1 < nums.len() => {
                let (x0, y0) = (cur_x, cur_y);
                let (x1, y1) = if matches!(last_cmd, 'Q'|'q'|'T'|'t') { (2.0 * cur_x - last_ctrl_x, 2.0 * cur_y - last_ctrl_y) } else { (cur_x, cur_y) };
                let (x2, y2) = (cur_x + nums[idx], cur_y + nums[idx+1]);
                quadratic_bezier_bounds(x0, y0, x1, y1, x2, y2, &mut track);
                last_ctrl_x = x1; last_ctrl_y = y1; cur_x = x2; cur_y = y2; idx += 2;
            }
            'A' if idx + 6 < nums.len() => {
                let (rx, ry, phi, large_arc, sweep) = (nums[idx].abs(), nums[idx+1].abs(), nums[idx+2], nums[idx+3] != 0.0, nums[idx+4] != 0.0);
                let (x2, y2) = (nums[idx+5], nums[idx+6]);
                arc_bounds(cur_x, cur_y, rx, ry, phi, large_arc, sweep, x2, y2, &mut track);
                cur_x = x2; cur_y = y2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; idx += 7;
            }
            'a' if idx + 6 < nums.len() => {
                let (rx, ry, phi, large_arc, sweep) = (nums[idx].abs(), nums[idx+1].abs(), nums[idx+2], nums[idx+3] != 0.0, nums[idx+4] != 0.0);
                let (x2, y2) = (cur_x + nums[idx+5], cur_y + nums[idx+6]);
                arc_bounds(cur_x, cur_y, rx, ry, phi, large_arc, sweep, x2, y2, &mut track);
                cur_x = x2; cur_y = y2; last_ctrl_x = cur_x; last_ctrl_y = cur_y; idx += 7;
            }
            'Z' | 'z' => { cur_x = start_x; cur_y = start_y; last_ctrl_x = cur_x; last_ctrl_y = cur_y; }
            _ => {}
        }
        last_cmd = cmd;
    }
    if min_x == f32::MAX { (0.0, 0.0, 0.0, 0.0) } else { (min_x, min_y, max_x - min_x, max_y - min_y) }
}

/// Compute cubic Bezier bounds by finding extrema
fn cubic_bezier_bounds(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, track: &mut impl FnMut(f32, f32)) {
    track(x0, y0); track(x3, y3);
    // Find t values where derivative = 0 for x and y separately
    for (p0, p1, p2, p3, is_x) in [(x0, x1, x2, x3, true), (y0, y1, y2, y3, false)] {
        // B'(t) = 3(1-t)²(p1-p0) + 6(1-t)t(p2-p1) + 3t²(p3-p2) = 0
        // Simplifies to: at² + bt + c = 0
        let a = -p0 + 3.0*p1 - 3.0*p2 + p3;
        let b = 2.0*(p0 - 2.0*p1 + p2);
        let c = -p0 + p1;
        for t in solve_quadratic(a, b, c) {
            if t > 0.0 && t < 1.0 {
                let val = cubic_at(t, p0, p1, p2, p3);
                if is_x { track(val, cubic_at(t, y0, y1, y2, y3)); }
                else { track(cubic_at(t, x0, x1, x2, x3), val); }
            }
        }
    }
}

/// Compute quadratic Bezier bounds by finding extrema
fn quadratic_bezier_bounds(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, track: &mut impl FnMut(f32, f32)) {
    track(x0, y0); track(x2, y2);
    // B'(t) = 2(1-t)(p1-p0) + 2t(p2-p1) = 0 => t = (p0-p1)/(p0-2p1+p2)
    for (p0, p1, p2, is_x) in [(x0, x1, x2, true), (y0, y1, y2, false)] {
        let denom = p0 - 2.0*p1 + p2;
        if denom.abs() > 1e-10 {
            let t = (p0 - p1) / denom;
            if t > 0.0 && t < 1.0 {
                let val = quadratic_at(t, p0, p1, p2);
                if is_x { track(val, quadratic_at(t, y0, y1, y2)); }
                else { track(quadratic_at(t, x0, x1, x2), val); }
            }
        }
    }
}

/// Compute arc bounds using endpoint parameterization
fn arc_bounds(x1: f32, y1: f32, mut rx: f32, mut ry: f32, phi_deg: f32, large_arc: bool, sweep: bool, x2: f32, y2: f32, track: &mut impl FnMut(f32, f32)) {
    track(x1, y1); track(x2, y2);
    if rx < 1e-10 || ry < 1e-10 { return; }
    
    let phi = phi_deg.to_radians();
    let (cos_phi, sin_phi) = (phi.cos(), phi.sin());
    
    // Transform to unit circle space
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;
    
    // Correct radii if too small
    let lambda = (x1p / rx).powi(2) + (y1p / ry).powi(2);
    if lambda > 1.0 { let s = lambda.sqrt(); rx *= s; ry *= s; }
    
    // Center point
    let sq = ((rx*ry).powi(2) - (rx*y1p).powi(2) - (ry*x1p).powi(2)) / ((rx*y1p).powi(2) + (ry*x1p).powi(2));
    let coef = if large_arc != sweep { sq.max(0.0).sqrt() } else { -sq.max(0.0).sqrt() };
    let cxp = coef * rx * y1p / ry;
    let cyp = -coef * ry * x1p / rx;
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;
    
    // Compute angle range
    let theta1 = ((y1p - cyp) / ry).atan2((x1p - cxp) / rx);
    let mut dtheta = (((-y1p - cyp) / ry).atan2((-x1p - cxp) / rx) - theta1).rem_euclid(std::f32::consts::TAU);
    if !sweep { dtheta -= std::f32::consts::TAU; }
    
    // Check cardinal directions for extrema
    for angle in [0.0_f32, std::f32::consts::FRAC_PI_2, std::f32::consts::PI, 3.0 * std::f32::consts::FRAC_PI_2] {
        let t = (angle - theta1).rem_euclid(std::f32::consts::TAU);
        if (sweep && t <= dtheta) || (!sweep && t >= dtheta.abs() - std::f32::consts::TAU) || dtheta.abs() >= std::f32::consts::TAU - 1e-6 {
            let px = cx + rx * angle.cos() * cos_phi - ry * angle.sin() * sin_phi;
            let py = cy + rx * angle.cos() * sin_phi + ry * angle.sin() * cos_phi;
            track(px, py);
        }
    }
}

#[inline] fn cubic_at(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let mt = 1.0 - t;
    mt*mt*mt*p0 + 3.0*mt*mt*t*p1 + 3.0*mt*t*t*p2 + t*t*t*p3
}

#[inline] fn quadratic_at(t: f32, p0: f32, p1: f32, p2: f32) -> f32 {
    let mt = 1.0 - t;
    mt*mt*p0 + 2.0*mt*t*p1 + t*t*p2
}

fn solve_quadratic(a: f32, b: f32, c: f32) -> Vec<f32> {
    if a.abs() < 1e-10 {
        return if b.abs() < 1e-10 { vec![] } else { vec![-c / b] };
    }
    let disc = b*b - 4.0*a*c;
    if disc < 0.0 { vec![] }
    else if disc < 1e-10 { vec![-b / (2.0 * a)] }
    else { let sq = disc.sqrt(); vec![(-b - sq) / (2.0 * a), (-b + sq) / (2.0 * a)] }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_rect_bounds() { assert_eq!(Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0, rx: 0.0, style: Style::default(), transform: None }.bounds(), (10.0, 20.0, 100.0, 50.0)); }
    #[test] fn test_circle_bounds() { assert_eq!(Circle { cx: 100.0, cy: 100.0, r: 50.0, style: Style::default(), transform: None }.bounds(), (50.0, 50.0, 100.0, 100.0)); }
    
    #[test] fn test_path_bounds_line() {
        let (x, y, w, h) = parse_path_bounds("M0 0 L100 50");
        assert!((x - 0.0).abs() < 0.01 && (y - 0.0).abs() < 0.01);
        assert!((w - 100.0).abs() < 0.01 && (h - 50.0).abs() < 0.01);
    }
    
    #[test] fn test_path_bounds_cubic() {
        // Cubic with control points outside endpoints
        let (x, y, w, h) = parse_path_bounds("M0 50 C0 0, 100 0, 100 50");
        assert!(y < 50.0); // curve goes above start/end y
        assert!(x >= -0.01 && (x + w) <= 100.01);
    }
    
    #[test] fn test_path_bounds_quadratic() {
        let (x, y, w, h) = parse_path_bounds("M0 0 Q50 100, 100 0");
        assert!(y >= -0.01 && (y + h) >= 45.0); // control point at y=100 should expand bounds
    }
    
    #[test] fn test_path_bounds_arc() {
        // Semicircle arc
        let (x, y, w, h) = parse_path_bounds("M0 50 A50 50 0 0 1 100 50");
        assert!((w - 100.0).abs() < 1.0);
        assert!(y <= 50.0 && (y + h) >= 50.0);
    }
    
    #[test] fn test_path_bounds_smooth_cubic() {
        let (x, y, w, h) = parse_path_bounds("M0 0 C10 20 20 20 30 0 S50 -20 60 0");
        assert!(x >= -0.01 && (x + w) <= 60.01);
    }
    
    #[test] fn test_path_bounds_smooth_quadratic() {
        let (x, y, w, h) = parse_path_bounds("M0 0 Q25 50 50 0 T100 0");
        assert!(x >= -0.01 && (x + w) <= 100.01);
        assert!((y + h) >= 20.0); // smooth continuation should create a curve
    }
}
