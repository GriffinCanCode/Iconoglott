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
    let bounds = crate::path::parse_path_bounds(d);
    serde_wasm_bindgen::to_value(&[bounds.0, bounds.1, bounds.2, bounds.3]).unwrap_or(JsValue::NULL)
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

// ─────────────────────────────────────────────────────────────────────────────
// Symbol & Use (Component Reuse)
// ─────────────────────────────────────────────────────────────────────────────

/// Render a symbol definition (goes in <defs>)
/// content: inner SVG elements as string
/// viewbox: optional [x, y, width, height]
#[wasm_bindgen]
pub fn render_symbol(id: &str, content: &str, viewbox: JsValue) -> String {
    let vb: Option<[f32; 4]> = serde_wasm_bindgen::from_value(viewbox).ok();
    let viewbox_attr = vb.map_or(String::new(), |[x, y, w, h]| 
        format!(r#" viewBox="{} {} {} {}""#, x, y, w, h));
    format!(r#"<symbol id="{}"{}>{}</symbol>"#, html_escape(id), viewbox_attr, content)
}

/// Render a use element (references a symbol)
#[wasm_bindgen]
pub fn render_use(href: &str, x: f32, y: f32, width: JsValue, height: JsValue, style: JsValue, transform: Option<String>) -> String {
    let style = WasmStyle::from_js(style);
    let w: Option<f32> = serde_wasm_bindgen::from_value(width).ok();
    let h: Option<f32> = serde_wasm_bindgen::from_value(height).ok();
    let size = match (w, h) {
        (Some(wv), Some(hv)) => format!(r#" width="{}" height="{}""#, wv, hv),
        (Some(wv), None) => format!(r#" width="{}""#, wv),
        (None, Some(hv)) => format!(r#" height="{}""#, hv),
        _ => String::new(),
    };
    let tf = transform.map_or(String::new(), |t| format!(r#" transform="{}""#, t));
    format!("<use href=\"#{}\" x=\"{}\" y=\"{}\"{}{}{}/>" , 
        html_escape(href), x, y, size, style.to_svg_attrs(), tf)
}

// ─────────────────────────────────────────────────────────────────────────────
// Path Boolean Operations
// ─────────────────────────────────────────────────────────────────────────────

/// Boolean operation type for path operations
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum WasmBoolOp {
    Union = 0,
    Intersection = 1,
    Difference = 2,
    Xor = 3,
}

impl From<WasmBoolOp> for crate::path::BoolOp {
    fn from(op: WasmBoolOp) -> Self {
        match op {
            WasmBoolOp::Union => crate::path::BoolOp::Union,
            WasmBoolOp::Intersection => crate::path::BoolOp::Intersection,
            WasmBoolOp::Difference => crate::path::BoolOp::Difference,
            WasmBoolOp::Xor => crate::path::BoolOp::Xor,
        }
    }
}

/// Perform boolean operation on two SVG paths
/// 
/// # Arguments
/// * `path_a` - First SVG path d attribute
/// * `path_b` - Second SVG path d attribute
/// * `op` - Boolean operation (Union=0, Intersection=1, Difference=2, Xor=3)
/// * `tolerance` - Curve flattening tolerance (smaller = more accurate but slower)
/// 
/// # Returns
/// Combined SVG path d attribute string
#[wasm_bindgen]
pub fn path_boolean_op(path_a: &str, path_b: &str, op: WasmBoolOp, tolerance: f64) -> String {
    crate::path::path_boolean(path_a, path_b, op.into(), tolerance)
}

/// Perform union of two SVG paths (combine both areas)
#[wasm_bindgen]
pub fn path_union(path_a: &str, path_b: &str, tolerance: f64) -> String {
    crate::path::path_boolean(path_a, path_b, crate::path::BoolOp::Union, tolerance)
}

/// Perform intersection of two SVG paths (common area only)
#[wasm_bindgen]
pub fn path_intersection(path_a: &str, path_b: &str, tolerance: f64) -> String {
    crate::path::path_boolean(path_a, path_b, crate::path::BoolOp::Intersection, tolerance)
}

/// Perform difference of two SVG paths (A minus B)
#[wasm_bindgen]
pub fn path_difference(path_a: &str, path_b: &str, tolerance: f64) -> String {
    crate::path::path_boolean(path_a, path_b, crate::path::BoolOp::Difference, tolerance)
}

/// Perform XOR of two SVG paths (area in either but not both)
#[wasm_bindgen]
pub fn path_xor(path_a: &str, path_b: &str, tolerance: f64) -> String {
    crate::path::path_boolean(path_a, path_b, crate::path::BoolOp::Xor, tolerance)
}

/// Flatten an SVG path to line segments
/// Returns an array of [x, y] coordinates
#[wasm_bindgen]
pub fn flatten_svg_path(d: &str, tolerance: f64) -> JsValue {
    let polygon = crate::path::flatten_path(d, tolerance);
    let coords: Vec<[f64; 2]> = polygon.vertices.iter().map(|p| [p.x, p.y]).collect();
    serde_wasm_bindgen::to_value(&coords).unwrap_or(JsValue::NULL)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests (native - no JsValue)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{
        fnv1a_hash, render_line, render_text, render_linear_gradient, render_radial_gradient,
        render_shadow_filter, render_blur_filter, render_edge, render_arrow_markers, 
        render_scene, WasmStyle, html_escape,
    };
    use crate::path::parse_path_bounds;

    // ─────────────────────────────────────────────────────────────────────────
    // Hashing Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_fnv1a_deterministic() {
        let h1 = fnv1a_hash("hello");
        let h2 = fnv1a_hash("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_fnv1a_different_inputs() {
        let h1 = fnv1a_hash("rect");
        let h2 = fnv1a_hash("circle");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_fnv1a_empty_string() {
        let h = fnv1a_hash("");
        assert_eq!(h.len(), 16); // 64-bit hex = 16 chars
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Shape Rendering Tests (no JsValue)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_render_line() {
        let svg = render_line(0.0, 0.0, 100.0, 100.0, "#000", 2.0, None);
        assert!(svg.contains("<line"));
        assert!(svg.contains("stroke=\"#000\""));
        assert!(svg.contains("stroke-width=\"2\""));
    }

    #[test]
    fn test_render_text() {
        let svg = render_text(50.0, 50.0, "Hello", "Arial", 16.0, "bold", "middle", "#000", None);
        assert!(svg.contains("<text"));
        assert!(svg.contains("Hello"));
        assert!(svg.contains(r#"font-family="Arial""#));
        assert!(svg.contains(r#"font-size="16""#));
    }

    #[test]
    fn test_render_text_escapes_html() {
        let svg = render_text(0.0, 0.0, "<script>&", "Arial", 12.0, "normal", "start", "#000", None);
        assert!(svg.contains("&lt;script&gt;&amp;"));
        assert!(!svg.contains("<script>"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Gradient & Filter Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_render_linear_gradient() {
        let svg = render_linear_gradient("grad1", "#ff0000", "#0000ff", 90.0);
        assert!(svg.contains("<linearGradient"));
        assert!(svg.contains(r#"id="grad1""#));
        assert!(svg.contains("#ff0000"));
        assert!(svg.contains("#0000ff"));
    }

    #[test]
    fn test_render_radial_gradient() {
        let svg = render_radial_gradient("grad2", "#fff", "#000");
        assert!(svg.contains("<radialGradient"));
        assert!(svg.contains(r#"id="grad2""#));
    }

    #[test]
    fn test_render_shadow_filter() {
        let svg = render_shadow_filter("shadow1", 2.0, 2.0, 4.0, "#333");
        assert!(svg.contains("<filter"));
        assert!(svg.contains("<feDropShadow"));
        assert!(svg.contains(r#"dx="2""#));
    }

    #[test]
    fn test_render_blur_filter() {
        let svg = render_blur_filter("blur1", 5.0);
        assert!(svg.contains("<filter"));
        assert!(svg.contains("<feGaussianBlur"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Path Bounds Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_path_bounds_simple_rect() {
        let bounds = parse_path_bounds("M10 10 L100 10 L100 100 L10 100 Z");
        assert!((bounds.0 - 10.0).abs() < 0.1);  // min_x
        assert!((bounds.1 - 10.0).abs() < 0.1);  // min_y
        assert!((bounds.2 - 90.0).abs() < 0.1);  // width
        assert!((bounds.3 - 90.0).abs() < 0.1);  // height
    }

    #[test]
    fn test_path_bounds_relative() {
        let bounds = parse_path_bounds("m0 0 l50 0 l0 50 l-50 0 z");
        assert!((bounds.0).abs() < 0.1);
        assert!((bounds.1).abs() < 0.1);
        assert!((bounds.2 - 50.0).abs() < 0.1);
        assert!((bounds.3 - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_path_bounds_empty() {
        let bounds = parse_path_bounds("");
        assert_eq!(bounds, (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_path_bounds_horizontal_vertical() {
        let bounds = parse_path_bounds("M0 0 H100 V50");
        assert!((bounds.2 - 100.0).abs() < 0.1);
        assert!((bounds.3 - 50.0).abs() < 0.1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Graph/Edge Tests (no JsValue)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_render_edge_straight() {
        let svg = render_edge(0.0, 0.0, 100.0, 100.0, "straight", "none", None, "#333", 2.0);
        assert!(svg.contains("<path"));
        assert!(svg.contains("M0,0 L100,100"));
    }

    #[test]
    fn test_render_edge_curved() {
        let svg = render_edge(0.0, 0.0, 100.0, 0.0, "curved", "forward", None, "#333", 2.0);
        assert!(svg.contains("C"));  // Bezier curve command
        assert!(svg.contains("marker-end"));
    }

    #[test]
    fn test_render_edge_orthogonal() {
        let svg = render_edge(0.0, 0.0, 100.0, 100.0, "orthogonal", "both", Some("->".into()), "#333", 2.0);
        assert!(svg.contains("marker-start"));
        assert!(svg.contains("marker-end"));
    }

    #[test]
    fn test_render_arrow_markers() {
        let svg = render_arrow_markers("#333");
        assert!(svg.contains("<marker"));
        assert!(svg.contains(r#"id="arrow-start""#));
        assert!(svg.contains(r#"id="arrow-end""#));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Scene Rendering Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_render_scene() {
        let svg = render_scene("medium", "#fff", "", "<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\"/>");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("width=\"64\""));  // medium = 64
        assert!(svg.contains("height=\"64\""));
        assert!(svg.contains("fill=\"#fff\""));
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_render_scene_with_defs() {
        let svg = render_scene("small", "#000", "<linearGradient id=\"g1\"/>", "");
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("<linearGradient"));
        assert!(svg.contains("</defs>"));
    }

    #[test]
    fn test_render_scene_invalid_size_fallback() {
        let svg = render_scene("invalid", "#fff", "", "");
        assert!(svg.contains("width=\"64\""));  // Falls back to 64x64
    }

    #[test]
    fn test_render_scene_all_sizes() {
        for (name, expected) in [("nano", 16), ("micro", 24), ("tiny", 32), ("small", 48),
                                   ("medium", 64), ("large", 96), ("xlarge", 128), 
                                   ("huge", 192), ("massive", 256), ("giant", 512)] {
            let svg = render_scene(name, "#fff", "", "");
            assert!(svg.contains(&format!("width=\"{}\"", expected)), "Failed for size {}", name);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Style Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_wasm_style_default() {
        let style = WasmStyle::default();
        assert!(style.fill.is_none());
        assert!(style.stroke.is_none());
        assert!((style.opacity - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_wasm_style_to_svg_attrs() {
        let style = WasmStyle {
            fill: Some("#ff0".into()),
            stroke: Some("#000".into()),
            stroke_width: 2.0,
            opacity: 0.5,
            corner: 0.0,
            filter: None,
        };
        let attrs = style.to_svg_attrs();
        assert!(attrs.contains("fill=\"#ff0\""));
        assert!(attrs.contains("stroke=\"#000\""));
        assert!(attrs.contains("stroke-width=\"2\""));
        assert!(attrs.contains("opacity=\"0.5\""));
    }

    #[test]
    fn test_wasm_style_with_filter() {
        let style = WasmStyle {
            fill: None,
            stroke: None,
            stroke_width: 0.0,
            opacity: 1.0,
            corner: 0.0,
            filter: Some("shadow1".into()),
        };
        let attrs = style.to_svg_attrs();
        assert!(attrs.contains("filter=\"url(#shadow1)\""));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // HTML Escape Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("&"), "&amp;");
        assert_eq!(html_escape("<"), "&lt;");
        assert_eq!(html_escape(">"), "&gt;");
        assert_eq!(html_escape("\""), "&quot;");
        assert_eq!(html_escape("<a href=\"x\">"), "&lt;a href=&quot;x&quot;&gt;");
    }

    #[test]
    fn test_html_escape_combined() {
        assert_eq!(html_escape("<script>alert('&')</script>"), "&lt;script&gt;alert('&amp;')&lt;/script&gt;");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Path Bounds - Complex Cases
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_path_bounds_cubic_bezier() {
        // Cubic curve that goes beyond endpoints
        let bounds = parse_path_bounds("M0 50 C0 0, 100 0, 100 50");
        assert!(bounds.1 < 50.0); // Should have min_y above the endpoints
    }

    #[test]
    fn test_path_bounds_quadratic_bezier() {
        let bounds = parse_path_bounds("M0 0 Q50 100, 100 0");
        assert!(bounds.3 > 0.0); // Height should account for control point influence
    }

    #[test]
    fn test_path_bounds_arc() {
        let bounds = parse_path_bounds("M0 50 A50 50 0 0 1 100 50");
        assert!(bounds.2 >= 100.0); // Width should be at least 100
    }

    #[test]
    fn test_path_bounds_smooth_cubic() {
        let bounds = parse_path_bounds("M0 0 C10 20 20 20 30 0 S50 -20 60 0");
        assert!(bounds.3 > 0.0); // Should have height from curves
    }
}
