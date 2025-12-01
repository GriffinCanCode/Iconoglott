# Root Makefile - Iconoglott
SHELL := /bin/bash
.PHONY: all clean install dev test start stop killports lint build core

ROOT_DIR := $(shell pwd)
VENV := .venv
PYTHON := $(VENV)/bin/python3
PIP := $(VENV)/bin/pip
MATURIN := $(ROOT_DIR)/$(VENV)/bin/maturin

# Setup venv, install deps, and build Rust core (required)
install: $(VENV) core
	@echo "Installation complete with Rust core"

$(VENV):
	python3 -m venv $(VENV)
	$(PIP) install -e ".[dev]"

# Build Rust core module (required for the interpreter)
core: $(VENV)
	@echo "Building Rust core..."
	@cd source/core && $(MATURIN) develop --release
	@echo "Rust core ready"

# Dev install (alias for install)
dev: install

# Delegate to source Makefile
killports:
	@$(MAKE) -C source killports

start: install
	@$(MAKE) -C source start

start-bg: install
	@$(MAKE) -C source start-bg

stop:
	@$(MAKE) -C source stop

test: install
	@$(MAKE) -C source test

lint:
	@$(MAKE) -C source lint

# Clean everything
clean:
	@$(MAKE) -C source clean
	@rm -rf $(VENV) *.egg-info dist build source/core/target
	@echo "Cleaned all"

# Build distribution (includes builder for all targets)
build: clean install
	@echo "Running builder..."
	@builder build
	$(PYTHON) -m build
