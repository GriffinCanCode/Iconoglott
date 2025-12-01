//! Lexer for the iconoglott DSL
//!
//! Tokenizes DSL source into a stream of tokens with indentation tracking.

use lazy_static::lazy_static;
use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Token types for lexical analysis
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub enum TokenType {
    Ident,
    Number,
    Percent,     // 50%, 100%
    String,
    Color,
    Var,
    Pair,
    PercentPair, // 50%,50% or 50%x50%
    Size,
    Colon,
    Equals,
    Arrow,
    LBracket,
    RBracket,
    Newline,
    Indent,
    Dedent,
    Eof,
}

/// Standard canvas sizes (10-tier system)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub enum CanvasSize {
    Nano = 16,      // 16×16 - Favicons, tiny UI
    Micro = 24,     // 24×24 - Small UI icons
    Tiny = 32,      // 32×32 - Standard UI icons
    Small = 48,     // 48×48 - Toolbar icons
    Medium = 64,    // 64×64 - Medium icons
    Large = 96,     // 96×96 - Large display icons
    XLarge = 128,   // 128×128 - Small app icons
    Huge = 192,     // 192×192 - Touch/PWA icons
    Massive = 256,  // 256×256 - Medium app icons
    Giant = 512,    // 512×512 - Large app icons
}

impl CanvasSize {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nano" => Some(Self::Nano),
            "micro" => Some(Self::Micro),
            "tiny" => Some(Self::Tiny),
            "small" => Some(Self::Small),
            "medium" => Some(Self::Medium),
            "large" => Some(Self::Large),
            "xlarge" | "xl" => Some(Self::XLarge),
            "huge" => Some(Self::Huge),
            "massive" => Some(Self::Massive),
            "giant" => Some(Self::Giant),
            _ => None,
        }
    }
    pub fn pixels(self) -> u32 { self as u32 }
    pub fn dimensions(self) -> (u32, u32) { let p = self.pixels(); (p, p) }
    
    /// All valid size names for error messages
    pub fn all_names() -> &'static [&'static str] {
        &["nano", "micro", "tiny", "small", "medium", "large", "xlarge", "huge", "massive", "giant"]
    }
}

impl std::fmt::Display for CanvasSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Nano => "nano", Self::Micro => "micro", Self::Tiny => "tiny",
            Self::Small => "small", Self::Medium => "medium", Self::Large => "large",
            Self::XLarge => "xlarge", Self::Huge => "huge", Self::Massive => "massive",
            Self::Giant => "giant",
        };
        write!(f, "{}", name)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl CanvasSize {
    #[staticmethod]
    fn from_name(name: &str) -> Option<Self> { Self::from_str(name) }
    fn to_pixels(&self) -> u32 { self.pixels() }
    fn to_dimensions(&self) -> (u32, u32) { self.dimensions() }
    fn __repr__(&self) -> String { format!("CanvasSize.{} ({}px)", self, self.pixels()) }
}

#[cfg(feature = "python")]
#[pymethods]
impl TokenType {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

/// Token value variants
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TokenValue {
    None,
    Str(String),
    Num(f64),
    Pair(f64, f64),
    /// Percentage pair (both values are percentages 0-100)
    PercentPair(f64, f64),
}

impl Default for TokenValue {
    fn default() -> Self { Self::None }
}

/// A single token from the lexer
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Token {
    pub ttype: TokenType,
    pub value: TokenValue,
    pub line: usize,
    pub col: usize,
}

#[cfg(feature = "python")]
#[pymethods]
impl Token {
    #[getter]
    fn get_ttype(&self) -> TokenType { self.ttype }

    #[getter]
    fn get_line(&self) -> usize { self.line }

    #[getter]
    fn get_col(&self) -> usize { self.col }

    #[getter]
    fn value_str(&self) -> Option<String> {
        match &self.value {
            TokenValue::Str(s) => Some(s.clone()),
            _ => None,
        }
    }

    #[getter]
    fn value_num(&self) -> Option<f64> {
        match &self.value {
            TokenValue::Num(n) => Some(*n),
            _ => None,
        }
    }

    #[getter]
    fn value_pair(&self) -> Option<(f64, f64)> {
        match &self.value {
            TokenValue::Pair(a, b) => Some((*a, *b)),
            _ => None,
        }
    }

    /// Get value as Python object (str, float, tuple, or None)
    #[getter]
    fn value(&self, py: Python<'_>) -> PyObject {
        match &self.value {
            TokenValue::None => py.None(),
            TokenValue::Str(s) => s.clone().into_py(py),
            TokenValue::Num(n) => n.into_py(py),
            TokenValue::Pair(a, b) | TokenValue::PercentPair(a, b) => (*a, *b).into_py(py),
        }
    }

    fn __repr__(&self) -> String {
        format!("Token({:?}, {:?}, {}:{})", self.ttype, self.value, self.line, self.col)
    }
}

impl Token {
    pub fn new(ttype: TokenType, value: TokenValue, line: usize, col: usize) -> Self {
        Self { ttype, value, line, col }
    }
}

/// Pattern for token matching
struct Pattern {
    regex: Regex,
    ttype: Option<TokenType>,
}

lazy_static! {
    /// Cached lexer patterns - built once, reused across all Lexer instances
    static ref PATTERNS: Vec<Pattern> = vec![
        Pattern { regex: Regex::new(r"^//[^\n]*").unwrap(), ttype: None }, // Comments
        Pattern { regex: Regex::new(r"^\$[a-zA-Z_][a-zA-Z0-9_]*").unwrap(), ttype: Some(TokenType::Var) },
        Pattern { regex: Regex::new(r"^#[0-9a-fA-F]{3,8}\b").unwrap(), ttype: Some(TokenType::Color) },
        // Percent pairs must come before regular pairs (50%,50% or 50%x50%)
        Pattern { regex: Regex::new(r"^-?\d+\.?\d*%[,x]-?\d+\.?\d*%").unwrap(), ttype: Some(TokenType::PercentPair) },
        Pattern { regex: Regex::new(r"^-?\d+\.?\d*[,x]-?\d+\.?\d*").unwrap(), ttype: Some(TokenType::Pair) },
        // Single percentage (50%)
        Pattern { regex: Regex::new(r"^-?\d+\.?\d*%").unwrap(), ttype: Some(TokenType::Percent) },
        Pattern { regex: Regex::new(r#"^"[^"]*""#).unwrap(), ttype: Some(TokenType::String) },
        Pattern { regex: Regex::new(r"^'[^']*'").unwrap(), ttype: Some(TokenType::String) },
        Pattern { regex: Regex::new(r"^-?\d+\.?\d*").unwrap(), ttype: Some(TokenType::Number) },
        Pattern { regex: Regex::new(r"^\[").unwrap(), ttype: Some(TokenType::LBracket) },
        Pattern { regex: Regex::new(r"^\]").unwrap(), ttype: Some(TokenType::RBracket) },
        Pattern { regex: Regex::new(r"^->").unwrap(), ttype: Some(TokenType::Arrow) },
        Pattern { regex: Regex::new(r"^:").unwrap(), ttype: Some(TokenType::Colon) },
        Pattern { regex: Regex::new(r"^=").unwrap(), ttype: Some(TokenType::Equals) },
        // Size keywords before general identifiers
        Pattern { regex: Regex::new(r"^(nano|micro|tiny|small|medium|large|xlarge|xl|huge|massive|giant)\b").unwrap(), ttype: Some(TokenType::Size) },
        Pattern { regex: Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*").unwrap(), ttype: Some(TokenType::Ident) },
    ];
}

/// Lexer for tokenizing DSL source
#[cfg_attr(feature = "python", pyclass)]
pub struct Lexer {
    lines: Vec<String>,
    indent_stack: Vec<usize>,
    line_idx: usize,
}

impl Lexer {
    /// Create a new lexer for the given source
    pub fn new(source: &str) -> Self {
        Self {
            lines: source.split('\n').map(String::from).collect(),
            indent_stack: vec![0],
            line_idx: 0,
        }
    }

    /// Tokenize the source and return all tokens
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let num_lines = self.lines.len();

        for lineno in 0..num_lines {
            self.line_idx = lineno;
            // Clone the line to avoid borrowing self.lines while mutating self
            let line = self.lines[lineno].clone();
            let stripped = line.trim_start();

            // Skip empty and comment-only lines
            if stripped.is_empty() || stripped.starts_with("//") {
                continue;
            }

            let indent = line.len() - stripped.len();
            let line_len = line.len();
            tokens.extend(self.handle_indent(indent, lineno));
            tokens.extend(self.tokenize_line(stripped, lineno));
            tokens.push(Token::new(TokenType::Newline, TokenValue::Str("\n".into()), lineno, line_len));
        }

        // Close remaining indents
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::new(TokenType::Dedent, TokenValue::None, num_lines.saturating_sub(1), 0));
        }

        tokens.push(Token::new(TokenType::Eof, TokenValue::None, num_lines.saturating_sub(1), 0));
        tokens
    }

    fn handle_indent(&mut self, indent: usize, line: usize) -> Vec<Token> {
        let mut tokens = Vec::new();
        let current = *self.indent_stack.last().unwrap_or(&0);

        if indent > current {
            self.indent_stack.push(indent);
            tokens.push(Token::new(TokenType::Indent, TokenValue::None, line, 0));
        } else {
            while indent < *self.indent_stack.last().unwrap_or(&0) {
                self.indent_stack.pop();
                tokens.push(Token::new(TokenType::Dedent, TokenValue::None, line, 0));
            }
        }
        tokens
    }

    fn tokenize_line(&self, line: &str, lineno: usize) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos = 0;

        while pos < line.len() {
            let remaining = &line[pos..];

            // Skip whitespace
            if remaining.starts_with(char::is_whitespace) {
                pos += 1;
                continue;
            }

            let mut matched = false;
            for pattern in PATTERNS.iter() {
                if let Some(m) = pattern.regex.find(remaining) {
                    if let Some(ttype) = pattern.ttype {
                        let raw = m.as_str();
                        let value = Self::parse_value(raw, ttype);
                        tokens.push(Token::new(ttype, value, lineno, pos));
                    }
                    pos += m.len();
                    matched = true;
                    break;
                }
            }

            if !matched {
                pos += 1; // Skip unknown character
            }
        }
        tokens
    }

    fn parse_value(raw: &str, ttype: TokenType) -> TokenValue {
        match ttype {
            TokenType::Number => {
                if raw.contains('.') {
                    TokenValue::Num(raw.parse().unwrap_or(0.0))
                } else {
                    TokenValue::Num(raw.parse::<i64>().unwrap_or(0) as f64)
                }
            }
            TokenType::Percent => {
                // Strip % and parse as number (stored as 0-100)
                let num = raw.trim_end_matches('%');
                TokenValue::Num(num.parse().unwrap_or(0.0))
            }
            TokenType::String => {
                TokenValue::Str(raw[1..raw.len() - 1].to_string()) // Strip quotes
            }
            TokenType::Pair => {
                let sep = if raw.contains('x') { 'x' } else { ',' };
                let parts: Vec<&str> = raw.split(sep).collect();
                if parts.len() == 2 {
                    let a = parts[0].parse().unwrap_or(0.0);
                    let b = parts[1].parse().unwrap_or(0.0);
                    TokenValue::Pair(a, b)
                } else {
                    TokenValue::Pair(0.0, 0.0)
                }
            }
            TokenType::PercentPair => {
                // Parse percentage pairs (50%,50% or 50%x50%)
                let sep = if raw.contains('x') { 'x' } else { ',' };
                let parts: Vec<&str> = raw.split(sep).collect();
                if parts.len() == 2 {
                    let a = parts[0].trim_end_matches('%').parse().unwrap_or(0.0);
                    let b = parts[1].trim_end_matches('%').parse().unwrap_or(0.0);
                    TokenValue::PercentPair(a, b)
                } else {
                    TokenValue::PercentPair(0.0, 0.0)
                }
            }
            TokenType::Size => TokenValue::Str(raw.to_lowercase()),
            _ => TokenValue::Str(raw.to_string()),
        }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Lexer {
    #[new]
    fn py_new(source: &str) -> Self {
        Self::new(source)
    }

    /// Tokenize and return list of tokens
    fn py_tokenize(&mut self) -> Vec<Token> {
        self.tokenize()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WASM bindings
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn tokenize(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    serde_json::to_string(&tokens).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_empty() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].ttype, TokenType::Eof);
    }

    #[test]
    fn test_lexer_basic_rect() {
        let mut lexer = Lexer::new("rect at 100,200");
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Ident && matches!(&t.value, TokenValue::Str(s) if s == "rect")));
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Pair));
    }

    #[test]
    fn test_lexer_color() {
        let mut lexer = Lexer::new("fill #ff0000");
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Color));
    }

    #[test]
    fn test_lexer_string() {
        let mut lexer = Lexer::new(r#"text "Hello""#);
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.ttype == TokenType::String && matches!(&t.value, TokenValue::Str(s) if s == "Hello")));
    }

    #[test]
    fn test_lexer_variable() {
        let mut lexer = Lexer::new("$accent = #ff0");
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Var));
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Equals));
    }

    #[test]
    fn test_lexer_indent_dedent() {
        let mut lexer = Lexer::new("rect\n  fill #fff\ntext");
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Indent));
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Dedent));
    }

    #[test]
    fn test_lexer_comment() {
        let mut lexer = Lexer::new("// comment\nrect");
        let tokens = lexer.tokenize();
        // Comment should be skipped, first real token should be rect
        assert!(tokens.iter().any(|t| matches!(&t.value, TokenValue::Str(s) if s == "rect")));
    }

    #[test]
    fn test_lexer_number() {
        let mut lexer = Lexer::new("radius 50");
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Number && matches!(&t.value, TokenValue::Num(n) if (*n - 50.0).abs() < 0.001)));
    }

    #[test]
    fn test_lexer_negative_number() {
        let mut lexer = Lexer::new("translate -10,20");
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.ttype == TokenType::Pair && matches!(&t.value, TokenValue::Pair(a, b) if (*a + 10.0).abs() < 0.001 && (*b - 20.0).abs() < 0.001)));
    }

    #[test]
    fn test_lexer_brackets() {
        let mut lexer = Lexer::new("[100,200 300,400]");
        let tokens = lexer.tokenize();
        assert!(tokens.iter().any(|t| t.ttype == TokenType::LBracket));
        assert!(tokens.iter().any(|t| t.ttype == TokenType::RBracket));
    }
}

