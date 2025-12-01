//! Tests for the parser

#![cfg(test)]

use super::ast::*;
use super::core::Parser;
use super::symbols::resolve;
use super::layout::{LayoutSolver, LayoutContext};
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

// ─────────────────────────────────────────────────────────────────────────────
// Symbol Table / Resolution Pass Tests
// ─────────────────────────────────────────────────────────────────────────────

fn parse_and_resolve(source: &str) -> (AstNode, Vec<ParseError>) {
    let (ast, mut parse_errors) = parse_with_errors(source);
    let result = resolve(ast);
    parse_errors.extend(result.errors);
    (result.ast, parse_errors)
}

#[test]
fn test_undefined_variable_error() {
    let (_, errors) = parse_and_resolve("circle $undefined");
    
    // Should have error for undefined variable
    assert!(errors.iter().any(|e| e.kind == ErrorKind::UndefinedVariable));
    assert!(errors.iter().any(|e| e.message.contains("undefined")));
}

#[test]
fn test_defined_variable_resolved() {
    let (ast, errors) = parse_and_resolve("$accent = #ff0\ncircle $accent");
    
    // Should have no resolution errors
    let resolution_errors: Vec<_> = errors.iter().filter(|e| e.kind == ErrorKind::UndefinedVariable).collect();
    assert!(resolution_errors.is_empty(), "Unexpected errors: {:?}", resolution_errors);
    
    // Variable should be resolved in the shape
    if let AstNode::Scene(children) = ast {
        // Check that circle exists and has the resolved color
        let shape = children.iter().find_map(|n| match n {
            AstNode::Shape(s) if s.kind == "circle" => Some(s),
            _ => None
        });
        assert!(shape.is_some(), "Should have a circle shape");
    }
}

#[test]
fn test_variable_forward_reference() {
    // Variable used before definition
    let (_, errors) = parse_and_resolve("circle $accent\n$accent = #ff0");
    
    // With symbol table pass, forward references should work
    // The symbol table collects all definitions first
    let resolution_errors: Vec<_> = errors.iter().filter(|e| e.kind == ErrorKind::UndefinedVariable).collect();
    assert!(resolution_errors.is_empty(), "Forward reference should work: {:?}", resolution_errors);
}

#[test]
fn test_undefined_variable_has_suggestion() {
    let (_, errors) = parse_and_resolve("rect $missing");
    
    let undefined_err = errors.iter().find(|e| e.kind == ErrorKind::UndefinedVariable);
    assert!(undefined_err.is_some());
    assert!(undefined_err.unwrap().suggestion.is_some(), "Should have a suggestion for undefined variable");
}

#[test]
fn test_multiple_undefined_variables() {
    let (_, errors) = parse_and_resolve("rect $a\ncircle $b\ntext $c");
    
    let undefined_count = errors.iter().filter(|e| e.kind == ErrorKind::UndefinedVariable).count();
    assert_eq!(undefined_count, 3, "Should report all undefined variables");
}

#[test]
fn test_variable_in_nested_block() {
    let (ast, errors) = parse_and_resolve("$color = #ff0\nrect at 100,100\n  fill $color");
    
    let resolution_errors: Vec<_> = errors.iter().filter(|e| e.kind == ErrorKind::UndefinedVariable).collect();
    assert!(resolution_errors.is_empty(), "Block variable should resolve: {:?}", resolution_errors);
    
    if let AstNode::Scene(children) = ast {
        if let Some(AstNode::Shape(shape)) = children.iter().find(|n| matches!(n, AstNode::Shape(s) if s.kind == "rect")) {
            assert_eq!(shape.style.fill, Some("#ff0".into()));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Layout System Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_layout_basic_stack() {
    let ast = parse_source("stack vertical gap 10");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            assert_eq!(s.kind, "layout");
            assert!(matches!(s.props.get("direction"), Some(PropValue::Str(d)) if d == "vertical"));
            assert!(matches!(s.props.get("gap"), Some(PropValue::Num(n)) if (*n - 10.0).abs() < 0.001));
        } else {
            panic!("Expected Shape");
        }
    }
}

#[test]
fn test_layout_row_with_justify() {
    let ast = parse_source("row justify center align center");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            assert_eq!(s.kind, "layout");
            assert!(matches!(s.props.get("direction"), Some(PropValue::Str(d)) if d == "horizontal"));
            assert!(matches!(s.props.get("justify"), Some(PropValue::Str(j)) if j == "center"));
            assert!(matches!(s.props.get("align"), Some(PropValue::Str(a)) if a == "center"));
        }
    }
}

#[test]
fn test_layout_justify_space_between() {
    let ast = parse_source("row justify space-between");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            assert!(matches!(s.props.get("justify"), Some(PropValue::Str(j)) if j == "spacebetween"));
        }
    }
}

#[test]
fn test_layout_center_shorthand() {
    let ast = parse_source("stack center");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            assert!(matches!(s.props.get("justify"), Some(PropValue::Str(j)) if j == "center"));
            assert!(matches!(s.props.get("align"), Some(PropValue::Str(a)) if a == "center"));
        }
    }
}

#[test]
fn test_layout_percentage_size() {
    let ast = parse_source("stack size 50%x100%");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            if let Some(PropValue::DimPair(dp)) = s.props.get("size") {
                assert!(matches!(dp.width, Dimension::Percent(p) if (p - 50.0).abs() < 0.001));
                assert!(matches!(dp.height, Dimension::Percent(p) if (p - 100.0).abs() < 0.001));
            } else {
                panic!("Expected DimPair size");
            }
        }
    }
}

#[test]
fn test_layout_percentage_width() {
    let ast = parse_source("stack width 75%");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            if let Some(PropValue::Dim(d)) = s.props.get("width") {
                assert!(matches!(d, Dimension::Percent(p) if (p - 75.0).abs() < 0.001));
            } else {
                panic!("Expected Dim width");
            }
        }
    }
}

#[test]
fn test_layout_auto_dimension() {
    let ast = parse_source("stack width auto height 50");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            if let Some(PropValue::Dim(d)) = s.props.get("width") {
                assert!(matches!(d, Dimension::Auto));
            } else {
                panic!("Expected Dim width");
            }
            if let Some(PropValue::Dim(d)) = s.props.get("height") {
                assert!(matches!(d, Dimension::Px(h) if (h - 50.0).abs() < 0.001));
            }
        }
    }
}

#[test]
fn test_layout_with_children() {
    let ast = parse_source("stack gap 10\n  rect size 50x50\n  circle radius 25");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            assert_eq!(s.kind, "layout");
            assert_eq!(s.children.len(), 2);
            assert_eq!(s.children[0].kind, "rect");
            assert_eq!(s.children[1].kind, "circle");
        }
    }
}

#[test]
fn test_layout_nested() {
    let ast = parse_source("row gap 20\n  stack gap 10\n    rect size 30x30\n  rect size 50x50");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            assert_eq!(s.kind, "layout");
            assert_eq!(s.children.len(), 2);
            
            // First child is a nested stack
            let nested = &s.children[0];
            assert_eq!(nested.kind, "layout");
            assert!(matches!(nested.props.get("direction"), Some(PropValue::Str(d)) if d == "vertical"));
        }
    }
}

#[test]
fn test_layout_percentage_position() {
    let ast = parse_source("stack at 50%,25%");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            if let Some(PropValue::PercentPair(x, y)) = s.props.get("at") {
                assert!((*x - 50.0).abs() < 0.001, "x should be 50%, got {}", x);
                assert!((*y - 25.0).abs() < 0.001, "y should be 25%, got {}", y);
            } else {
                panic!("Expected PercentPair for at");
            }
        }
    }
}

#[test]
fn test_layout_solver_percentages() {
    let mut shape = AstShape::new("rect");
    shape.props.insert("width".into(), PropValue::Dim(Dimension::Percent(50.0)));
    shape.props.insert("height".into(), PropValue::Dim(Dimension::Px(40.0)));
    
    let ctx = LayoutContext::new(200.0, 100.0);
    let solver = LayoutSolver::new();
    let rect = solver.resolve(&shape, &mut ctx.clone());
    
    assert!((rect.width - 100.0).abs() < 0.001, "50% of 200 = 100, got {}", rect.width);
    assert!((rect.height - 40.0).abs() < 0.001, "height = 40, got {}", rect.height);
}

#[test]
fn test_layout_solver_center_constraint() {
    let mut shape = AstShape::new("rect");
    shape.props.insert("_center_x".into(), PropValue::Num(1.0));
    shape.props.insert("_center_y".into(), PropValue::Num(1.0));
    shape.props.insert("width".into(), PropValue::Dim(Dimension::Px(60.0)));
    shape.props.insert("height".into(), PropValue::Dim(Dimension::Px(40.0)));
    
    let ctx = LayoutContext::new(200.0, 100.0);
    let solver = LayoutSolver::new();
    let rect = solver.resolve(&shape, &mut ctx.clone());
    
    // Center of 200x100 for 60x40 shape: x=(200-60)/2=70, y=(100-40)/2=30
    assert!((rect.x - 70.0).abs() < 0.001, "x should be 70, got {}", rect.x);
    assert!((rect.y - 30.0).abs() < 0.001, "y should be 30, got {}", rect.y);
}

#[test]
fn test_layout_wrap_property() {
    let ast = parse_source("row wrap");
    if let AstNode::Scene(children) = ast {
        if let AstNode::Shape(s) = &children[0] {
            assert!(matches!(s.props.get("wrap"), Some(PropValue::Num(n)) if *n > 0.0));
        }
    }
}

