//! Parser for the iconoglott DSL
//!
//! Parses token stream into AST with error collection and recovery.

mod ast;
mod core;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(test)]
mod tests;

// Re-export AST types
pub use ast::{
    AstCanvas, AstGraph, AstNode, AstShape, AstStyle, AstTransform,
    FullStyle, GradientDef, GraphEdge, GraphNode, ParseError, PropValue, ShadowDef,
};

// Re-export error types for diagnostics
pub use ast::{ErrorKind, ErrorSeverity, Span};

// Re-export parser
pub use self::core::Parser;

// Re-export WASM bindings
#[cfg(feature = "wasm")]
pub use wasm::{parse, parse_with_errors};

