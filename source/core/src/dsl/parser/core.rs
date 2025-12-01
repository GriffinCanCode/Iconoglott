//! Parser for the iconoglott DSL
//!
//! Parses token stream into AST with error collection and recovery.
//! Uses synchronization tokens (Newline, Dedent) for error recovery.

use super::ast::*;
use super::super::lexer::{CanvasSize, Token, TokenType, TokenValue};
use std::collections::{HashMap, HashSet};

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Synchronization tokens for error recovery (Newline, Dedent, EOF)
pub const SYNC_TOKENS: &[TokenType] = &[TokenType::Newline, TokenType::Dedent, TokenType::Eof];

/// Statement-starting tokens (used to find recovery points)
pub const STMT_STARTERS: &[TokenType] = &[TokenType::Ident, TokenType::Var];

// ─────────────────────────────────────────────────────────────────────────────
// Parser Constants
// ─────────────────────────────────────────────────────────────────────────────

lazy_static::lazy_static! {
    pub(crate) static ref SHAPES: HashSet<&'static str> = {
        ["rect", "circle", "ellipse", "line", "path", "polygon", "text", "image", "arc", "curve", "diamond"]
            .into_iter().collect()
    };
    pub(crate) static ref STYLE_PROPS: HashSet<&'static str> = {
        ["fill", "stroke", "opacity", "corner", "shadow", "gradient", "blur"]
            .into_iter().collect()
    };
    pub(crate) static ref TEXT_PROPS: HashSet<&'static str> = {
        ["font", "bold", "italic", "center", "middle", "end"]
            .into_iter().collect()
    };
    pub(crate) static ref TRANSFORM_PROPS: HashSet<&'static str> = {
        ["translate", "rotate", "scale", "origin"]
            .into_iter().collect()
    };
    pub(crate) static ref LAYOUT_PROPS: HashSet<&'static str> = {
        ["gap", "padding", "justify", "align", "wrap", "width", "height", "size", "anchor", "fill-parent", "center-in"]
            .into_iter().collect()
    };
    pub(crate) static ref JUSTIFY_VALUES: HashSet<&'static str> = {
        ["start", "end", "center", "space-between", "space-around", "space-evenly"]
            .into_iter().collect()
    };
    pub(crate) static ref ALIGN_VALUES: HashSet<&'static str> = {
        ["start", "end", "center", "stretch", "baseline"]
            .into_iter().collect()
    };
    pub(crate) static ref NODE_SHAPES: HashSet<&'static str> = {
        ["rect", "circle", "ellipse", "diamond"]
            .into_iter().collect()
    };
    pub(crate) static ref EDGE_STYLES: HashSet<&'static str> = {
        ["straight", "curved", "orthogonal"]
            .into_iter().collect()
    };
    pub(crate) static ref ARROW_TYPES: HashSet<&'static str> = {
        ["none", "forward", "backward", "both"]
            .into_iter().collect()
    };
    pub(crate) static ref GRAPH_LAYOUTS: HashSet<&'static str> = {
        ["hierarchical", "force", "grid", "tree", "manual"]
            .into_iter().collect()
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────────────

/// Parser for the iconoglott DSL
#[cfg_attr(feature = "python", pyclass)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    pub(crate) variables: HashMap<String, TokenValue>,
    pub errors: Vec<ParseError>,
    /// Track indent depth for recovery
    indent_depth: usize,
    /// Panic mode flag - true when recovering from error
    panic_mode: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            variables: HashMap::new(),
            errors: Vec::new(),
            indent_depth: 0,
            panic_mode: false,
        }
    }

    pub(crate) fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub(crate) fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    #[allow(dead_code)]
    pub(crate) fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }

    pub(crate) fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if let Some(t) = tok {
            match t.ttype {
                TokenType::Indent => self.indent_depth += 1,
                TokenType::Dedent => self.indent_depth = self.indent_depth.saturating_sub(1),
                _ => {}
            }
        }
        self.pos += 1;
        tok
    }

    pub(crate) fn matches(&self, types: &[TokenType]) -> bool {
        self.current().map(|t| types.contains(&t.ttype)).unwrap_or(false)
    }

    pub(crate) fn skip_newlines(&mut self) {
        while self.matches(&[TokenType::Newline]) {
            self.advance();
        }
    }

    /// Resolve a token value, returning VarRef for unresolved variables.
    /// Final resolution happens in the symbol table pass.
    pub(crate) fn resolve(&self, tok: &Token) -> TokenValue {
        if tok.ttype == TokenType::Var {
            if let TokenValue::Str(name) = &tok.value {
                // Check local scope first (for backward compatibility in same-block vars)
                if let Some(val) = self.variables.get(name) {
                    return val.clone();
                }
                // Return as unresolved - will be resolved in symbol pass
                return TokenValue::Str(format!("$VAR:{}", name));
            }
        }
        tok.value.clone()
    }

    /// Create a VarRef PropValue for deferred resolution
    #[allow(dead_code)] // Available for future use in property parsing
    pub(crate) fn var_ref(&self, tok: &Token) -> PropValue {
        if let TokenValue::Str(name) = &tok.value {
            let name = name.strip_prefix('$').unwrap_or(name);
            PropValue::VarRef(name.to_string(), tok.line, tok.col)
        } else {
            PropValue::None
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Error Handling & Recovery
    // ─────────────────────────────────────────────────────────────────────────

    /// Record an error at current position
    #[allow(dead_code)]
    pub(crate) fn error(&mut self, msg: &str) {
        self.error_at_current(msg, ErrorKind::UnexpectedToken, None);
    }

    /// Record error with full details at current token
    fn error_at_current(&mut self, msg: &str, kind: ErrorKind, suggestion: Option<&str>) {
        if self.panic_mode { return; } // Suppress cascade errors
        
        let (line, col) = self.current().map(|t| (t.line, t.col)).unwrap_or((0, 0));
        let mut err = ParseError::new(msg, kind, line, col);
        if let Some(s) = suggestion { err = err.with_suggestion(s); }
        self.errors.push(err);
    }

    /// Record error and immediately synchronize to recover
    fn error_and_sync(&mut self, msg: &str, kind: ErrorKind, suggestion: Option<&str>) {
        self.error_at_current(msg, kind, suggestion);
        self.synchronize();
    }

    /// Enter panic mode and skip to next synchronization point
    fn synchronize(&mut self) {
        self.panic_mode = true;
        
        // Skip to next newline/dedent at current indent level or statement starter
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Eof { break; }
            
            // Found sync token - exit panic mode after it
            if SYNC_TOKENS.contains(&tok.ttype) {
                if tok.ttype == TokenType::Newline {
                    self.advance();
                    self.skip_newlines();
                    
                    // Check if next token could start a statement
                    if self.matches(STMT_STARTERS) {
                        self.panic_mode = false;
                        return;
                    }
                } else if tok.ttype == TokenType::Dedent {
                    // Don't consume dedent - let caller handle block end
                    self.panic_mode = false;
                    return;
                }
            }
            self.advance();
        }
        self.panic_mode = false;
    }

    /// Synchronize to end of current block (consume until dedent or eof)
    #[allow(dead_code)]
    fn sync_to_block_end(&mut self) {
        while let Some(tok) = self.current() {
            match tok.ttype {
                TokenType::Dedent => { self.advance(); break; }
                TokenType::Eof => break,
                _ => { self.advance(); }
            }
        }
    }

    /// Synchronize to end of current line
    fn sync_to_line_end(&mut self) {
        while let Some(tok) = self.current() {
            if matches!(tok.ttype, TokenType::Newline | TokenType::Eof | TokenType::Dedent) { break; }
            self.advance();
        }
    }

    /// Try to consume expected token, record error if not found
    #[allow(dead_code)]
    fn expect(&mut self, expected: TokenType, msg: &str) -> bool {
        if self.matches(&[expected]) {
            self.advance();
            true
        } else {
            self.error_at_current(msg, ErrorKind::MissingToken, None);
            false
        }
    }

    /// Check if errors occurred
    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }

    /// Get error count
    pub fn error_count(&self) -> usize { self.errors.len() }

    /// Parse the token stream into an AST
    pub fn parse(&mut self) -> AstNode {
        let mut children = Vec::new();
        self.skip_newlines();

        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Eof {
                break;
            }
            if let Some(node) = self.parse_statement() {
                children.push(node);
            }
            self.skip_newlines();
        }

        AstNode::Scene(children)
    }

    pub(crate) fn parse_statement(&mut self) -> Option<AstNode> {
        let tok = self.current()?;

        if tok.ttype == TokenType::Var {
            return self.parse_variable();
        }

        // Handle unexpected token at statement start
        if tok.ttype != TokenType::Ident {
            let ttype = tok.ttype;
            self.error_and_sync(
                &format!("Expected command, found {:?}", ttype),
                ErrorKind::UnexpectedToken,
                Some("Statements should start with a command like 'rect', 'circle', 'canvas', etc.")
            );
            return None;
        }

        let cmd = match &tok.value {
            TokenValue::Str(s) => s.clone(),
            _ => {
                self.error_and_sync("Invalid command token", ErrorKind::UnexpectedToken, None);
                return None;
            }
        };
        self.advance();

        match cmd.as_str() {
            "canvas" => Some(self.parse_canvas()),
            "group" => Some(self.parse_group()),
            "stack" | "row" => Some(self.parse_layout(&cmd)),
            "graph" => Some(self.parse_graph()),
            "node" => Some(AstNode::Shape(self.parse_node_as_shape())),
            "edge" => Some(AstNode::Shape(self.parse_edge_as_shape())),
            "symbol" => Some(self.parse_symbol()),
            "use" => Some(self.parse_use()),
            _ if SHAPES.contains(cmd.as_str()) => Some(self.parse_shape(&cmd)),
            _ => {
                // Unknown command - suggest similar valid commands
                let suggestion = Self::suggest_command(&cmd);
                self.error_at_current(
                    &format!("Unknown command: '{}'", cmd),
                    ErrorKind::UnknownCommand,
                    suggestion.as_deref()
                );
                self.sync_to_line_end();
                None
            }
        }
    }

    /// Suggest similar valid commands for typos
    fn suggest_command(cmd: &str) -> Option<String> {
        let all_cmds = ["canvas", "group", "stack", "row", "graph", "node", "edge",
                        "symbol", "use", "rect", "circle", "ellipse", "line", "path", 
                        "polygon", "text", "image", "arc", "curve", "diamond"];
        
        // Simple Levenshtein-style matching for common typos
        let cmd_lower = cmd.to_lowercase();
        for valid in all_cmds {
            if cmd_lower.starts_with(&valid[..1.min(valid.len())]) && 
               (cmd_lower.len() as i32 - valid.len() as i32).abs() <= 2 {
                return Some(format!("Did you mean '{}'?", valid));
            }
        }
        
        // Check for prefix matches
        for valid in all_cmds {
            if valid.starts_with(&cmd_lower) || cmd_lower.starts_with(valid) {
                return Some(format!("Did you mean '{}'?", valid));
            }
        }
        
        Some(format!("Valid commands: {}", all_cmds[..8].join(", ")))
    }

    fn parse_variable(&mut self) -> Option<AstNode> {
        let name_tok = self.advance()?;
        let name = match &name_tok.value {
            TokenValue::Str(s) => s.clone(),
            _ => return None,
        };

        if self.matches(&[TokenType::Equals]) {
            self.advance();
            if let Some(val_tok) = self.current() {
                if !self.matches(&[TokenType::Newline, TokenType::Eof]) {
                    self.variables.insert(name.clone(), val_tok.value.clone());
                    self.advance();
                }
            }
        }

        Some(AstNode::Variable {
            name: name.clone(),
            value: self.variables.get(&name).cloned(),
        })
    }

    fn parse_canvas(&mut self) -> AstNode {
        let mut canvas = AstCanvas::default();

        // Parse size (required - must be a valid size keyword)
        if self.matches(&[TokenType::Size]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(name) = tok.value.clone() {
                    if let Some(size) = CanvasSize::from_str(&name) {
                        canvas.size = size;
                    } else {
                        self.error_at_current(
                            &format!("Invalid canvas size '{}'", name),
                            ErrorKind::InvalidValue,
                            Some(&format!("Valid sizes: {}", CanvasSize::all_names().join(", ")))
                        );
                    }
                }
            }
        } else if self.matches(&[TokenType::Pair]) {
            // Legacy support: emit error for raw dimensions but continue
            self.error_at_current(
                "Raw pixel dimensions not allowed",
                ErrorKind::InvalidValue,
                Some(&format!("Use a standard size: {}", CanvasSize::all_names().join(", ")))
            );
            self.advance(); // consume and recover
        } else if !self.matches(&[TokenType::Ident, TokenType::Newline, TokenType::Eof]) {
            self.error_at_current(
                "Expected canvas size",
                ErrorKind::MissingToken,
                Some(&format!("Valid sizes: {}", CanvasSize::all_names().join(", ")))
            );
        }

        // Parse canvas properties with recovery
        while self.matches(&[TokenType::Ident, TokenType::Size]) {
            let prop = self.current().and_then(|t| match &t.value {
                TokenValue::Str(s) => Some(s.clone()),
                _ => None,
            });
            self.advance();

            match prop.as_deref() {
                Some("fill") => {
                    if self.matches(&[TokenType::Color, TokenType::Var, TokenType::Ident]) {
                        if let Some(tok) = self.current() {
                            if let TokenValue::Str(s) = self.resolve(tok) {
                                canvas.fill = s;
                            }
                            self.advance();
                        }
                    } else {
                        self.error_at_current(
                            "Expected color value after 'fill'",
                            ErrorKind::InvalidValue,
                            Some("Use a hex color like #fff or #1a2b3c")
                        );
                    }
                }
                Some(p) => {
                    self.error_at_current(
                        &format!("Unknown canvas property '{}'", p),
                        ErrorKind::InvalidProperty,
                        Some("Valid canvas properties: fill")
                    );
                    self.sync_to_line_end();
                }
                None => {}
            }
        }

        AstNode::Canvas(canvas)
    }

    fn parse_group(&mut self) -> AstNode {
        let mut shape = AstShape::new("group");

        if self.matches(&[TokenType::String]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(name) = &tok.value {
                    shape.props.insert("name".into(), PropValue::Str(name.clone()));
                }
            }
        }

        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_block(&mut shape);
        }

        AstNode::Shape(shape)
    }

    fn parse_layout(&mut self, kind: &str) -> AstNode {
        use super::ast::{Dimension, JustifyContent, AlignItems, LayoutProps};
        
        let mut shape = AstShape::new("layout");
        let direction = if kind == "stack" { "vertical" } else { "horizontal" };
        shape.props.insert("direction".into(), PropValue::Str(direction.into()));
        
        // Initialize layout props with defaults
        let mut layout = LayoutProps::default();
        layout.direction = Some(direction.into());

        while let Some(tok) = self.current() {
            if self.matches(&[TokenType::Newline, TokenType::Eof]) { break; }
            
            match tok.ttype {
                TokenType::Ident => {
                    let prop = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };
                    self.advance();

                    match prop.as_str() {
                        "vertical" | "horizontal" => {
                            layout.direction = Some(prop.clone());
                            shape.props.insert("direction".into(), PropValue::Str(prop));
                        }
                        "gap" => {
                            layout.gap = self.parse_dimension_value();
                            if let Dimension::Px(n) = layout.gap {
                                shape.props.insert("gap".into(), PropValue::Num(n));
                            }
                        }
                        "justify" => {
                            layout.justify = self.parse_justify_content();
                            shape.props.insert("justify".into(), PropValue::Str(format!("{:?}", layout.justify).to_lowercase()));
                        }
                        "align" => {
                            layout.align = self.parse_align_items();
                            shape.props.insert("align".into(), PropValue::Str(format!("{:?}", layout.align).to_lowercase()));
                        }
                        "wrap" => {
                            layout.wrap = true;
                            shape.props.insert("wrap".into(), PropValue::Num(1.0));
                        }
                        "at" => {
                            if self.matches(&[TokenType::Pair]) {
                                if let Some(t) = self.advance() {
                                    if let TokenValue::Pair(a, b) = t.value {
                                        shape.props.insert("at".into(), PropValue::Pair(a, b));
                                    }
                                }
                            } else if self.matches(&[TokenType::PercentPair]) {
                                if let Some(t) = self.advance() {
                                    if let TokenValue::PercentPair(a, b) = t.value {
                                        shape.props.insert("at".into(), PropValue::PercentPair(a, b));
                                    }
                                }
                            }
                        }
                        "size" => {
                            let dim_pair = self.parse_dimension_pair();
                            shape.props.insert("size".into(), PropValue::DimPair(dim_pair));
                        }
                        "width" => {
                            let dim = self.parse_dimension_value();
                            shape.props.insert("width".into(), PropValue::Dim(dim));
                        }
                        "height" => {
                            let dim = self.parse_dimension_value();
                            shape.props.insert("height".into(), PropValue::Dim(dim));
                        }
                        "padding" => {
                            layout.padding = Some(self.parse_padding());
                        }
                        "center" => {
                            // Shorthand: center = justify center + align center
                            layout.justify = JustifyContent::Center;
                            layout.align = AlignItems::Center;
                            shape.props.insert("justify".into(), PropValue::Str("center".into()));
                            shape.props.insert("align".into(), PropValue::Str("center".into()));
                        }
                        _ => {}
                    }
                }
                TokenType::Number => {
                    // Bare number is gap
                    if let TokenValue::Num(n) = tok.value {
                        layout.gap = Dimension::Px(n);
                        shape.props.insert("gap".into(), PropValue::Num(n));
                    }
                    self.advance();
                }
                TokenType::Percent => {
                    // Bare percentage for gap
                    if let TokenValue::Num(n) = tok.value {
                        layout.gap = Dimension::Percent(n);
                        shape.props.insert("gap".into(), PropValue::Dim(Dimension::Percent(n)));
                    }
                    self.advance();
                }
                _ => { self.advance(); }
            }
        }

        // Store full layout props
        shape.props.insert("_layout".into(), PropValue::Layout(Box::new(layout)));

        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_layout_block(&mut shape);
        }

        AstNode::Shape(shape)
    }
    
    /// Parse a dimension value (number, percentage, or 'auto')
    fn parse_dimension_value(&mut self) -> Dimension {
        use super::ast::Dimension;
        
        if let Some(tok) = self.current() {
            match tok.ttype {
                TokenType::Number => {
                    if let TokenValue::Num(n) = tok.value {
                        self.advance();
                        return Dimension::Px(n);
                    }
                }
                TokenType::Percent => {
                    if let TokenValue::Num(n) = tok.value {
                        self.advance();
                        return Dimension::Percent(n);
                    }
                }
                TokenType::Ident => {
                    if let TokenValue::Str(s) = &tok.value {
                        if s == "auto" {
                            self.advance();
                            return Dimension::Auto;
                        }
                    }
                }
                _ => {}
            }
        }
        Dimension::Auto
    }
    
    /// Parse a dimension pair for width/height
    fn parse_dimension_pair(&mut self) -> DimensionPair {
        use super::ast::{Dimension, DimensionPair};
        
        if let Some(tok) = self.current() {
            match tok.ttype {
                TokenType::Pair => {
                    if let TokenValue::Pair(w, h) = tok.value {
                        self.advance();
                        return DimensionPair { width: Dimension::Px(w), height: Dimension::Px(h) };
                    }
                }
                TokenType::PercentPair => {
                    if let TokenValue::PercentPair(w, h) = tok.value {
                        self.advance();
                        return DimensionPair { width: Dimension::Percent(w), height: Dimension::Percent(h) };
                    }
                }
                TokenType::Ident if matches!(&tok.value, TokenValue::Str(s) if s == "auto") => {
                    self.advance();
                    return DimensionPair { width: Dimension::Auto, height: Dimension::Auto };
                }
                _ => {}
            }
        }
        DimensionPair::default()
    }
    
    /// Parse justify-content value
    fn parse_justify_content(&mut self) -> JustifyContent {
        use super::ast::JustifyContent;
        
        if self.matches(&[TokenType::Ident]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(s) = &tok.value {
                    return match s.as_str() {
                        "start" => JustifyContent::Start,
                        "end" => JustifyContent::End,
                        "center" => JustifyContent::Center,
                        "space-between" => JustifyContent::SpaceBetween,
                        "space-around" => JustifyContent::SpaceAround,
                        "space-evenly" => JustifyContent::SpaceEvenly,
                        _ => JustifyContent::Start,
                    };
                }
            }
        }
        JustifyContent::Start
    }
    
    /// Parse align-items value
    fn parse_align_items(&mut self) -> AlignItems {
        use super::ast::AlignItems;
        
        if self.matches(&[TokenType::Ident]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(s) = &tok.value {
                    return match s.as_str() {
                        "start" => AlignItems::Start,
                        "end" => AlignItems::End,
                        "center" => AlignItems::Center,
                        "stretch" => AlignItems::Stretch,
                        "baseline" => AlignItems::Baseline,
                        _ => AlignItems::Start,
                    };
                }
            }
        }
        AlignItems::Start
    }
    
    /// Parse padding values (1, 2, or 4 values)
    fn parse_padding(&mut self) -> (Dimension, Dimension, Dimension, Dimension) {
        use super::ast::Dimension;
        
        let mut values = Vec::new();
        
        // Collect up to 4 dimension values
        while values.len() < 4 && self.matches(&[TokenType::Number, TokenType::Percent]) {
            values.push(self.parse_dimension_value());
        }
        
        match values.len() {
            1 => (values[0].clone(), values[0].clone(), values[0].clone(), values[0].clone()),
            2 => (values[0].clone(), values[1].clone(), values[0].clone(), values[1].clone()),
            4 => (values[0].clone(), values[1].clone(), values[2].clone(), values[3].clone()),
            _ => (Dimension::Px(0.0), Dimension::Px(0.0), Dimension::Px(0.0), Dimension::Px(0.0)),
        }
    }
    
    /// Parse layout block (like parse_block but with layout-specific handling)
    fn parse_layout_block(&mut self, shape: &mut AstShape) {
        #![allow(unused_imports)]
        use super::ast::Dimension;
        
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Dedent {
                self.advance();
                break;
            }
            if tok.ttype == TokenType::Eof {
                self.error_at_current("Unexpected end of file in layout block", ErrorKind::UnterminatedBlock, None);
                break;
            }

            self.skip_newlines();
            if self.matches(&[TokenType::Dedent]) {
                self.advance();
                break;
            }

            if let Some(tok) = self.current() {
                if tok.ttype == TokenType::Ident {
                    let prop = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };

                    // Check for nested shapes
                    if SHAPES.contains(prop.as_str()) || prop == "stack" || prop == "row" {
                        match self.parse_statement() {
                            Some(AstNode::Shape(mut child)) => {
                                // Check for child layout constraints
                                self.apply_child_layout_props(&mut child);
                                shape.children.push(child);
                            }
                            _ => {}
                        }
                    } else if LAYOUT_PROPS.contains(prop.as_str()) {
                        self.parse_layout_prop(shape);
                    } else if STYLE_PROPS.contains(prop.as_str()) {
                        self.parse_style_prop(shape);
                    } else if TEXT_PROPS.contains(prop.as_str()) {
                        self.parse_text_prop(&mut shape.style);
                    } else if TRANSFORM_PROPS.contains(prop.as_str()) {
                        self.parse_transform_prop(&mut shape.transform);
                    } else {
                        self.error_at_current(
                            &format!("Unknown property '{}' in layout block", prop),
                            ErrorKind::InvalidProperty,
                            Some("Valid layout properties: gap, justify, align, padding, wrap, width, height")
                        );
                        self.advance();
                        self.sync_to_line_end();
                    }
                } else {
                    self.advance();
                }
            }
        }
    }
    
    /// Parse a layout-specific property
    fn parse_layout_prop(&mut self, shape: &mut AstShape) {
        use super::ast::Dimension;
        
        let prop = match self.advance().and_then(|t| match &t.value {
            TokenValue::Str(s) => Some(s.clone()),
            _ => None,
        }) {
            Some(p) => p,
            None => return,
        };

        match prop.as_str() {
            "gap" => {
                let dim = self.parse_dimension_value();
                if let Dimension::Px(n) = dim {
                    shape.props.insert("gap".into(), PropValue::Num(n));
                } else {
                    shape.props.insert("gap".into(), PropValue::Dim(dim));
                }
            }
            "justify" => {
                let val = self.parse_justify_content();
                shape.props.insert("justify".into(), PropValue::Str(format!("{:?}", val).to_lowercase()));
            }
            "align" => {
                let val = self.parse_align_items();
                shape.props.insert("align".into(), PropValue::Str(format!("{:?}", val).to_lowercase()));
            }
            "width" => {
                let dim = self.parse_dimension_value();
                shape.props.insert("width".into(), PropValue::Dim(dim));
            }
            "height" => {
                let dim = self.parse_dimension_value();
                shape.props.insert("height".into(), PropValue::Dim(dim));
            }
            "size" => {
                let dim_pair = self.parse_dimension_pair();
                shape.props.insert("size".into(), PropValue::DimPair(dim_pair));
            }
            "padding" => {
                let padding = self.parse_padding();
                // Store as prop for now (serialization-friendly)
                if let (Dimension::Px(t), Dimension::Px(r), Dimension::Px(b), Dimension::Px(l)) = &padding {
                    shape.props.insert("padding".into(), PropValue::Points(vec![(*t, *r), (*b, *l)]));
                }
            }
            "wrap" => {
                shape.props.insert("wrap".into(), PropValue::Num(1.0));
            }
            "fill-parent" => {
                // Shorthand for width 100%, height 100%
                shape.props.insert("width".into(), PropValue::Dim(Dimension::Percent(100.0)));
                shape.props.insert("height".into(), PropValue::Dim(Dimension::Percent(100.0)));
            }
            "center-in" => {
                // Constraint: center in parent
                shape.props.insert("_center_x".into(), PropValue::Num(1.0));
                shape.props.insert("_center_y".into(), PropValue::Num(1.0));
            }
            "anchor" => {
                // Parse anchor constraint: anchor top 10 or anchor left 20%
                if self.matches(&[TokenType::Ident]) {
                    let edge = self.advance().and_then(|t| match &t.value {
                        TokenValue::Str(s) => Some(s.clone()),
                        _ => None,
                    });
                    if let Some(edge) = edge {
                        let offset = self.parse_dimension_value();
                        shape.props.insert(format!("_anchor_{}", edge), PropValue::Dim(offset));
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Apply layout-specific properties to child shapes
    fn apply_child_layout_props(&mut self, _child: &mut AstShape) {
        // Child layout properties like flex-grow, align-self can be handled here
        // For now, this is a placeholder for future extension
    }

    fn parse_graph(&mut self) -> AstNode {
        let mut graph = AstGraph::default();

        // Parse inline graph properties
        while self.matches(&[TokenType::Ident]) {
            let prop = self.current().and_then(|t| match &t.value {
                TokenValue::Str(s) => Some(s.clone()),
                _ => None,
            });
            self.advance();

            match prop.as_deref() {
                Some(p) if GRAPH_LAYOUTS.contains(p) => graph.layout = p.to_string(),
                Some("vertical") | Some("horizontal") => {
                    if let Some(p) = prop { graph.direction = p; }
                }
                Some("spacing") if self.matches(&[TokenType::Number]) => {
                    if let Some(tok) = self.advance() {
                        if let TokenValue::Num(n) = tok.value { graph.spacing = n; }
                    }
                }
                _ => {}
            }
        }

        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_graph_block(&mut graph);
        }

        AstNode::Graph(graph)
    }

    fn parse_graph_block(&mut self, graph: &mut AstGraph) {
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Dedent {
                self.advance();
                break;
            }
            if tok.ttype == TokenType::Eof {
                self.error_at_current("Unexpected end of file in graph block", ErrorKind::UnterminatedBlock, None);
                break;
            }

            self.skip_newlines();
            if self.matches(&[TokenType::Dedent]) {
                self.advance();
                break;
            }

            if let Some(tok) = self.current() {
                if tok.ttype == TokenType::Ident {
                    let cmd = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };
                    self.advance();

                    match cmd.as_str() {
                        "node" => graph.nodes.push(self.parse_graph_node()),
                        "edge" => graph.edges.push(self.parse_graph_edge()),
                        "layout" => {
                            if self.matches(&[TokenType::Ident]) {
                                let layout_val = self.advance().and_then(|t| {
                                    if let TokenValue::Str(s) = &t.value { Some(s.clone()) } else { None }
                                });
                                if let Some(s) = layout_val {
                                    if GRAPH_LAYOUTS.contains(s.as_str()) {
                                        graph.layout = s;
                                    } else {
                                        self.error_at_current(
                                            &format!("Invalid layout '{}'", s),
                                            ErrorKind::InvalidValue,
                                            Some(&format!("Valid layouts: {}", GRAPH_LAYOUTS.iter().copied().collect::<Vec<_>>().join(", ")))
                                        );
                                    }
                                }
                            } else {
                                self.error_at_current("Expected layout name", ErrorKind::MissingToken, None);
                            }
                        }
                        "direction" => {
                            if self.matches(&[TokenType::Ident]) {
                                let dir_val = self.advance().and_then(|t| {
                                    if let TokenValue::Str(s) = &t.value { Some(s.clone()) } else { None }
                                });
                                if let Some(s) = dir_val {
                                    if s == "vertical" || s == "horizontal" {
                                        graph.direction = s;
                                    } else {
                                        self.error_at_current(
                                            &format!("Invalid direction '{}'", s),
                                            ErrorKind::InvalidValue,
                                            Some("Use 'vertical' or 'horizontal'")
                                        );
                                    }
                                }
                            } else {
                                self.error_at_current("Expected direction value", ErrorKind::MissingToken, None);
                            }
                        }
                        "spacing" => {
                            if self.matches(&[TokenType::Number]) {
                                if let Some(t) = self.advance() {
                                    if let TokenValue::Num(n) = t.value { graph.spacing = n; }
                                }
                            } else {
                                self.error_at_current("Expected number after 'spacing'", ErrorKind::MissingToken, None);
                            }
                        }
                        _ => {
                            self.error_at_current(
                                &format!("Unknown graph property '{}'", cmd),
                                ErrorKind::InvalidProperty,
                                Some("Valid graph properties: node, edge, layout, direction, spacing")
                            );
                            self.sync_to_line_end();
                        }
                    }
                } else {
                    self.error_at_current(
                        &format!("Unexpected {:?} in graph block", tok.ttype),
                        ErrorKind::UnexpectedToken,
                        None
                    );
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn parse_graph_node(&mut self) -> GraphNode {
        let mut node = GraphNode::default();

        // First token should be the ID (string)
        if self.matches(&[TokenType::String]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(s) = &tok.value { node.id = s.clone(); }
            }
        }

        // Parse inline properties
        while let Some(tok) = self.current() {
            if self.matches(&[TokenType::Newline, TokenType::Eof]) { break; }

            match tok.ttype {
                TokenType::Pair => {
                    if let TokenValue::Pair(a, b) = self.advance().map(|t| &t.value).unwrap() {
                        if node.at.is_none() { node.at = Some((*a, *b)); }
                        else if node.size.is_none() { node.size = Some((*a, *b)); }
                    }
                }
                TokenType::Color | TokenType::Var => {
                    let val = self.resolve(tok);
                    self.advance();
                    if let TokenValue::Str(s) = val { node.style.fill = Some(s); }
                }
                TokenType::Ident => {
                    let key = match &tok.value { TokenValue::Str(s) => s.clone(), _ => { self.advance(); continue; } };
                    self.advance();

                    match key.as_str() {
                        "at" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value { node.at = Some((a, b)); }
                            }
                        }
                        "size" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value { node.size = Some((a, b)); }
                            }
                        }
                        "shape" if self.matches(&[TokenType::Ident]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value {
                                    if NODE_SHAPES.contains(s.as_str()) { node.shape = s.clone(); }
                                }
                            }
                        }
                        "label" if self.matches(&[TokenType::String]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value { node.label = Some(s.clone()); }
                            }
                        }
                        _ => {}
                    }
                }
                _ => { self.advance(); }
            }
        }

        // Parse block if present
        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_node_block(&mut node);
        }

        node
    }

    fn parse_node_block(&mut self, node: &mut GraphNode) {
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Dedent { self.advance(); break; }

            self.skip_newlines();
            if self.matches(&[TokenType::Dedent]) { self.advance(); break; }

            if let Some(tok) = self.current() {
                if tok.ttype == TokenType::Ident {
                    let prop = match &tok.value { TokenValue::Str(s) => s.clone(), _ => { self.advance(); continue; } };
                    self.advance();

                    match prop.as_str() {
                        "shape" if self.matches(&[TokenType::Ident]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value {
                                    if NODE_SHAPES.contains(s.as_str()) { node.shape = s.clone(); }
                                }
                            }
                        }
                        "label" if self.matches(&[TokenType::String]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value { node.label = Some(s.clone()); }
                            }
                        }
                        "fill" if self.matches(&[TokenType::Color, TokenType::Var]) => {
                            if let Some(t) = self.current() {
                                if let TokenValue::Str(s) = self.resolve(t) { node.style.fill = Some(s); }
                                self.advance();
                            }
                        }
                        "stroke" if self.matches(&[TokenType::Color, TokenType::Var]) => {
                            if let Some(t) = self.current() {
                                if let TokenValue::Str(s) = self.resolve(t) { node.style.stroke = Some(s); }
                                self.advance();
                            }
                        }
                        _ => {}
                    }
                } else {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn parse_graph_edge(&mut self) -> GraphEdge {
        let mut edge = GraphEdge::default();

        // Parse: "from" -> "to"
        if self.matches(&[TokenType::String]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(s) = &tok.value { edge.from = s.clone(); }
            }
        }

        // Expect arrow
        if self.matches(&[TokenType::Arrow]) { self.advance(); }

        if self.matches(&[TokenType::String]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(s) = &tok.value { edge.to = s.clone(); }
            }
        }

        // Parse inline properties
        while let Some(tok) = self.current() {
            if self.matches(&[TokenType::Newline, TokenType::Eof]) { break; }

            match tok.ttype {
                TokenType::Color | TokenType::Var => {
                    let val = self.resolve(tok);
                    self.advance();
                    if let TokenValue::Str(s) = val { edge.stroke = Some(s); }
                }
                TokenType::Number => {
                    if let TokenValue::Num(n) = tok.value { edge.stroke_width = n; }
                    self.advance();
                }
                TokenType::Ident => {
                    let key = match &tok.value { TokenValue::Str(s) => s.clone(), _ => { self.advance(); continue; } };
                    self.advance();

                    match key.as_str() {
                        "style" if self.matches(&[TokenType::Ident]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value {
                                    if EDGE_STYLES.contains(s.as_str()) { edge.style = s.clone(); }
                                }
                            }
                        }
                        "arrow" if self.matches(&[TokenType::Ident]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value {
                                    if ARROW_TYPES.contains(s.as_str()) { edge.arrow = s.clone(); }
                                }
                            }
                        }
                        "label" if self.matches(&[TokenType::String]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value { edge.label = Some(s.clone()); }
                            }
                        }
                        "stroke" if self.matches(&[TokenType::Color, TokenType::Var]) => {
                            if let Some(t) = self.current() {
                                if let TokenValue::Str(s) = self.resolve(t) { edge.stroke = Some(s); }
                                self.advance();
                            }
                        }
                        k if EDGE_STYLES.contains(k) => edge.style = k.to_string(),
                        k if ARROW_TYPES.contains(k) => edge.arrow = k.to_string(),
                        _ => {}
                    }
                }
                _ => { self.advance(); }
            }
        }

        // Parse block if present
        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_edge_block(&mut edge);
        }

        edge
    }

    fn parse_edge_block(&mut self, edge: &mut GraphEdge) {
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Dedent { self.advance(); break; }

            self.skip_newlines();
            if self.matches(&[TokenType::Dedent]) { self.advance(); break; }

            if let Some(tok) = self.current() {
                if tok.ttype == TokenType::Ident {
                    let prop = match &tok.value { TokenValue::Str(s) => s.clone(), _ => { self.advance(); continue; } };
                    self.advance();

                    match prop.as_str() {
                        "style" if self.matches(&[TokenType::Ident]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value {
                                    if EDGE_STYLES.contains(s.as_str()) { edge.style = s.clone(); }
                                }
                            }
                        }
                        "arrow" if self.matches(&[TokenType::Ident]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value {
                                    if ARROW_TYPES.contains(s.as_str()) { edge.arrow = s.clone(); }
                                }
                            }
                        }
                        "label" if self.matches(&[TokenType::String]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value { edge.label = Some(s.clone()); }
                            }
                        }
                        "stroke" if self.matches(&[TokenType::Color, TokenType::Var]) => {
                            if let Some(t) = self.current() {
                                if let TokenValue::Str(s) = self.resolve(t) { edge.stroke = Some(s); }
                                self.advance();
                            }
                        }
                        k if EDGE_STYLES.contains(k) => edge.style = k.to_string(),
                        k if ARROW_TYPES.contains(k) => edge.arrow = k.to_string(),
                        _ => {}
                    }
                } else {
                    self.advance();
                }
            }
        }
    }

    /// Parse standalone node (outside graph) - returns as shape with special kind
    fn parse_node_as_shape(&mut self) -> AstShape {
        let node = self.parse_graph_node();
        let mut shape = AstShape::new("node");
        shape.props.insert("id".into(), PropValue::Str(node.id));
        shape.props.insert("shape".into(), PropValue::Str(node.shape));
        if let Some(label) = node.label { shape.props.insert("label".into(), PropValue::Str(label)); }
        if let Some((x, y)) = node.at { shape.props.insert("at".into(), PropValue::Pair(x, y)); }
        if let Some((w, h)) = node.size { shape.props.insert("size".into(), PropValue::Pair(w, h)); }
        shape.style = node.style;
        shape
    }

    /// Parse standalone edge (outside graph) - returns as shape with special kind
    fn parse_edge_as_shape(&mut self) -> AstShape {
        let edge = self.parse_graph_edge();
        let mut shape = AstShape::new("edge");
        shape.props.insert("from".into(), PropValue::Str(edge.from));
        shape.props.insert("to".into(), PropValue::Str(edge.to));
        shape.props.insert("style".into(), PropValue::Str(edge.style));
        shape.props.insert("arrow".into(), PropValue::Str(edge.arrow));
        if let Some(label) = edge.label { shape.props.insert("label".into(), PropValue::Str(label)); }
        if let Some(stroke) = edge.stroke { shape.style.stroke = Some(stroke); }
        shape.style.stroke_width = edge.stroke_width;
        shape
    }

    /// Parse symbol definition for component reuse (SVG <symbol>)
    fn parse_symbol(&mut self) -> AstNode {
        use super::ast::AstSymbol;
        let mut symbol = AstSymbol::default();

        // Parse symbol ID (required)
        if self.matches(&[TokenType::String]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(s) = &tok.value { symbol.id = s.clone(); }
            }
        } else {
            self.error_at_current("Expected symbol ID (string)", ErrorKind::MissingToken, Some("symbol \"my-icon\""));
        }

        // Parse optional viewbox
        while let Some(tok) = self.current() {
            if self.matches(&[TokenType::Newline, TokenType::Eof]) { break; }
            match tok.ttype {
                TokenType::Ident => {
                    let key = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };
                    self.advance();
                    if key == "viewbox" && self.matches(&[TokenType::Pair]) {
                        // Parse viewbox as two pairs: origin and size
                        if let Some(t1) = self.advance() {
                            if let TokenValue::Pair(x, y) = t1.value {
                                if self.matches(&[TokenType::Pair]) {
                                    if let Some(t2) = self.advance() {
                                        if let TokenValue::Pair(w, h) = t2.value {
                                            symbol.viewbox = Some((x, y, w, h));
                                        }
                                    }
                                } else {
                                    // Single pair - treat as size with 0,0 origin
                                    symbol.viewbox = Some((0.0, 0.0, x, y));
                                }
                            }
                        }
                    }
                }
                _ => { self.advance(); }
            }
        }

        // Parse block with child shapes
        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_symbol_block(&mut symbol);
        }

        AstNode::Symbol(symbol)
    }

    fn parse_symbol_block(&mut self, symbol: &mut super::ast::AstSymbol) {
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Dedent { self.advance(); break; }
            if tok.ttype == TokenType::Eof {
                self.error_at_current("Unexpected end of file in symbol block", ErrorKind::UnterminatedBlock, None);
                break;
            }

            self.skip_newlines();
            if self.matches(&[TokenType::Dedent]) { self.advance(); break; }

            if let Some(tok) = self.current() {
                if tok.ttype == TokenType::Ident {
                    let cmd = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };

                    if SHAPES.contains(cmd.as_str()) || cmd == "group" {
                        match self.parse_statement() {
                            Some(AstNode::Shape(child)) => symbol.children.push(child),
                            _ => {}
                        }
                    } else {
                        self.error_at_current(
                            &format!("Only shapes allowed in symbol block, found '{}'", cmd),
                            ErrorKind::InvalidProperty,
                            Some("Use rect, circle, path, etc. inside symbol blocks")
                        );
                        self.advance();
                        self.sync_to_line_end();
                    }
                } else {
                    self.advance();
                }
            }
        }
    }

    /// Parse use reference to instantiate a symbol (SVG <use>)
    fn parse_use(&mut self) -> AstNode {
        use super::ast::AstUse;
        let mut use_ref = AstUse::default();

        // Parse symbol reference (required)
        if self.matches(&[TokenType::String]) {
            if let Some(tok) = self.advance() {
                if let TokenValue::Str(s) = &tok.value { use_ref.href = s.clone(); }
            }
        } else {
            self.error_at_current("Expected symbol reference (string)", ErrorKind::MissingToken, Some("use \"my-icon\""));
        }

        // Parse inline properties
        while let Some(tok) = self.current() {
            if self.matches(&[TokenType::Newline, TokenType::Eof]) { break; }
            match tok.ttype {
                TokenType::Pair => {
                    if let TokenValue::Pair(a, b) = self.advance().map(|t| &t.value).unwrap() {
                        if use_ref.at.is_none() {
                            use_ref.at = Some((*a, *b));
                        } else if use_ref.size.is_none() {
                            use_ref.size = Some((*a, *b));
                        }
                    }
                }
                TokenType::Ident => {
                    let key = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };
                    self.advance();

                    match key.as_str() {
                        "at" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value {
                                    use_ref.at = Some((a, b));
                                }
                            }
                        }
                        "size" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value {
                                    use_ref.size = Some((a, b));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                TokenType::Color | TokenType::Var => {
                    let val = self.resolve(tok);
                    self.advance();
                    if let TokenValue::Str(s) = val { use_ref.style.fill = Some(s); }
                }
                _ => { self.advance(); }
            }
        }

        // Parse optional block for transform/style
        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_use_block(&mut use_ref);
        }

        AstNode::Use(use_ref)
    }

    fn parse_use_block(&mut self, use_ref: &mut super::ast::AstUse) {
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Dedent { self.advance(); break; }
            if tok.ttype == TokenType::Eof { break; }

            self.skip_newlines();
            if self.matches(&[TokenType::Dedent]) { self.advance(); break; }

            if let Some(tok) = self.current() {
                if tok.ttype == TokenType::Ident {
                    let prop = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };

                    if STYLE_PROPS.contains(prop.as_str()) {
                        // Create temporary shape to parse style
                        let mut temp = AstShape::new("_temp");
                        self.parse_style_prop(&mut temp);
                        use_ref.style = temp.style;
                    } else if TRANSFORM_PROPS.contains(prop.as_str()) {
                        self.parse_transform_prop(&mut use_ref.transform);
                    } else {
                        self.advance();
                    }
                } else {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn parse_shape(&mut self, kind: &str) -> AstNode {
        let mut shape = AstShape::new(kind);

        while let Some(tok) = self.current() {
            if self.matches(&[TokenType::Newline, TokenType::Eof]) {
                break;
            }

            match tok.ttype {
                TokenType::Pair => {
                    if let TokenValue::Pair(a, b) = self.advance().map(|t| &t.value).unwrap() {
                        if !shape.props.contains_key("at") {
                            shape.props.insert("at".into(), PropValue::Pair(*a, *b));
                        } else if !shape.props.contains_key("size") {
                            shape.props.insert("size".into(), PropValue::Pair(*a, *b));
                        }
                    }
                }
                TokenType::Number => {
                    if let TokenValue::Num(n) = self.advance().map(|t| &t.value).unwrap() {
                        if kind == "circle" && !shape.props.contains_key("radius") {
                            shape.props.insert("radius".into(), PropValue::Num(*n));
                        } else if !shape.props.contains_key("width") {
                            shape.props.insert("width".into(), PropValue::Num(*n));
                        }
                    }
                }
                TokenType::String => {
                    if let TokenValue::Str(s) = self.advance().map(|t| t.value.clone()).unwrap() {
                        shape.props.insert("content".into(), PropValue::Str(s));
                    }
                }
                TokenType::LBracket if kind == "polygon" => {
                    shape.props.insert("points".into(), PropValue::Points(self.parse_points()));
                }
                TokenType::Ident => {
                    let key = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };
                    self.advance();

                    match key.as_str() {
                        "at" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value {
                                    shape.props.insert("at".into(), PropValue::Pair(a, b));
                                }
                            }
                        }
                        "size" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value {
                                    shape.props.insert("size".into(), PropValue::Pair(a, b));
                                }
                            }
                        }
                        "radius" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value {
                                    shape.props.insert("radius".into(), PropValue::Pair(a, b));
                                }
                            }
                        }
                        "radius" if self.matches(&[TokenType::Number]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Num(n) = t.value {
                                    shape.props.insert("radius".into(), PropValue::Num(n));
                                }
                            }
                        }
                        "from" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value {
                                    shape.props.insert("from".into(), PropValue::Pair(a, b));
                                }
                            }
                        }
                        "to" if self.matches(&[TokenType::Pair]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Pair(a, b) = t.value {
                                    shape.props.insert("to".into(), PropValue::Pair(a, b));
                                }
                            }
                        }
                        "d" if self.matches(&[TokenType::String]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value {
                                    shape.props.insert("d".into(), PropValue::Str(s.clone()));
                                }
                            }
                        }
                        "points" if self.matches(&[TokenType::LBracket]) => {
                            shape.props.insert("points".into(), PropValue::Points(self.parse_points()));
                        }
                        "href" if self.matches(&[TokenType::String]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Str(s) = &t.value {
                                    shape.props.insert("href".into(), PropValue::Str(s.clone()));
                                }
                            }
                        }
                        // Arc properties
                        "start" if self.matches(&[TokenType::Number]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Num(n) = t.value {
                                    shape.props.insert("start".into(), PropValue::Num(n));
                                }
                            }
                        }
                        "end" if self.matches(&[TokenType::Number]) => {
                            if let Some(t) = self.advance() {
                                if let TokenValue::Num(n) = t.value {
                                    shape.props.insert("end".into(), PropValue::Num(n));
                                }
                            }
                        }
                        // Curve properties
                        "smooth" => {
                            shape.props.insert("smooth".into(), PropValue::Num(1.0));
                        }
                        "sharp" => {
                            shape.props.insert("smooth".into(), PropValue::Num(0.0));
                        }
                        "closed" => {
                            shape.props.insert("closed".into(), PropValue::Num(1.0));
                        }
                        _ => {}
                    }
                }
                TokenType::Color | TokenType::Var => {
                    if !shape.props.contains_key("fill") {
                        let val = self.resolve(tok);
                        self.advance();
                        if let TokenValue::Str(s) = val {
                            shape.props.insert("fill".into(), PropValue::Str(s));
                        }
                    } else {
                        self.advance();
                    }
                }
                _ => { self.advance(); }
            }
        }

        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_block(&mut shape);
        }

        AstNode::Shape(shape)
    }

    pub(crate) fn parse_block(&mut self, shape: &mut AstShape) {
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Dedent {
                self.advance();
                break;
            }
            if tok.ttype == TokenType::Eof {
                self.error_at_current(
                    "Unexpected end of file in block",
                    ErrorKind::UnterminatedBlock,
                    Some("Block was never closed")
                );
                break;
            }

            self.skip_newlines();
            if self.matches(&[TokenType::Dedent]) {
                self.advance();
                break;
            }

            if let Some(tok) = self.current() {
                if tok.ttype == TokenType::Ident {
                    let prop = match &tok.value {
                        TokenValue::Str(s) => s.clone(),
                        _ => { self.advance(); continue; }
                    };

                    if SHAPES.contains(prop.as_str()) {
                        match self.parse_statement() {
                            Some(AstNode::Shape(child)) => shape.children.push(child),
                            _ => {} // Error already recorded, continue with next
                        }
                    } else if STYLE_PROPS.contains(prop.as_str()) {
                        self.parse_style_prop(shape);
                    } else if TEXT_PROPS.contains(prop.as_str()) {
                        self.parse_text_prop(&mut shape.style);
                    } else if TRANSFORM_PROPS.contains(prop.as_str()) {
                        self.parse_transform_prop(&mut shape.transform);
                    } else if prop == "width" && self.peek_next().map(|t| t.ttype == TokenType::Number).unwrap_or(false) {
                        self.advance();
                        if let Some(t) = self.advance() {
                            if let TokenValue::Num(n) = t.value {
                                shape.style.stroke_width = n;
                            }
                        }
                    } else if prop == "d" && self.peek_next().map(|t| t.ttype == TokenType::String).unwrap_or(false) {
                        self.advance();
                        if let Some(t) = self.advance() {
                            if let TokenValue::Str(s) = &t.value {
                                shape.props.insert("d".into(), PropValue::Str(s.clone()));
                            }
                        }
                    } else if prop == "points" && self.peek_next().map(|t| t.ttype == TokenType::LBracket).unwrap_or(false) {
                        self.advance();
                        shape.props.insert("points".into(), PropValue::Points(self.parse_points()));
                    } else {
                        // Unknown property in block - report and skip line
                        self.error_at_current(
                            &format!("Unknown property '{}' in {} block", prop, shape.kind),
                            ErrorKind::InvalidProperty,
                            Self::suggest_property(&prop, &shape.kind).as_deref()
                        );
                        self.advance();
                        self.sync_to_line_end();
                    }
                } else {
                    // Unexpected token in block
                    let ttype = tok.ttype;
                    self.error_at_current(
                        &format!("Unexpected {:?} in block", ttype),
                        ErrorKind::UnexpectedToken,
                        Some("Expected property name or nested shape")
                    );
                    self.advance();
                }
            }
        }
    }

    /// Suggest similar property names
    fn suggest_property(prop: &str, kind: &str) -> Option<String> {
        let all_props: Vec<&str> = STYLE_PROPS.iter()
            .chain(TEXT_PROPS.iter())
            .chain(TRANSFORM_PROPS.iter())
            .copied()
            .collect();
        
        let prop_lower = prop.to_lowercase();
        for valid in &all_props {
            if valid.starts_with(&prop_lower) || prop_lower.starts_with(*valid) {
                return Some(format!("Did you mean '{}'?", valid));
            }
        }
        
        Some(format!("Valid {} properties: fill, stroke, opacity, transform, etc.", kind))
    }

    fn parse_style_prop(&mut self, shape: &mut AstShape) {
        let prop = match self.advance().and_then(|t| match &t.value {
            TokenValue::Str(s) => Some(s.clone()),
            _ => None,
        }) {
            Some(p) => p,
            None => return,
        };

        match prop.as_str() {
            "fill" => {
                if self.matches(&[TokenType::Color, TokenType::Var, TokenType::Ident]) {
                    if let Some(tok) = self.current() {
                        if let TokenValue::Str(s) = self.resolve(tok) {
                            shape.style.fill = Some(s);
                        }
                        self.advance();
                    }
                }
            }
            "stroke" => {
                if self.matches(&[TokenType::Color, TokenType::Var]) {
                    if let Some(tok) = self.current() {
                        if let TokenValue::Str(s) = self.resolve(tok) {
                            shape.style.stroke = Some(s);
                        }
                        self.advance();
                    }
                }
                if self.matches(&[TokenType::Number]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Num(n) = t.value {
                            shape.style.stroke_width = n;
                        }
                    }
                }
                if self.matches(&[TokenType::Ident]) {
                    if let Some(tok) = self.current() {
                        if matches!(&tok.value, TokenValue::Str(s) if s == "width") {
                            self.advance();
                            if self.matches(&[TokenType::Number]) {
                                if let Some(t) = self.advance() {
                                    if let TokenValue::Num(n) = t.value {
                                        shape.style.stroke_width = n;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "opacity" => {
                if self.matches(&[TokenType::Number]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Num(n) = t.value {
                            shape.style.opacity = n;
                        }
                    }
                }
            }
            "corner" => {
                if self.matches(&[TokenType::Number]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Num(n) = t.value {
                            shape.style.corner = n;
                        }
                    }
                }
            }
            "shadow" => {
                shape.shadow = Some(self.parse_shadow());
            }
            "gradient" => {
                shape.gradient = Some(self.parse_gradient());
            }
            _ => {}
        }
    }

    fn parse_text_prop(&mut self, style: &mut AstStyle) {
        let prop = match self.advance().and_then(|t| match &t.value {
            TokenValue::Str(s) => Some(s.clone()),
            _ => None,
        }) {
            Some(p) => p,
            None => return,
        };

        match prop.as_str() {
            "font" => {
                if self.matches(&[TokenType::String]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Str(s) = &t.value {
                            style.font = Some(s.clone());
                        }
                    }
                }
                if self.matches(&[TokenType::Number]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Num(n) = t.value {
                            style.font_size = n;
                        }
                    }
                }
            }
            "bold" => style.font_weight = "bold".into(),
            "italic" => style.font_weight = "italic".into(),
            "center" => style.text_anchor = "middle".into(),
            "end" => style.text_anchor = "end".into(),
            _ => {}
        }
    }

    fn parse_transform_prop(&mut self, transform: &mut AstTransform) {
        let prop = match self.advance().and_then(|t| match &t.value {
            TokenValue::Str(s) => Some(s.clone()),
            _ => None,
        }) {
            Some(p) => p,
            None => return,
        };

        match prop.as_str() {
            "translate" => {
                if self.matches(&[TokenType::Pair]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Pair(a, b) = t.value {
                            transform.translate = Some((a, b));
                        }
                    }
                }
            }
            "rotate" => {
                if self.matches(&[TokenType::Number]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Num(n) = t.value {
                            transform.rotate = n;
                        }
                    }
                }
            }
            "scale" => {
                if self.matches(&[TokenType::Pair]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Pair(a, b) = t.value {
                            transform.scale = Some((a, b));
                        }
                    }
                } else if self.matches(&[TokenType::Number]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Num(n) = t.value {
                            transform.scale = Some((n, n));
                        }
                    }
                }
            }
            "origin" => {
                if self.matches(&[TokenType::Pair]) {
                    if let Some(t) = self.advance() {
                        if let TokenValue::Pair(a, b) = t.value {
                            transform.origin = Some((a, b));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn parse_shadow(&mut self) -> ShadowDef {
        let mut shadow = ShadowDef {
            x: 0.0, y: 4.0, blur: 8.0, color: "#0004".into(),
        };

        if self.matches(&[TokenType::Pair]) {
            if let Some(t) = self.advance() {
                if let TokenValue::Pair(a, b) = t.value {
                    shadow.x = a;
                    shadow.y = b;
                }
            }
        }
        if self.matches(&[TokenType::Number]) {
            if let Some(t) = self.advance() {
                if let TokenValue::Num(n) = t.value {
                    shadow.blur = n;
                }
            }
        }
        if self.matches(&[TokenType::Color]) {
            if let Some(t) = self.advance() {
                if let TokenValue::Str(s) = &t.value {
                    shadow.color = s.clone();
                }
            }
        }

        shadow
    }

    fn parse_gradient(&mut self) -> GradientDef {
        let mut gradient = GradientDef {
            gtype: "linear".into(),
            from: "#fff".into(),
            to: "#000".into(),
            angle: 90.0,
        };

        while self.matches(&[TokenType::Ident, TokenType::Color, TokenType::Number]) {
            if let Some(tok) = self.current() {
                match tok.ttype {
                    TokenType::Ident => {
                        let val = match &tok.value {
                            TokenValue::Str(s) => s.clone(),
                            _ => { self.advance(); continue; }
                        };
                        self.advance();

                        match val.as_str() {
                            "linear" | "radial" => gradient.gtype = val,
                            "from" if self.matches(&[TokenType::Color]) => {
                                if let Some(t) = self.advance() {
                                    if let TokenValue::Str(s) = &t.value {
                                        gradient.from = s.clone();
                                    }
                                }
                            }
                            "to" if self.matches(&[TokenType::Color]) => {
                                if let Some(t) = self.advance() {
                                    if let TokenValue::Str(s) = &t.value {
                                        gradient.to = s.clone();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    TokenType::Color => {
                        if let TokenValue::Str(s) = &tok.value {
                            if gradient.from == "#fff" {
                                gradient.from = s.clone();
                            } else {
                                gradient.to = s.clone();
                            }
                        }
                        self.advance();
                    }
                    TokenType::Number => {
                        if let TokenValue::Num(n) = tok.value {
                            gradient.angle = n;
                        }
                        self.advance();
                    }
                    _ => { self.advance(); }
                }
            } else {
                break;
            }
        }

        gradient
    }

    pub(crate) fn parse_points(&mut self) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        
        if !self.matches(&[TokenType::LBracket]) {
            self.error_at_current("Expected '[' to start points list", ErrorKind::MissingToken, None);
            return points;
        }
        self.advance(); // consume [

        while let Some(tok) = self.current() {
            match tok.ttype {
                TokenType::RBracket => {
                    self.advance();
                    break;
                }
                TokenType::Pair => {
                    if let TokenValue::Pair(a, b) = tok.value {
                        points.push((a, b));
                    }
                    self.advance();
                }
                TokenType::Eof => {
                    self.error_at_current(
                        "Unclosed points list",
                        ErrorKind::UnterminatedBlock,
                        Some("Add ']' to close the points list")
                    );
                    break;
                }
                TokenType::Newline => {
                    // Allow newlines in points list
                    self.advance();
                }
                _ => {
                    self.error_at_current(
                        &format!("Expected point pair (x,y), found {:?}", tok.ttype),
                        ErrorKind::InvalidValue,
                        Some("Points should be in format: [100,200 300,400]")
                    );
                    self.advance();
                }
            }
        }

        points
    }
}

