"""Comprehensive tests for the DSL interpreter with snapshot testing."""

import pytest
from hypothesis import given, strategies as st, assume, settings
from pathlib import Path
import json

from lang import Interpreter


class TestInterpreterBasics:
    """Basic interpreter functionality tests."""

    def test_empty_source(self):
        state = Interpreter().eval("")
        assert state.canvas.size == "medium"
        assert state.canvas.width == 64
        assert state.canvas.height == 64

    def test_canvas_only(self):
        state = Interpreter().eval("canvas massive fill #1a1a2e")
        assert state.canvas.size == "massive"
        assert state.canvas.width == 256
        assert state.canvas.height == 256
        assert state.canvas.fill == "#1a1a2e"

    def test_single_rect(self):
        source = """canvas massive
rect at 10,10 size 100x50
  fill #f00"""
        state = Interpreter().eval(source)
        assert len(state.shapes) == 1
        assert state.shapes[0]['kind'] == 'rect'
        assert list(state.shapes[0]['props']['at']) == [10.0, 10.0]
        assert list(state.shapes[0]['props']['size']) == [100.0, 50.0]


class TestInterpreterShapes:
    """Shape rendering tests."""

    def test_circle(self):
        source = """circle at 100,100 radius 50
  fill #0f0"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['kind'] == 'circle'
        assert state.shapes[0]['props']['radius'] == 50

    def test_ellipse(self):
        source = """ellipse at 100,100 radius 80,40
  fill #00f"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['kind'] == 'ellipse'

    def test_line(self):
        source = """line from 0,0 to 100,100
  stroke #000 2"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['kind'] == 'line'
        assert list(state.shapes[0]['props']['from']) == [0.0, 0.0]
        assert list(state.shapes[0]['props']['to']) == [100.0, 100.0]

    def test_path(self):
        source = 'path d "M 0 0 L 100 100 L 100 0 Z"'
        state = Interpreter().eval(source)
        assert state.shapes[0]['kind'] == 'path'

    def test_polygon(self):
        source = """polygon points [0,0 100,0 50,100]
  fill #ff0"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['kind'] == 'polygon'
        assert len(state.shapes[0]['props']['points']) == 3

    def test_text(self):
        source = '''text at 10,20 "Hello World"
  font "Arial" 24
  fill #000'''
        state = Interpreter().eval(source)
        assert state.shapes[0]['kind'] == 'text'
        assert state.shapes[0]['props']['content'] == "Hello World"

    def test_image(self):
        source = 'image at 0,0 size 100x100 href "test.png"'
        state = Interpreter().eval(source)
        assert state.shapes[0]['kind'] == 'image'
        assert state.shapes[0]['props']['href'] == "test.png"


class TestInterpreterStyles:
    """Style rendering tests."""

    def test_fill_stroke(self):
        source = """rect at 0,0 size 100x100
  fill #f00
  stroke #000 2"""
        state = Interpreter().eval(source)
        style = state.shapes[0]['style']
        assert style['fill'] == '#f00'
        assert style['stroke'] == '#000'
        assert style['stroke_width'] == 2

    def test_opacity(self):
        source = """rect at 0,0 size 100x100
  opacity 0.5"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['style']['opacity'] == 0.5

    def test_corner(self):
        source = """rect at 0,0 size 100x100
  corner 10"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['style']['corner'] == 10


class TestInterpreterTransforms:
    """Transform rendering tests."""

    def test_rotate(self):
        source = """rect at 50,50 size 100x100
  rotate 45"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['transform']['rotate'] == 45

    def test_scale(self):
        source = """rect at 0,0 size 100x100
  scale 1.5,2.0"""
        state = Interpreter().eval(source)
        assert list(state.shapes[0]['transform']['scale']) == [1.5, 2.0]

    def test_translate(self):
        source = """rect at 0,0 size 100x100
  translate 10,20"""
        state = Interpreter().eval(source)
        assert list(state.shapes[0]['transform']['translate']) == [10.0, 20.0]


class TestInterpreterGradients:
    """Gradient rendering tests."""

    def test_linear_gradient(self):
        source = """rect at 0,0 size 100x100
  gradient linear #f00 #00f"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['style']['gradient'] is not None
        assert state.shapes[0]['style']['gradient']['gtype'] == 'linear'

    def test_radial_gradient(self):
        source = """rect at 0,0 size 100x100
  gradient radial #fff #000"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['style']['gradient']['gtype'] == 'radial'


class TestInterpreterVariables:
    """Variable resolution tests."""

    def test_color_variable(self):
        source = """$primary = #e94560
rect at 0,0 size 100x100 $primary"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['style']['fill'] == '#e94560'

    def test_multiple_variables(self):
        source = """$primary = #e94560
$secondary = #16213e
rect at 0,0 size 100x100 $primary
circle at 200,200 radius 50 $secondary"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['style']['fill'] == '#e94560'
        # Note: circle uses $secondary in props.fill
        assert state.shapes[1]['props'].get('fill') == '#16213e'


class TestInterpreterLayouts:
    """Layout rendering tests."""

    def test_stack_layout(self):
        source = """stack at 0,0 gap 10
  rect size 50x30
  rect size 50x30
  rect size 50x30"""
        state = Interpreter().eval(source)
        assert len(state.shapes) == 1
        assert state.shapes[0]['kind'] == 'layout'
        assert len(state.shapes[0].get('children', [])) == 3

    def test_row_layout(self):
        source = """row at 0,0 gap 10
  rect size 30x50
  rect size 30x50"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['props']['direction'] == 'horizontal'


class TestInterpreterSVGOutput:
    """SVG output tests."""

    def test_svg_contains_xmlns(self):
        state = Interpreter().eval("canvas large")
        svg = state.to_svg()
        assert 'xmlns="http://www.w3.org/2000/svg"' in svg

    def test_svg_contains_dimensions(self):
        state = Interpreter().eval("canvas massive")
        svg = state.to_svg()
        assert 'width="256"' in svg
        assert 'height="256"' in svg

    def test_svg_contains_background(self):
        state = Interpreter().eval("canvas large fill #1a1a2e")
        svg = state.to_svg()
        assert 'fill="#1a1a2e"' in svg

    def test_svg_contains_rect(self):
        source = """canvas large
rect at 10,10 size 50x30
  fill #f00"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<rect' in svg
        assert 'x="10"' in svg
        assert 'fill="#f00"' in svg

    def test_svg_contains_circle(self):
        source = """canvas large
circle at 50,50 radius 20
  fill #0f0"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<circle' in svg
        assert 'r="20"' in svg

    def test_svg_gradient_defs(self):
        source = """canvas large
rect at 0,0 size 100x100
  gradient linear #f00 #00f"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<defs>' in svg
        assert '<linearGradient' in svg
        assert '</defs>' in svg


class TestInterpreterErrors:
    """Error handling tests."""

    def test_error_collection(self):
        source = "unknown_command"
        state = Interpreter().eval(source)
        assert len(state.error_infos) > 0

    def test_undefined_variable_uses_literal(self):
        # Undefined variables are passed through as literal strings
        source = "rect $undefined"
        state = Interpreter().eval(source)
        # The variable name is used as-is when undefined
        assert state.shapes[0]['props'].get('fill') == '$undefined'


class TestInterpreterPropertyBased:
    """Property-based tests using hypothesis."""

    @given(
        st.integers(min_value=100, max_value=2000),
        st.integers(min_value=100, max_value=2000)
    )
    def test_canvas_dimensions_in_svg(self, w, h):
        """Canvas dimensions should appear in SVG output."""
        state = Interpreter().eval(f"canvas {w}x{h}")
        svg = state.to_svg()
        assert f'width="{w}"' in svg
        assert f'height="{h}"' in svg

    @given(st.from_regex(r'#[0-9a-fA-F]{6}', fullmatch=True))
    def test_fill_color_in_svg(self, color):
        """Fill colors should appear in SVG output."""
        source = f"""rect at 0,0 size 50x50
  fill {color}"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert color.lower() in svg.lower()

    @given(
        st.integers(min_value=0, max_value=500),
        st.integers(min_value=0, max_value=500),
        st.integers(min_value=1, max_value=200),
        st.integers(min_value=1, max_value=200)
    )
    def test_rect_coords_in_svg(self, x, y, w, h):
        """Rect coordinates should appear in SVG output."""
        source = f"""rect at {x},{y} size {w}x{h}
  fill #f00"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert f'x="{x}"' in svg
        assert f'y="{y}"' in svg

    @given(
        st.integers(min_value=0, max_value=500),
        st.integers(min_value=0, max_value=500),
        st.integers(min_value=1, max_value=100)
    )
    def test_circle_coords_in_svg(self, cx, cy, r):
        """Circle coordinates should appear in SVG output."""
        source = f"""circle at {cx},{cy} radius {r}
  fill #0f0"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert f'cx="{cx}"' in svg or f'cx="{float(cx)}"' in svg
        assert f'r="{r}"' in svg or f'r="{float(r)}"' in svg


class TestInterpreterSnapshots:
    """Snapshot tests for SVG output."""

    def _compare_or_write_snapshot(self, snapshot_dir: Path, name: str, svg: str):
        """Compare SVG to snapshot or write new snapshot."""
        snapshot_file = snapshot_dir / f"{name}.svg"
        if snapshot_file.exists():
            expected = snapshot_file.read_text()
            assert svg == expected, f"Snapshot mismatch for {name}"
        else:
            snapshot_file.write_text(svg)

    def test_snapshot_empty_canvas(self, snapshot_dir):
        state = Interpreter().eval("canvas large fill #fff")
        svg = state.to_svg()
        self._compare_or_write_snapshot(snapshot_dir, "empty_canvas", svg)

    def test_snapshot_basic_rect(self, snapshot_dir):
        source = """canvas huge fill #fff
rect at 25,25 size 150x150
  fill #e94560
  corner 10"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        self._compare_or_write_snapshot(snapshot_dir, "basic_rect", svg)

    def test_snapshot_basic_circle(self, snapshot_dir):
        source = """canvas huge fill #1a1a2e
circle at 100,100 radius 50
  fill #e94560
  stroke #fff 2"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        self._compare_or_write_snapshot(snapshot_dir, "basic_circle", svg)

    def test_snapshot_gradient(self, snapshot_dir):
        source = """canvas huge fill #fff
rect at 25,25 size 150x150
  gradient linear #e94560 #16213e"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        self._compare_or_write_snapshot(snapshot_dir, "gradient", svg)

    def test_snapshot_multiple_shapes(self, snapshot_dir):
        source = """canvas massive fill #1a1a2e
rect at 20,20 size 80x60
  fill #e94560
circle at 200,50 radius 30
  fill #16213e
text at 50,150 "Hello"
  fill #fff"""
        state = Interpreter().eval(source)
        svg = state.to_svg()
        self._compare_or_write_snapshot(snapshot_dir, "multiple_shapes", svg)

