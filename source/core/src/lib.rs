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

mod cache;
mod diff;
mod id;
mod render;
mod scene;
mod shape;

#[cfg(feature = "wasm")]
mod wasm;

// ─────────────────────────────────────────────────────────────────────────────
// Python Bindings (via PyO3)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use render::{compute_patches, index_scene, needs_redraw, RenderPatch};

/// Python module entry point
#[cfg(feature = "python")]
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
    // Diffing
    m.add_class::<RenderPatch>()?;
    m.add_function(wrap_pyfunction!(compute_patches, m)?)?;
    m.add_function(wrap_pyfunction!(needs_redraw, m)?)?;
    m.add_function(wrap_pyfunction!(index_scene, m)?)?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Re-exports for library consumers
// ─────────────────────────────────────────────────────────────────────────────

pub use diff::{DiffOp, DiffResult, IndexedScene};
pub use id::{ContentHash, ElementId, ElementKind, Fnv1a, IdGen};
pub use scene::{Element, Filter, Gradient, Scene};
pub use shape::{Circle, Color, Ellipse, Image, Line, Path, Polygon, Rect, Style, Text};

