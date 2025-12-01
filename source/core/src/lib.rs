//! Iconoglott Core - Rust rendering engine with Python bindings

mod render;
mod scene;
mod shape;

use pyo3::prelude::*;
use scene::Scene;
use shape::{Circle, Rect, Text};

/// Python module entry point
#[pymodule]
fn core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Scene>()?;
    m.add_class::<Rect>()?;
    m.add_class::<Circle>()?;
    m.add_class::<Text>()?;
    Ok(())
}

