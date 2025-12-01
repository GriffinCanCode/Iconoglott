# Root Makefile - Iconoglott
SHELL := /bin/bash
.PHONY: all clean install dev test start stop killports lint build

VENV := .venv
PYTHON := $(VENV)/bin/python3
PIP := $(VENV)/bin/pip

# Setup venv and install deps
install: $(VENV)
	$(PIP) install -e ".[dev]"

$(VENV):
	python3 -m venv $(VENV)

# Dev install
dev: install
	@echo "Dev environment ready"

# Delegate to source Makefile
killports:
	@$(MAKE) -C source killports

start:
	@$(MAKE) -C source start

start-bg:
	@$(MAKE) -C source start-bg

stop:
	@$(MAKE) -C source stop

test:
	@$(MAKE) -C source test

lint:
	@$(MAKE) -C source lint

# Clean everything
clean:
	@$(MAKE) -C source clean
	@rm -rf $(VENV) *.egg-info dist build
	@echo "Cleaned all"

# Build distribution
build: clean
	$(PYTHON) -m build

