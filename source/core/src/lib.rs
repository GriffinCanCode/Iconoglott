//! Iconoglott Core - Rust rendering engine with Python bindings

mod render;
mod scene;
mod shape;

use pyo3::prelude::*;
use scene::{Filter, Gradient, Scene};
use shape::{Circle, Color, Ellipse, Image, Line, Path, Polygon, Rect, Style, Text};

/// Python module entry point
#[pymodule]
fn iconoglott_core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Scene & definitions
    m.add_class::<Scene>()?;
    m.add_class::<Gradient>()?;
    m.add_class::<Filter>()?;
    // Shapes
    m.add_class::<Rect>()?;
    m.add_class::<Circle>()?;
    m.add_class::<Ellipse>()?;
    m.add_class::<Line>()?;
    m.add_class::<Path>()?;
    m.add_class::<Polygon>()?;
    m.add_class::<Text>()?;
    m.add_class::<Image>()?;
    // Utilities
    m.add_class::<Style>()?;
    m.add_class::<Color>()?;
    Ok(())
}

