# Root Makefile - Iconoglott
SHELL := /bin/bash
.PHONY: all clean install dev test start stop killports lint build core ts-types

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

# Run tests with coverage
test-cov: install
	@$(MAKE) -C source test-cov

# Run Rust tests
test-rust: core
	@$(MAKE) -C source test-rust

# Generate TypeScript types from Rust structs
ts-types: core
	@echo "Generating TypeScript types from Rust..."
	@cd source/core && cargo test export_ts_bindings --features python -- --ignored
	@echo "TypeScript types generated at distribution/npm/src/core/generated/"

# Run TypeScript tests
test-ts:
	@cd distribution/npm && npm test

# Run all tests (Python + Rust + TypeScript)
test-all: test-rust test test-ts
	@echo "All tests completed"

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
