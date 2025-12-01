"""Type definitions for the DSL - backward compatibility wrappers over Rust core.

The Rust core (iconoglott_core) is the single source of truth.
These types exist only for backward compatibility with existing Python code.
"""

from dataclasses import dataclass, field
from typing import Union

# Re-export Rust types directly
try:
    from iconoglott_core import (
        TokenType, Token, CanvasSize,
        AstStyle, AstTransform, AstCanvas, AstShape,
        ShadowDef, GradientDef, ParseError as RustParseError,
    )
except ImportError:
    # Fallback for when Rust core isn't built (e.g., during type checking)
    TokenType = None  # type: ignore
    Token = None  # type: ignore
    CanvasSize = None  # type: ignore

# Standard canvas sizes (10-tier system)
CANVAS_SIZES = {
    "nano": 16, "micro": 24, "tiny": 32, "small": 48, "medium": 64,
    "large": 96, "xlarge": 128, "huge": 192, "massive": 256, "giant": 512,
}

from .errors import ErrorInfo, ErrorCode


# ─────────────────────────────────────────────────────────────────────────────
# Backward compatibility wrappers (thin layers over Rust types)
# ─────────────────────────────────────────────────────────────────────────────

@dataclass(slots=True)
class Style:
    """Shape styling properties. Wraps Rust AstStyle."""
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
    """Transform properties. Wraps Rust AstTransform."""
    translate: tuple[float, float] | None = None
    rotate: float = 0.0
    scale: tuple[float, float] | None = None
    origin: tuple[float, float] | None = None


@dataclass(slots=True)
class Canvas:
    """Canvas definition using standardized sizes. Wraps Rust AstCanvas."""
    size: str = "medium"  # nano|micro|tiny|small|medium|large|xlarge|huge|massive|giant
    fill: str = "#fff"
    
    @property
    def width(self) -> int:
        return CANVAS_SIZES.get(self.size, 64)
    
    @property
    def height(self) -> int:
        return CANVAS_SIZES.get(self.size, 64)


@dataclass(slots=True)
class Shape:
    """Generic shape with properties. Wraps Rust AstShape."""
    kind: str
    props: dict = field(default_factory=dict)
    style: Style = field(default_factory=Style)
    transform: Transform = field(default_factory=Transform)
    children: list["Shape"] = field(default_factory=list)


@dataclass(slots=True)
class Node:
    """AST node for backward compatibility."""
    type: str
    value: Union[str, float, Canvas, Shape, dict, None] = None
    children: list["Node"] = field(default_factory=list)


# Deprecated: Use ErrorInfo from errors.py instead
@dataclass(slots=True)
class ParseError:
    """Syntax error with location. Deprecated: use ErrorInfo or Rust ParseError."""
    message: str
    line: int
    col: int = 0
    code: ErrorCode = ErrorCode.PARSE_UNEXPECTED_TOKEN
    
    def to_error_info(self) -> ErrorInfo:
        return ErrorInfo(self.code, self.message, self.line, self.col)
    
    def __str__(self) -> str:
        return f"E{self.code.value} Line {self.line + 1}: {self.message}"
