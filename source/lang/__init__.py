"""Iconoglott DSL - Visual language interpreter with Rust-powered core.

The Rust core (iconoglott_core) is the SINGLE SOURCE OF TRUTH for:
- DSL lexing and parsing
- SVG rendering
- Scene diffing

Python provides:
- LangChain/AI tool integrations
- WebSocket server
- High-level convenience wrappers

Example:
    >>> from lang import render
    >>> svg = render('''
    ...     canvas 400x300 fill #1a1a2e
    ...     circle at 200,150 radius 50
    ...       fill #e94560
    ... ''')

For AI/LangChain integration:
    >>> from lang.tools import create_tool
    >>> tool = create_tool()  # Ready for LangChain agent
"""

# Import Rust core (required)
try:
    import iconoglott_core as rust
except ImportError as e:
    raise ImportError(
        "Rust core module 'iconoglott_core' not found. "
        "Build it with: cd source/core && maturin develop --release"
    ) from e

# Re-export Rust types for backwards compatibility
from iconoglott_core import (
    # Lexer
    TokenType, Token, Lexer,
    # Parser
    Parser, AstCanvas, AstShape, AstStyle, AstTransform,
    ShadowDef, GradientDef, ParseError,
    # Scene/Shapes (for rendering)
    Scene, Gradient, Filter,
    Rect, Circle, Ellipse, Line, Path, Polygon, Text, Image,
    Style, Color,
)

# Python-specific types for backward compat (thin wrappers over Rust)
from .types import Node, Canvas, Shape, Transform
from .types import Style as PyStyle  # Alias to avoid collision
from .errors import (
    ErrorCode, ErrorInfo, Severity, ErrorList,
    IconoglottError, LexerError, ParserError, EvalError, WSError, RenderError,
    collect_errors, errors_to_response, has_fatal, error_summary,
)
from .eval import Interpreter, SceneState

__version__ = "0.1.0"


def render(source: str) -> str:
    """Render iconoglott DSL code to SVG string.

    Args:
        source: DSL source code

    Returns:
        SVG string
    """
    return Interpreter().eval(source).to_svg()


def parse(source: str) -> SceneState:
    """Parse DSL and return scene state for inspection/manipulation.

    Args:
        source: DSL source code

    Returns:
        SceneState with canvas, shapes, and any errors
    """
    return Interpreter().eval(source)


__all__ = [
    # High-level API
    "render", "parse", "Interpreter", "SceneState",
    # Rust core types
    "TokenType", "Token", "Lexer",
    "Parser", "AstCanvas", "AstShape", "AstStyle", "AstTransform",
    "ShadowDef", "GradientDef", "ParseError",
    "Scene", "Gradient", "Filter",
    "Rect", "Circle", "Ellipse", "Line", "Path", "Polygon", "Text", "Image",
    "Style", "Color",
    # Python types
    "Node", "Canvas", "Shape", "Transform", "PyStyle",
    # Errors
    "ErrorCode", "ErrorInfo", "Severity", "ErrorList",
    "IconoglottError", "LexerError", "ParserError", "EvalError", "WSError", "RenderError",
    "collect_errors", "errors_to_response", "has_fatal", "error_summary",
    "__version__",
]
