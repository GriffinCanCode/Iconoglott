# iconoglott

Visual DSL for AI-generated graphics with Rust-powered rendering and LangChain integration.

## Installation

```bash
pip install iconoglott

# With LangChain integration
pip install iconoglott[langchain]
```

> **Note**: This package includes a native Rust extension for high-performance SVG rendering.
> Pre-built wheels are available for most platforms (Linux, macOS, Windows on x86_64 and ARM64).

## Quick Start

### Basic Usage

```python
from iconoglott import render

svg = render('''
    canvas 400x300 fill #1a1a2e
    
    circle at 200,150 radius 50
      fill #e94560
      
    text "Hello" at 200,160
      font "system-ui" 24
      fill #fff
      center
''')

# svg is now a string containing the SVG markup
print(svg)
```

### LangChain Integration

```python
from langchain.agents import initialize_agent, AgentType
from langchain_openai import ChatOpenAI
from iconoglott.tools import create_tool

llm = ChatOpenAI(model="gpt-4")
tools = [create_tool()]

agent = initialize_agent(tools, llm, agent=AgentType.OPENAI_FUNCTIONS)
result = agent.run("Create a simple logo with a red circle and white text saying 'AI'")
```

### OpenAI Function Calling

```python
from iconoglott.tools import get_openai_schema
from iconoglott import render
import openai

schema = get_openai_schema()

response = openai.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "user", "content": "Create a bar chart with 3 bars"}
    ],
    tools=[{"type": "function", "function": schema}]
)

# Execute the function call
if response.choices[0].message.tool_calls:
    call = response.choices[0].message.tool_calls[0]
    code = json.loads(call.function.arguments)["code"]
    svg = render(code)
```

## DSL Syntax

### Canvas

```
canvas 800x600 fill #1a1a2e
```

### Shapes

```
rect at 100,100 size 200x150
  fill #e94560
  stroke #fff 2
  corner 12

circle at 400,300 radius 80
  fill #0f3460
  gradient radial #4ecdc4 #0f3460

text "Hello World" at 50,50
  font "Fira Code" 24
  fill #fff
  bold
  center

line from 0,0 to 100,100
  stroke #fff 2

path "M 10 80 Q 95 10 180 80"
  stroke #e94560 3
```

### Variables

```
$accent = #e94560
$size = 40

circle at 100,100 radius $size
  fill $accent
```

### Effects

```
rect at 100,100 size 200x100
  shadow 0,8 20 #0008
  gradient linear #e94560 #0f3460
  opacity 0.9
```

### Transforms

```
rect at 100,100 size 50x50
  rotate 45
  scale 1.5,1.5
  translate 10,20
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

### Grouping

```
group "buttons"
  rect at 0,0 size 100x50
    fill #e94560
  text "Click" at 50,30
    fill #fff
    center
```

## API Reference

### `render(source: str) -> str`

Render DSL code to SVG string.

### `parse(source: str) -> SceneState`

Parse DSL and return scene state for manipulation.

### `create_tool() -> IconoglottTool`

Create LangChain tool for agent integration.

### `get_openai_schema() -> dict`

Get OpenAI function calling schema.

## Frontend Integration

For browser rendering, use the companion NPM package:

```bash
npm install @iconoglott/renderer
```

```tsx
import { IconoglottCanvas } from '@iconoglott/renderer/react'

<IconoglottCanvas code={dslCode} />
```

See [@iconoglott/renderer](https://www.npmjs.com/package/@iconoglott/renderer) for details.

## License

MIT

