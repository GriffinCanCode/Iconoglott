//! WebAssembly bindings for browser/Node.js usage
//!
//! Exposes the core rendering engine to JavaScript via wasm-bindgen.
//! TypeScript retains DSL parsing while Rust handles performance-critical:
//! - Scene diffing with stable IDs
//! - SVG fragment rendering
//! - Content-addressed hashing

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use crate::CanvasSize;

// Initialize panic hook for better error messages in WASM
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();
}

// ─────────────────────────────────────────────────────────────────────────────
// Canvas Size System
// ─────────────────────────────────────────────────────────────────────────────

/// Get pixel dimensions for a named size
/// Returns [width, height] or null if invalid
#[wasm_bindgen]
pub fn size_to_pixels(name: &str) -> JsValue {
    match CanvasSize::from_str(name) {
        Some(size) => {
            let (w, h) = size.dimensions();
            serde_wasm_bindgen::to_value(&[w, h]).unwrap_or(JsValue::NULL)
        }
        None => JsValue::NULL,
    }
}

/// Check if a size name is valid
#[wasm_bindgen]
pub fn is_valid_size(name: &str) -> bool {
    CanvasSize::from_str(name).is_some()
}

/// Get all valid size names as array
#[wasm_bindgen]
pub fn get_all_sizes() -> JsValue {
    serde_wasm_bindgen::to_value(&CanvasSize::all_names()).unwrap_or(JsValue::NULL)
}

/// Get size info as object: {name, width, height}
#[wasm_bindgen]
pub fn get_size_info(name: &str) -> JsValue {
    match CanvasSize::from_str(name) {
        Some(size) => {
            let (w, h) = size.dimensions();
            #[derive(Serialize)]
            struct SizeInfo { name: String, width: u32, height: u32 }
            serde_wasm_bindgen::to_value(&SizeInfo { name: size.to_string(), width: w, height: h })
                .unwrap_or(JsValue::NULL)
        }
        None => JsValue::NULL,
    }
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
pub fn compute_element_id(order: u32, kind: &str, key: JsValue) -> String {
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
    
    // Hash key properties (serialize to JSON for consistent hashing)
    let key_str = js_sys::JSON::stringify(&key)
        .map(|s| s.as_string().unwrap_or_default())
        .unwrap_or_default();
    for byte in key_str.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    
    format!("{:016x}", hash)
}

// ─────────────────────────────────────────────────────────────────────────────
// Style
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WasmStyle {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub corner: f32,
    pub filter: Option<String>,
}

impl WasmStyle {
    fn from_js(v: JsValue) -> Self {
        serde_wasm_bindgen::from_value(v).unwrap_or_default()
    }

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
pub fn render_rect(x: f32, y: f32, w: f32, h: f32, rx: f32, style: JsValue, transform: Option<String>) -> String {
    let style = WasmStyle::from_js(style);
    let rx_attr = if rx > 0.0 { format!(r#" rx="{}""#, rx) } else { String::new() };
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<rect x="{}" y="{}" width="{}" height="{}"{}{}{}/>"#, x, y, w, h, rx_attr, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_circle(cx: f32, cy: f32, r: f32, style: JsValue, transform: Option<String>) -> String {
    let style = WasmStyle::from_js(style);
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<circle cx="{}" cy="{}" r="{}"{}{}/>"#, cx, cy, r, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_ellipse(cx: f32, cy: f32, rx: f32, ry: f32, style: JsValue, transform: Option<String>) -> String {
    let style = WasmStyle::from_js(style);
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{}{}/>"#, cx, cy, rx, ry, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_line(x1: f32, y1: f32, x2: f32, y2: f32, stroke: &str, stroke_width: f32, transform: Option<String>) -> String {
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"{}/>"#, x1, y1, x2, y2, stroke, stroke_width, tf)
}

#[wasm_bindgen]
pub fn render_path(d: &str, style: JsValue, transform: Option<String>) -> String {
    let style = WasmStyle::from_js(style);
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<path d="{}"{}{}/>"#, d, style.to_svg_attrs(), tf)
}

#[wasm_bindgen]
pub fn render_polygon(points: JsValue, style: JsValue, transform: Option<String>) -> String {
    let points: Vec<(f32, f32)> = serde_wasm_bindgen::from_value(points).unwrap_or_default();
    let style = WasmStyle::from_js(style);
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
/// Returns {width, height, ascender, descender}
#[wasm_bindgen]
pub fn measure_text(content: &str, font: &str, size: f32) -> JsValue {
    let m = crate::font::measure_text(content, font, size);
    #[derive(Serialize)]
    struct Metrics { width: f32, height: f32, ascender: f32, descender: f32 }
    serde_wasm_bindgen::to_value(&Metrics { 
        width: m.width, height: m.height, ascender: m.ascender, descender: m.descender 
    }).unwrap_or(JsValue::NULL)
}

/// Compute text bounding box accounting for anchor position
/// Returns [x, y, width, height]
#[wasm_bindgen]
pub fn compute_text_bounds(x: f32, y: f32, content: &str, font: &str, size: f32, anchor: &str) -> JsValue {
    let m = crate::font::measure_text(content, font, size);
    let adj_x = match anchor {
        "middle" => x - m.width / 2.0,
        "end" => x - m.width,
        _ => x,
    };
    serde_wasm_bindgen::to_value(&[adj_x, y - m.ascender, m.width, m.height]).unwrap_or(JsValue::NULL)
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
    size: String,
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

/// Diff two scenes and return operations
#[wasm_bindgen]
pub fn diff_scenes(old: JsValue, new: JsValue) -> JsValue {
    let old: DiffInput = match serde_wasm_bindgen::from_value(old) {
        Ok(v) => v,
        Err(_) => return full_redraw_result(),
    };
    
    let new: DiffInput = match serde_wasm_bindgen::from_value(new) {
        Ok(v) => v,
        Err(_) => return full_redraw_result(),
    };

    // Canvas change = full redraw
    if old.canvas.size != new.canvas.size || old.canvas.fill != new.canvas.fill {
        return serde_wasm_bindgen::to_value(&DiffResult {
            ops: vec![DiffOp { op_type: "full_redraw".into(), id: None, idx: None, svg: None, from_idx: None, to_idx: None }],
            canvas_changed: true,
        }).unwrap_or_else(|_| full_redraw_result());
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

    serde_wasm_bindgen::to_value(&DiffResult { ops, canvas_changed: false })
        .unwrap_or_else(|_| full_redraw_result())
}

fn full_redraw_result() -> JsValue {
    let result = DiffResult {
        ops: vec![DiffOp { op_type: "full_redraw".into(), id: None, idx: None, svg: None, from_idx: None, to_idx: None }],
        canvas_changed: true,
    };
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Check if two scenes need any updates (fast path)
#[wasm_bindgen]
pub fn scenes_equal(old: JsValue, new: JsValue) -> bool {
    let old: Result<DiffInput, _> = serde_wasm_bindgen::from_value(old);
    let new: Result<DiffInput, _> = serde_wasm_bindgen::from_value(new);
    
    match (old, new) {
        (Ok(o), Ok(n)) => {
            o.canvas.size == n.canvas.size &&
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

/// Render complete scene SVG using standardized size
#[wasm_bindgen]
pub fn render_scene(size_name: &str, background: &str, defs: &str, elements_svg: &str) -> String {
    let (width, height) = CanvasSize::from_str(size_name)
        .map(|s| s.dimensions())
        .unwrap_or((64, 64)); // Default to medium if invalid
    
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
pub fn compute_path_bounds(d: &str) -> JsValue {
    let bounds = parse_path_bounds(d);
    serde_wasm_bindgen::to_value(&[bounds.0, bounds.1, bounds.2, bounds.3]).unwrap_or(JsValue::NULL)
}

fn parse_path_bounds(d: &str) -> (f32, f32, f32, f32) {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let (mut cur_x, mut cur_y, mut start_x, mut start_y) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    let (mut last_ctrl_x, mut last_ctrl_y) = (0.0_f32, 0.0_f32);
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
    for (p0, p1, p2, p3, is_x) in [(x0, x1, x2, x3, true), (y0, y1, y2, y3, false)] {
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
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;
    
    let lambda = (x1p / rx).powi(2) + (y1p / ry).powi(2);
    if lambda > 1.0 { let s = lambda.sqrt(); rx *= s; ry *= s; }
    
    let sq = ((rx*ry).powi(2) - (rx*y1p).powi(2) - (ry*x1p).powi(2)) / ((rx*y1p).powi(2) + (ry*x1p).powi(2));
    let coef = if large_arc != sweep { sq.max(0.0).sqrt() } else { -sq.max(0.0).sqrt() };
    let cxp = coef * rx * y1p / ry;
    let cyp = -coef * ry * x1p / rx;
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;
    
    let theta1 = ((y1p - cyp) / ry).atan2((x1p - cxp) / rx);
    let mut dtheta = (((-y1p - cyp) / ry).atan2((-x1p - cxp) / rx) - theta1).rem_euclid(std::f32::consts::TAU);
    if !sweep { dtheta -= std::f32::consts::TAU; }
    
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
    if a.abs() < 1e-10 { return if b.abs() < 1e-10 { vec![] } else { vec![-c / b] }; }
    let disc = b*b - 4.0*a*c;
    if disc < 0.0 { vec![] }
    else if disc < 1e-10 { vec![-b / (2.0 * a)] }
    else { let sq = disc.sqrt(); vec![(-b - sq) / (2.0 * a), (-b + sq) / (2.0 * a)] }
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

// ─────────────────────────────────────────────────────────────────────────────
// Graph/Flowchart Primitives
// ─────────────────────────────────────────────────────────────────────────────

/// Render a diamond shape (rotated rectangle for flowcharts)
#[wasm_bindgen]
pub fn render_diamond(cx: f32, cy: f32, w: f32, h: f32, style: JsValue, transform: Option<String>) -> String {
    let style = WasmStyle::from_js(style);
    let pts = format!("{},{} {},{} {},{} {},{}",
        cx, cy - h / 2.0, cx + w / 2.0, cy, cx, cy + h / 2.0, cx - w / 2.0, cy);
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!(r#"<polygon points="{}"{}{}/>"#, pts, style.to_svg_attrs(), tf)
}

/// Render a graph node (shape + label)
#[wasm_bindgen]
pub fn render_node(id: &str, shape: &str, cx: f32, cy: f32, w: f32, h: f32, label: Option<String>, style: JsValue) -> String {
    let style = WasmStyle::from_js(style);
    
    let shape_svg = match shape {
        "circle" => {
            let r = w.min(h) / 2.0;
            format!(r#"<circle cx="{}" cy="{}" r="{}"{}/>"#, cx, cy, r, style.to_svg_attrs())
        }
        "ellipse" => {
            format!(r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}"{}/>"#, cx, cy, w / 2.0, h / 2.0, style.to_svg_attrs())
        }
        "diamond" => {
            let pts = format!("{},{} {},{} {},{} {},{}",
                cx, cy - h / 2.0, cx + w / 2.0, cy, cx, cy + h / 2.0, cx - w / 2.0, cy);
            format!(r#"<polygon points="{}"{}/>"#, pts, style.to_svg_attrs())
        }
        _ => { // rect
            let x = cx - w / 2.0;
            let y = cy - h / 2.0;
            let rx = if style.corner > 0.0 { format!(r#" rx="{}""#, style.corner) } else { String::new() };
            format!(r#"<rect x="{}" y="{}" width="{}" height="{}"{}{}/>"#, x, y, w, h, rx, style.to_svg_attrs())
        }
    };
    
    let label_svg = label.map_or(String::new(), |lbl| {
        format!(r##"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" fill="#000">{}</text>"##, 
            cx, cy, html_escape(&lbl))
    });
    
    format!(r##"<g id="node-{}">{}{}</g>"##, html_escape(id), shape_svg, label_svg)
}

/// Render an edge (connector with optional arrow)
#[wasm_bindgen]
pub fn render_edge(from_x: f32, from_y: f32, to_x: f32, to_y: f32, edge_style: &str, arrow: &str, label: Option<String>, stroke: &str, stroke_width: f32) -> String {
    let path_d = match edge_style {
        "curved" => {
            let mx = (from_x + to_x) / 2.0;
            let my = (from_y + to_y) / 2.0;
            if (to_y - from_y).abs() > (to_x - from_x).abs() {
                format!("M{},{} C{},{} {},{} {},{}", from_x, from_y, from_x, my, to_x, my, to_x, to_y)
            } else {
                let offset = ((to_x - from_x).abs().max((to_y - from_y).abs())) * 0.3;
                format!("M{},{} C{},{} {},{} {},{}", from_x, from_y, mx, from_y + offset, mx, to_y - offset, to_x, to_y)
            }
        }
        "orthogonal" => {
            let mx = (from_x + to_x) / 2.0;
            format!("M{},{} L{},{} L{},{} L{},{}", from_x, from_y, mx, from_y, mx, to_y, to_x, to_y)
        }
        _ => format!("M{},{} L{},{}", from_x, from_y, to_x, to_y), // straight
    };
    
    let markers = match arrow {
        "forward" => r#" marker-end="url(#arrow-end)""#,
        "backward" => r#" marker-start="url(#arrow-start)""#,
        "both" => r#" marker-start="url(#arrow-start)" marker-end="url(#arrow-end)""#,
        _ => "",
    };
    
    let label_svg = label.map_or(String::new(), |lbl| {
        let mx = (from_x + to_x) / 2.0;
        let my = (from_y + to_y) / 2.0;
        format!(r##"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" font-size="12" fill="#666">{}</text>"##, 
            mx, my - 8.0, html_escape(&lbl))
    });
    
    format!(r##"<path d="{}" fill="none" stroke="{}" stroke-width="{}"{}/>{}"##, path_d, stroke, stroke_width, markers, label_svg)
}

/// Render arrow marker definitions (call once per SVG if using edges)
#[wasm_bindgen]
pub fn render_arrow_markers(color: &str) -> String {
    format!(
        r#"<marker id="arrow-start" markerWidth="10" markerHeight="7" refX="0" refY="3.5" orient="auto-start-reverse"><polygon points="10 0, 10 7, 0 3.5" fill="{color}"/></marker><marker id="arrow-end" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="{color}"/></marker>"#,
        color = color
    )
}

/// Compute best anchor points for an edge between two nodes
/// Returns {from: [x, y], to: [x, y]}
#[wasm_bindgen]
pub fn compute_edge_anchors(from_cx: f32, from_cy: f32, from_w: f32, from_h: f32, to_cx: f32, to_cy: f32, to_w: f32, to_h: f32) -> JsValue {
    #[derive(Serialize)]
    struct EdgeAnchors { from: [f32; 2], to: [f32; 2] }
    
    let dx = to_cx - from_cx;
    let dy = to_cy - from_cy;
    
    let (from_pt, to_pt) = if dy.abs() > dx.abs() {
        if dy > 0.0 {
            ([from_cx, from_cy + from_h / 2.0], [to_cx, to_cy - to_h / 2.0])
        } else {
            ([from_cx, from_cy - from_h / 2.0], [to_cx, to_cy + to_h / 2.0])
        }
    } else if dx > 0.0 {
        ([from_cx + from_w / 2.0, from_cy], [to_cx - to_w / 2.0, to_cy])
    } else {
        ([from_cx - from_w / 2.0, from_cy], [to_cx + to_w / 2.0, to_cy])
    };
    
    serde_wasm_bindgen::to_value(&EdgeAnchors { from: from_pt, to: to_pt }).unwrap_or(JsValue::NULL)
}

#[derive(Deserialize)]
struct NodeIn { id: String, w: f32, h: f32 }

#[derive(Serialize)]
struct NodeOut { id: String, cx: f32, cy: f32 }

/// Apply hierarchical layout to nodes
/// Input: array of {id, w, h}
/// Output: array of {id, cx, cy}
#[wasm_bindgen]
pub fn layout_hierarchical(nodes: JsValue, direction: &str, spacing: f32) -> JsValue {
    let nodes: Vec<NodeIn> = serde_wasm_bindgen::from_value(nodes).unwrap_or_default();
    let is_vertical = direction != "horizontal";
    
    let mut pos = spacing;
    let outputs: Vec<NodeOut> = nodes.iter().map(|n| {
        let (cx, cy) = if is_vertical {
            let cy = pos;
            pos += n.h + spacing;
            (spacing * 2.0, cy + n.h / 2.0)
        } else {
            let cx = pos;
            pos += n.w + spacing;
            (cx + n.w / 2.0, spacing * 2.0)
        };
        NodeOut { id: n.id.clone(), cx, cy }
    }).collect();
    
    serde_wasm_bindgen::to_value(&outputs).unwrap_or(JsValue::NULL)
}

/// Apply grid layout to nodes
/// Input: array of {id, w, h}
/// Output: array of {id, cx, cy}
#[wasm_bindgen]
pub fn layout_grid(nodes: JsValue, spacing: f32) -> JsValue {
    let nodes: Vec<NodeIn> = serde_wasm_bindgen::from_value(nodes).unwrap_or_default();
    if nodes.is_empty() { return serde_wasm_bindgen::to_value(&Vec::<NodeOut>::new()).unwrap_or(JsValue::NULL); }
    
    let cols = (nodes.len() as f32).sqrt().ceil() as usize;
    
    let outputs: Vec<NodeOut> = nodes.iter().enumerate().map(|(i, n)| {
        let row = i / cols;
        let col = i % cols;
        let cx = spacing + (col as f32) * (n.w + spacing) + n.w / 2.0;
        let cy = spacing + (row as f32) * (n.h + spacing) + n.h / 2.0;
        NodeOut { id: n.id.clone(), cx, cy }
    }).collect();
    
    serde_wasm_bindgen::to_value(&outputs).unwrap_or(JsValue::NULL)
}
