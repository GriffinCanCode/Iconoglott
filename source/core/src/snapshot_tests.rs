//! Snapshot tests for SVG output using insta
//!
//! These tests verify that rendered SVG output matches expected snapshots.
//! Uses direct Scene construction to test the rendering pipeline.

#![cfg(all(test, any(feature = "python", feature = "bench")))]

use insta::assert_snapshot;
use crate::CanvasSize;
use crate::scene::{
    Scene, Element, Gradient, Filter, Symbol,
    Rect, Circle, Ellipse, Line, Polygon, Text, Diamond, Path,
    Style, Use,
};

// ─────────────────────────────────────────────────────────────────────────────
// Basic Shape Snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_empty_canvas() {
    let scene = Scene::new(CanvasSize::Medium, "#fff".into());
    assert_snapshot!("empty_canvas", scene.render_svg());
}

#[test]
fn snapshot_basic_rect() {
    let mut scene = Scene::new(CanvasSize::Small, "#fff".into());
    scene.push(Element::Rect(Rect {
        x: 10.0, y: 10.0, w: 30.0, h: 30.0, rx: 0.0,
        style: Style::with_fill("#ff0"), transform: None,
    }));
    assert_snapshot!("basic_rect", scene.render_svg());
}

#[test]
fn snapshot_basic_circle() {
    let mut scene = Scene::new(CanvasSize::Small, "#fff".into());
    scene.push(Element::Circle(Circle {
        cx: 24.0, cy: 24.0, r: 16.0,
        style: Style::with_fill("#0af"), transform: None,
    }));
    assert_snapshot!("basic_circle", scene.render_svg());
}

#[test]
fn snapshot_basic_ellipse() {
    let mut scene = Scene::new(CanvasSize::Small, "#fff".into());
    scene.push(Element::Ellipse(Ellipse {
        cx: 24.0, cy: 24.0, rx: 20.0, ry: 12.0,
        style: Style::with_fill("#f0a"), transform: None,
    }));
    assert_snapshot!("basic_ellipse", scene.render_svg());
}

#[test]
fn snapshot_basic_line() {
    let mut scene = Scene::new(CanvasSize::Small, "#fff".into());
    scene.push(Element::Line(Line {
        x1: 10.0, y1: 10.0, x2: 40.0, y2: 40.0,
        style: Style { stroke: Some("#333".into()), stroke_width: 2.0, ..Default::default() },
        transform: None,
    }));
    assert_snapshot!("basic_line", scene.render_svg());
}

#[test]
fn snapshot_basic_polygon() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Polygon(Polygon {
        points: vec![(32.0, 8.0), (56.0, 56.0), (8.0, 56.0)],
        style: Style::with_fill("#fa0"), transform: None,
    }));
    assert_snapshot!("basic_polygon", scene.render_svg());
}

#[test]
fn snapshot_basic_text() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Text(Text {
        x: 32.0, y: 32.0, content: "Hello".into(),
        font: "sans-serif".into(), size: 14.0, weight: "normal".into(), anchor: "start".into(),
        style: Style::with_fill("#333"), transform: None,
    }));
    assert_snapshot!("basic_text", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Style Snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_styled_rect() {
    let mut scene = Scene::new(CanvasSize::Medium, "#f0f0f0".into());
    scene.push(Element::Rect(Rect {
        x: 12.0, y: 12.0, w: 40.0, h: 40.0, rx: 8.0,
        style: Style {
            fill: Some("#3b82f6".into()),
            stroke: Some("#1e40af".into()),
            stroke_width: 2.0,
            opacity: 0.9,
            ..Default::default()
        },
        transform: None,
    }));
    assert_snapshot!("styled_rect", scene.render_svg());
}

#[test]
fn snapshot_gradient_rect() {
    let mut scene = Scene::new(CanvasSize::Medium, "#1a1a2e".into());
    scene.push_gradient(Gradient {
        id: "grad1".into(),
        kind: "linear".into(),
        from_color: "#ff6b6b".into(),
        to_color: "#4ecdc4".into(),
        angle: 45.0,
    });
    scene.push(Element::Rect(Rect {
        x: 8.0, y: 8.0, w: 48.0, h: 48.0, rx: 4.0,
        style: Style { fill: Some("url(#grad1)".into()), ..Default::default() },
        transform: None,
    }));
    assert_snapshot!("gradient_rect", scene.render_svg());
}

#[test]
fn snapshot_shadow_circle() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push_filter(Filter {
        id: "shadow1".into(),
        kind: "shadow".into(),
        dx: 2.0, dy: 4.0, blur: 8.0,
        color: "#0004".into(),
    });
    scene.push(Element::Circle(Circle {
        cx: 32.0, cy: 32.0, r: 20.0,
        style: Style { fill: Some("#8b5cf6".into()), filter: Some("shadow1".into()), ..Default::default() },
        transform: None,
    }));
    assert_snapshot!("shadow_circle", scene.render_svg());
}

#[test]
fn snapshot_radial_gradient() {
    let mut scene = Scene::new(CanvasSize::Medium, "#0a0a1a".into());
    scene.push_gradient(Gradient {
        id: "radial1".into(),
        kind: "radial".into(),
        from_color: "#fff".into(),
        to_color: "#000".into(),
        angle: 0.0,
    });
    scene.push(Element::Circle(Circle {
        cx: 32.0, cy: 32.0, r: 24.0,
        style: Style { fill: Some("url(#radial1)".into()), ..Default::default() },
        transform: None,
    }));
    assert_snapshot!("radial_gradient", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Transform Snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_rotated_rect() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Rect(Rect {
        x: 20.0, y: 20.0, w: 24.0, h: 24.0, rx: 0.0,
        style: Style::with_fill("#f59e0b"),
        transform: Some("rotate(45 32 32)".into()),
    }));
    assert_snapshot!("rotated_rect", scene.render_svg());
}

#[test]
fn snapshot_scaled_circle() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Circle(Circle {
        cx: 32.0, cy: 32.0, r: 16.0,
        style: Style::with_fill("#10b981"),
        transform: Some("scale(1.5 0.75)".into()),
    }));
    assert_snapshot!("scaled_circle", scene.render_svg());
}

#[test]
fn snapshot_translated_shape() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Rect(Rect {
        x: 10.0, y: 10.0, w: 20.0, h: 20.0, rx: 0.0,
        style: Style::with_fill("#ef4444"),
        transform: Some("translate(15 15)".into()),
    }));
    assert_snapshot!("translated_shape", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Composition Snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_multiple_shapes() {
    let mut scene = Scene::new(CanvasSize::Medium, "#1e293b".into());
    scene.push(Element::Rect(Rect {
        x: 8.0, y: 8.0, w: 20.0, h: 20.0, rx: 0.0,
        style: Style::with_fill("#ef4444"), transform: None,
    }));
    scene.push(Element::Circle(Circle {
        cx: 48.0, cy: 24.0, r: 12.0,
        style: Style::with_fill("#10b981"), transform: None,
    }));
    scene.push(Element::Rect(Rect {
        x: 36.0, y: 40.0, w: 20.0, h: 16.0, rx: 0.0,
        style: Style::with_fill("#3b82f6"), transform: None,
    }));
    assert_snapshot!("multiple_shapes", scene.render_svg());
}

#[test]
fn snapshot_nested_group() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Group(vec![
        Element::Rect(Rect {
            x: 10.0, y: 10.0, w: 44.0, h: 44.0, rx: 0.0,
            style: Style::with_fill("#f0f0f0"), transform: None,
        }),
        Element::Circle(Circle {
            cx: 32.0, cy: 32.0, r: 16.0,
            style: Style::with_fill("#3b82f6"), transform: None,
        }),
    ], None));
    assert_snapshot!("nested_group", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Canvas Size Snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_nano_canvas() {
    let scene = Scene::new(CanvasSize::Nano, "#3b82f6".into());
    assert_snapshot!("canvas_nano", scene.render_svg());
}

#[test]
fn snapshot_giant_canvas() {
    let mut scene = Scene::new(CanvasSize::Giant, "#0f172a".into());
    scene.push(Element::Circle(Circle {
        cx: 256.0, cy: 256.0, r: 128.0,
        style: Style::with_fill("#3b82f6"), transform: None,
    }));
    assert_snapshot!("canvas_giant", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Diamond Shape Snapshot
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_diamond() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Diamond(Diamond {
        cx: 32.0, cy: 32.0, w: 40.0, h: 30.0,
        style: Style {
            fill: Some("#8b5cf6".into()),
            stroke: Some("#6d28d9".into()),
            stroke_width: 2.0,
            ..Default::default()
        },
        transform: None,
    }));
    assert_snapshot!("diamond", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Text Styling Snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_text_bold() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Text(Text {
        x: 32.0, y: 32.0, content: "Bold".into(),
        font: "sans-serif".into(), size: 16.0, weight: "bold".into(), anchor: "middle".into(),
        style: Style::with_fill("#1f2937"), transform: None,
    }));
    assert_snapshot!("text_bold", scene.render_svg());
}

#[test]
fn snapshot_text_anchors() {
    let mut scene = Scene::new(CanvasSize::Large, "#fff".into());
    scene.push(Element::Text(Text {
        x: 8.0, y: 24.0, content: "Start".into(),
        font: "sans-serif".into(), size: 12.0, weight: "normal".into(), anchor: "start".into(),
        style: Style::with_fill("#333"), transform: None,
    }));
    scene.push(Element::Text(Text {
        x: 48.0, y: 48.0, content: "Center".into(),
        font: "sans-serif".into(), size: 12.0, weight: "normal".into(), anchor: "middle".into(),
        style: Style::with_fill("#333"), transform: None,
    }));
    scene.push(Element::Text(Text {
        x: 88.0, y: 72.0, content: "End".into(),
        font: "sans-serif".into(), size: 12.0, weight: "normal".into(), anchor: "end".into(),
        style: Style::with_fill("#333"), transform: None,
    }));
    assert_snapshot!("text_anchors", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Symbol/Use Snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_symbol_use() {
    let mut scene = Scene::new(CanvasSize::Large, "#1a1a2e".into());
    scene.push_symbol(Symbol {
        id: "dot".into(),
        viewbox: None,
        children: vec![Element::Circle(Circle {
            cx: 8.0, cy: 8.0, r: 8.0,
            style: Style::with_fill("#10b981"), transform: None,
        })],
    });
    for (x, y) in [(20.0, 20.0), (50.0, 20.0), (80.0, 20.0), (35.0, 50.0), (65.0, 50.0)] {
        scene.push(Element::Use(Use {
            href: "dot".into(), x, y, width: None, height: None,
            style: Style::default(), transform: None,
        }));
    }
    assert_snapshot!("symbol_use", scene.render_svg());
}

#[test]
fn snapshot_symbol_with_transform() {
    let mut scene = Scene::new(CanvasSize::Medium, "#0f172a".into());
    scene.push_symbol(Symbol {
        id: "star".into(),
        viewbox: Some((0.0, 0.0, 24.0, 24.0)),
        children: vec![Element::Polygon(Polygon {
            points: vec![
                (12.0, 2.0), (15.0, 9.0), (22.0, 9.0), (17.0, 14.0), (19.0, 22.0),
                (12.0, 17.0), (5.0, 22.0), (7.0, 14.0), (2.0, 9.0), (9.0, 9.0),
            ],
            style: Style::with_fill("#f59e0b"), transform: None,
        })],
    });
    scene.push(Element::Use(Use {
        href: "star".into(), x: 8.0, y: 20.0, width: None, height: None,
        style: Style::default(), transform: None,
    }));
    scene.push(Element::Use(Use {
        href: "star".into(), x: 32.0, y: 20.0, width: None, height: None,
        style: Style::default(), transform: Some("scale(1.5)".into()),
    }));
    scene.push(Element::Use(Use {
        href: "star".into(), x: 56.0, y: 24.0, width: None, height: None,
        style: Style::default(), transform: Some("rotate(15)".into()),
    }));
    assert_snapshot!("symbol_with_transform", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Path Snapshot
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_path() {
    let mut scene = Scene::new(CanvasSize::Medium, "#fff".into());
    scene.push(Element::Path(Path {
        d: "M 10 30 Q 25 10 40 30 T 70 30".into(),
        style: Style {
            fill: None,
            stroke: Some("#3b82f6".into()),
            stroke_width: 2.0,
            ..Default::default()
        },
        transform: None,
        bounds_hint: None,
    }));
    assert_snapshot!("path", scene.render_svg());
}

// ─────────────────────────────────────────────────────────────────────────────
// Opacity Snapshot
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_opacity_layers() {
    let mut scene = Scene::new(CanvasSize::Medium, "#1e293b".into());
    scene.push(Element::Circle(Circle {
        cx: 24.0, cy: 32.0, r: 20.0,
        style: Style { fill: Some("#ef4444".into()), opacity: 0.7, ..Default::default() },
        transform: None,
    }));
    scene.push(Element::Circle(Circle {
        cx: 40.0, cy: 32.0, r: 20.0,
        style: Style { fill: Some("#3b82f6".into()), opacity: 0.7, ..Default::default() },
        transform: None,
    }));
    assert_snapshot!("opacity_layers", scene.render_svg());
}
