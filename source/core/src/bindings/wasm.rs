//! WebAssembly bindings for browser/Node.js usage
//!
//! Exposes the core rendering engine to JavaScript via wasm-bindgen.
//! TypeScript retains DSL parsing while Rust handles performance-critical:
//! - Scene diffing with stable IDs
//! - SVG fragment rendering
//! - Content-addressed hashing

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Initialize panic hook for better error messages in WASM
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();
}

// ─────────────────────────────────────────────────────────────────────────────
// Hashing (FNV-1a)
// ─────────────────────────────────────────────────────────────────────────────

const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// Compute FNV-1a hash of string data
#[wasm_bindgen]
pub fn fnv1a_hash(data: &str) -> String {
    let mut hash = FNV_OFFSET;
    for byte in data.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{:016x}", hash)
}

/// Compute stable element ID from order, kind, and key properties
#[wasm_bindgen]
pub fn compute_element_id(order: u32, kind: &str, key_json: &str) -> String {
    let mut hash = FNV_OFFSET;
    
    // Hash order
    for byte in order.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    
    // Hash kind
    for byte in kind.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    
    // Hash key properties
    for byte in key_json.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    
    format!("{:016x}", hash)
}

// ─────────────────────────────────────────────────────────────────────────────
// Style
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmStyle {
    fill: Option<String>,
    stroke: Option<String>,
    stroke_width: f32,
    opacity: f32,
    corner: f32,
    filter: Option<String>,
}

#[wasm_bindgen]
impl WasmStyle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { fill: None, stroke: None, stroke_width: 1.0, opacity: 1.0, corner: 0.0, filter: None }
    }

    #[wasm_bindgen(setter)]
    pub fn set_fill(&mut self, v: Option<String>) { self.fill = v; }
    
    #[wasm_bindgen(setter)]
    pub fn set_stroke(&mut self, v: Option<String>) { self.stroke = v; }
    
    #[wasm_bindgen(setter)]
    pub fn set_stroke_width(&mut self, v: f32) { self.stroke_width = v; }
    
    #[wasm_bindgen(setter)]
    pub fn set_opacity(&mut self, v: f32) { self.opacity = v; }
    
    #[wasm_bindgen(setter)]
    pub fn set_corner(&mut self, v: f32) { self.corner = v; }
    
    #[wasm_bindgen(setter)]
    pub fn set_filter(&mut self, v: Option<String>) { self.filter = v; }

    fn to_svg_attrs(&self) -> String {
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

// ─────────────────────────────────────────────────────────────────────────────
// Shape Primitives
// ─────────────────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn render_rect(x: f32, y: f32, w: f32, h: f32, rx: f32, style_json: &str, transform: Option<String>) -> String {
    let style: WasmStyle = serde_json::from_str(style_json).unwrap_or_default();
    let rx_attr = if rx > 0.0 { format!(r#" rx="{}""#, rx) } else { String::new() };
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<rect x="{}" y="{}" width="{}" height="{}"{}{}{}/>"#, x, y, w, h, rx_attr, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_circle(cx: f32, cy: f32, r: f32, style_json: &str, transform: Option<String>) -> String {
    let style: WasmStyle = serde_json::from_str(style_json).unwrap_or_default();
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<circle cx="{}" cy="{}" r="{}"{}{}/>"#, cx, cy, r, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_ellipse(cx: f32, cy: f32, rx: f32, ry: f32, style_json: &str, transform: Option<String>) -> String {
    let style: WasmStyle = serde_json::from_str(style_json).unwrap_or_default();
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{}{}/>"#, cx, cy, rx, ry, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_line(x1: f32, y1: f32, x2: f32, y2: f32, stroke: &str, stroke_width: f32, transform: Option<String>) -> String {
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"{}/>"#, x1, y1, x2, y2, stroke, stroke_width, tf)
}

#[wasm_bindgen]
pub fn render_path(d: &str, style_json: &str, transform: Option<String>) -> String {
    let style: WasmStyle = serde_json::from_str(style_json).unwrap_or_default();
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<path d="{}"{}{}/>"#, d, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_polygon(points_json: &str, style_json: &str, transform: Option<String>) -> String {
    let points: Vec<(f32, f32)> = serde_json::from_str(points_json).unwrap_or_default();
    let style: WasmStyle = serde_json::from_str(style_json).unwrap_or_default();
    let pts: String = points.iter().map(|(x, y)| format!("{},{}", x, y)).collect::<Vec<_>>().join(" ");
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<polygon points="{}"{}{}/>"#, pts, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_text(x: f32, y: f32, content: &str, font: &str, size: f32, weight: &str, anchor: &str, fill: &str, transform: Option<String>) -> String {
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    let escaped = html_escape(content);
    format!(
        r#"<text x="{}" y="{}" font-family="{}" font-size="{}" font-weight="{}" text-anchor="{}" fill="{}"{}>{}</text>"#,
        x, y, font, size, weight, anchor, fill, tf, escaped
    )
}

/// Measure text dimensions using font metrics
/// Returns JSON: {"width": f32, "height": f32, "ascender": f32, "descender": f32}
#[wasm_bindgen]
pub fn measure_text(content: &str, font: &str, size: f32) -> String {
    let m = crate::font::measure_text(content, font, size);
    format!(r#"{{"width":{},"height":{},"ascender":{},"descender":{}}}"#, 
        m.width, m.height, m.ascender, m.descender)
}

/// Compute text bounding box accounting for anchor position
/// Returns JSON: [x, y, width, height]
#[wasm_bindgen]
pub fn compute_text_bounds(x: f32, y: f32, content: &str, font: &str, size: f32, anchor: &str) -> String {
    let m = crate::font::measure_text(content, font, size);
    let adj_x = match anchor {
        "middle" => x - m.width / 2.0,
        "end" => x - m.width,
        _ => x,
    };
    format!("[{},{},{},{}]", adj_x, y - m.ascender, m.width, m.height)
}

#[wasm_bindgen]
pub fn render_image(x: f32, y: f32, w: f32, h: f32, href: &str, transform: Option<String>) -> String {
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<image x="{}" y="{}" width="{}" height="{}" href="{}"{}/>"#, x, y, w, h, html_escape(href), tf)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

// ─────────────────────────────────────────────────────────────────────────────
// Gradient & Filter Definitions
// ─────────────────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn render_linear_gradient(id: &str, from_color: &str, to_color: &str, angle: f32) -> String {
    let rad = (angle - 90.0_f32).to_radians();
    let x2 = 50.0 + 50.0 * rad.cos();
    let y2 = 50.0 + 50.0 * rad.sin();
    format!(
        r#"<linearGradient id="{}" x1="0%" y1="0%" x2="{:.1}%" y2="{:.1}%"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="{}"/></linearGradient>"#,
        id, x2, y2, from_color, to_color
    )
}

#[wasm_bindgen]
pub fn render_radial_gradient(id: &str, from_color: &str, to_color: &str) -> String {
    format!(
        r#"<radialGradient id="{}"><stop offset="0%" stop-color="{}"/><stop offset="100%" stop-color="{}"/></radialGradient>"#,
        id, from_color, to_color
    )
}

#[wasm_bindgen]
pub fn render_shadow_filter(id: &str, dx: f32, dy: f32, blur: f32, color: &str) -> String {
    format!(
        r#"<filter id="{}" x="-50%" y="-50%" width="200%" height="200%"><feDropShadow dx="{}" dy="{}" stdDeviation="{}" flood-color="{}"/></filter>"#,
        id, dx, dy, blur, color
    )
}

#[wasm_bindgen]
pub fn render_blur_filter(id: &str, blur: f32) -> String {
    format!(r#"<filter id="{}"><feGaussianBlur stdDeviation="{}"/></filter>"#, id, blur)
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene Diffing
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct DiffInput {
    canvas: CanvasInput,
    elements: Vec<ElementInput>,
    defs: String,
}

#[derive(Serialize, Deserialize)]
struct CanvasInput {
    width: u32,
    height: u32,
    fill: String,
}

#[derive(Serialize, Deserialize)]
struct ElementInput {
    id: String,
    kind: String,
    svg: String,
}

#[derive(Serialize, Deserialize)]
struct DiffOp {
    #[serde(rename = "type")]
    op_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idx: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    svg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_idx: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_idx: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct DiffResult {
    ops: Vec<DiffOp>,
    canvas_changed: bool,
}

/// Diff two scenes and return JSON array of operations
#[wasm_bindgen]
pub fn diff_scenes(old_json: &str, new_json: &str) -> String {
    let old: DiffInput = match serde_json::from_str(old_json) {
        Ok(v) => v,
        Err(_) => return r#"{"ops":[{"type":"full_redraw"}],"canvas_changed":true}"#.to_string(),
    };
    
    let new: DiffInput = match serde_json::from_str(new_json) {
        Ok(v) => v,
        Err(_) => return r#"{"ops":[{"type":"full_redraw"}],"canvas_changed":true}"#.to_string(),
    };

    // Canvas change = full redraw
    if old.canvas.width != new.canvas.width || 
       old.canvas.height != new.canvas.height || 
       old.canvas.fill != new.canvas.fill {
        return serde_json::to_string(&DiffResult {
            ops: vec![DiffOp { op_type: "full_redraw".into(), id: None, idx: None, svg: None, from_idx: None, to_idx: None }],
            canvas_changed: true,
        }).unwrap_or_default();
    }

    // Build old index
    let mut old_map: std::collections::HashMap<&str, (usize, &str)> = std::collections::HashMap::new();
    for (i, el) in old.elements.iter().enumerate() {
        old_map.insert(&el.id, (i, &el.svg));
    }

    let mut ops = Vec::new();
    let mut matched = vec![false; old.elements.len()];

    // Pass 1: Match new to old
    for (new_idx, new_el) in new.elements.iter().enumerate() {
        if let Some(&(old_idx, old_svg)) = old_map.get(new_el.id.as_str()) {
            matched[old_idx] = true;
            
            // Content changed
            if old_svg != new_el.svg {
                ops.push(DiffOp {
                    op_type: "update".into(),
                    id: Some(new_el.id.clone()),
                    idx: Some(new_idx),
                    svg: Some(new_el.svg.clone()),
                    from_idx: None,
                    to_idx: None,
                });
            }
            
            // Position changed
            if old_idx != new_idx {
                ops.push(DiffOp {
                    op_type: "move".into(),
                    id: Some(new_el.id.clone()),
                    idx: None,
                    svg: None,
                    from_idx: Some(old_idx),
                    to_idx: Some(new_idx),
                });
            }
        } else {
            // New element
            ops.push(DiffOp {
                op_type: "add".into(),
                id: Some(new_el.id.clone()),
                idx: Some(new_idx),
                svg: Some(new_el.svg.clone()),
                from_idx: None,
                to_idx: None,
            });
        }
    }

    // Pass 2: Remove unmatched (reverse for stable indices)
    for (i, was_matched) in matched.iter().enumerate().rev() {
        if !was_matched {
            ops.push(DiffOp {
                op_type: "remove".into(),
                id: Some(old.elements[i].id.clone()),
                idx: Some(i),
                svg: None,
                from_idx: None,
                to_idx: None,
            });
        }
    }

    // Defs changed
    if old.defs != new.defs {
        ops.push(DiffOp {
            op_type: "update_defs".into(),
            id: None,
            idx: None,
            svg: Some(new.defs),
            from_idx: None,
            to_idx: None,
        });
    }

    serde_json::to_string(&DiffResult { ops, canvas_changed: false }).unwrap_or_default()
}

/// Check if two scenes need any updates (fast path)
#[wasm_bindgen]
pub fn scenes_equal(old_json: &str, new_json: &str) -> bool {
    let old: Result<DiffInput, _> = serde_json::from_str(old_json);
    let new: Result<DiffInput, _> = serde_json::from_str(new_json);
    
    match (old, new) {
        (Ok(o), Ok(n)) => {
            o.canvas.width == n.canvas.width &&
            o.canvas.height == n.canvas.height &&
            o.canvas.fill == n.canvas.fill &&
            o.elements.len() == n.elements.len() &&
            o.defs == n.defs &&
            o.elements.iter().zip(n.elements.iter()).all(|(a, b)| a.id == b.id && a.svg == b.svg)
        }
        _ => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene Rendering (Full SVG output)
// ─────────────────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn render_scene(width: u32, height: u32, background: &str, defs: &str, elements_svg: &str) -> String {
    let mut svg = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#, width, height);
    svg.push_str(&format!(r#"<rect width="100%" height="100%" fill="{}"/>"#, background));
    
    if !defs.is_empty() {
        svg.push_str("<defs>");
        svg.push_str(defs);
        svg.push_str("</defs>");
    }
    
    svg.push_str(elements_svg);
    svg.push_str("</svg>");
    svg
}

// ─────────────────────────────────────────────────────────────────────────────
// Path Bounds Calculation
// ─────────────────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn compute_path_bounds(d: &str) -> String {
    let bounds = parse_path_bounds(d);
    serde_json::to_string(&bounds).unwrap_or_else(|_| "[0,0,0,0]".to_string())
}

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
    let cmds: Vec<char> = d.chars().filter(|c| matches!(c, 'M'|'m'|'L'|'l'|'H'|'h'|'V'|'v'|'C'|'c'|'S'|'s'|'Q'|'q'|'T'|'t'|'A'|'a'|'Z'|'z')).collect();
    let mut idx = 0;

    for cmd in cmds {
        match cmd {
            'M' if idx + 1 < nums.len() => {
                cur_x = nums[idx]; cur_y = nums[idx + 1];
                start_x = cur_x; start_y = cur_y;
                track(cur_x, cur_y); idx += 2;
            }
            'm' if idx + 1 < nums.len() => {
                cur_x += nums[idx]; cur_y += nums[idx + 1];
                start_x = cur_x; start_y = cur_y;
                track(cur_x, cur_y); idx += 2;
            }
            'L' if idx + 1 < nums.len() => {
                cur_x = nums[idx]; cur_y = nums[idx + 1];
                track(cur_x, cur_y); idx += 2;
            }
            'l' if idx + 1 < nums.len() => {
                cur_x += nums[idx]; cur_y += nums[idx + 1];
                track(cur_x, cur_y); idx += 2;
            }
            'H' if idx < nums.len() => { cur_x = nums[idx]; track(cur_x, cur_y); idx += 1; }
            'h' if idx < nums.len() => { cur_x += nums[idx]; track(cur_x, cur_y); idx += 1; }
            'V' if idx < nums.len() => { cur_y = nums[idx]; track(cur_x, cur_y); idx += 1; }
            'v' if idx < nums.len() => { cur_y += nums[idx]; track(cur_x, cur_y); idx += 1; }
            'C' if idx + 5 < nums.len() => {
                for i in (0..6).step_by(2) { track(nums[idx + i], nums[idx + i + 1]); }
                cur_x = nums[idx + 4]; cur_y = nums[idx + 5]; idx += 6;
            }
            'c' if idx + 5 < nums.len() => {
                for i in (0..6).step_by(2) { track(cur_x + nums[idx + i], cur_y + nums[idx + i + 1]); }
                cur_x += nums[idx + 4]; cur_y += nums[idx + 5]; idx += 6;
            }
            'Q' if idx + 3 < nums.len() => {
                track(nums[idx], nums[idx + 1]); track(nums[idx + 2], nums[idx + 3]);
                cur_x = nums[idx + 2]; cur_y = nums[idx + 3]; idx += 4;
            }
            'q' if idx + 3 < nums.len() => {
                track(cur_x + nums[idx], cur_y + nums[idx + 1]);
                cur_x += nums[idx + 2]; cur_y += nums[idx + 3];
                track(cur_x, cur_y); idx += 4;
            }
            'A' if idx + 6 < nums.len() => {
                let rx = nums[idx].abs(); let ry = nums[idx + 1].abs();
                track(cur_x - rx, cur_y - ry); track(cur_x + rx, cur_y + ry);
                cur_x = nums[idx + 5]; cur_y = nums[idx + 6];
                track(cur_x, cur_y); idx += 7;
            }
            'a' if idx + 6 < nums.len() => {
                let rx = nums[idx].abs(); let ry = nums[idx + 1].abs();
                track(cur_x - rx, cur_y - ry); track(cur_x + rx, cur_y + ry);
                cur_x += nums[idx + 5]; cur_y += nums[idx + 6];
                track(cur_x, cur_y); idx += 7;
            }
            'Z' | 'z' => { cur_x = start_x; cur_y = start_y; }
            _ => {}
        }
    }

    if min_x == f32::MAX { (0.0, 0.0, 0.0, 0.0) }
    else { (min_x, min_y, max_x - min_x, max_y - min_y) }
}

fn extract_numbers(d: &str) -> Vec<f32> {
    let mut nums = Vec::new();
    let mut buf = String::new();
    
    for c in d.chars() {
        if c.is_ascii_digit() || c == '.' || (c == '-' && buf.is_empty()) || (c == '-' && buf.ends_with('e')) {
            buf.push(c);
        } else if c == 'e' || c == 'E' {
            buf.push('e');
        } else {
            if !buf.is_empty() {
                if let Ok(n) = buf.parse::<f32>() { nums.push(n); }
                buf.clear();
            }
            if c == '-' { buf.push(c); }
        }
    }
    if !buf.is_empty() {
        if let Ok(n) = buf.parse::<f32>() { nums.push(n); }
    }
    nums
}

