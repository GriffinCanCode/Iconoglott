"""WebSocket handlers for real-time DSL interpretation."""

import asyncio
import json
import logging
from dataclasses import dataclass, field
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


@dataclass
class ConnectionState:
    """Per-connection backpressure state for message coalescing."""
    rendering: bool = False
    pending_source: str | None = None
    lock: asyncio.Lock = field(default_factory=asyncio.Lock)


class ConnectionManager:
    """Manage active WebSocket connections with backpressure."""

    def __init__(self):
        self.active: Set[WebSocket] = set()
        self.states: dict[WebSocket, ConnectionState] = {}
        self.interpreter = Interpreter()

    async def connect(self, ws: WebSocket):
        await ws.accept()
        self.active.add(ws)
        self.states[ws] = ConnectionState()

    def disconnect(self, ws: WebSocket):
        self.active.discard(ws)
        self.states.pop(ws, None)

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
            result["errors"] = errors_to_response(state.error_infos) if state.error_infos else []
            return result
        except Exception as e:
            logger.exception("Processing failed")
            return {
                "type": "error",
                "message": str(e),
                "errors": [ErrorInfo(ErrorCode.EVAL_INVALID_SHAPE, str(e)).to_dict()]
            }

    async def process_with_backpressure(self, ws: WebSocket, source: str) -> dict | None:
        """Process source with coalescing - drops older pending sources."""
        state = self.states.get(ws)
        if not state:
            return await self.process(source)

        async with state.lock:
            if state.rendering:
                # Replace pending source - drop old, keep newest
                state.pending_source = source
                return None  # Signal: coalesced, no immediate response
            state.rendering = True

        try:
            result = await self.process(source)
            await ws.send_text(json.dumps(result))

            # Process any pending source that arrived during render
            while True:
                async with state.lock:
                    next_source = state.pending_source
                    state.pending_source = None
                    if next_source is None:
                        state.rendering = False
                        break

                result = await self.process(next_source)
                await ws.send_text(json.dumps(result))

            return None  # Already sent
        except Exception:
            async with state.lock:
                state.rendering = False
                state.pending_source = None
            raise


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
    """Handle WebSocket connections for real-time rendering with backpressure."""
    await manager.connect(ws)
    try:
        while True:
            data = await ws.receive_text()
            try:
                msg = json.loads(data)
                msg_type = msg.get("type")

                if msg_type == "source":
                    # Backpressure: coalesces if render in progress
                    await manager.process_with_backpressure(ws, msg.get("payload", ""))
                elif msg_type == "ping":
                    await ws.send_text(json.dumps({"type": "pong"}))
                else:
                    await ws.send_text(json.dumps(
                        _ws_error(ErrorCode.WS_INVALID_MESSAGE, f"Unknown message type: {msg_type}")
                    ))
            except json.JSONDecodeError:
                # Raw source for backward compatibility
                await manager.process_with_backpressure(ws, data)
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
