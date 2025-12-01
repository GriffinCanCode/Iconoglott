"""Pytest configuration and shared fixtures."""

import os
import sys
from pathlib import Path

import pytest

# Add source to path
sys.path.insert(0, str(Path(__file__).parent.parent))

# Snapshot directory
SNAPSHOTS_DIR = Path(__file__).parent / "snapshots"
SNAPSHOTS_DIR.mkdir(exist_ok=True)


@pytest.fixture
def snapshot_dir():
    """Return snapshot directory for SVG snapshots."""
    return SNAPSHOTS_DIR


@pytest.fixture
def basic_canvas_source():
    """Basic canvas DSL source."""
    return "canvas 800x600 fill #1a1a2e"


@pytest.fixture
def rect_source():
    """Rectangle DSL source."""
    return """canvas 400x300
rect at 10,10 size 100x50
  fill #e94560"""


@pytest.fixture
def complex_scene_source():
    """Complex scene with multiple shapes."""
    return """canvas 800x600 fill #1a1a2e
$primary = #e94560
$secondary = #16213e

rect at 50,50 size 200x100
  fill $primary
  corner 8
  
circle at 300,100 radius 40
  fill $secondary
  stroke #fff 2

text at 100,250 "Hello World"
  font "Arial" 24
  bold
  fill #fff"""


@pytest.fixture
def transform_source():
    """Source with transforms."""
    return """canvas 400x400
rect at 200,200 size 100x100
  fill #f00
  rotate 45
  origin 250,250"""

