"""Lexer for the visual DSL."""

import re
from typing import Iterator
from .types import Token, TokenType


class Lexer:
    """Tokenize DSL source into a stream of tokens."""

    PATTERNS = [
        # Comments (skip)
        (r'//[^\n]*', None),
        # Variables
        (r'\$[a-zA-Z_][a-zA-Z0-9_]*', TokenType.VAR),
        # Colors
        (r'#[0-9a-fA-F]{3,8}\b', TokenType.COLOR),
        # Pairs: 100,200 or 100x200
        (r'-?\d+\.?\d*[,x]-?\d+\.?\d*', TokenType.PAIR),
        # Strings
        (r'"[^"]*"', TokenType.STRING),
        (r"'[^']*'", TokenType.STRING),
        # Numbers
        (r'-?\d+\.?\d*', TokenType.NUMBER),
        # Operators
        (r'->', TokenType.ARROW),
        (r':', TokenType.COLON),
        (r'=', TokenType.EQUALS),
        # Identifiers
        (r'[a-zA-Z_][a-zA-Z0-9_-]*', TokenType.IDENT),
    ]

    def __init__(self, source: str):
        self.source = source
        self.lines = source.split('\n')
        self.indent_stack = [0]

    def tokenize(self) -> Iterator[Token]:
        """Generate tokens from source."""
        for lineno, line in enumerate(self.lines):
            stripped = line.lstrip()
            
            # Skip empty and comment-only lines
            if not stripped or stripped.startswith('//'):
                continue

            indent = len(line) - len(stripped)
            yield from self._handle_indent(indent, lineno)
            yield from self._tokenize_line(stripped, lineno)
            yield Token(TokenType.NEWLINE, '\n', lineno, len(line))

        # Close remaining indents
        while len(self.indent_stack) > 1:
            self.indent_stack.pop()
            yield Token(TokenType.DEDENT, '', len(self.lines) - 1, 0)

        yield Token(TokenType.EOF, '', len(self.lines) - 1, 0)

    def _handle_indent(self, indent: int, line: int) -> Iterator[Token]:
        """Handle indentation changes."""
        if indent > self.indent_stack[-1]:
            self.indent_stack.append(indent)
            yield Token(TokenType.INDENT, '', line, 0)
        else:
            while indent < self.indent_stack[-1]:
                self.indent_stack.pop()
                yield Token(TokenType.DEDENT, '', line, 0)

    def _tokenize_line(self, line: str, lineno: int) -> Iterator[Token]:
        """Tokenize a single line."""
        pos = 0
        while pos < len(line):
            # Skip whitespace
            if line[pos].isspace():
                pos += 1
                continue

            matched = False
            for pattern, ttype in self.PATTERNS:
                if m := re.match(pattern, line[pos:]):
                    if ttype is not None:  # Skip comments
                        val = self._parse_value(m.group(), ttype)
                        yield Token(ttype, val, lineno, pos)
                    pos += len(m.group())
                    matched = True
                    break
            
            if not matched:
                pos += 1  # Skip unknown char

    def _parse_value(self, raw: str, ttype: TokenType):
        """Parse token value based on type."""
        match ttype:
            case TokenType.NUMBER:
                return float(raw) if '.' in raw else int(raw)
            case TokenType.STRING:
                return raw[1:-1]  # Strip quotes
            case TokenType.PAIR:
                sep = 'x' if 'x' in raw else ','
                a, b = raw.split(sep)
                return (float(a) if '.' in a else int(a),
                        float(b) if '.' in b else int(b))
            case _:
                return raw
