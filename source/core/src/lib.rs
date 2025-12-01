//! Iconoglott Core - High-performance SVG rendering engine
//!
//! Features:
//! - Stable element IDs via content-addressed hashing
//! - Incremental scene diffing with O(n) reconciliation
//! - SVG fragment memoization for render caching
//!
//! Targets:
//! - Python: `cargo build --features python` (PyO3 bindings)
//! - WASM: `wasm-pack build --features wasm` (wasm-bindgen)

// Core modules (always compiled)
mod id;

// Python-specific modules (only with python feature)
#[cfg(feature = "python")]
mod cache;
#[cfg(feature = "python")]
mod diff;
#[cfg(feature = "python")]
mod render;
#[cfg(feature = "python")]
mod scene;
#[cfg(feature = "python")]
mod shape;

// WASM module (only with wasm feature)
#[cfg(feature = "wasm")]
mod wasm;

// ─────────────────────────────────────────────────────────────────────────────
// Python Bindings (via PyO3)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Python module entry point
#[cfg(feature = "python")]
#[pymodule]
fn iconoglott_core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Scene & definitions
    m.add_class::<scene::Scene>()?;
    m.add_class::<scene::Gradient>()?;
    m.add_class::<scene::Filter>()?;
    // Shapes
    m.add_class::<shape::Rect>()?;
    m.add_class::<shape::Circle>()?;
    m.add_class::<shape::Ellipse>()?;
    m.add_class::<shape::Line>()?;
    m.add_class::<shape::Path>()?;
    m.add_class::<shape::Polygon>()?;
    m.add_class::<shape::Text>()?;
    m.add_class::<shape::Image>()?;
    // Utilities
    m.add_class::<shape::Style>()?;
    m.add_class::<shape::Color>()?;
    // Diffing
    m.add_class::<render::RenderPatch>()?;
    m.add_function(wrap_pyfunction!(render::compute_patches, m)?)?;
    m.add_function(wrap_pyfunction!(render::needs_redraw, m)?)?;
    m.add_function(wrap_pyfunction!(render::index_scene, m)?)?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Re-exports for library consumers
// ─────────────────────────────────────────────────────────────────────────────

pub use id::{ContentHash, ElementId, ElementKind, Fnv1a, IdGen};

#[cfg(feature = "python")]
pub use diff::{DiffOp, DiffResult, IndexedScene};

#[cfg(feature = "python")]
pub use scene::{Element, Filter, Gradient, Scene};

#[cfg(feature = "python")]
pub use shape::{Circle, Color, Ellipse, Image, Line, Path, Polygon, Rect, Style, Text};
