//! Parser for the iconoglott DSL
//!
//! Parses token stream into AST with error collection and recovery.

use super::lexer::{CanvasSize, Token, TokenType, TokenValue};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[cfg(feature = "python")]
use pyo3::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// AST Types
// ─────────────────────────────────────────────────────────────────────────────

/// Style properties for shapes
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct AstStyle {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f64,
    pub opacity: f64,
    pub corner: f64,
    pub font: Option<String>,
    pub font_size: f64,
    pub font_weight: String,
    pub text_anchor: String,
}

/// Extended style with shadow/gradient (separate for Python compat)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FullStyle {
    pub base: AstStyle,
    pub shadow: Option<ShadowDef>,
    pub gradient: Option<GradientDef>,
}

impl AstStyle {
    pub fn new() -> Self {
        Self {
            stroke_width: 1.0,
            opacity: 1.0,
            font_size: 16.0,
            font_weight: "normal".into(),
            text_anchor: "start".into(),
            ..Default::default()
        }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl AstStyle {
    #[new]
    fn py_new() -> Self { Self::new() }
}

/// Shadow definition
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct ShadowDef {
    pub x: f64,
    pub y: f64,
    pub blur: f64,
    pub color: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl ShadowDef {
    #[new]
    #[pyo3(signature = (x=0.0, y=4.0, blur=8.0, color="#0004".to_string()))]
    fn py_new(x: f64, y: f64, blur: f64, color: String) -> Self {
        Self { x, y, blur, color }
    }
}

/// Gradient definition
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct GradientDef {
    pub gtype: String, // "linear" or "radial"
    pub from: String,
    pub to: String,
    pub angle: f64,
}

#[cfg(feature = "python")]
#[pymethods]
impl GradientDef {
    #[new]
    #[pyo3(signature = (gtype="linear".to_string(), from="#fff".to_string(), to="#000".to_string(), angle=90.0))]
    fn py_new(gtype: String, from: String, to: String, angle: f64) -> Self {
        Self { gtype, from, to, angle }
    }
}

/// Transform properties
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass)]
pub struct AstTransform {
    pub translate: Option<(f64, f64)>,
    pub rotate: f64,
    pub scale: Option<(f64, f64)>,
    pub origin: Option<(f64, f64)>,
}

#[cfg(feature = "python")]
#[pymethods]
impl AstTransform {
    #[new]
    fn py_new() -> Self { Self::default() }

    #[getter]
    fn get_translate(&self) -> Option<(f64, f64)> { self.translate }
    #[setter]
    fn set_translate(&mut self, v: Option<(f64, f64)>) { self.translate = v; }

    #[getter]
    fn get_rotate(&self) -> f64 { self.rotate }
    #[setter]
    fn set_rotate(&mut self, v: f64) { self.rotate = v; }

    #[getter]
    fn get_scale(&self) -> Option<(f64, f64)> { self.scale }
    #[setter]
    fn set_scale(&mut self, v: Option<(f64, f64)>) { self.scale = v; }

    #[getter]
    fn get_origin(&self) -> Option<(f64, f64)> { self.origin }
    #[setter]
    fn set_origin(&mut self, v: Option<(f64, f64)>) { self.origin = v; }
}

/// Canvas definition using standardized sizes
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct AstCanvas {
    pub size: CanvasSize,
    pub fill: String,
}

impl AstCanvas {
    pub fn width(&self) -> u32 { self.size.pixels() }
    pub fn height(&self) -> u32 { self.size.pixels() }
    pub fn dimensions(&self) -> (u32, u32) { self.size.dimensions() }
}

impl Default for AstCanvas {
    fn default() -> Self {
        Self { size: CanvasSize::Medium, fill: "#fff".into() }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl AstCanvas {
    #[new]
    #[pyo3(signature = (size=CanvasSize::Medium, fill="#fff".to_string()))]
    fn py_new(size: CanvasSize, fill: String) -> Self {
        Self { size, fill }
    }
    
    #[getter]
    fn get_width(&self) -> u32 { self.width() }
    
    #[getter]
    fn get_height(&self) -> u32 { self.height() }
}

/// Property value types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropValue {
    None,
    Str(String),
    Num(f64),
    Pair(f64, f64),
    Points(Vec<(f64, f64)>),
}

impl Default for PropValue {
    fn default() -> Self { Self::None }
}

/// Shape in the AST
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass)]
pub struct AstShape {
    pub kind: String,
    pub props: HashMap<String, PropValue>,
    pub style: AstStyle,
    pub shadow: Option<ShadowDef>,
    pub gradient: Option<GradientDef>,
    pub transform: AstTransform,
    pub children: Vec<AstShape>,
}

impl AstShape {
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.into(),
            props: HashMap::new(),
            style: AstStyle::new(),
            shadow: None,
            gradient: None,
            transform: AstTransform::default(),
            children: Vec::new(),
        }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl AstShape {
    #[new]
    fn py_new(kind: &str) -> Self { Self::new(kind) }

    #[getter]
    fn get_kind(&self) -> String { self.kind.clone() }

    #[getter]
    fn get_props(&self, py: Python<'_>) -> PyObject {
        use pyo3::types::PyDict;
        let dict = PyDict::new(py);
        for (k, v) in &self.props {
            let val: PyObject = match v {
                PropValue::None => py.None(),
                PropValue::Str(s) => s.clone().into_py(py),
                PropValue::Num(n) => n.into_py(py),
                PropValue::Pair(a, b) => (*a, *b).into_py(py),
                PropValue::Points(pts) => pts.clone().into_py(py),
            };
            dict.set_item(k, val).ok();
        }
        dict.into()
    }

    #[getter]
    fn get_style(&self) -> AstStyle { self.style.clone() }

    #[getter]
    fn get_shadow(&self) -> Option<ShadowDef> { self.shadow.clone() }

    #[getter]
    fn get_gradient(&self) -> Option<GradientDef> { self.gradient.clone() }

    #[getter]
    fn get_transform(&self) -> AstTransform { self.transform.clone() }

    #[getter]
    fn get_children(&self) -> Vec<AstShape> { self.children.clone() }
}

/// AST node types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    Scene(Vec<AstNode>),
    Canvas(AstCanvas),
    Shape(AstShape),
    Variable { name: String, value: Option<TokenValue> },
}

/// Parse error
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(get_all))]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

#[cfg(feature = "python")]
#[pymethods]
impl ParseError {
    fn __repr__(&self) -> String {
        format!("ParseError({:?}, {}:{})", self.message, self.line, self.col)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────────────

lazy_static::lazy_static! {
    static ref SHAPES: HashSet<&'static str> = {
        ["rect", "circle", "ellipse", "line", "path", "polygon", "text", "image", "arc", "curve"]
            .into_iter().collect()
    };
    static ref STYLE_PROPS: HashSet<&'static str> = {
        ["fill", "stroke", "opacity", "corner", "shadow", "gradient", "blur"]
            .into_iter().collect()
    };
    static ref TEXT_PROPS: HashSet<&'static str> = {
        ["font", "bold", "italic", "center", "middle", "end"]
            .into_iter().collect()
    };
    static ref TRANSFORM_PROPS: HashSet<&'static str> = {
        ["translate", "rotate", "scale", "origin"]
            .into_iter().collect()
    };
}

/// Parser for the iconoglott DSL
#[cfg_attr(feature = "python", pyclass)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    variables: HashMap<String, TokenValue>,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            variables: HashMap::new(),
            errors: Vec::new(),
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    fn matches(&self, types: &[TokenType]) -> bool {
        self.current().map(|t| types.contains(&t.ttype)).unwrap_or(false)
    }

    fn skip_newlines(&mut self) {
        while self.matches(&[TokenType::Newline]) {
            self.advance();
        }
    }

    fn resolve(&self, tok: &Token) -> TokenValue {
        if tok.ttype == TokenType::Var {
            if let TokenValue::Str(name) = &tok.value {
                if let Some(val) = self.variables.get(name) {
                    return val.clone();
                }
            }
        }
        tok.value.clone()
    }

    fn error(&mut self, msg: &str) {
        let (line, col) = self.current().map(|t| (t.line, t.col)).unwrap_or((0, 0));
        self.errors.push(ParseError { message: msg.into(), line, col });
    }

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

    fn parse_statement(&mut self) -> Option<AstNode> {
        let tok = self.current()?;

        if tok.ttype == TokenType::Var {
            return self.parse_variable();
        }

        if tok.ttype != TokenType::Ident {
            self.advance();
            return None;
        }

        let cmd = match &tok.value {
            TokenValue::Str(s) => s.clone(),
            _ => return None,
        };
        self.advance();

        match cmd.as_str() {
            "canvas" => Some(self.parse_canvas()),
            "group" => Some(self.parse_group()),
            "stack" | "row" => Some(self.parse_layout(&cmd)),
            _ if SHAPES.contains(cmd.as_str()) => Some(self.parse_shape(&cmd)),
            _ => {
                self.error(&format!("Unknown command: {}", cmd));
                None
            }
        }
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
                        self.error(&format!("Invalid canvas size '{}'. Valid sizes: {}", name, CanvasSize::all_names().join(", ")));
                    }
                }
            }
        } else if self.matches(&[TokenType::Pair]) {
            // Legacy support: emit error for raw dimensions
            self.error(&format!("Raw pixel dimensions are not allowed. Use a standard size: {}", CanvasSize::all_names().join(", ")));
            self.advance(); // consume the pair token
        }

        while self.matches(&[TokenType::Ident, TokenType::Size]) {
            let prop = self.current().and_then(|t| match &t.value {
                TokenValue::Str(s) => Some(s.clone()),
                _ => None,
            });
            self.advance();

            if let Some(p) = prop {
                if p == "fill" {
                    if let Some(tok) = self.current() {
                        canvas.fill = match self.resolve(tok) {
                            TokenValue::Str(s) => s,
                            _ => canvas.fill,
                        };
                        self.advance();
                    }
                }
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
        let mut shape = AstShape::new("layout");
        let direction = if kind == "stack" { "vertical" } else { "horizontal" };
        shape.props.insert("direction".into(), PropValue::Str(direction.into()));
        shape.props.insert("gap".into(), PropValue::Num(0.0));

        while self.matches(&[TokenType::Ident]) {
            let prop = self.current().and_then(|t| match &t.value {
                TokenValue::Str(s) => Some(s.clone()),
                _ => None,
            });
            self.advance();

            match prop.as_deref() {
                Some("vertical") | Some("horizontal") => {
                    if let Some(p) = prop {
                        shape.props.insert("direction".into(), PropValue::Str(p));
                    }
                }
                Some("gap") if self.matches(&[TokenType::Number]) => {
                    if let Some(tok) = self.advance() {
                        if let TokenValue::Num(n) = tok.value {
                            shape.props.insert("gap".into(), PropValue::Num(n));
                        }
                    }
                }
                Some("at") if self.matches(&[TokenType::Pair]) => {
                    if let Some(tok) = self.advance() {
                        if let TokenValue::Pair(a, b) = tok.value {
                            shape.props.insert("at".into(), PropValue::Pair(a, b));
                        }
                    }
                }
                _ => {}
            }
        }

        self.skip_newlines();
        if self.matches(&[TokenType::Indent]) {
            self.advance();
            self.parse_block(&mut shape);
        }

        AstNode::Shape(shape)
    }

    fn parse_shape(&mut self, kind: &str) -> AstNode {
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

    fn parse_block(&mut self, shape: &mut AstShape) {
        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::Dedent {
                self.advance();
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
                        if let Some(AstNode::Shape(child)) = self.parse_statement() {
                            shape.children.push(child);
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
                        self.advance();
                    }
                } else {
                    self.advance();
                }
            }
        }
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

    fn parse_points(&mut self) -> Vec<(f64, f64)> {
        let mut points = Vec::new();
        self.advance(); // consume [

        while let Some(tok) = self.current() {
            if tok.ttype == TokenType::RBracket {
                self.advance();
                break;
            }
            if tok.ttype == TokenType::Pair {
                if let TokenValue::Pair(a, b) = tok.value {
                    points.push((a, b));
                }
            }
            self.advance();
        }

        points
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Parser {
    #[new]
    fn py_new(tokens: Vec<Token>) -> Self {
        Self::new(tokens)
    }

    /// Parse and return the AST as native Python objects
    fn parse_py(&mut self, py: Python<'_>) -> PyObject {
        ast_node_to_py(py, &self.parse())
    }

    /// Get parse errors
    fn get_errors(&self) -> Vec<ParseError> {
        self.errors.clone()
    }
}

/// Convert AstNode to Python object directly
#[cfg(feature = "python")]
fn ast_node_to_py(py: Python<'_>, node: &AstNode) -> PyObject {
    use pyo3::types::{PyDict, PyList};
    let dict = PyDict::new(py);
    
    match node {
        AstNode::Scene(children) => {
            let list = PyList::new(py, children.iter().map(|c| ast_node_to_py(py, c)));
            dict.set_item("Scene", list).ok();
        }
        AstNode::Canvas(c) => {
            let canvas = PyDict::new(py);
            canvas.set_item("size", c.size.to_string()).ok();
            canvas.set_item("width", c.width()).ok();
            canvas.set_item("height", c.height()).ok();
            canvas.set_item("fill", &c.fill).ok();
            dict.set_item("Canvas", canvas).ok();
        }
        AstNode::Shape(s) => {
            dict.set_item("Shape", ast_shape_to_py(py, s)).ok();
        }
        AstNode::Variable { name, value } => {
            let var = PyDict::new(py);
            var.set_item("name", name).ok();
            var.set_item("value", token_value_to_py(py, value.as_ref())).ok();
            dict.set_item("Variable", var).ok();
        }
    }
    dict.into()
}

/// Convert AstShape to Python dict
#[cfg(feature = "python")]
fn ast_shape_to_py(py: Python<'_>, shape: &AstShape) -> PyObject {
    use pyo3::types::{PyDict, PyList};
    let dict = PyDict::new(py);
    
    dict.set_item("kind", &shape.kind).ok();
    
    // Convert props HashMap to PyDict
    let props = PyDict::new(py);
    for (k, v) in &shape.props {
        props.set_item(k, prop_value_to_py(py, v)).ok();
    }
    dict.set_item("props", props).ok();
    
    // Convert style
    let style = PyDict::new(py);
    style.set_item("fill", shape.style.fill.as_deref()).ok();
    style.set_item("stroke", shape.style.stroke.as_deref()).ok();
    style.set_item("stroke_width", shape.style.stroke_width).ok();
    style.set_item("opacity", shape.style.opacity).ok();
    style.set_item("corner", shape.style.corner).ok();
    style.set_item("font", shape.style.font.as_deref()).ok();
    style.set_item("font_size", shape.style.font_size).ok();
    style.set_item("font_weight", &shape.style.font_weight).ok();
    style.set_item("text_anchor", &shape.style.text_anchor).ok();
    dict.set_item("style", style).ok();
    
    // Convert shadow
    if let Some(shadow) = &shape.shadow {
        let s = PyDict::new(py);
        s.set_item("x", shadow.x).ok();
        s.set_item("y", shadow.y).ok();
        s.set_item("blur", shadow.blur).ok();
        s.set_item("color", &shadow.color).ok();
        dict.set_item("shadow", s).ok();
    }
    
    // Convert gradient
    if let Some(grad) = &shape.gradient {
        let g = PyDict::new(py);
        g.set_item("gtype", &grad.gtype).ok();
        g.set_item("from", &grad.from).ok();
        g.set_item("to", &grad.to).ok();
        g.set_item("angle", grad.angle).ok();
        dict.set_item("gradient", g).ok();
    }
    
    // Convert transform
    let transform = PyDict::new(py);
    transform.set_item("translate", shape.transform.translate).ok();
    transform.set_item("rotate", shape.transform.rotate).ok();
    transform.set_item("scale", shape.transform.scale).ok();
    transform.set_item("origin", shape.transform.origin).ok();
    dict.set_item("transform", transform).ok();
    
    // Convert children recursively
    let children = PyList::new(py, shape.children.iter().map(|c| ast_shape_to_py(py, c)));
    dict.set_item("children", children).ok();
    
    dict.into()
}

/// Convert PropValue to Python object
#[cfg(feature = "python")]
fn prop_value_to_py(py: Python<'_>, val: &PropValue) -> PyObject {
    use pyo3::types::PyList;
    match val {
        PropValue::None => py.None(),
        PropValue::Str(s) => s.into_py(py),
        PropValue::Num(n) => n.into_py(py),
        PropValue::Pair(a, b) => (*a, *b).into_py(py),
        PropValue::Points(pts) => PyList::new(py, pts.iter().map(|(a, b)| (*a, *b))).into(),
    }
}

/// Convert TokenValue to Python object
#[cfg(feature = "python")]
fn token_value_to_py(py: Python<'_>, val: Option<&TokenValue>) -> PyObject {
    match val {
        None | Some(TokenValue::None) => py.None(),
        Some(TokenValue::Str(s)) => s.into_py(py),
        Some(TokenValue::Num(n)) => n.into_py(py),
        Some(TokenValue::Pair(a, b)) => (*a, *b).into_py(py),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WASM bindings
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Parse DSL source and return AST as JSON
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse(source: &str) -> String {
    let mut lexer = super::lexer::Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    serde_json::to_string(&ast).unwrap_or_else(|_| "null".to_string())
}

/// Parse and return errors as JSON
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn parse_with_errors(source: &str) -> String {
    let mut lexer = super::lexer::Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    #[derive(Serialize)]
    struct ParseResult {
        ast: AstNode,
        errors: Vec<ParseError>,
    }
    
    serde_json::to_string(&ParseResult { ast, errors: parser.errors })
        .unwrap_or_else(|_| r#"{"ast":null,"errors":[]}"#.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::Lexer;

    fn parse_source(source: &str) -> AstNode {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_empty_source() {
        let ast = parse_source("");
        assert!(matches!(ast, AstNode::Scene(children) if children.is_empty()));
    }

    #[test]
    fn test_canvas() {
        let ast = parse_source("canvas large fill #1a1a2e");
        if let AstNode::Scene(children) = ast {
            assert_eq!(children.len(), 1);
            if let AstNode::Canvas(c) = &children[0] {
                assert_eq!(c.size, CanvasSize::Large);
                assert_eq!(c.width(), 96);
                assert_eq!(c.height(), 96);
                assert_eq!(c.fill, "#1a1a2e");
            } else {
                panic!("Expected Canvas");
            }
        } else {
            panic!("Expected Scene");
        }
    }
    
    #[test]
    fn test_canvas_sizes() {
        for (name, expected_px) in [("nano", 16), ("micro", 24), ("tiny", 32), ("small", 48), 
                                     ("medium", 64), ("large", 96), ("xlarge", 128), 
                                     ("huge", 192), ("massive", 256), ("giant", 512)] {
            let ast = parse_source(&format!("canvas {}", name));
            if let AstNode::Scene(children) = ast {
                if let AstNode::Canvas(c) = &children[0] {
                    assert_eq!(c.width(), expected_px as u32, "Size {} should be {}px", name, expected_px);
                } else {
                    panic!("Expected Canvas for size {}", name);
                }
            }
        }
    }

    #[test]
    fn test_rect() {
        let ast = parse_source("rect at 100,200 size 50x30 #ff0");
        if let AstNode::Scene(children) = ast {
            if let AstNode::Shape(s) = &children[0] {
                assert_eq!(s.kind, "rect");
                assert!(matches!(s.props.get("at"), Some(PropValue::Pair(a, b)) if (*a - 100.0).abs() < 0.001 && (*b - 200.0).abs() < 0.001));
            }
        }
    }

    #[test]
    fn test_circle() {
        let ast = parse_source("circle at 200,200 radius 50");
        if let AstNode::Scene(children) = ast {
            if let AstNode::Shape(s) = &children[0] {
                assert_eq!(s.kind, "circle");
                assert!(matches!(s.props.get("radius"), Some(PropValue::Num(n)) if (*n - 50.0).abs() < 0.001));
            }
        }
    }

    #[test]
    fn test_nested_style() {
        let ast = parse_source("rect\n  fill #ff0\n  stroke #000 2");
        if let AstNode::Scene(children) = ast {
            if let AstNode::Shape(s) = &children[0] {
                assert_eq!(s.style.fill, Some("#ff0".into()));
                assert_eq!(s.style.stroke, Some("#000".into()));
                assert!((s.style.stroke_width - 2.0).abs() < 0.001);
            }
        }
    }

    #[test]
    fn test_variable() {
        let ast = parse_source("$accent = #ff0\ncircle $accent");
        if let AstNode::Scene(children) = ast {
            assert!(matches!(&children[0], AstNode::Variable { .. }));
        }
    }

    #[test]
    fn test_arc() {
        let ast = parse_source("arc at 200,200 radius 50 start 0 end 180");
        if let AstNode::Scene(children) = ast {
            if let AstNode::Shape(s) = &children[0] {
                assert_eq!(s.kind, "arc");
                assert!(matches!(s.props.get("at"), Some(PropValue::Pair(a, b)) if (*a - 200.0).abs() < 0.001 && (*b - 200.0).abs() < 0.001));
                assert!(matches!(s.props.get("radius"), Some(PropValue::Num(n)) if (*n - 50.0).abs() < 0.001));
                assert!(matches!(s.props.get("start"), Some(PropValue::Num(n)) if n.abs() < 0.001));
                assert!(matches!(s.props.get("end"), Some(PropValue::Num(n)) if (*n - 180.0).abs() < 0.001));
            } else {
                panic!("Expected Shape");
            }
        }
    }

    #[test]
    fn test_curve() {
        let ast = parse_source("curve points [100,100 150,50 200,100] smooth");
        if let AstNode::Scene(children) = ast {
            if let AstNode::Shape(s) = &children[0] {
                assert_eq!(s.kind, "curve");
                assert!(matches!(s.props.get("points"), Some(PropValue::Points(pts)) if pts.len() == 3));
                assert!(matches!(s.props.get("smooth"), Some(PropValue::Num(n)) if (*n - 1.0).abs() < 0.001));
            } else {
                panic!("Expected Shape");
            }
        }
    }

    #[test]
    fn test_curve_sharp() {
        let ast = parse_source("curve points [0,0 50,50 100,0] sharp closed");
        if let AstNode::Scene(children) = ast {
            if let AstNode::Shape(s) = &children[0] {
                assert_eq!(s.kind, "curve");
                assert!(matches!(s.props.get("smooth"), Some(PropValue::Num(n)) if n.abs() < 0.001));
                assert!(matches!(s.props.get("closed"), Some(PropValue::Num(n)) if (*n - 1.0).abs() < 0.001));
            } else {
                panic!("Expected Shape");
            }
        }
    }
}
