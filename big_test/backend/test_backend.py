#!/usr/bin/env python3
"""Backend integration tests for iconoglott.

Tests:
1. Direct rendering via Rust core
2. LangChain tool integration
3. Error handling
4. Interpreter state management
5. Advanced DSL features

Run with: python backend/test_backend.py (from big_test dir with venv active)
"""

import sys
from pathlib import Path

# Add source paths for local imports
ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT / "source"))

# Pre-import check for Rust core
try:
    import iconoglott_core
except ImportError:
    print("ERROR: Rust core not found. Build with: cd source/core && maturin develop --release")
    sys.exit(1)


def test_render():
    """Test direct DSL → SVG rendering via Rust core."""
    from lang import render
    
    dsl = '''canvas 200x150 fill #1a1a2e
circle at 100,75 radius 40 fill #e94560
text "Hello" at 100,75 font "Arial" 16 fill #fff center'''
    
    svg = render(dsl)
    assert '<svg' in svg, "Should produce valid SVG"
    assert 'circle' in svg or 'ellipse' in svg, "Should contain circle"
    
    print("✓ Core rendering works!")
    print(f"  SVG length: {len(svg)} chars")


def test_langchain_tool():
    """Test LangChain tool integration."""
    from lang.tools import create_tool, DSL_REFERENCE, get_openai_schema
    
    tool = create_tool()
    assert tool.name == "iconoglott"
    
    result = tool._run("canvas 100x100 fill #000\ncircle at 50,50 radius 20 fill #f00")
    assert '<svg' in result, "Tool should produce SVG"
    
    schema = get_openai_schema()
    assert schema["name"] == "render_iconoglott"
    
    print("✓ LangChain tool works!")
    print(f"  Tool: {tool.name}, Schema: {list(schema.keys())}")


def test_error_handling():
    """Test that errors are properly reported."""
    from lang import parse
    
    state = parse("canvas 100x100 fill #fff\ncircle at 50,50 radius 10 fill #f00")
    assert '<svg' in state.to_svg()
    
    state2 = parse("canvas 100x100\nunknown_shape at 50,50")
    assert '<svg' in state2.to_svg()  # Should still produce canvas
    
    print("✓ Error handling works!")


def test_interpreter():
    """Test interpreter state management."""
    from lang import Interpreter
    
    interp = Interpreter()
    
    state1 = interp.eval("canvas 100x100 fill #111\nrect at 10,10 size 30x30 fill #f00")
    state2 = interp.eval("canvas 100x100 fill #222\ncircle at 50,50 radius 20 fill #0f0")
    
    svg1, svg2 = state1.to_svg(), state2.to_svg()
    assert '111' in svg1 and '222' in svg2
    
    print("✓ Interpreter state management works!")


def test_advanced_features():
    """Test advanced DSL features: variables, groups, transforms."""
    from lang import render
    
    dsl = '''canvas 400x300 fill #0a0f1a
$accent = #e94560
group "logo"
  circle at 100,100 radius 40 fill $accent
rect at 200,50 size 150x100 gradient linear #3b82f6 #8b5cf6 45 corner 12'''
    
    svg = render(dsl)
    assert '<svg' in svg and 'rect' in svg
    
    print("✓ Advanced DSL features work!")
    print(f"  SVG length: {len(svg)} chars")


def main():
    """Run all backend tests."""
    print("\n" + "═" * 60)
    print("  ICONOGLOTT BACKEND INTEGRATION TESTS")
    print("═" * 60)
    
    tests = [
        ("Core Rendering", test_render),
        ("LangChain Tool", test_langchain_tool),
        ("Error Handling", test_error_handling),
        ("Interpreter State", test_interpreter),
        ("Advanced Features", test_advanced_features),
    ]
    
    passed = failed = 0
    
    for name, fn in tests:
        print(f"\n▸ Testing: {name}")
        try:
            fn()
            passed += 1
        except Exception as e:
            print(f"✗ {name} FAILED: {e}")
            failed += 1
    
    print("\n" + "─" * 60)
    print(f"Results: {passed} passed, {failed} failed")
    print("─" * 60 + "\n")
    
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())

