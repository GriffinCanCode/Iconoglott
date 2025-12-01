//! Scene graph management

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shape::{Circle, Rect, Text};

/// A renderable element in the scene
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Element {
    Rect(Rect),
    Circle(Circle),
    Text(Text),
}

impl Element {
    pub fn to_svg(&self) -> String {
        match self {
            Element::Rect(r) => r.to_svg(),
            Element::Circle(c) => c.to_svg(),
            Element::Text(t) => t.to_svg(),
        }
    }
}

/// Scene container with canvas properties
#[derive(Clone, Debug, Default)]
#[pyclass]
pub struct Scene {
    #[pyo3(get, set)]
    pub width: u32,
    #[pyo3(get, set)]
    pub height: u32,
    #[pyo3(get, set)]
    pub background: String,
    elements: Vec<Element>,
}

#[pymethods]
impl Scene {
    #[new]
    #[pyo3(signature = (width=800, height=600, background="#fff".to_string()))]
    fn new(width: u32, height: u32, background: String) -> Self {
        Self { width, height, background, elements: Vec::new() }
    }

    fn add_rect(&mut self, rect: Rect) {
        self.elements.push(Element::Rect(rect));
    }

    fn add_circle(&mut self, circle: Circle) {
        self.elements.push(Element::Circle(circle));
    }

    fn add_text(&mut self, text: Text) {
        self.elements.push(Element::Text(text));
    }

    fn clear(&mut self) {
        self.elements.clear();
    }

    fn count(&self) -> usize {
        self.elements.len()
    }

    /// Render scene to SVG
    fn to_svg(&self) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" style="background:{}">"#,
            self.width, self.height, self.background
        );
        for el in &self.elements {
            svg.push_str(&el.to_svg());
        }
        svg.push_str("</svg>");
        svg
    }

    /// Render to JSON for diffing
    fn to_json(&self) -> String {
        serde_json::json!({
            "width": self.width,
            "height": self.height,
            "background": self.background,
            "elements": self.elements.iter().map(|e| {
                match e {
                    Element::Rect(r) => serde_json::to_value(r).unwrap(),
                    Element::Circle(c) => serde_json::to_value(c).unwrap(),
                    Element::Text(t) => serde_json::to_value(t).unwrap(),
                }
            }).collect::<Vec<_>>()
        }).to_string()
    }
}

