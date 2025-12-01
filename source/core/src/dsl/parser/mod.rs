//! Parser for the iconoglott DSL
//!
//! Parses token stream into AST with error collection and recovery.
//! Variable resolution is done in a separate pass via the symbols module.
//! Layout resolution is done via the layout module.
//! Animation primitives via the anim module.

mod anim;
mod ast;
mod core;
mod layout;
mod symbols;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod proptest_tests;

// Re-export AST types
pub use ast::{
    AstCanvas, AstGraph, AstNode, AstShape, AstStyle, AstTransform, AstSymbol, AstUse,
    FullStyle, GradientDef, GraphEdge, GraphNode, ParseError, PropValue, ShadowDef,
};

// Re-export dimension and layout types (allow unused - used externally)
#[allow(unused_imports)]
pub use ast::{
    Dimension, DimensionPair, JustifyContent, AlignItems, 
    Constraint, Edge, Axis, LayoutProps,
};

// Re-export error types for diagnostics
pub use ast::{ErrorKind, ErrorSeverity, Span};

// Re-export parser
pub use self::core::Parser;

// Re-export symbol table and resolution
#[allow(unused_imports)] // Public API for external use
pub use symbols::{resolve, Scope, Symbol, SymbolTable, ResolveResult};

// Re-export layout solver (allow unused - used externally)
#[allow(unused_imports)]
pub use layout::{LayoutSolver, LayoutContext, LayoutRect, resolve_layout};

// Re-export animation primitives
pub use anim::{
    Animation, AnimationState, AnimatableProperty, Direction, Duration,
    Easing, FillMode, Interpolation, Iteration, Keyframes, KeyframeStep,
    PlayState, StepPosition, Transition,
};

// Re-export WASM bindings
#[cfg(feature = "wasm")]
pub use wasm::{parse, parse_with_errors};

