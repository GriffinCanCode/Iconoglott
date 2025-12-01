"""Iconoglott DSL - Visual language interpreter with Rust-powered rendering.

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

from .types import Token, Node, Canvas, Shape
from .lexer import Lexer
from .parser import Parser
from .eval import Interpreter, SceneState
from .errors import (
    ErrorCode, ErrorInfo, Severity, ErrorList,
    IconoglottError, LexerError, ParserError, EvalError, WSError, RenderError,
    collect_errors, errors_to_response, has_fatal, error_summary,
)

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
    "render", "parse", "Interpreter", "SceneState",
    "Token", "Node", "Canvas", "Shape", "Lexer", "Parser",
    "ErrorCode", "ErrorInfo", "Severity", "ErrorList",
    "IconoglottError", "LexerError", "ParserError", "EvalError", "WSError", "RenderError",
    "collect_errors", "errors_to_response", "has_fatal", "error_summary",
    "__version__",
]

