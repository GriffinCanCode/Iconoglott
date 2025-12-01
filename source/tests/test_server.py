"""Tests for WebSocket server."""

import pytest
from fastapi.testclient import TestClient
from server.app import app


class TestServer:
    def test_index(self):
        client = TestClient(app)
        resp = client.get("/")
        assert resp.status_code == 200

    def test_websocket_render(self):
        client = TestClient(app)
        with client.websocket_connect("/ws") as ws:
            ws.send_json({"type": "source", "payload": "canvas 100 100"})
            msg = ws.receive_json()
            assert msg["type"] == "render"
            assert "<svg" in msg["svg"]

