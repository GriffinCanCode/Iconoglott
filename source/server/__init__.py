"""Iconoglott Server - Real-time WebSocket server."""

from .app import create_app
from lang import (
    ErrorCode, ErrorInfo, Severity,
    IconoglottError, WSError, errors_to_response,
)

__all__ = [
    "create_app",
    "ErrorCode", "ErrorInfo", "Severity",
    "IconoglottError", "WSError", "errors_to_response",
]

