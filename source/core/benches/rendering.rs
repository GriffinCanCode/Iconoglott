//! Criterion benchmarks for iconoglott-core
//!
//! Run with: cargo bench --features bench
//!
//! Benchmarks:
//! - Scene construction (N elements)
//! - Full render pipeline (DSL -> SVG string)
//! - Incremental diff (changing 1 element in N)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use iconoglott_core::{
    Lexer, Parser, AstNode, CanvasSize,
    Scene, Element, Rect, Circle, Style,
    IndexedScene, render,
};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn make_style(fill: &str) -> Style {
    Style {
        fill: Some(fill.into()),
        opacity: 1.0,
        stroke_width: 1.0,
        ..Style::default()
    }
}

fn make_rect(i: usize) -> Rect {
    Rect {
        x: (i % 100) as f32 * 10.0,
        y: (i / 100) as f32 * 10.0,
        w: 50.0,
        h: 50.0,
        rx: 0.0,
        style: make_style("#ff0"),
        transform: None,
    }
}

fn make_circle(i: usize) -> Circle {
    Circle {
        cx: (i % 100) as f32 * 10.0 + 25.0,
        cy: (i / 100) as f32 * 10.0 + 25.0,
        r: 20.0,
        style: make_style("#0ff"),
        transform: None,
    }
}

fn build_scene_with_n_elements(n: usize) -> Scene {
    let mut scene = Scene::new(CanvasSize::Giant, "#1a1a2e".into());
    for i in 0..n {
        if i % 2 == 0 {
            scene.push(Element::Rect(make_rect(i)));
        } else {
            scene.push(Element::Circle(make_circle(i)));
        }
    }
    scene
}

fn generate_dsl_source(n: usize) -> String {
    let mut src = String::with_capacity(n * 40);
    src.push_str("canvas giant fill #1a1a2e\n");
    for i in 0..n {
        let x = (i % 100) * 10;
        let y = (i / 100) * 10;
        if i % 2 == 0 {
            src.push_str(&format!("rect at {},{} size 50x50 #ff0\n", x, y));
        } else {
            src.push_str(&format!("circle at {},{} radius 20 #0ff\n", x + 25, y + 25));
        }
    }
    src
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Scene Construction
// ─────────────────────────────────────────────────────────────────────────────

fn bench_scene_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_construction");
    
    for count in [10, 100, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("elements", count),
            count,
            |b, &n| {
                b.iter(|| {
                    let scene = build_scene_with_n_elements(n);
                    black_box(scene.elements().len())
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Scene Rendering (to SVG)
// ─────────────────────────────────────────────────────────────────────────────

fn bench_scene_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_render");
    
    for count in [10, 100, 1000].iter() {
        let scene = build_scene_with_n_elements(*count);
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("to_svg", count),
            &scene,
            |b, scene| {
                b.iter(|| {
                    let svg = scene.render_svg();
                    black_box(svg.len())
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Full Render Pipeline (DSL -> SVG)
// ─────────────────────────────────────────────────────────────────────────────

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");
    
    for count in [10, 100, 500].iter() {
        let source = generate_dsl_source(*count);
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("dsl_to_svg", count),
            &source,
            |b, src| {
                b.iter(|| {
                    // Lex
                    let mut lexer = Lexer::new(src);
                    let tokens = lexer.tokenize();
                    
                    // Parse
                    let mut parser = Parser::new(tokens);
                    let ast = parser.parse();
                    
                    // Convert AST to scene (simplified - just count nodes)
                    let node_count = match &ast {
                        AstNode::Scene(nodes) => nodes.len(),
                        _ => 0,
                    };
                    
                    black_box(node_count)
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Lexer
// ─────────────────────────────────────────────────────────────────────────────

fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    
    for count in [10, 100, 500].iter() {
        let source = generate_dsl_source(*count);
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("tokenize", count),
            &source,
            |b, src| {
                b.iter(|| {
                    let mut lexer = Lexer::new(src);
                    let tokens = lexer.tokenize();
                    black_box(tokens.len())
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Parser
// ─────────────────────────────────────────────────────────────────────────────

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    
    for count in [10, 100, 500].iter() {
        let source = generate_dsl_source(*count);
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        
        group.throughput(Throughput::Elements(tokens.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parse", count),
            &tokens,
            |b, toks| {
                b.iter(|| {
                    let mut parser = Parser::new(toks.clone());
                    let ast = parser.parse();
                    black_box(ast)
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Scene Indexing
// ─────────────────────────────────────────────────────────────────────────────

fn bench_scene_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_indexing");
    
    for count in [10, 100, 1000, 5000].iter() {
        let scene = build_scene_with_n_elements(*count);
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("index", count),
            &scene,
            |b, scene| {
                b.iter(|| {
                    let indexed = IndexedScene::from_scene(scene);
                    black_box(indexed.len())
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Incremental Diff (identical scenes)
// ─────────────────────────────────────────────────────────────────────────────

fn bench_diff_identical(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_identical");
    
    for count in [10, 100, 1000].iter() {
        let scene1 = build_scene_with_n_elements(*count);
        let scene2 = build_scene_with_n_elements(*count);
        
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("no_change", count),
            &(scene1, scene2),
            |b, (s1, s2)| {
                b.iter(|| {
                    let result = render::diff(s1, s2);
                    black_box(result.is_empty())
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Incremental Diff (single element changed)
// ─────────────────────────────────────────────────────────────────────────────

fn bench_diff_single_change(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_single_change");
    
    for count in [10, 100, 1000].iter() {
        let scene1 = build_scene_with_n_elements(*count);
        let mut scene2 = build_scene_with_n_elements(*count);
        
        // Modify the middle element
        let mid = *count / 2;
        if mid < scene2.elements().len() {
            let els = scene2.elements_mut();
            els[mid] = Element::Rect(Rect {
                x: 999.0,
                y: 999.0,
                w: 100.0,
                h: 100.0,
                rx: 10.0,
                style: make_style("#f00"),
                transform: None,
            });
        }
        
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("one_change", count),
            &(scene1, scene2),
            |b, (s1, s2)| {
                b.iter(|| {
                    let result = render::diff(s1, s2);
                    black_box(result.ops.len())
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Incremental Diff (all elements changed)
// ─────────────────────────────────────────────────────────────────────────────

fn bench_diff_all_changed(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_all_changed");
    
    for count in [10, 100, 500].iter() {
        let scene1 = build_scene_with_n_elements(*count);
        
        // Different scene with all different elements (shifted positions)
        let mut scene2 = Scene::new(CanvasSize::Giant, "#1a1a2e".into());
        for i in 0..*count {
            scene2.push(Element::Rect(Rect {
                x: ((i % 100) as f32 * 10.0) + 5.0, // slightly offset
                y: ((i / 100) as f32 * 10.0) + 5.0,
                w: 50.0,
                h: 50.0,
                rx: 0.0,
                style: make_style("#f0f"),
                transform: None,
            }));
        }
        
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("all_changed", count),
            &(scene1, scene2),
            |b, (s1, s2)| {
                b.iter(|| {
                    let result = render::diff(s1, s2);
                    black_box(result.ops.len())
                })
            },
        );
    }
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: Element SVG Generation
// ─────────────────────────────────────────────────────────────────────────────

fn bench_element_to_svg(c: &mut Criterion) {
    let mut group = c.benchmark_group("element_to_svg");
    
    let rect = Rect {
        x: 100.0, y: 200.0, w: 300.0, h: 150.0, rx: 8.0,
        style: Style {
            fill: Some("#ff6b6b".into()),
            stroke: Some("#333".into()),
            stroke_width: 2.0,
            opacity: 0.9,
            ..Default::default()
        },
        transform: Some("rotate(15)".into()),
    };
    
    let circle = Circle {
        cx: 250.0, cy: 250.0, r: 100.0,
        style: Style {
            fill: Some("#4ecdc4".into()),
            opacity: 1.0,
            ..Default::default()
        },
        transform: None,
    };
    
    group.bench_function("rect", |b| {
        b.iter(|| black_box(rect.to_svg()))
    });
    
    group.bench_function("circle", |b| {
        b.iter(|| black_box(circle.to_svg()))
    });
    
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark: FNV-1a Hashing
// ─────────────────────────────────────────────────────────────────────────────

fn bench_hashing(c: &mut Criterion) {
    use iconoglott_core::{Fnv1a, ContentHash};
    
    let mut group = c.benchmark_group("hashing");
    
    let small = "<rect x=\"10\" y=\"20\" width=\"100\" height=\"50\"/>";
    let medium = small.repeat(10);
    let large = small.repeat(100);
    
    group.bench_function("fnv1a_small", |b| {
        b.iter(|| {
            let mut h = Fnv1a::default();
            h.update(small.as_bytes());
            black_box(h.finish())
        })
    });
    
    group.bench_function("fnv1a_medium", |b| {
        b.iter(|| {
            let mut h = Fnv1a::default();
            h.update(medium.as_bytes());
            black_box(h.finish())
        })
    });
    
    group.bench_function("fnv1a_large", |b| {
        b.iter(|| {
            let mut h = Fnv1a::default();
            h.update(large.as_bytes());
            black_box(h.finish())
        })
    });
    
    group.bench_function("content_hash_small", |b| {
        b.iter(|| black_box(ContentHash::from_svg(small)))
    });
    
    group.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_scene_construction,
    bench_scene_render,
    bench_full_pipeline,
    bench_lexer,
    bench_parser,
    bench_scene_indexing,
    bench_diff_identical,
    bench_diff_single_change,
    bench_diff_all_changed,
    bench_element_to_svg,
    bench_hashing,
);

criterion_main!(benches);
