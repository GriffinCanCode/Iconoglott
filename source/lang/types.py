"""Type definitions for the DSL."""

from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Union


class TokenType(Enum):
    """Token types for lexical analysis."""
    # Literals
    IDENT = auto()      # keywords, shape names
    NUMBER = auto()     # 100, 3.14
    STRING = auto()     # "hello"
    COLOR = auto()      # #fff, #e94560
    VAR = auto()        # $name
    
    # Compound literals
    PAIR = auto()       # 100,200 or 100x200
    
    # Operators
    COLON = auto()      # :
    EQUALS = auto()     # =
    ARROW = auto()      # ->
    
    # Structure
    NEWLINE = auto()
    INDENT = auto()
    DEDENT = auto()
    EOF = auto()
    
    # Comments
    COMMENT = auto()    # // comment


@dataclass(frozen=True, slots=True)
class Token:
    """Lexical token."""
    type: TokenType
    value: Union[str, float, tuple]
    line: int = 0
    col: int = 0


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
