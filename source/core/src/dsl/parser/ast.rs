//! AST types for the iconoglott DSL

use super::super::lexer::{CanvasSize, TokenValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

#[cfg(feature = "python")]
use pyo3::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// AST Types
// ─────────────────────────────────────────────────────────────────────────────

/// Style properties for shapes
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub struct AstTransform {
    pub translate: Option<(f64, f64)>,
    pub rotate: f64,
    pub scale: Option<(f64, f64)>,
    pub origin: Option<(f64, f64)>,
}

/// Node definition for graphs/flowcharts
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct GraphNode {
    pub id: String,
    pub shape: String,       // rect, circle, ellipse, diamond
    pub label: Option<String>,
    pub at: Option<(f64, f64)>,
    pub size: Option<(f64, f64)>,
    pub style: AstStyle,
}

impl Default for GraphNode {
    fn default() -> Self {
        Self { id: String::new(), shape: "rect".into(), label: None, at: None, size: None, style: AstStyle::new() }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl GraphNode {
    #[new]
    #[pyo3(signature = (id, shape="rect".to_string()))]
    fn py_new(id: String, shape: String) -> Self { Self { id, shape, ..Default::default() } }
}

/// Edge/connector between nodes
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub style: String,       // straight, curved, orthogonal
    pub arrow: String,       // none, forward, backward, both
    pub label: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f64,
}

impl Default for GraphEdge {
    fn default() -> Self {
        Self { from: String::new(), to: String::new(), style: "straight".into(), arrow: "forward".into(), label: None, stroke: Some("#333".into()), stroke_width: 2.0 }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl GraphEdge {
    #[new]
    fn py_new(from: String, to: String) -> Self { Self { from, to, ..Default::default() } }
}

/// Graph container with layout
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
pub struct AstGraph {
    pub layout: String,      // hierarchical, force, grid, tree, manual
    pub direction: String,   // vertical, horizontal
    pub spacing: f64,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl Default for AstGraph {
    fn default() -> Self {
        Self { layout: "manual".into(), direction: "vertical".into(), spacing: 50.0, nodes: Vec::new(), edges: Vec::new() }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl AstGraph {
    #[new]
    fn py_new() -> Self { Self::default() }
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AstNode {
    Scene(Vec<AstNode>),
    Canvas(AstCanvas),
    Shape(AstShape),
    Graph(AstGraph),
    Variable { name: String, value: Option<TokenValue> },
}

/// Error severity levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Hint,
}

/// Error categories for structured diagnostics
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass)]
pub enum ErrorKind {
    UnexpectedToken,
    UnknownCommand,
    InvalidValue,
    MissingToken,
    InvalidIndentation,
    UnterminatedBlock,
    InvalidProperty,
}

impl ErrorKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::UnexpectedToken => "E001",
            Self::UnknownCommand => "E002",
            Self::InvalidValue => "E003",
            Self::MissingToken => "E004",
            Self::InvalidIndentation => "E005",
            Self::UnterminatedBlock => "E006",
            Self::InvalidProperty => "E007",
        }
    }
}

/// Source span for error locations
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all))]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn point(line: usize, col: usize) -> Self {
        Self { start_line: line, start_col: col, end_line: line, end_col: col + 1 }
    }

    pub fn range(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self { start_line, start_col, end_line, end_col }
    }
}

/// Parse error with recovery context
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[cfg_attr(feature = "python", pyclass(get_all))]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub kind: ErrorKind,
    pub severity: ErrorSeverity,
    pub span: Span,
    pub suggestion: Option<String>,
    pub recovered: bool,
}

impl ParseError {
    pub fn new(message: impl Into<String>, kind: ErrorKind, line: usize, col: usize) -> Self {
        Self {
            message: message.into(),
            line, col,
            kind,
            severity: ErrorSeverity::Error,
            span: Span::point(line, col),
            suggestion: None,
            recovered: false,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self { self.span = span; self }
    pub fn with_suggestion(mut self, s: impl Into<String>) -> Self { self.suggestion = Some(s.into()); self }
    pub fn with_severity(mut self, sev: ErrorSeverity) -> Self { self.severity = sev; self }
    pub fn as_recovered(mut self) -> Self { self.recovered = true; self }
}

#[cfg(feature = "python")]
#[pymethods]
impl ParseError {
    fn __repr__(&self) -> String {
        format!("ParseError[{}]({:?}, {}:{}{})", 
            self.kind.code(), self.message, self.line, self.col,
            self.suggestion.as_ref().map(|s| format!(", suggestion={:?}", s)).unwrap_or_default())
    }
    
    #[getter]
    fn code(&self) -> &'static str { self.kind.code() }
}

