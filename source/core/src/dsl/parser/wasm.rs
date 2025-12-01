//! WASM bindings for the parser

#![cfg(feature = "wasm")]

use super::ast::{AstNode, ParseError};
use super::core::Parser;
use super::symbols::resolve;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Parse DSL source and return AST as JSON
#[wasm_bindgen]
pub fn parse(source: &str) -> String {
    let mut lexer = super::super::lexer::Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    // Run resolution pass to resolve variable references
    let result = resolve(ast);
    serde_json::to_string(&result.ast).unwrap_or_else(|_| "null".to_string())
}

/// Parse and return errors as JSON (includes both parse and resolution errors)
#[wasm_bindgen]
pub fn parse_with_errors(source: &str) -> String {
    let mut lexer = super::super::lexer::Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    let mut errors = parser.errors;
    
    // Run resolution pass
    let result = resolve(ast);
    errors.extend(result.errors);
    
    #[derive(Serialize)]
    struct ParseResult {
        ast: AstNode,
        errors: Vec<ParseError>,
    }
    
    serde_json::to_string(&ParseResult { ast: result.ast, errors })
        .unwrap_or_else(|_| r#"{"ast":null,"errors":[]}"#.to_string())
}

