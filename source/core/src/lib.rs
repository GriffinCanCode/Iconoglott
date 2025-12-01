//! Iconoglott Core - High-performance SVG rendering engine
//!
//! Features:
//! - DSL lexer and parser (single source of truth)
//! - Stable element IDs via content-addressed hashing
//! - Incremental scene diffing with O(n) reconciliation
//! - SVG fragment memoization for render caching
//!
//! Targets:
//! - Python: `cargo build --features python` (PyO3 bindings)
//! - WASM: `wasm-pack build --features wasm` (wasm-bindgen)
//! - Bench: `cargo bench --features bench` (Criterion benchmarks)

// Core modules (always compiled)
mod hash;
mod dsl;
pub mod font;
pub mod path;

// Scene/rendering modules (python or bench feature)
#[cfg(any(feature = "python", feature = "bench"))]
pub mod scene;
#[cfg(any(feature = "python", feature = "bench"))]
pub mod render;

// TypeScript type export (test only)
#[cfg(all(test, any(feature = "python", feature = "bench")))]
mod ts_export;

// Platform bindings (WASM, etc.)
#[cfg(feature = "wasm")]
mod bindings;

// ─────────────────────────────────────────────────────────────────────────────
// Python Bindings (via PyO3)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Python module entry point
#[cfg(feature = "python")]
#[pymodule]
fn iconoglott_core(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Canvas sizing
    m.add_class::<dsl::CanvasSize>()?;
    // Lexer & Parser (core DSL processing)
    m.add_class::<dsl::TokenType>()?;
    m.add_class::<dsl::Token>()?;
    m.add_class::<dsl::Lexer>()?;
    m.add_class::<dsl::Parser>()?;
    m.add_class::<dsl::AstCanvas>()?;
    m.add_class::<dsl::AstShape>()?;
    m.add_class::<dsl::AstStyle>()?;
    m.add_class::<dsl::AstTransform>()?;
    m.add_class::<dsl::ShadowDef>()?;
    m.add_class::<dsl::GradientDef>()?;
    m.add_class::<dsl::ParseError>()?;
    // Scene & definitions
    m.add_class::<scene::Scene>()?;
    m.add_class::<scene::Gradient>()?;
    m.add_class::<scene::Filter>()?;
    // Shapes
    m.add_class::<scene::Rect>()?;
    m.add_class::<scene::Circle>()?;
    m.add_class::<scene::Ellipse>()?;
    m.add_class::<scene::Line>()?;
    m.add_class::<scene::Path>()?;
    m.add_class::<scene::Polygon>()?;
    m.add_class::<scene::Text>()?;
    m.add_class::<scene::Image>()?;
    // Utilities
    m.add_class::<scene::Style>()?;
    m.add_class::<scene::Color>()?;
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

// Core ID/hashing (always available)
pub use hash::{ContentHash, ElementId, ElementKind, Fnv1a, IdGen};

// Font metrics (always available)
pub use font::{get_metrics, measure_text, FontMetrics, TextMetrics};

// Lexer & Parser (always available) - re-export from dsl module
pub use dsl::{
    AstCanvas, AstGraph, AstNode, AstShape, AstStyle, AstTransform, CanvasSize,
    ErrorKind, ErrorSeverity, FullStyle, GradientDef, GraphEdge, GraphNode,
    Lexer, ParseError, Parser, PropValue, ShadowDef, Span,
    Token, TokenType, TokenValue,
};

// Aliased modules for compatibility
pub mod lexer { pub use crate::dsl::*; }
pub mod parser { pub use crate::dsl::*; }
pub mod id { pub use crate::hash::*; }

#[cfg(any(feature = "python", feature = "bench"))]
pub use render::{DiffOp, DiffResult, IndexedScene};

#[cfg(any(feature = "python", feature = "bench"))]
pub use scene::{
    ArrowType, Circle, Color, Diamond, Edge, EdgeStyle, Element, Ellipse,
    Filter, Gradient, GraphContainer, Image, Line, Node, Path, Polygon,
    Rect, Scene, Style, Text,
};

// Shape module alias for compatibility
#[cfg(any(feature = "python", feature = "bench"))]
pub mod shape { pub use crate::scene::*; }

// Diff module alias for compatibility  
#[cfg(any(feature = "python", feature = "bench"))]
pub mod diff { pub use crate::render::*; }
