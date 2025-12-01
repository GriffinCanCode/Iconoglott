//! DSL lexer and parser modules

mod lexer;
mod parser;

pub use lexer::{CanvasSize, Lexer, Token, TokenType, TokenValue};
pub use parser::{
    AstCanvas, AstGraph, AstNode, AstShape, AstStyle, AstTransform,
    ErrorKind, ErrorSeverity, FullStyle, GradientDef, GraphEdge, GraphNode,
    ParseError, Parser, PropValue, ShadowDef, Span,
};

// Re-export WASM bindings
#[cfg(feature = "wasm")]
pub use lexer::tokenize;
#[cfg(feature = "wasm")]
pub use parser::{parse, parse_with_errors};

