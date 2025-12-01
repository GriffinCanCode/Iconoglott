"""Tests for DSL interpreter."""

import pytest
from lang import Lexer, Parser, Interpreter
from lang.types import TokenType


class TestLexer:
    def test_tokenize_canvas(self):
        tokens = list(Lexer("canvas 800x600").tokenize())
        assert tokens[0].type == TokenType.IDENT
        assert tokens[0].value == "canvas"
        assert tokens[1].type == TokenType.PAIR
        assert tokens[1].value == (800, 600)

    def test_tokenize_color(self):
        tokens = list(Lexer("fill #ff0000").tokenize())
        assert tokens[1].type == TokenType.COLOR
        assert tokens[1].value == "#ff0000"

    def test_tokenize_variable(self):
        tokens = list(Lexer("$primary = #e94560").tokenize())
        assert tokens[0].type == TokenType.VAR
        assert tokens[0].value == "$primary"
        assert tokens[1].type == TokenType.EQUALS

    def test_tokenize_pair(self):
        tokens = list(Lexer("at 100,200").tokenize())
        assert tokens[1].type == TokenType.PAIR
        assert tokens[1].value == (100, 200)

    def test_tokenize_string(self):
        tokens = list(Lexer('text "Hello World"').tokenize())
        assert tokens[1].type == TokenType.STRING
        assert tokens[1].value == "Hello World"


class TestParser:
    def test_parse_canvas(self):
        tokens = Lexer("canvas 800x600 fill #000").tokenize()
        ast = Parser(tokens).parse()
        assert ast.children[0].type == "canvas"
        assert ast.children[0].value.width == 800
        assert ast.children[0].value.fill == "#000"

    def test_parse_shape_at(self):
        tokens = Lexer("rect at 100,200 size 50x30").tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.kind == "rect"
        assert shape.props["at"] == (100, 200)
        assert shape.props["size"] == (50, 30)

    def test_parse_variable(self):
        tokens = Lexer("$color = #f00\nrect $color").tokenize()
        ast = Parser(tokens).parse()
        # Variable should be resolved in shape
        shape = ast.children[1].value
        assert shape.props.get("fill") == "#f00"

    def test_parse_nested_style(self):
        source = """rect at 10,10 size 100x50
  fill #e94560
  stroke #000 2
  corner 8"""
        tokens = Lexer(source).tokenize()
        ast = Parser(tokens).parse()
        shape = ast.children[0].value
        assert shape.style.fill == "#e94560"
        assert shape.style.stroke == "#000"
        assert shape.style.stroke_width == 2
        assert shape.style.corner == 8


class TestInterpreter:
    def test_eval_canvas(self):
        state = Interpreter().eval("canvas 400x300 fill #1a1a2e")
        assert state.canvas.width == 400
        assert state.canvas.height == 300
        assert state.canvas.fill == "#1a1a2e"

    def test_eval_rect(self):
        source = """canvas 400x300
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
        assert state.shapes[0]['props']['radius'] == 50

    def test_to_svg_basic(self):
        source = "canvas 100x100\nrect at 0,0 size 50x50\n  fill #fff"
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
        assert state.shapes[0]['transform']['rotate'] == 45
        assert state.shapes[0]['transform']['scale'] == (1.5, 1.5)
