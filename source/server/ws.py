"""WebSocket handlers for real-time DSL interpretation."""

import json
from typing import Set
from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from pydantic import BaseModel

from lang import Interpreter

router = APIRouter()


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
        for ws in self.active.copy():
            try:
                await ws.send_text(data)
            except Exception:
                self.active.discard(ws)

    async def process(self, source: str) -> dict:
        """Interpret DSL source and return SVG."""
        try:
            state = self.interpreter.eval(source)
            return {"type": "render", "svg": state.to_svg()}
        except Exception as e:
            return {"type": "error", "message": str(e)}


manager = ConnectionManager()


@router.websocket("/ws")
async def websocket_endpoint(ws: WebSocket):
    """Handle WebSocket connections for real-time rendering."""
    await manager.connect(ws)
    try:
        while True:
            data = await ws.receive_text()
            try:
                msg = json.loads(data)
                if msg.get("type") == "source":
                    result = await manager.process(msg.get("payload", ""))
                    await ws.send_text(json.dumps(result))
            except json.JSONDecodeError:
                # Treat raw text as source code
                result = await manager.process(data)
                await ws.send_text(json.dumps(result))
    except WebSocketDisconnect:
        manager.disconnect(ws)

