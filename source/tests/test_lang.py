"""Tests for DSL interpreter using Rust core."""

import pytest
import iconoglott_core as rust
from lang.eval import Interpreter


class TestLexer:
    """Test Rust lexer via Python bindings."""
    
    def test_tokenize_canvas(self):
        lexer = rust.Lexer("canvas giant")
        tokens = lexer.py_tokenize()
        assert tokens[0].ttype == rust.TokenType.Ident
        assert tokens[0].value == "canvas"
        assert tokens[1].ttype == rust.TokenType.Size
        assert tokens[1].value == "giant"

    def test_tokenize_color(self):
        lexer = rust.Lexer("fill #ff0000")
        tokens = lexer.py_tokenize()
        assert tokens[1].ttype == rust.TokenType.Color
        assert tokens[1].value == "#ff0000"

    def test_tokenize_variable(self):
        lexer = rust.Lexer("$primary = #e94560")
        tokens = lexer.py_tokenize()
        assert tokens[0].ttype == rust.TokenType.Var
        assert tokens[0].value == "$primary"
        assert tokens[1].ttype == rust.TokenType.Equals

    def test_tokenize_pair(self):
        lexer = rust.Lexer("at 100,200")
        tokens = lexer.py_tokenize()
        assert tokens[1].ttype == rust.TokenType.Pair
        assert tokens[1].value == (100.0, 200.0)

    def test_tokenize_string(self):
        lexer = rust.Lexer('text "Hello World"')
        tokens = lexer.py_tokenize()
        assert tokens[1].ttype == rust.TokenType.String
        assert tokens[1].value == "Hello World"


class TestParser:
    """Test Rust parser via Python bindings."""
    
    def _parse(self, source: str):
        lexer = rust.Lexer(source)
        tokens = lexer.py_tokenize()
        parser = rust.Parser(tokens)
        return parser.parse_py()
    
    def test_parse_canvas(self):
        ast = self._parse("canvas giant fill #000")
        canvas = ast['Scene'][0]['Canvas']
        assert canvas['size'] == 'giant'
        assert canvas['width'] == 512
        assert canvas['height'] == 512
        assert canvas['fill'] == "#000"

    def test_parse_shape_at(self):
        ast = self._parse("rect at 100,200 size 50x30")
        shape = ast['Scene'][0]['Shape']
        assert shape['kind'] == "rect"
        assert shape['props']['at'] == (100.0, 200.0)
        assert shape['props']['size'] == (50.0, 30.0)

    def test_parse_variable(self):
        ast = self._parse("$color = #f00\nrect $color")
        # Variable should be in first child
        assert 'Variable' in ast['Scene'][0]
        # Shape should have fill resolved
        shape = ast['Scene'][1]['Shape']
        assert shape['props']['fill'] == "#f00"

    def test_parse_nested_style(self):
        source = """rect at 10,10 size 100x50
  fill #e94560
  stroke #000 2
  corner 8"""
        ast = self._parse(source)
        shape = ast['Scene'][0]['Shape']
        assert shape['style']['fill'] == "#e94560"
        assert shape['style']['stroke'] == "#000"
        assert shape['style']['stroke_width'] == 2.0
        assert shape['style']['corner'] == 8.0


class TestInterpreter:
    """Test full evaluation pipeline."""
    
    def test_eval_canvas(self):
        state = Interpreter().eval("canvas massive fill #1a1a2e")
        assert state.canvas.size == "massive"
        assert state.canvas.width == 256
        assert state.canvas.height == 256
        assert state.canvas.fill == "#1a1a2e"

    def test_eval_rect(self):
        source = """canvas massive
rect at 10,10 size 100x50
  fill #f00"""
        state = Interpreter().eval(source)
        assert len(state.shapes) == 1
        assert state.shapes[0]['kind'] == 'rect'
        assert state.shapes[0]['style']['fill'] == '#f00'

    def test_eval_circle(self):
        source = "circle at 100,100 radius 50\n  fill #0f0"
        state = Interpreter().eval(source)
        assert state.shapes[0]['kind'] == 'circle'
        assert state.shapes[0]['props']['radius'] == 50.0

    def test_to_svg_basic(self):
        source = "canvas large\nrect at 0,0 size 50x50\n  fill #fff"
        state = Interpreter().eval(source)
        svg = state.to_svg()
        assert '<svg' in svg
        assert '<rect' in svg
        assert 'fill="#fff"' in svg

    def test_eval_gradient(self):
        source = """rect at 0,0 size 100x100
  gradient linear #f00 #00f"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['style']['gradient'] is not None

    def test_eval_transform(self):
        source = """rect at 50,50 size 100x100
  rotate 45
  scale 1.5,1.5"""
        state = Interpreter().eval(source)
        assert state.shapes[0]['transform']['rotate'] == 45.0
        scale = state.shapes[0]['transform']['scale']
        assert list(scale) == [1.5, 1.5]
