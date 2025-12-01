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

    /// Parse hex color (#RRGGBB or #RGB)
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
}

#[pymethods]
impl Style {
    #[new]
    #[pyo3(signature = (fill=None, stroke=None, stroke_width=1.0))]
    fn new(fill: Option<String>, stroke: Option<String>, stroke_width: f32) -> Self {
        Self { fill, stroke, stroke_width }
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
    pub style: Style,
}

#[pymethods]
impl Rect {
    #[new]
    #[pyo3(signature = (x, y, w, h, style=None))]
    fn new(x: f32, y: f32, w: f32, h: f32, style: Option<Style>) -> Self {
        Self { x, y, w, h, style: style.unwrap_or_default() }
    }

    fn to_svg(&self) -> String {
        let mut attrs = format!(r#"x="{}" y="{}" width="{}" height="{}""#, self.x, self.y, self.w, self.h);
        if let Some(ref fill) = self.style.fill {
            attrs.push_str(&format!(r#" fill="{}""#, fill));
        }
        if let Some(ref stroke) = self.style.stroke {
            attrs.push_str(&format!(r#" stroke="{}" stroke-width="{}""#, stroke, self.style.stroke_width));
        }
        format!("<rect {}/>", attrs)
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

    fn to_svg(&self) -> String {
        let mut attrs = format!(r#"cx="{}" cy="{}" r="{}""#, self.cx, self.cy, self.r);
        if let Some(ref fill) = self.style.fill {
            attrs.push_str(&format!(r#" fill="{}""#, fill));
        }
        if let Some(ref stroke) = self.style.stroke {
            attrs.push_str(&format!(r#" stroke="{}" stroke-width="{}""#, stroke, self.style.stroke_width));
        }
        format!("<circle {}/>", attrs)
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
    pub style: Style,
}

#[pymethods]
impl Text {
    #[new]
    #[pyo3(signature = (x, y, content, font="sans-serif".to_string(), size=16.0, style=None))]
    fn new(x: f32, y: f32, content: String, font: String, size: f32, style: Option<Style>) -> Self {
        Self { x, y, content, font, size, style: style.unwrap_or_default() }
    }

    fn to_svg(&self) -> String {
        let fill = self.style.fill.as_deref().unwrap_or("#000");
        format!(
            r#"<text x="{}" y="{}" font-family="{}" font-size="{}" fill="{}">{}</text>"#,
            self.x, self.y, self.font, self.size, fill, self.content
        )
    }
}

