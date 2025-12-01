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

fn parse_with_errors(source: &str) -> (AstNode, Vec<ParseError>) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    (ast, parser.errors)
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

// ─────────────────────────────────────────────────────────────────────────────
// Error Recovery Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_error_recovery_unknown_command() {
    let (ast, errors) = parse_with_errors("foobar\nrect at 100,100");
    
    // Should have one error for unknown command
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind, ErrorKind::UnknownCommand);
    assert!(errors[0].message.contains("foobar"));
    
    // Should still parse the valid rect
    if let AstNode::Scene(children) = ast {
        assert_eq!(children.len(), 1);
        if let AstNode::Shape(s) = &children[0] {
            assert_eq!(s.kind, "rect");
        }
    }
}

#[test]
fn test_error_recovery_multiple_errors() {
    let (ast, errors) = parse_with_errors("badcmd\nrect at 100,100\nanotherbad\ncircle 50");
    
    // Should collect multiple errors
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|e| e.kind == ErrorKind::UnknownCommand));
    
    // Should parse both valid shapes
    if let AstNode::Scene(children) = ast {
        assert_eq!(children.len(), 2);
        assert!(matches!(&children[0], AstNode::Shape(s) if s.kind == "rect"));
        assert!(matches!(&children[1], AstNode::Shape(s) if s.kind == "circle"));
    }
}

#[test]
fn test_error_recovery_invalid_canvas_size() {
    let (ast, errors) = parse_with_errors("canvas invalidsize\nrect at 50,50");
    
    // Should have error for invalid size
    assert!(!errors.is_empty());
    
    // Should still parse with default canvas and the rect
    if let AstNode::Scene(children) = ast {
        assert!(children.len() >= 1);
    }
}

#[test]
fn test_error_recovery_block_with_errors() {
    let (ast, errors) = parse_with_errors("rect at 100,100\n  fill #ff0\n  badprop value\n  stroke #000");
    
    // Should have error for unknown property
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.kind == ErrorKind::InvalidProperty));
    
    // Should still parse valid properties
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            assert_eq!(s.style.fill, Some("#ff0".into()));
            assert_eq!(s.style.stroke, Some("#000".into()));
        }
    }
}

#[test]
fn test_error_recovery_graph_block() {
    let (ast, errors) = parse_with_errors("graph\n  node \"A\"\n  badcmd\n  node \"B\"");
    
    // Should have error for bad command in graph
    assert!(!errors.is_empty());
    
    // Should parse both valid nodes
    if let AstNode::Scene(children) = ast {
        if let AstNode::Graph(g) = &children[0] {
            assert_eq!(g.nodes.len(), 2);
        }
    }
}

#[test]
fn test_error_has_suggestion() {
    let (_, errors) = parse_with_errors("rekt at 100,100"); // typo: rekt instead of rect
    
    assert!(!errors.is_empty());
    // Should have a suggestion
    assert!(errors[0].suggestion.is_some());
}

#[test]
fn test_error_spans() {
    let (_, errors) = parse_with_errors("rect at 100,100\nbadcommand");
    
    assert!(!errors.is_empty());
    // Error should be on line 1 (0-indexed)
    assert_eq!(errors[0].line, 1);
    assert_eq!(errors[0].span.start_line, 1);
}

#[test]
fn test_error_codes() {
    let (_, errors) = parse_with_errors("unknowncmd");
    
    assert!(!errors.is_empty());
    assert_eq!(errors[0].kind.code(), "E002"); // UnknownCommand
}

#[test]
fn test_unclosed_points_recovery() {
    let (ast, errors) = parse_with_errors("polygon points [100,100 200,200\nrect at 50,50");
    
    // Should have error for unclosed points
    assert!(!errors.is_empty());
    
    // Should still attempt to parse subsequent content
    if let AstNode::Scene(children) = ast {
        assert!(!children.is_empty());
    }
}

