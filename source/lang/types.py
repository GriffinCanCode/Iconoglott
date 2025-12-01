"""Type definitions for the DSL."""

from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Union

from .errors import ErrorInfo, ErrorCode, Severity


class TokenType(Enum):
    """Token types for lexical analysis."""
    IDENT = auto()
    NUMBER = auto()
    STRING = auto()
    COLOR = auto()
    VAR = auto()
    PAIR = auto()
    COLON = auto()
    EQUALS = auto()
    ARROW = auto()
    LBRACKET = auto()
    RBRACKET = auto()
    NEWLINE = auto()
    INDENT = auto()
    DEDENT = auto()
    EOF = auto()
    COMMENT = auto()


@dataclass(frozen=True, slots=True)
class Token:
    """Lexical token."""
    type: TokenType
    value: Union[str, float, tuple]
    line: int = 0
    col: int = 0


# Deprecated: Use ErrorInfo from errors.py instead
# Kept for backward compatibility during migration
@dataclass(slots=True)
class ParseError:
    """Syntax error with location. Deprecated: use ErrorInfo."""
    message: str
    line: int
    col: int = 0
    code: ErrorCode = ErrorCode.PARSE_UNEXPECTED_TOKEN
    
    def to_error_info(self) -> ErrorInfo:
        return ErrorInfo(self.code, self.message, self.line, self.col)
    
    def __str__(self) -> str:
        return f"E{self.code.value} Line {self.line + 1}: {self.message}"


@dataclass(slots=True)
class Style:
    """Shape styling properties."""
    fill: str | None = None
    stroke: str | None = None
    stroke_width: float = 1.0
    opacity: float = 1.0
    corner: float = 0.0
    font: str | None = None
    font_size: float = 16.0
    font_weight: str = "normal"
    text_anchor: str = "start"
    shadow: dict | None = None
    gradient: dict | None = None


@dataclass(slots=True)
class Transform:
    """Transform properties."""
    translate: tuple[float, float] | None = None
    rotate: float = 0.0
    scale: tuple[float, float] | None = None
    origin: tuple[float, float] | None = None


@dataclass(slots=True)
class Canvas:
    """Canvas definition."""
    width: int = 800
    height: int = 600
    fill: str = "#fff"


@dataclass(slots=True)
class Shape:
    """Generic shape with properties."""
    kind: str
    props: dict = field(default_factory=dict)
    style: Style = field(default_factory=Style)
    transform: Transform = field(default_factory=Transform)
    children: list["Shape"] = field(default_factory=list)


@dataclass(slots=True)
class Node:
    """AST node."""
    type: str
    value: Union[str, float, Canvas, Shape, dict, None] = None
    children: list["Node"] = field(default_factory=list)
