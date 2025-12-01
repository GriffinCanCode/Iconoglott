# Source Makefile - Iconoglott
SHELL := /bin/bash
.PHONY: all clean install dev test start stop killports lint

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

# Lint
lint:
	@$(PYTHON) -m ruff check .

# Clean pycache
clean:
	@find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@find . -type f -name "*.pyc" -delete 2>/dev/null || true
	@echo "Cleaned pycache"

