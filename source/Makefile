# Source Makefile - Iconoglott
SHELL := /bin/bash
.PHONY: all clean install dev test start stop killports lint wasm wasm-dev

# Paths
SCRIPTS := $(dir $(lastword $(MAKEFILE_LIST)))scripts
VENV := $(shell cd .. && pwd)/.venv
PYTHON := $(VENV)/bin/python3

# Kill active ports
killports:
	@chmod +x $(SCRIPTS)/killports.sh
	@$(SCRIPTS)/killports.sh

# Start backend server
start: killports
	@echo "Starting backend on port 8765..."
	$(PYTHON) -m uvicorn server.app:app --host 0.0.0.0 --port 8765 --reload

# Start in background
start-bg: killports
	@echo "Starting backend in background..."
	@nohup $(PYTHON) -m uvicorn server.app:app --host 0.0.0.0 --port 8765 --reload > /dev/null 2>&1 &
	@echo "Backend started on port 8765"

# Stop server
stop: killports
	@echo "Backend stopped"

# Run tests
test:
	@cd .. && $(PYTHON) -m pytest source/tests -v

# Run tests with coverage
test-cov:
	@cd .. && $(PYTHON) -m pytest source/tests -v --cov=source/lang --cov=source/server --cov-report=term-missing --cov-report=html

# Run Rust tests
test-rust:
	@cd core && cargo test

# Run all tests (Python + Rust)
test-all: test-rust test

# Lint
lint:
	@$(PYTHON) -m ruff check .

# Clean pycache
clean:
	@find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@find . -type f -name "*.pyc" -delete 2>/dev/null || true
	@echo "Cleaned pycache"

# Build WASM module (production)
wasm:
	@echo "Building WASM module..."
	@cd core && wasm-pack build --target web --features wasm -- --no-default-features
	@cp -r core/pkg ../distribution/npm/src/wasm
	@echo "WASM built to distribution/npm/src/wasm"

# Build WASM module (development)
wasm-dev:
	@echo "Building WASM module (dev)..."
	@cd core && wasm-pack build --target web --features wasm --dev -- --no-default-features
	@cp -r core/pkg ../distribution/npm/src/wasm
	@echo "WASM built (dev) to distribution/npm/src/wasm"

# Build playground
playground:
	@echo "Building playground..."
	@cd ../distribution/npm && npm run build
	@cd ../distribution/npm/playground && npm install && npm run build
	@echo "Playground built to distribution/npm/playground/dist"

# Full build (WASM + npm + playground)
build-all: wasm
	@cd ../distribution/npm && npm install && npm run build
	@cd ../distribution/npm/playground && npm install && npm run build
	@echo "Full build complete"

