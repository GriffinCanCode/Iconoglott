# Iconoglott Integration Test Suite
# Tests both Python backend (LangChain) and Frontend (NPM package)

.PHONY: all install backend frontend server clean help

# Use project venv
VENV := ../.venv/bin
PYTHON := $(VENV)/python

# Default: run all tests
all: backend frontend
	@echo ""
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  ALL TESTS COMPLETE"
	@echo "═══════════════════════════════════════════════════════════════"

# ─────────────────────────────────────────────────────────────────────────────
# Installation
# ─────────────────────────────────────────────────────────────────────────────

install: install-deps install-frontend
	@echo "✓ All dependencies installed"

install-deps:
	@echo "Installing Python dependencies..."
	$(VENV)/pip install langchain-core pydantic -q

install-frontend:
	@echo "Installing NPM dependencies..."
	@cd frontend && npm install --silent

# Build npm package if needed
build-npm:
	@echo "Building NPM package..."
	cd ../distribution/npm && npm run build

# Build Rust core if needed
build-rust:
	@echo "Building Rust core..."
	cd ../source/core && maturin develop --release

# ─────────────────────────────────────────────────────────────────────────────
# Tests
# ─────────────────────────────────────────────────────────────────────────────

# Python backend tests (LangChain integration)
backend:
	@echo ""
	@echo "Running Python backend tests..."
	@$(PYTHON) backend/test_backend.py

# Frontend package tests (Node.js based)
frontend:
	@echo ""
	@echo "Running Frontend package tests..."
	@cd frontend && npm test

# ─────────────────────────────────────────────────────────────────────────────
# Development Servers
# ─────────────────────────────────────────────────────────────────────────────

# Start Python backend server
server:
	@echo "Starting Python server on :8765..."
	cd .. && $(PYTHON) -m uvicorn source.server.app:app --host 0.0.0.0 --port 8765 --reload

# Start frontend dev server (proxies to backend)
dev:
	@echo "Starting frontend dev server on :3456..."
	@cd frontend && npm run dev

# Start both servers (run in separate terminals or use tmux)
dev-all:
	@echo "Starting both servers..."
	@echo "Backend: http://localhost:8765"
	@echo "Frontend: http://localhost:3456"
	@$(MAKE) -j2 server dev

# ─────────────────────────────────────────────────────────────────────────────
# Cleanup
# ─────────────────────────────────────────────────────────────────────────────

clean:
	rm -rf frontend/node_modules frontend/dist
	rm -rf backend/__pycache__ backend/*.egg-info
	@echo "✓ Cleaned"

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────

help:
	@echo ""
	@echo "Iconoglott Integration Tests"
	@echo "──────────────────────────────────────────────────────────"
	@echo ""
	@echo "  make all          Run all tests (backend + frontend)"
	@echo "  make install      Install all dependencies"
	@echo "  make backend      Run Python/LangChain tests only"
	@echo "  make frontend     Run NPM package tests only"
	@echo ""
	@echo "  make server       Start Python backend server (:8765)"
	@echo "  make dev          Start frontend dev server (:3456)"
	@echo "  make dev-all      Start both servers"
	@echo ""
	@echo "  make build-rust   Rebuild Rust core (maturin)"
	@echo "  make build-npm    Rebuild NPM package"
	@echo "  make clean        Remove build artifacts"
	@echo ""

