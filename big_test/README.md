# Iconoglott Integration Tests

End-to-end tests for both Python backend (LangChain) and TypeScript frontend (NPM package).

## Structure

```
big_test/
├── backend/           # Python tests
│   ├── pyproject.toml
│   └── test_backend.py
├── frontend/          # TypeScript tests
│   ├── package.json
│   ├── test.ts        # Node.js tests
│   ├── index.html     # Browser demo
│   └── main.ts        # Browser entry
├── Makefile
└── README.md
```

## Quick Start

```bash
# Run all tests
make all

# Or separately:
make backend    # Python/LangChain tests
make frontend   # NPM package tests
```

## Prerequisites

1. **Rust core must be built first:**
   ```bash
   cd source/core && maturin develop --release
   ```

2. **NPM package must be built:**
   ```bash
   cd distribution/npm && npm run build
   ```

## Backend Tests

Tests the Python package from `source/lang`:
- Direct DSL → SVG rendering via Rust core
- LangChain tool integration
- Error handling
- Interpreter state management

```bash
make backend
```

## Frontend Tests

Tests the NPM package from `distribution/npm`:
- Type exports verification
- Module imports
- WASM bridge accessibility
- WebSocket client API
- Package structure

```bash
make frontend
```

## Development Mode

Start the servers for interactive testing:

```bash
# Terminal 1: Python backend
make server

# Terminal 2: Frontend dev server (with HMR)
make dev
```

Then open http://localhost:3456 to see the live playground.

## What's Being Tested

### Backend (Python)
| Test | Description |
|------|-------------|
| Core Rendering | `lang.render()` produces valid SVG |
| LangChain Tool | `IconoglottTool` works with agents |
| Error Handling | Graceful error reporting |
| Interpreter | State management between renders |
| Advanced DSL | Variables, groups, transforms |

### Frontend (TypeScript)
| Test | Description |
|------|-------------|
| Type Exports | `index.d.ts` has all required types |
| Module Imports | Package can be imported |
| WASM Bridge | `initWasm()`, `tryGetWasm()` work |
| WebSocket Client | `createClient()` API available |
| Package Structure | Exports map is correct |

