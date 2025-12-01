//! DSL lexer and parser modules

mod lexer;
mod parser;

pub use lexer::{CanvasSize, Lexer, Token, TokenType, TokenValue};
pub use parser::{
    AstCanvas, AstNode, AstShape, AstStyle, AstTransform,
    GradientDef, ParseError, Parser, PropValue, ShadowDef,
};

// Re-export WASM bindings
#[cfg(feature = "wasm")]
pub use lexer::tokenize;
#[cfg(feature = "wasm")]
pub use parser::{parse, parse_with_errors};

