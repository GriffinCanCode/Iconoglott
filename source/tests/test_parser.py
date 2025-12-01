"""Comprehensive tests for the DSL parser with property-based testing."""

import pytest
from hypothesis import given, strategies as st, assume, settings

from lang.lexer import Lexer
from lang.parser import Parser
from lang.types import TokenType


class TestParserCanvas:
    """Canvas parsing tests."""

    def test_canvas_basic(self):
        tokens = Lexer("canvas 800x600").tokenize()
        ast = Parser(tokens).parse()
        canvas = ast.children[0]
        assert canvas.type == "canvas"
        assert canvas.value.width == 800
        assert canvas.value.height == 600

    def test_canvas_with_fill(self):
        tokens = Lexer("canvas 400x300 fill #1a1a2e").tokenize()
        ast = Parser(tokens).parse()
        canvas = ast.children[0]
        assert canvas.value.fill == "#1a1a2e"

    def test_canvas_default_values(self):
        tokens = Lexer("canvas").tokenize()
        ast = Parser(tokens).parse()
        canvas = ast.children[0]
        assert canvas.value.width == 800
        assert canvas.value.height == 600


class TestParserShapes:
    """Shape parsing tests."""

    def test_rect_basic(self):
        tokens = Lexer("rect at 10,20 size 100x50").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "rect"
        assert shape.props["at"] == (10, 20)
        assert shape.props["size"] == (100, 50)

    def test_rect_with_color(self):
        tokens = Lexer("rect at 0,0 size 50x50 #f00").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.props.get("fill") == "#f00"

    def test_circle_basic(self):
        tokens = Lexer("circle at 100,100 radius 50").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "circle"
        assert shape.props["radius"] == 50

    def test_circle_short(self):
        tokens = Lexer("circle 100,100 50").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "circle"
        assert shape.props["at"] == (100, 100)
        assert shape.props["radius"] == 50

    def test_ellipse_basic(self):
        tokens = Lexer("ellipse at 100,100 radius 50,30").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "ellipse"
        assert shape.props["radius"] == (50, 30)

    def test_line_basic(self):
        tokens = Lexer("line from 0,0 to 100,100").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "line"
        assert shape.props["from"] == (0, 0)
        assert shape.props["to"] == (100, 100)

    def test_path_basic(self):
        tokens = Lexer('path d "M 0 0 L 100 100"').tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "path"
        assert shape.props["d"] == "M 0 0 L 100 100"

    def test_polygon_basic(self):
        tokens = Lexer("polygon points [0,0 100,0 50,100]").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "polygon"
        assert shape.props["points"] == [(0, 0), (100, 0), (50, 100)]

    def test_text_basic(self):
        tokens = Lexer('text at 10,20 "Hello World"').tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "text"
        assert shape.props["content"] == "Hello World"

    def test_image_basic(self):
        tokens = Lexer('image at 0,0 size 100x100 href "test.png"').tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "image"
        assert shape.props["href"] == "test.png"


class TestParserStyles:
    """Style parsing tests."""

    def test_fill_style(self):
        source = """rect at 0,0 size 100x50
  fill #e94560"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.fill == "#e94560"

    def test_stroke_style(self):
        source = """rect at 0,0 size 100x50
  stroke #000 2"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.stroke == "#000"
        assert shape.style.stroke_width == 2

    def test_opacity_style(self):
        source = """rect at 0,0 size 100x50
  opacity 0.5"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.opacity == 0.5

    def test_corner_style(self):
        source = """rect at 0,0 size 100x50
  corner 8"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.corner == 8

    def test_combined_styles(self):
        source = """rect at 0,0 size 100x50
  fill #f00
  stroke #000 2
  opacity 0.8
  corner 5"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.fill == "#f00"
        assert shape.style.stroke == "#000"
        assert shape.style.stroke_width == 2
        assert shape.style.opacity == 0.8
        assert shape.style.corner == 5


class TestParserTextStyles:
    """Text-specific style parsing tests."""

    def test_font_style(self):
        source = '''text at 0,0 "Test"
  font "Arial" 24'''
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.font == "Arial"
        assert shape.style.font_size == 24

    def test_bold_style(self):
        source = '''text at 0,0 "Test"
  bold'''
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.font_weight == "bold"

    def test_center_style(self):
        source = '''text at 0,0 "Test"
  center'''
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.text_anchor == "middle"


class TestParserTransforms:
    """Transform parsing tests."""

    def test_translate_transform(self):
        source = """rect at 0,0 size 50x50
  translate 10,20"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.transform.translate == (10, 20)

    def test_rotate_transform(self):
        source = """rect at 0,0 size 50x50
  rotate 45"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.transform.rotate == 45

    def test_scale_transform_pair(self):
        source = """rect at 0,0 size 50x50
  scale 1.5,2.0"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.transform.scale == (1.5, 2.0)

    def test_scale_transform_single(self):
        source = """rect at 0,0 size 50x50
  scale 2"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.transform.scale == (2, 2)

    def test_origin_transform(self):
        source = """rect at 0,0 size 50x50
  origin 25,25"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.transform.origin == (25, 25)


class TestParserGradients:
    """Gradient parsing tests."""

    def test_linear_gradient(self):
        source = """rect at 0,0 size 100x100
  gradient linear #f00 #00f"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.gradient is not None
        assert shape.style.gradient["type"] == "linear"
        assert shape.style.gradient["from"] == "#f00"
        assert shape.style.gradient["to"] == "#00f"

    def test_radial_gradient(self):
        source = """rect at 0,0 size 100x100
  gradient radial #fff #000"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.gradient["type"] == "radial"

    def test_gradient_with_angle(self):
        source = """rect at 0,0 size 100x100
  gradient linear #f00 #00f 45"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.gradient["angle"] == 45


class TestParserShadows:
    """Shadow parsing tests."""

    def test_shadow_basic(self):
        source = """rect at 0,0 size 100x100
  shadow 2,4 8 #0004"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.shadow is not None
        assert shape.style.shadow["x"] == 2
        assert shape.style.shadow["y"] == 4
        assert shape.style.shadow["blur"] == 8
        assert shape.style.shadow["color"] == "#0004"

    def test_shadow_default(self):
        source = """rect at 0,0 size 100x100
  shadow"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.shadow is not None


class TestParserVariables:
    """Variable parsing tests."""

    def test_variable_assignment(self):
        tokens = Lexer("$primary = #e94560").tokenize()
        ast = Parser(tokens).parse()
        var_node = ast.children[0]
        assert var_node.type == "variable"

    def test_variable_usage(self):
        source = """$color = #f00
rect $color"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[1].value
        assert shape.props.get("fill") == "#f00"


class TestParserGroups:
    """Group and layout parsing tests."""

    def test_group_basic(self):
        source = """group "my-group"
  rect at 0,0 size 50x50"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "group"
        assert len(shape.children) == 1

    def test_stack_layout(self):
        source = """stack at 0,0 gap 10
  rect size 50x30
  rect size 50x30"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "layout"
        assert shape.props["direction"] == "vertical"
        assert shape.props["gap"] == 10

    def test_row_layout(self):
        source = """row at 0,0 gap 10
  rect size 30x50
  rect size 30x50"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "layout"
        assert shape.props["direction"] == "horizontal"


class TestParserErrors:
    """Error handling tests."""

    def test_unknown_command(self):
        tokens = Lexer("unknown 100,200").tokenize()
        parser = Parser(tokens)
        parser.parse()
        assert len(parser.errors) > 0

    def test_missing_bracket(self):
        tokens = Lexer("polygon points [0,0 100,0").tokenize()
        parser = Parser(tokens)
        parser.parse()
        assert len(parser.errors) > 0


class TestParserErrorRecovery:
    """Error recovery tests - parser continues after errors."""

    def test_recovery_after_unknown_command(self):
        """Parser recovers after unknown command and parses next statement."""
        source = """unknown 100,200
rect at 0,0 size 50x50"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        # Should have error for unknown command
        assert len(parser.errors) >= 1
        # But should still parse the rect
        assert len(ast.children) == 1
        assert ast.children[0].value.kind == "rect"

    def test_recovery_multiple_errors(self):
        """Parser continues after multiple errors."""
        source = """invalid1
rect at 0,0 size 50x50
invalid2
circle at 100,100 radius 30"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        # Should have 2 errors
        assert len(parser.errors) >= 2
        # Should parse both valid shapes
        assert len(ast.children) == 2
        assert ast.children[0].value.kind == "rect"
        assert ast.children[1].value.kind == "circle"

    def test_recovery_partial_ast(self):
        """Produces partial AST with valid nodes despite errors."""
        source = """canvas 800x600
rect at 0,0 size 100x100
badcmd
circle at 200,200 radius 50
text at 300,300 "Hello" """
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        # Should have error for badcmd
        assert len(parser.errors) >= 1
        # Should still have canvas + 3 shapes
        assert len(ast.children) == 4

    def test_recovery_in_block(self):
        """Parser recovers from errors within indented blocks."""
        source = """rect at 0,0 size 100x100
  fill #f00
  opacity
  stroke #000 2"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        shape = ast.children[0].value
        # Should have error for missing opacity value
        assert len(parser.errors) >= 1
        # But should still parse fill and stroke
        assert shape.style.fill == "#f00"
        assert shape.style.stroke == "#000"

    def test_recovery_missing_values(self):
        """Parser records errors but continues parsing."""
        source = """rect at 0,0 size 100x100
  translate
  rotate 45"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        shape = ast.children[0].value
        # Error for missing translate value
        assert len(parser.errors) >= 1
        # But rotate should still work
        assert shape.transform.rotate == 45

    def test_recovery_undefined_variable(self):
        """Undefined variables recorded as error but parsing continues."""
        source = """rect $undefined
circle at 100,100 radius 30"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        # Should have error for undefined var
        assert len(parser.errors) >= 1
        # Both shapes should be parsed
        assert len(ast.children) == 2

    def test_recovery_unclosed_bracket(self):
        """Parser recovers from unclosed brackets."""
        source = """polygon points [0,0 100,0
rect at 0,0 size 50x50"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        # Should have error for missing ]
        assert len(parser.errors) >= 1
        # Should still parse shapes
        assert len(ast.children) >= 1

    def test_recovery_nested_groups(self):
        """Parser recovers within nested group structures."""
        source = """group "outer"
  rect at 0,0 size 50x50
  badprop
  circle at 100,100 radius 25"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        group = ast.children[0].value
        # Group should have both valid shapes
        assert len(group.children) == 2

    def test_error_list_populated(self):
        """Both errors and error_infos lists are populated."""
        source = """unknown
rect"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        parser.parse()
        # Both error tracking mechanisms should have errors
        assert len(parser.errors) > 0
        assert len(parser.error_infos) > 0
        assert len(parser.errors) == len(parser.error_infos)


class TestParserPropertyBased:
    """Property-based tests using hypothesis."""

    @given(
        st.integers(min_value=100, max_value=2000),
        st.integers(min_value=100, max_value=2000)
    )
    def test_canvas_size_roundtrip(self, w, h):
        """Canvas dimensions should parse correctly."""
        tokens = Lexer(f"canvas {w}x{h}").tokenize()
        ast = Parser(tokens).parse()
        canvas = ast.children[0]
        assert canvas.value.width == w
        assert canvas.value.height == h

    @given(
        st.integers(min_value=-500, max_value=500),
        st.integers(min_value=-500, max_value=500),
        st.integers(min_value=1, max_value=500),
        st.integers(min_value=1, max_value=500)
    )
    def test_rect_props_roundtrip(self, x, y, w, h):
        """Rect properties should parse correctly."""
        tokens = Lexer(f"rect at {x},{y} size {w}x{h}").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.props["at"] == (x, y)
        assert shape.props["size"] == (w, h)

    @given(
        st.integers(min_value=-500, max_value=500),
        st.integers(min_value=-500, max_value=500),
        st.integers(min_value=1, max_value=200)
    )
    def test_circle_props_roundtrip(self, cx, cy, r):
        """Circle properties should parse correctly."""
        tokens = Lexer(f"circle at {cx},{cy} radius {r}").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.props["at"] == (cx, cy)
        assert shape.props["radius"] == r

    @given(st.floats(min_value=0.0, max_value=1.0, allow_nan=False))
    @settings(max_examples=50)
    def test_opacity_roundtrip(self, opacity):
        """Opacity values should parse correctly."""
        opacity = round(opacity, 2)
        source = f"""rect at 0,0 size 50x50
  opacity {opacity}"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert abs(shape.style.opacity - opacity) < 0.01

    @given(st.integers(min_value=0, max_value=360))
    def test_rotate_roundtrip(self, angle):
        """Rotation angles should parse correctly."""
        source = f"""rect at 0,0 size 50x50
  rotate {angle}"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.transform.rotate == angle

    @given(st.lists(
        st.tuples(
            st.integers(min_value=0, max_value=200),
            st.integers(min_value=0, max_value=200)
        ),
        min_size=3,
        max_size=10
    ))
    def test_polygon_points_roundtrip(self, points):
        """Polygon points should parse correctly."""
        points_str = " ".join(f"{x},{y}" for x, y in points)
        source = f"polygon points [{points_str}]"
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.props["points"] == points

    @given(st.text(
        alphabet=st.characters(whitelist_categories=['L', 'N', 'S', 'P', 'Z'], blacklist_characters='"\''),
        min_size=1,
        max_size=50
    ))
    def test_text_content_roundtrip(self, content):
        """Text content should parse correctly."""
        assume('\n' not in content and '"' not in content)
        source = f'text at 0,0 "{content}"'
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.props["content"] == content


class TestParserSceneIntegrity:
    """Tests ensuring complete scene parsing."""

    def test_multiple_shapes(self):
        source = """canvas 800x600
rect at 0,0 size 100x100
circle at 200,200 radius 50
text at 100,100 "Hello" """
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        assert len(ast.children) == 4  # canvas + 3 shapes

    def test_nested_styles(self):
        source = """rect at 0,0 size 100x100
  fill #f00
  stroke #000 2
  opacity 0.8
  corner 10
  shadow 2,4 8 #0004
  gradient linear #f00 #00f
  rotate 45
  translate 10,10"""
        tokens = Lexer(source).tokenize()
        parser = Parser(tokens)
        ast = parser.parse()
        assert len(parser.errors) == 0
        shape = ast.children[0].value
        assert shape.style.fill is not None
        assert shape.style.stroke is not None
        assert shape.style.shadow is not None
        assert shape.style.gradient is not None

