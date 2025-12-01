# Iconoglott

A real-time visual DSL with Rust-powered rendering and Python accessibility.

## Architecture

```
iconoglott/
├── core/       # Rust rendering engine (PyO3) - future
├── lang/       # DSL interpreter (lexer → parser → eval)
├── server/     # FastAPI + WebSocket real-time server
├── static/     # Minimal frontend
└── tests/      # Test suite
```

## DSL Syntax

### Canvas

```
canvas giant fill #1a1a2e
```

### Variables

```
$accent = #e94560
$radius = 40
```

### Shapes

```
// Rectangle with position and size
rect at 100,100 size 200x150
  fill $accent
  stroke #16213e 2
  corner 12

// Circle with center and radius
circle at 400,300 radius 80
  fill #0f3460
  gradient radial #4ecdc4 #0f3460

// Text with font and styling
text "Hello World" at 50,50
  font "Fira Code" 24
  fill #fff
  bold
  center

// Line from point to point
line from 0,0 to 100,100
  stroke #fff 2

// SVG path
path "M 10 80 Q 95 10 180 80"
  stroke #e94560 3
  fill none
```

### Style Properties

| Property   | Syntax                    | Description              |
|------------|---------------------------|--------------------------|
| `fill`     | `fill #color` or `fill $var` | Shape fill color       |
| `stroke`   | `stroke #color width`     | Stroke color and width   |
| `corner`   | `corner 12`               | Border radius            |
| `opacity`  | `opacity 0.5`             | Transparency (0-1)       |
| `gradient` | `gradient linear/radial #from #to` | Gradient fill |
| `shadow`   | `shadow x,y blur #color`  | Drop shadow              |

### Text Properties

| Property | Syntax              | Description           |
|----------|---------------------|------------------------|
| `font`   | `font "Name" size`  | Font family and size   |
| `bold`   | `bold`              | Bold weight            |
| `italic` | `italic`            | Italic style           |
| `center` | `center`            | Center align           |

### Transforms

```
rect at 100,100 size 50x50
  rotate 45
  scale 1.5,1.5
  translate 10,20
  origin 125,125
```

### Grouping

```
group "name"
  rect at 0,0 size 100x50
    fill #e94560
  text "Button" at 50,30
    fill #fff
    center
```

### Layout

```
stack vertical gap 20 at 50,100
  circle radius 10 fill #f00
  circle radius 10 fill #0f0
  circle radius 10 fill #00f

row horizontal gap 10 at 50,200
  rect size 30x30 fill #f00
  rect size 30x30 fill #0f0
```

## Example

```
// Complete example with all features

canvas giant fill #1a1a2e

$accent = #e94560
$teal = #4ecdc4

// Hero text
text "Iconoglott" at 50,60
  font "Outfit" 48
  fill #fff
  bold

// Card with shadow
rect at 100,120 size 280x180
  fill #16213e
  corner 16
  shadow 0,8 20 #0008

// Gradient circle
circle at 500,200 radius 60
  gradient radial $teal #0f3460

// CTA button
rect at 120,320 size 160x48
  fill $accent
  corner 24
  shadow 0,4 8 #0004

text "Get Started" at 200,352
  font "Outfit" 16
  fill #fff
  center
```

## Development

```bash
# Setup
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"

# Run server
python3 -m uvicorn server.app:app --reload --port 8765

# Run tests
pytest tests/ -v
```

## Design Philosophy

1. **Readable**: Named parameters (`at`, `size`, `radius`) over positional
2. **Composable**: Variables, groups, and nested styling
3. **Immediate**: Sub-100ms interpretation, WebSocket-first updates
4. **Layered**: DSL → Python → (future) Rust for progressive optimization

## Stack

- **Interpreter**: Python 3.11+ (pattern matching, slots, dataclasses)
- **Server**: FastAPI + Uvicorn + WebSockets 13.1
- **Render**: SVG (browser-native, scalable)
- **Future**: Rust via PyO3 for performance-critical paths
