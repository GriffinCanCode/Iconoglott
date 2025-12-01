# @iconoglott/renderer

Real-time visual DSL renderer with WebSocket backend support. Connect to the Iconoglott Python server and render graphics instantly as you type.

## Installation

```bash
npm install @iconoglott/renderer
```

## Quick Start

### React Canvas (Recommended)

The easiest way to get started. Connect to your backend and render DSL in real-time:

```tsx
import { useState } from 'react'
import { IconoglottCanvas } from '@iconoglott/renderer/react'

function Editor() {
  const [code, setCode] = useState(`
    canvas massive fill #1a1a2e
    circle at 200,150 radius 50
      fill #e94560
  `)
  const [connected, setConnected] = useState(false)

  return (
    <div>
      <span>{connected ? '🟢' : '🔴'}</span>
      <textarea value={code} onChange={e => setCode(e.target.value)} />
      <IconoglottCanvas
        ws="ws://localhost:8765/ws"
        code={code}
        onConnectionChange={setConnected}
        onError={errors => errors.forEach(e => console.warn(e.message))}
      />
    </div>
  )
}
```

### React Hook (More Control)

For fine-grained control over the rendering pipeline:

```tsx
import { useIconoglott } from '@iconoglott/renderer/react'

function MyComponent() {
  const { svg, errors, connected, render } = useIconoglott({
    url: 'ws://localhost:8765/ws',
  })

  return (
    <div>
      <button onClick={() => render('canvas massive fill #1a1a2e')}>
        Render
      </button>
      <span>{connected ? 'Connected' : 'Disconnected'}</span>
      {errors.map((e, i) => (
        <p key={i} style={{ color: 'red' }}>{e.message}</p>
      ))}
      <div dangerouslySetInnerHTML={{ __html: svg }} />
    </div>
  )
}
```

### Vanilla JavaScript

Use the WebSocket client directly without React:

```typescript
import { createClient } from '@iconoglott/renderer'

const client = createClient({
  url: 'ws://localhost:8765/ws',
  onRender: (svg, errors) => {
    document.getElementById('canvas').innerHTML = svg
    if (errors.length) console.warn('Parse errors:', errors)
  },
  onError: (msg) => console.error('Error:', msg),
  onConnectionChange: (connected) => {
    document.getElementById('status').textContent = connected ? '🟢' : '🔴'
  },
})

// Send DSL to render (debounced automatically)
document.getElementById('editor').addEventListener('input', (e) => {
  client.send(e.target.value)
})

// Cleanup when done
window.addEventListener('beforeunload', () => client.disconnect())
```

### Offline Rendering

Render locally without a backend using the built-in TypeScript interpreter:

```tsx
import { useLocalRender } from '@iconoglott/renderer/react'

function OfflineEditor() {
  const { svg, error, render } = useLocalRender()

  return (
    <div>
      <textarea onChange={e => render(e.target.value)} />
      {error && <p style={{ color: 'red' }}>{error}</p>}
      <div dangerouslySetInnerHTML={{ __html: svg }} />
    </div>
  )
}
```

Or use the render function directly:

```typescript
import { render } from '@iconoglott/renderer'

const svg = render(`
  canvas massive fill #1a1a2e
  circle at 200,150 radius 50
    fill #e94560
`)

document.getElementById('canvas').innerHTML = svg
```

## API Reference

### `<IconoglottCanvas />`

React component for rendering DSL via WebSocket.

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `ws` | `string` | required | WebSocket URL (e.g., `ws://localhost:8765/ws`) |
| `code` | `string` | - | DSL code to render |
| `debounce` | `number` | `80` | Debounce delay in ms |
| `reconnect` | `boolean` | `true` | Auto-reconnect on disconnect |
| `onRender` | `(svg: string) => void` | - | Callback when render completes |
| `onError` | `(errors: ParseError[]) => void` | - | Callback for errors/warnings |
| `onConnectionChange` | `(connected: boolean) => void` | - | Connection state callback |
| `className` | `string` | - | Container className |
| `style` | `CSSProperties` | - | Container style |

### `useIconoglott(options)`

Hook for fine-grained WebSocket rendering control.

```typescript
interface UseIconoglottOptions {
  url: string           // WebSocket URL
  initialCode?: string  // Initial DSL to render
  debounce?: number     // Debounce delay (default: 80)
  reconnect?: boolean   // Auto-reconnect (default: true)
}

interface UseIconoglottReturn {
  svg: string                  // Current rendered SVG
  errors: ParseError[]         // Current parse errors
  connected: boolean           // Connection state
  render: (code: string) => void  // Send code to render
  disconnect: () => void       // Disconnect from backend
}
```

### `useLocalRender(options?)`

Hook for client-side rendering without a backend.

```typescript
interface UseLocalRenderReturn {
  svg: string                        // Current rendered SVG
  error: string | null               // Current error (if any)
  render: (code: string) => string | null  // Render DSL locally
}
```

### `createClient(options)`

Create a WebSocket client for direct use.

```typescript
interface IconoglottClientOptions {
  url: string                                    // WebSocket URL
  onRender?: (svg: string, errors: ParseError[]) => void
  onError?: (message: string, errors: ParseError[]) => void
  onConnectionChange?: (connected: boolean) => void
  debounce?: number    // Default: 80
  reconnect?: boolean  // Default: true
  reconnectDelay?: number  // Default: 2000
}

interface IconoglottClient {
  send: (source: string) => void
  disconnect: () => void
  readonly connected: boolean
}
```

### `render(source: string): string`

Render DSL to SVG string (client-side, no backend).

### `parse(source: string): SceneState`

Parse DSL and return scene state for manipulation.

## DSL Syntax

### Canvas

```
canvas giant fill #1a1a2e
```

### Shapes

```
rect at 100,100 size 200x150
  fill #e94560
  stroke #fff 2
  corner 12

circle at 400,300 radius 80
  gradient radial #4ecdc4 #0f3460

text "Hello World" at 50,50
  font "Fira Code" 24
  fill #fff
  bold
  center

line from 0,0 to 100,100
  stroke #fff 2
```

### Variables

```
$accent = #e94560
$size = 40

circle at 100,100 radius $size
  fill $accent
```

### Transforms

```
rect at 100,100 size 50x50
  rotate 45
  scale 1.5,1.5
```

### Effects

```
rect at 100,100 size 200x100
  shadow 0,8 20 #0008
  gradient linear #e94560 #0f3460
  opacity 0.9
```

### Layout

```
stack vertical gap 20 at 50,100
  circle radius 10 fill #f00
  circle radius 10 fill #0f0
  circle radius 10 fill #00f
```

## License

MIT
