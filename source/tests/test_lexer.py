"""Comprehensive tests for the DSL lexer with property-based testing."""

import pytest
from hypothesis import given, strategies as st, assume, settings

from lang.lexer import Lexer
from lang.types import TokenType


class TestLexerBasics:
    """Basic lexer functionality tests."""

    def test_empty_input(self):
        tokens = list(Lexer("").tokenize())
        assert tokens[-1].type == TokenType.EOF

    def test_whitespace_only(self):
        tokens = list(Lexer("   \t  ").tokenize())
        assert tokens[-1].type == TokenType.EOF

    def test_comment_only(self):
        tokens = list(Lexer("// this is a comment").tokenize())
        assert tokens[-1].type == TokenType.EOF

    def test_multiple_comments(self):
        tokens = list(Lexer("// comment 1\n// comment 2").tokenize())
        assert tokens[-1].type == TokenType.EOF


class TestLexerIdentifiers:
    """Identifier tokenization tests."""

    def test_simple_identifier(self):
        tokens = list(Lexer("canvas").tokenize())
        assert tokens[0].type == TokenType.IDENT
        assert tokens[0].value == "canvas"

    def test_identifier_with_numbers(self):
        tokens = list(Lexer("layer1").tokenize())
        assert tokens[0].type == TokenType.IDENT
        assert tokens[0].value == "layer1"

    def test_identifier_with_hyphen(self):
        tokens = list(Lexer("my-shape").tokenize())
        assert tokens[0].type == TokenType.IDENT
        assert tokens[0].value == "my-shape"

    def test_identifier_with_underscore(self):
        tokens = list(Lexer("my_shape").tokenize())
        assert tokens[0].type == TokenType.IDENT
        assert tokens[0].value == "my_shape"


class TestLexerNumbers:
    """Number tokenization tests."""

    def test_integer(self):
        tokens = list(Lexer("100").tokenize())
        assert tokens[0].type == TokenType.NUMBER
        assert tokens[0].value == 100

    def test_negative_integer(self):
        tokens = list(Lexer("-50").tokenize())
        assert tokens[0].type == TokenType.NUMBER
        assert tokens[0].value == -50

    def test_float(self):
        tokens = list(Lexer("3.14").tokenize())
        assert tokens[0].type == TokenType.NUMBER
        assert tokens[0].value == 3.14

    def test_negative_float(self):
        tokens = list(Lexer("-2.5").tokenize())
        assert tokens[0].type == TokenType.NUMBER
        assert tokens[0].value == -2.5


class TestLexerPairs:
    """Pair (coordinate) tokenization tests."""

    def test_pair_comma(self):
        tokens = list(Lexer("100,200").tokenize())
        assert tokens[0].type == TokenType.PAIR
        assert tokens[0].value == (100, 200)

    def test_pair_x(self):
        tokens = list(Lexer("100x200").tokenize())
        assert tokens[0].type == TokenType.PAIR
        assert tokens[0].value == (100, 200)

    def test_pair_floats(self):
        tokens = list(Lexer("10.5,20.5").tokenize())
        assert tokens[0].type == TokenType.PAIR
        assert tokens[0].value == (10.5, 20.5)

    def test_pair_negative(self):
        tokens = list(Lexer("-10,20").tokenize())
        assert tokens[0].type == TokenType.PAIR
        assert tokens[0].value == (-10, 20)


class TestLexerColors:
    """Color tokenization tests."""

    def test_short_hex(self):
        tokens = list(Lexer("#fff").tokenize())
        assert tokens[0].type == TokenType.COLOR
        assert tokens[0].value == "#fff"

    def test_long_hex(self):
        tokens = list(Lexer("#e94560").tokenize())
        assert tokens[0].type == TokenType.COLOR
        assert tokens[0].value == "#e94560"

    def test_hex_with_alpha(self):
        tokens = list(Lexer("#00000080").tokenize())
        assert tokens[0].type == TokenType.COLOR
        assert tokens[0].value == "#00000080"


class TestLexerStrings:
    """String tokenization tests."""

    def test_double_quoted(self):
        tokens = list(Lexer('"Hello World"').tokenize())
        assert tokens[0].type == TokenType.STRING
        assert tokens[0].value == "Hello World"

    def test_single_quoted(self):
        tokens = list(Lexer("'Hello World'").tokenize())
        assert tokens[0].type == TokenType.STRING
        assert tokens[0].value == "Hello World"

    def test_empty_string(self):
        tokens = list(Lexer('""').tokenize())
        assert tokens[0].type == TokenType.STRING
        assert tokens[0].value == ""


class TestLexerVariables:
    """Variable tokenization tests."""

    def test_simple_variable(self):
        tokens = list(Lexer("$primary").tokenize())
        assert tokens[0].type == TokenType.VAR
        assert tokens[0].value == "$primary"

    def test_variable_with_numbers(self):
        tokens = list(Lexer("$color1").tokenize())
        assert tokens[0].type == TokenType.VAR
        assert tokens[0].value == "$color1"


class TestLexerOperators:
    """Operator tokenization tests."""

    def test_equals(self):
        tokens = list(Lexer("=").tokenize())
        assert tokens[0].type == TokenType.EQUALS

    def test_colon(self):
        tokens = list(Lexer(":").tokenize())
        assert tokens[0].type == TokenType.COLON

    def test_arrow(self):
        tokens = list(Lexer("->").tokenize())
        assert tokens[0].type == TokenType.ARROW

    def test_brackets(self):
        tokens = list(Lexer("[]").tokenize())
        assert tokens[0].type == TokenType.LBRACKET
        assert tokens[1].type == TokenType.RBRACKET


class TestLexerIndentation:
    """Indentation handling tests."""

    def test_simple_indent(self):
        source = "parent\n  child"
        tokens = list(Lexer(source).tokenize())
        types = [t.type for t in tokens]
        assert TokenType.INDENT in types

    def test_dedent(self):
        source = "parent\n  child\nsibling"
        tokens = list(Lexer(source).tokenize())
        types = [t.type for t in tokens]
        assert TokenType.DEDENT in types

    def test_multiple_indent_levels(self):
        source = "level0\n  level1\n    level2\nback"
        tokens = list(Lexer(source).tokenize())
        types = [t.type for t in tokens]
        indent_count = types.count(TokenType.INDENT)
        dedent_count = types.count(TokenType.DEDENT)
        assert indent_count == 2
        assert dedent_count == 2


class TestLexerComplex:
    """Complex tokenization scenarios."""

    def test_canvas_with_size(self):
        tokens = list(Lexer("canvas 800x600").tokenize())
        assert tokens[0].type == TokenType.IDENT
        assert tokens[0].value == "canvas"
        assert tokens[1].type == TokenType.PAIR
        assert tokens[1].value == (800, 600)

    def test_variable_assignment(self):
        tokens = list(Lexer("$primary = #e94560").tokenize())
        assert tokens[0].type == TokenType.VAR
        assert tokens[1].type == TokenType.EQUALS
        assert tokens[2].type == TokenType.COLOR

    def test_shape_with_properties(self):
        tokens = list(Lexer("rect at 10,20 size 100x50").tokenize())
        types = [t.type for t in tokens]
        assert types[0] == TokenType.IDENT  # rect
        assert types[1] == TokenType.IDENT  # at
        assert types[2] == TokenType.PAIR   # 10,20
        assert types[3] == TokenType.IDENT  # size
        assert types[4] == TokenType.PAIR   # 100x50


class TestLexerPropertyBased:
    """Property-based tests using hypothesis."""

    @given(st.integers(min_value=-10000, max_value=10000))
    def test_integer_roundtrip(self, n):
        """Any integer should tokenize correctly."""
        tokens = list(Lexer(str(n)).tokenize())
        assert tokens[0].type == TokenType.NUMBER
        assert tokens[0].value == n

    @given(st.floats(min_value=-1000, max_value=1000, allow_nan=False, allow_infinity=False))
    @settings(max_examples=100)
    def test_float_roundtrip(self, n):
        """Floats should tokenize correctly."""
        n = round(n, 6)
        if n == int(n):
            return  # Skip integers
        # Skip values that would render as scientific notation (DSL doesn't support it)
        s = str(n)
        if 'e' in s or 'E' in s:
            return
        tokens = list(Lexer(s).tokenize())
        assert tokens[0].type == TokenType.NUMBER
        assert abs(tokens[0].value - n) < 0.0001

    @given(
        st.integers(min_value=0, max_value=1000),
        st.integers(min_value=0, max_value=1000)
    )
    def test_pair_roundtrip(self, x, y):
        """Integer pairs should tokenize correctly."""
        tokens = list(Lexer(f"{x},{y}").tokenize())
        assert tokens[0].type == TokenType.PAIR
        assert tokens[0].value == (x, y)

    @given(st.from_regex(r'[a-zA-Z_][a-zA-Z0-9_-]{0,20}', fullmatch=True))
    def test_identifier_valid(self, ident):
        """Valid identifiers should tokenize as IDENT."""
        tokens = list(Lexer(ident).tokenize())
        assert tokens[0].type == TokenType.IDENT
        assert tokens[0].value == ident

    @given(st.from_regex(r'#[0-9a-fA-F]{3}', fullmatch=True))
    def test_short_color_valid(self, color):
        """Short hex colors should tokenize correctly."""
        tokens = list(Lexer(color).tokenize())
        assert tokens[0].type == TokenType.COLOR
        assert tokens[0].value == color

    @given(st.from_regex(r'#[0-9a-fA-F]{6}', fullmatch=True))
    def test_long_color_valid(self, color):
        """Long hex colors should tokenize correctly."""
        tokens = list(Lexer(color).tokenize())
        assert tokens[0].type == TokenType.COLOR
        assert tokens[0].value == color

    @given(st.text(alphabet=st.characters(whitelist_categories=['L', 'N', 'P', 'S', 'Z'], blacklist_characters='"\''), min_size=0, max_size=50))
    def test_string_roundtrip(self, content):
        """String content should be preserved."""
        assume('\n' not in content)
        tokens = list(Lexer(f'"{content}"').tokenize())
        assert tokens[0].type == TokenType.STRING
        assert tokens[0].value == content

