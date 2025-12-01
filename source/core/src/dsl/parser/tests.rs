//! Tests for the parser

#![cfg(test)]

use super::ast::*;
use super::core::Parser;
use super::super::lexer::{CanvasSize, Lexer};

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

