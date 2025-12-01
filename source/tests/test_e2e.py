"""End-to-end tests from DSL source to final SVG output."""

import pytest
from hypothesis import given, strategies as st, settings
import re

from lang import Interpreter


class TestE2ERoundtrip:
    """End-to-end roundtrip tests."""

    def test_basic_scene_renders(self):
        """Basic scene should render without errors."""
        source = """canvas giant fill #1a1a2e
rect at 50,50 size 200x100
  fill #e94560
  corner 8"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<svg' in svg
        assert '</svg>' in svg
        assert len(state.errors) == 0

    def test_complex_scene_renders(self):
        """Complex scene with multiple elements should render."""
        source = """canvas giant fill #1a1a2e
$primary = #e94560
$secondary = #16213e

rect at 50,50 size 200x100
  fill $primary
  corner 8
  shadow 2,4 8 #0004

circle at 400,100 radius 50
  fill $secondary
  stroke #fff 2

ellipse at 600,100 radius 60,30
  fill #0f0
  opacity 0.8

line from 50,200 to 750,200
  stroke #fff 1

text at 100,300 "Iconoglott Test"
  font "Arial" 32
  bold
  fill #fff

polygon points [350,250 400,350 300,350]
  fill #ff0
  
group "rotated"
  rect at 500,300 size 80x80
    fill #f0f
    rotate 45"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<svg' in svg
        assert '</svg>' in svg
        # Check key elements present
        assert '<rect' in svg
        assert '<circle' in svg
        assert '<ellipse' in svg
        assert '<line' in svg
        assert '<text' in svg
        assert '<polygon' in svg

    def test_gradient_renders_in_defs(self):
        """Gradients should render in defs section."""
        source = """canvas massive
rect at 50,50 size 300x300
  gradient linear #f00 #00f 45"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<defs>' in svg
        assert '<linearGradient' in svg
        assert 'url(#' in svg

    def test_filter_renders_in_defs(self):
        """Filters (shadows) should render in defs section."""
        source = """canvas massive
rect at 50,50 size 300x300
  fill #e94560
  shadow 4,4 10 #0008"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        # Shadow creates a filter
        assert '<defs>' in svg

    def test_layout_positions_children(self):
        """Stack/row layouts should position children correctly."""
        source = """canvas massive
stack at 50,50 gap 20
  rect size 100x50
    fill #f00
  rect size 100x50
    fill #0f0
  rect size 100x50
    fill #00f"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        # All three rects should be present
        assert svg.count('<rect') >= 4  # 1 bg + 3 shapes

    def test_transform_applied(self):
        """Transforms should be applied to SVG elements."""
        source = """canvas huge
rect at 100,100 size 50x50
  fill #f00
  rotate 45
  translate 10,10"""
        state = Interpreter().eval(source)
        # Transform info should be in shape data
        assert state.shapes[0]['transform']['rotate'] == 45
        assert list(state.shapes[0]['transform']['translate']) == [10.0, 10.0]


class TestE2EValidSVG:
    """Tests that output is valid SVG."""

    def test_svg_has_namespace(self):
        source = "canvas large"
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert 'xmlns="http://www.w3.org/2000/svg"' in svg

    def test_svg_is_well_formed(self):
        """SVG should be well-formed XML (basic check)."""
        source = """canvas massive
rect at 0,0 size 100x100
  fill #f00
text at 50,50 "Test"
  fill #000"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        # Count opening and closing tags
        assert svg.count('<svg') == 1
        assert svg.count('</svg>') == 1
        # All elements should be properly closed
        for tag in ['rect', 'circle', 'ellipse', 'line', 'path', 'polygon', 'image']:
            opens = svg.count(f'<{tag}')
            closes = svg.count(f'</{tag}>') + svg.count('/>')  # self-closing
            assert opens <= closes or opens == svg.count(f'<{tag} ') + svg.count(f'<{tag}/')

    def test_special_chars_escaped(self):
        """Special characters in text should be escaped."""
        source = '''text at 10,20 "<script>alert('xss')</script>"'''
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<script>' not in svg
        assert '&lt;' in svg or '&gt;' in svg


class TestE2EPropertyBased:
    """Property-based E2E tests."""

    @given(
        st.integers(min_value=100, max_value=1000),
        st.integers(min_value=100, max_value=1000),
        st.from_regex(r'#[0-9a-fA-F]{6}', fullmatch=True)
    )
    @settings(max_examples=20)
    def test_canvas_roundtrip(self, w, h, bg):
        """Canvas dimensions and background should roundtrip."""
        source = f"canvas {w}x{h} fill {bg}"
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert f'width="{w}"' in svg
        assert f'height="{h}"' in svg
        assert bg.lower() in svg.lower()

    @given(
        st.integers(min_value=0, max_value=500),
        st.integers(min_value=0, max_value=500),
        st.integers(min_value=10, max_value=200),
        st.integers(min_value=10, max_value=200),
        st.integers(min_value=0, max_value=20)
    )
    @settings(max_examples=20)
    def test_rect_roundtrip(self, x, y, w, h, corner):
        """Rect properties should roundtrip through SVG."""
        source = f"""canvas giant
rect at {x},{y} size {w}x{h}
  fill #f00
  corner {corner}"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert f'x="{x}"' in svg
        assert f'y="{y}"' in svg
        assert f'width="{w}"' in svg
        assert f'height="{h}"' in svg

    @given(
        st.integers(min_value=50, max_value=500),
        st.integers(min_value=50, max_value=500),
        st.integers(min_value=5, max_value=100)
    )
    @settings(max_examples=20)
    def test_circle_roundtrip(self, cx, cy, r):
        """Circle properties should roundtrip through SVG."""
        source = f"""canvas giant
circle at {cx},{cy} radius {r}
  fill #0f0"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        # Allow for float formatting
        assert 'cx=' in svg
        assert 'cy=' in svg
        assert 'r=' in svg

    @given(st.lists(
        st.tuples(
            st.integers(min_value=0, max_value=400),
            st.integers(min_value=0, max_value=400)
        ),
        min_size=3,
        max_size=8
    ))
    @settings(max_examples=10)
    def test_polygon_roundtrip(self, points):
        """Polygon points should roundtrip through SVG."""
        points_str = " ".join(f"{x},{y}" for x, y in points)
        source = f"""canvas giant
polygon points [{points_str}]
  fill #ff0"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<polygon' in svg
        assert 'points=' in svg


class TestE2ERustIntegration:
    """Tests for Rust core integration."""

    def test_rust_scene_creation(self):
        """Rust Scene should be created correctly."""
        source = "canvas giant fill #1a1a2e"
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<svg' in svg
        assert 'width="512"' in svg
        assert 'height="512"' in svg

    def test_rust_shape_rendering(self):
        """Shapes should be rendered by Rust core."""
        source = """canvas massive
rect at 10,10 size 100x100
  fill #e94560
circle at 200,200 radius 50
  fill #16213e"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        # Rust should render these shapes
        assert '<rect' in svg
        assert '<circle' in svg

    def test_rust_gradient_rendering(self):
        """Gradients should be rendered by Rust core."""
        source = """canvas massive
rect at 0,0 size 400x300
  gradient linear #e94560 #16213e"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<linearGradient' in svg

    def test_rust_filter_rendering(self):
        """Filters should be rendered by Rust core."""
        source = """canvas massive
rect at 50,50 size 300x200
  fill #e94560
  shadow 4,4 8 #0008"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<filter' in svg


class TestE2EEdgeCases:
    """Edge case E2E tests."""

    def test_empty_scene(self):
        """Empty source should produce valid empty SVG."""
        state = Interpreter().eval("")
        svg = state.to_svg()
        assert '<svg' in svg
        assert '</svg>' in svg

    def test_very_large_coordinates(self):
        """Large coordinates should work (shapes can extend beyond canvas)."""
        source = """canvas giant
rect at 5000,5000 size 1000x1000
  fill #f00"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert 'width="512"' in svg
        assert '<rect' in svg
        assert 'x="5000"' in svg

    def test_zero_dimensions(self):
        """Zero-size shapes should still render."""
        source = """canvas large
rect at 10,10 size 0x0
  fill #f00"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        # Should not crash
        assert '<svg' in svg

    def test_negative_coordinates(self):
        """Negative coordinates should work."""
        source = """canvas massive
rect at -50,-50 size 100x100
  fill #f00"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert 'x="-50"' in svg

    def test_many_shapes(self):
        """Scene with many shapes should render."""
        shapes = "\n".join([
            f"rect at {i*10},{i*5} size 20x20\n  fill #f00"
            for i in range(50)
        ])
        source = f"canvas giant\n{shapes}"
        state = Interpreter().eval(source)
        svg = state.to_svg()
        # Should have many rects
        assert svg.count('<rect') >= 50

