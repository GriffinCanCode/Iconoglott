//! Property-based tests for the parser using proptest
//!
//! These tests verify parser invariants across a wide range of generated inputs.

#![cfg(test)]

use proptest::prelude::*;
use super::ast::*;
use super::core::Parser;
use super::super::lexer::{CanvasSize, Lexer};

// ─────────────────────────────────────────────────────────────────────────────
// AST Generators
// ─────────────────────────────────────────────────────────────────────────────

fn arb_canvas_size() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("nano"), Just("micro"), Just("tiny"), Just("small"),
        Just("medium"), Just("large"), Just("xlarge"), Just("huge"),
        Just("massive"), Just("giant"),
    ]
}

fn arb_color() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("#fff".to_string()),
        Just("#000".to_string()),
        Just("#ff0000".to_string()),
        Just("#1a2b3c".to_string()),
        "[0-9a-f]{6}".prop_map(|s| format!("#{}", s)),
    ]
}

fn arb_shape_kind() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("rect"), Just("circle"), Just("ellipse"),
        Just("line"), Just("polygon"), Just("text"),
        Just("arc"), Just("curve"), Just("diamond"),
    ]
}

fn arb_position() -> impl Strategy<Value = (f64, f64)> {
    (0.0..1000.0, 0.0..1000.0)
}

fn arb_size() -> impl Strategy<Value = (f64, f64)> {
    (1.0..500.0, 1.0..500.0)
}

fn arb_radius() -> impl Strategy<Value = f64> {
    1.0..200.0
}

fn arb_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("not keyword", |s| {
        !["canvas", "group", "stack", "row", "graph", "node", "edge", "symbol", "use",
          "rect", "circle", "ellipse", "line", "path", "polygon", "text", "image",
          "arc", "curve", "diamond", "fill", "stroke", "opacity"].contains(&s.as_str())
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Source Generators
// ─────────────────────────────────────────────────────────────────────────────

fn gen_canvas_source(size: &str, fill: &str) -> String {
    format!("canvas {} fill {}", size, fill)
}

fn gen_rect_source(x: f64, y: f64, w: f64, h: f64, fill: &str) -> String {
    format!("rect at {:.0},{:.0} size {:.0}x{:.0} {}", x, y, w, h, fill)
}

fn gen_circle_source(x: f64, y: f64, r: f64, fill: &str) -> String {
    format!("circle at {:.0},{:.0} radius {:.0} {}", x, y, r, fill)
}

fn gen_variable_source(name: &str, value: &str) -> String {
    format!("${} = {}", name, value)
}

// ─────────────────────────────────────────────────────────────────────────────
// Parse Helper
// ─────────────────────────────────────────────────────────────────────────────

fn parse(source: &str) -> (AstNode, Vec<ParseError>) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    (ast, parser.errors)
}

// ─────────────────────────────────────────────────────────────────────────────
// Property Tests
// ─────────────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// Valid canvas commands always parse without errors
    #[test]
    fn canvas_always_parses(size in arb_canvas_size(), fill in arb_color()) {
        let source = gen_canvas_source(size, &fill);
        let (ast, errors) = parse(&source);
        
        prop_assert!(errors.is_empty(), "Canvas should parse without errors: {:?}", errors);
        if let AstNode::Scene(children) = ast {
            prop_assert_eq!(children.len(), 1, "Should have exactly one canvas");
            if let AstNode::Canvas(c) = &children[0] {
                let expected_size = CanvasSize::from_str(size).unwrap();
                prop_assert_eq!(c.size, expected_size);
            } else {
                prop_assert!(false, "Expected Canvas node");
            }
        }
    }

    /// Valid rect commands parse with correct position
    #[test]
    fn rect_position_preserved((x, y) in arb_position(), (w, h) in arb_size(), fill in arb_color()) {
        let source = gen_rect_source(x, y, w, h, &fill);
        let (ast, errors) = parse(&source);
        
        prop_assert!(errors.is_empty(), "Rect should parse without errors: {:?}", errors);
        if let AstNode::Scene(children) = ast {
            prop_assert_eq!(children.len(), 1);
            if let AstNode::Shape(s) = &children[0] {
                prop_assert_eq!(s.kind.as_str(), "rect");
                if let Some(PropValue::Pair(px, py)) = s.props.get("at") {
                    prop_assert!((px - x.floor()).abs() < 1.0, "X position mismatch");
                    prop_assert!((py - y.floor()).abs() < 1.0, "Y position mismatch");
                }
            }
        }
    }

    /// Valid circle commands parse with correct radius
    #[test]
    fn circle_radius_preserved((x, y) in arb_position(), r in arb_radius(), fill in arb_color()) {
        let source = gen_circle_source(x, y, r, &fill);
        let (ast, errors) = parse(&source);
        
        prop_assert!(errors.is_empty(), "Circle should parse without errors: {:?}", errors);
        if let AstNode::Scene(children) = ast {
            if let Some(AstNode::Shape(s)) = children.first() {
                prop_assert_eq!(s.kind.as_str(), "circle");
                if let Some(PropValue::Num(parsed_r)) = s.props.get("radius") {
                    prop_assert!((parsed_r - r.floor()).abs() < 1.0, "Radius mismatch");
                }
            }
        }
    }

    /// Variables are always stored correctly
    #[test]
    fn variable_stored(name in arb_identifier(), color in arb_color()) {
        let source = gen_variable_source(&name, &color);
        let (ast, errors) = parse(&source);
        
        prop_assert!(errors.is_empty(), "Variable should parse without errors: {:?}", errors);
        if let AstNode::Scene(children) = ast {
            if let Some(AstNode::Variable { name: var_name, value }) = children.first() {
                prop_assert_eq!(var_name, &name);
                prop_assert!(value.is_some(), "Variable should have a value");
            }
        }
    }

    /// Parser never panics on arbitrary valid shape syntax
    #[test]
    fn no_panic_on_valid_shapes(
        kind in arb_shape_kind(),
        (x, y) in arb_position(),
        fill in arb_color()
    ) {
        let source = format!("{} at {:.0},{:.0} {}", kind, x, y, fill);
        let result = std::panic::catch_unwind(|| parse(&source));
        prop_assert!(result.is_ok(), "Parser should not panic");
    }

    /// Parser recovers from unknown commands
    #[test]
    fn recovers_from_unknown_command(
        bad_cmd in "[a-z]{3,8}".prop_filter("not valid", |s| {
            !["canvas", "group", "stack", "row", "graph", "rect", "circle",
              "ellipse", "line", "path", "polygon", "text", "image", "arc",
              "curve", "diamond", "node", "edge", "symbol", "use"].contains(&s.as_str())
        }),
        (x, y) in arb_position()
    ) {
        let source = format!("{}\nrect at {:.0},{:.0}", bad_cmd, x, y);
        let (ast, errors) = parse(&source);
        
        // Should have error for unknown command
        prop_assert!(!errors.is_empty(), "Should report unknown command error");
        prop_assert!(errors.iter().any(|e| e.kind == ErrorKind::UnknownCommand));
        
        // Should still parse the valid rect
        if let AstNode::Scene(children) = ast {
            prop_assert!(children.iter().any(|n| matches!(n, AstNode::Shape(s) if s.kind.as_str() == "rect")),
                "Should still parse valid rect after error recovery");
        }
    }

    /// Nested blocks maintain parent-child relationships
    #[test]
    fn nested_blocks_preserved(
        parent_kind in prop_oneof![Just("rect"), Just("group")],
        child_kind in arb_shape_kind(),
        fill in arb_color()
    ) {
        let source = format!("{}\n  {} 50,50 {}", parent_kind, child_kind, fill);
        let (ast, _) = parse(&source);
        
        if let AstNode::Scene(children) = ast {
            if let Some(AstNode::Shape(parent)) = children.first() {
                if parent.kind == "group" || parent.kind == "rect" {
                    // Groups should have children
                    if !parent.children.is_empty() {
                        prop_assert_eq!(parent.children[0].kind.as_str(), child_kind);
                    }
                }
            }
        }
    }

    /// Empty source always produces empty scene
    #[test]
    fn empty_source_empty_scene(whitespace in "[ \t\n]*") {
        let (ast, errors) = parse(&whitespace);
        prop_assert!(errors.is_empty());
        if let AstNode::Scene(children) = ast {
            prop_assert!(children.is_empty(), "Empty/whitespace source should produce empty scene");
        }
    }

    /// Comments are ignored
    #[test]
    fn comments_ignored(comment in "[a-zA-Z0-9 ]{0,20}") {
        let source = format!("// {}\nrect at 100,100", comment);
        let (ast, errors) = parse(&source);
        
        prop_assert!(errors.is_empty());
        if let AstNode::Scene(children) = ast {
            prop_assert_eq!(children.len(), 1);
            prop_assert!(matches!(&children[0], AstNode::Shape(s) if s.kind.as_str() == "rect"));
        }
    }

    /// Multiple shapes are parsed in order
    #[test]
    fn shapes_order_preserved(count in 1usize..5) {
        let source: String = (0..count)
            .map(|i| format!("rect at {},{}", i * 50, i * 50))
            .collect::<Vec<_>>()
            .join("\n");
        
        let (ast, errors) = parse(&source);
        prop_assert!(errors.is_empty());
        
        if let AstNode::Scene(children) = ast {
            prop_assert_eq!(children.len(), count, "Should have exactly {} shapes", count);
        }
    }

    /// Style properties are applied correctly
    #[test]
    fn style_properties_applied(fill in arb_color(), stroke in arb_color(), opacity in 0.0f64..1.0) {
        let source = format!("rect at 100,100\n  fill {}\n  stroke {}\n  opacity {:.2}", fill, stroke, opacity);
        let (ast, errors) = parse(&source);
        
        prop_assert!(errors.is_empty(), "Style parsing failed: {:?}", errors);
        if let AstNode::Scene(children) = ast {
            if let Some(AstNode::Shape(s)) = children.first() {
                prop_assert_eq!(s.style.fill.as_ref(), Some(&fill));
                prop_assert_eq!(s.style.stroke.as_ref(), Some(&stroke));
                prop_assert!((s.style.opacity - opacity).abs() < 0.01);
            }
        }
    }

    /// Transform properties preserve values
    #[test]
    fn transform_preserved(rotate in 0.0f64..360.0, scale in 0.1f64..2.0) {
        let source = format!("rect at 100,100\n  rotate {:.0}\n  scale {:.1},{:.1}", rotate, scale, scale);
        let (ast, errors) = parse(&source);
        
        prop_assert!(errors.is_empty());
        if let AstNode::Scene(children) = ast {
            if let Some(AstNode::Shape(s)) = children.first() {
                prop_assert!((s.transform.rotate - rotate.floor()).abs() < 1.0);
                if let Some((sx, sy)) = s.transform.scale {
                    prop_assert!((sx - scale).abs() < 0.2);
                    prop_assert!((sy - scale).abs() < 0.2);
                }
            }
        }
    }

    /// Polygon points are preserved
    #[test]
    fn polygon_points_preserved(n in 3usize..8) {
        let points: Vec<String> = (0..n)
            .map(|i| format!("{},{}", i * 20, i * 10))
            .collect();
        let source = format!("polygon points [{}]", points.join(" "));
        
        let (ast, errors) = parse(&source);
        prop_assert!(errors.is_empty());
        
        if let AstNode::Scene(children) = ast {
            if let Some(AstNode::Shape(s)) = children.first() {
                if let Some(PropValue::Points(pts)) = s.props.get("points") {
                    prop_assert_eq!(pts.len(), n, "Point count mismatch");
                }
            }
        }
    }

    /// Layout containers accept children
    #[test]
    fn layout_accepts_children(direction in prop_oneof![Just("stack"), Just("row")], gap in 0.0f64..50.0) {
        let source = format!("{} gap {:.0}\n  rect size 50x50\n  circle radius 25", direction, gap);
        let (ast, errors) = parse(&source);
        
        prop_assert!(errors.is_empty());
        if let AstNode::Scene(children) = ast {
            if let Some(AstNode::Shape(s)) = children.first() {
                prop_assert_eq!(s.kind.as_str(), "layout");
                prop_assert_eq!(s.children.len(), 2, "Layout should have 2 children");
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Roundtrip Property Tests
// ─────────────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Parsing is deterministic - same input always produces same output
    #[test]
    fn parsing_is_deterministic(size in arb_canvas_size(), fill in arb_color()) {
        let source = gen_canvas_source(size, &fill);
        let (ast1, errors1) = parse(&source);
        let (ast2, errors2) = parse(&source);
        
        prop_assert_eq!(errors1.len(), errors2.len(), "Error count should be deterministic");
        prop_assert_eq!(format!("{:?}", ast1), format!("{:?}", ast2), "AST should be deterministic");
    }

    /// Error positions are within source bounds
    #[test]
    fn error_positions_valid(bad_cmd in "[a-z]{5,10}") {
        let source = format!("{}\nrect at 100,100", bad_cmd);
        let line_count = source.lines().count();
        
        let (_, errors) = parse(&source);
        
        for error in errors {
            prop_assert!(error.line < line_count, "Error line {} exceeds source lines {}", error.line, line_count);
        }
    }
}

