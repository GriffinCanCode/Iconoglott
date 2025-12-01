"""WebSocket handlers for real-time DSL interpretation."""

import json
import logging
from typing import Set
from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from pydantic import BaseModel

from lang import Interpreter, ErrorCode, ErrorInfo, Severity, errors_to_response

router = APIRouter()
logger = logging.getLogger(__name__)


class Message(BaseModel):
    """WebSocket message schema."""
    type: str
    payload: str | dict


class ConnectionManager:
    """Manage active WebSocket connections."""

    def __init__(self):
        self.active: Set[WebSocket] = set()
        self.interpreter = Interpreter()

    async def connect(self, ws: WebSocket):
        await ws.accept()
        self.active.add(ws)

    def disconnect(self, ws: WebSocket):
        self.active.discard(ws)

    async def broadcast(self, message: dict):
        """Send message to all connected clients."""
        data = json.dumps(message)
        disconnected = []
        for ws in self.active:
            try:
                await ws.send_text(data)
            except Exception as e:
                logger.warning(f"Broadcast failed: {e}")
                disconnected.append(ws)
        for ws in disconnected:
            self.active.discard(ws)

    async def process(self, source: str) -> dict:
        """Interpret DSL source and return SVG with errors."""
        try:
            state = self.interpreter.eval(source)
            result = {"type": "render", "svg": state.to_svg()}
            
            # Use error_infos (new structured format) when available
            result["errors"] = errors_to_response(state.error_infos) if state.error_infos else []
            
            return result
        except Exception as e:
            logger.exception("Processing failed")
            return {
                "type": "error",
                "message": str(e),
                "errors": [ErrorInfo(ErrorCode.EVAL_INVALID_SHAPE, str(e)).to_dict()]
            }


manager = ConnectionManager()


def _ws_error(code: ErrorCode, msg: str) -> dict:
    """Build a WebSocket error response."""
    return {
        "type": "error",
        "message": msg,
        "errors": [ErrorInfo(code, msg, severity=Severity.ERROR).to_dict()]
    }


@router.websocket("/ws")
async def websocket_endpoint(ws: WebSocket):
    """Handle WebSocket connections for real-time rendering."""
    await manager.connect(ws)
    try:
        while True:
            data = await ws.receive_text()
            try:
                msg = json.loads(data)
                msg_type = msg.get("type")
                
                if msg_type == "source":
                    result = await manager.process(msg.get("payload", ""))
                elif msg_type == "ping":
                    result = {"type": "pong"}
                else:
                    result = _ws_error(ErrorCode.WS_INVALID_MESSAGE, f"Unknown message type: {msg_type}")
                
                await ws.send_text(json.dumps(result))
            except json.JSONDecodeError as e:
                # Try processing as raw source for backward compatibility
                result = await manager.process(data)
                await ws.send_text(json.dumps(result))
            except Exception as e:
                logger.exception("WebSocket handler error")
                await ws.send_text(json.dumps(
                    _ws_error(ErrorCode.WS_CONNECTION_ERROR, f"Internal error: {e}")
                ))
    except WebSocketDisconnect:
        manager.disconnect(ws)
    except Exception as e:
        logger.exception("WebSocket connection error")
        manager.disconnect(ws)
