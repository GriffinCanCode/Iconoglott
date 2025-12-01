//! Python bindings for the parser

#![cfg(feature = "python")]

use super::ast::*;
use super::core::Parser;
use super::super::lexer::TokenValue;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[pymethods]
impl Parser {
    #[new]
    fn py_new(tokens: Vec<super::super::lexer::Token>) -> Self {
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
pub fn ast_node_to_py(py: Python<'_>, node: &AstNode) -> PyObject {
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
        AstNode::Graph(g) => {
            dict.set_item("Graph", ast_graph_to_py(py, g)).ok();
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

/// Convert AstGraph to Python dict
pub fn ast_graph_to_py(py: Python<'_>, graph: &AstGraph) -> PyObject {
    let dict = PyDict::new(py);
    dict.set_item("layout", &graph.layout).ok();
    dict.set_item("direction", &graph.direction).ok();
    dict.set_item("spacing", graph.spacing).ok();
    let nodes = PyList::new(py, graph.nodes.iter().map(|n| graph_node_to_py(py, n)));
    dict.set_item("nodes", nodes).ok();
    let edges = PyList::new(py, graph.edges.iter().map(|e| graph_edge_to_py(py, e)));
    dict.set_item("edges", edges).ok();
    dict.into()
}

pub fn graph_node_to_py(py: Python<'_>, node: &GraphNode) -> PyObject {
    let dict = PyDict::new(py);
    dict.set_item("id", &node.id).ok();
    dict.set_item("shape", &node.shape).ok();
    dict.set_item("label", &node.label).ok();
    dict.set_item("at", node.at).ok();
    dict.set_item("size", node.size).ok();
    dict.set_item("fill", &node.style.fill).ok();
    dict.set_item("stroke", &node.style.stroke).ok();
    dict.into()
}

pub fn graph_edge_to_py(py: Python<'_>, edge: &GraphEdge) -> PyObject {
    let dict = PyDict::new(py);
    dict.set_item("from", &edge.from).ok();
    dict.set_item("to", &edge.to).ok();
    dict.set_item("style", &edge.style).ok();
    dict.set_item("arrow", &edge.arrow).ok();
    dict.set_item("label", &edge.label).ok();
    dict.set_item("stroke", &edge.stroke).ok();
    dict.set_item("stroke_width", edge.stroke_width).ok();
    dict.into()
}

/// Convert AstShape to Python dict
pub fn ast_shape_to_py(py: Python<'_>, shape: &AstShape) -> PyObject {
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
pub fn prop_value_to_py(py: Python<'_>, val: &PropValue) -> PyObject {
    match val {
        PropValue::None => py.None(),
        PropValue::Str(s) => s.into_py(py),
        PropValue::Num(n) => n.into_py(py),
        PropValue::Pair(a, b) => (*a, *b).into_py(py),
        PropValue::Points(pts) => PyList::new(py, pts.iter().map(|(a, b)| (*a, *b))).into(),
    }
}

/// Convert TokenValue to Python object
pub fn token_value_to_py(py: Python<'_>, val: Option<&TokenValue>) -> PyObject {
    match val {
        None | Some(TokenValue::None) => py.None(),
        Some(TokenValue::Str(s)) => s.into_py(py),
        Some(TokenValue::Num(n)) => n.into_py(py),
        Some(TokenValue::Pair(a, b)) => (*a, *b).into_py(py),
    }
}

