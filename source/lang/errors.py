"""Centralized error handling for the Iconoglott DSL."""

from dataclasses import dataclass, field
from enum import IntEnum
from typing import TypeAlias

# Error codes grouped by category (ranges of 100)
# 1000-1099: Lexer errors
# 2000-2099: Parser errors  
# 3000-3099: Runtime/Evaluation errors
# 4000-4099: WebSocket/Transport errors
# 5000-5099: Rendering errors


class ErrorCode(IntEnum):
    """Standardized error codes for the DSL."""
    
    # Lexer Errors (1000-1099)
    LEX_UNKNOWN_CHAR = 1001
    LEX_UNTERMINATED_STRING = 1002
    LEX_INVALID_NUMBER = 1003
    LEX_INVALID_COLOR = 1004
    LEX_INVALID_PAIR = 1005
    
    # Parser Errors (2000-2099)
    PARSE_UNEXPECTED_TOKEN = 2001
    PARSE_EXPECTED_VALUE = 2002
    PARSE_UNDEFINED_VAR = 2003
    PARSE_UNKNOWN_COMMAND = 2004
    PARSE_MISSING_BRACKET = 2005
    PARSE_INVALID_PROPERTY = 2006
    PARSE_EXPECTED_COLOR = 2007
    PARSE_EXPECTED_NUMBER = 2008
    PARSE_EXPECTED_PAIR = 2009
    PARSE_EXPECTED_STRING = 2010
    PARSE_MISSING_EQUALS = 2011
    PARSE_EMPTY_VALUE = 2012
    PARSE_RECOVERY = 2013  # Parser recovered from error at sync point
    
    # Runtime/Eval Errors (3000-3099)
    EVAL_INVALID_SHAPE = 3001
    EVAL_MISSING_PROPERTY = 3002
    EVAL_TYPE_MISMATCH = 3003
    EVAL_INVALID_CANVAS = 3004
    EVAL_INVALID_TRANSFORM = 3005
    
    # WebSocket Errors (4000-4099)
    WS_INVALID_MESSAGE = 4001
    WS_INVALID_PAYLOAD = 4002
    WS_CONNECTION_ERROR = 4003
    WS_BROADCAST_ERROR = 4004
    
    # Render Errors (5000-5099)
    RENDER_INVALID_SHAPE = 5001
    RENDER_RUST_ERROR = 5002
    RENDER_SVG_ERROR = 5003


class Severity(IntEnum):
    """Error severity levels."""
    INFO = 0
    WARNING = 1
    ERROR = 2
    FATAL = 3


@dataclass(frozen=True, slots=True)
class ErrorInfo:
    """Structured error information."""
    code: ErrorCode
    message: str
    line: int = 0
    col: int = 0
    severity: Severity = Severity.ERROR
    context: str | None = None
    
    @property
    def category(self) -> str:
        """Get error category from code range."""
        c = self.code.value // 1000
        return {1: "lexer", 2: "parser", 3: "runtime", 4: "websocket", 5: "render"}.get(c, "unknown")
    
    def to_dict(self) -> dict:
        """Convert to serializable dict for API responses."""
        return {
            "code": self.code.value,
            "category": self.category,
            "message": self.message,
            "line": self.line,
            "col": self.col,
            "severity": self.severity.name.lower(),
            **({"context": self.context} if self.context else {}),
        }
    
    def __str__(self) -> str:
        loc = f"[{self.line + 1}:{self.col}] " if self.line or self.col else ""
        return f"{loc}E{self.code.value}: {self.message}"


class IconoglottError(Exception):
    """Base exception for all DSL errors."""
    
    def __init__(self, info: ErrorInfo):
        self.info = info
        super().__init__(str(info))
    
    @classmethod
    def create(cls, code: ErrorCode, msg: str, line: int = 0, col: int = 0, 
               severity: Severity = Severity.ERROR, context: str | None = None):
        return cls(ErrorInfo(code, msg, line, col, severity, context))
    
    def to_dict(self) -> dict:
        return self.info.to_dict()


class LexerError(IconoglottError):
    """Lexer/tokenization errors."""
    pass


class ParserError(IconoglottError):
    """Parser/syntax errors."""
    pass


class EvalError(IconoglottError):
    """Runtime/evaluation errors."""
    pass


class WSError(IconoglottError):
    """WebSocket/transport errors."""
    pass


class RenderError(IconoglottError):
    """Rendering errors."""
    pass


# Type alias for error collections
ErrorList: TypeAlias = list[ErrorInfo]


def collect_errors(*errors: ErrorInfo | IconoglottError | None) -> ErrorList:
    """Collect and flatten errors into a list of ErrorInfo."""
    result: ErrorList = []
    for e in errors:
        if e is None:
            continue
        result.append(e.info if isinstance(e, IconoglottError) else e)
    return result


def errors_to_response(errors: ErrorList, include_warnings: bool = True) -> list[dict]:
    """Convert error list to API response format."""
    return [
        e.to_dict() for e in errors 
        if include_warnings or e.severity >= Severity.ERROR
    ]


def has_fatal(errors: ErrorList) -> bool:
    """Check if any error is fatal."""
    return any(e.severity == Severity.FATAL for e in errors)


def error_summary(errors: ErrorList) -> str:
    """Generate a summary string of all errors."""
    if not errors:
        return "No errors"
    return "; ".join(str(e) for e in errors)

