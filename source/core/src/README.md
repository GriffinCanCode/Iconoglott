# Iconoglott Core - Source Layout

This directory contains the Rust source for the iconoglott rendering engine.

## Module Structure

```
src/
├── lib.rs          # Crate root, re-exports & Python bindings
├── dsl/            # DSL parsing
│   ├── lexer.rs    # Tokenizer with indentation tracking
│   └── parser.rs   # AST generation with error recovery
├── hash/           # Identity & hashing
│   └── id.rs       # FNV-1a, ElementId, ContentHash
├── scene/          # Scene graph
│   ├── scene.rs    # Scene container, gradients, filters
│   └── shape.rs    # Shape primitives (rect, circle, etc.)
├── render/         # Rendering pipeline
│   ├── cache.rs    # SVG fragment memoization
│   ├── diff.rs     # Incremental scene diffing
│   └── render.rs   # Python render interface
└── bindings/       # Platform bindings
    └── wasm.rs     # WebAssembly API
```

## Build Targets

- **Python**: `cargo build --features python` (PyO3 bindings)
- **WASM**: `wasm-pack build --features wasm` (wasm-bindgen)
- **Library**: `cargo build` (core only)

## Testing

```bash
cargo test --features python
cargo test --features wasm
```

