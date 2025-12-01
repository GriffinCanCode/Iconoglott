import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { render } from '../index';
import { createClient, type IconoglottClient, type IconoglottClientOptions } from '../client/ws';
import type { ParseError } from '../core/types';

// ─────────────────────────────────────────────────────────────────────────────
// Canvas Component
// ─────────────────────────────────────────────────────────────────────────────

export interface IconoglottCanvasProps {
  /** WebSocket URL for backend connection (e.g., 'ws://localhost:8765/ws') */
  ws: string;
  /** DSL code to render */
  code?: string;
  /** Debounce delay in ms (default: 80) */
  debounce?: number;
  /** Auto-reconnect on disconnect (default: true) */
  reconnect?: boolean;
  /** Callback when render completes */
  onRender?: (svg: string) => void;
  /** Callback when errors occur (includes parse warnings) */
  onError?: (errors: ParseError[]) => void;
  /** Callback when connection state changes */
  onConnectionChange?: (connected: boolean) => void;
  /** Container className */
  className?: string;
  /** Container style */
  style?: React.CSSProperties;
}

/**
 * Canvas component that connects to Iconoglott Python backend and renders DSL.
 * 
 * @example
 * ```tsx
 * import { IconoglottCanvas } from '@iconoglott/renderer/react'
 * 
 * function Editor() {
 *   const [code, setCode] = useState('canvas 400x300 fill #1a1a2e')
 *   const [connected, setConnected] = useState(false)
 * 
 *   return (
 *     <div>
 *       <span>{connected ? '🟢' : '🔴'}</span>
 *       <textarea value={code} onChange={e => setCode(e.target.value)} />
 *       <IconoglottCanvas
 *         ws="ws://localhost:8765/ws"
 *         code={code}
 *         onConnectionChange={setConnected}
 *         onError={errors => errors.forEach(e => console.warn(e.message))}
 *       />
 *     </div>
 *   )
 * }
 * ```
 */
export function IconoglottCanvas({
  ws,
  code,
  debounce = 80,
  reconnect = true,
  onRender,
  onError,
  onConnectionChange,
  className,
  style,
}: IconoglottCanvasProps) {
  const [svg, setSvg] = useState('');
  const clientRef = useRef<IconoglottClient | null>(null);

  // Connect to WebSocket
  useEffect(() => {
    const client = createClient({
      url: ws,
      debounce,
      reconnect,
      onRender: (renderedSvg, errors) => {
        setSvg(renderedSvg);
        onRender?.(renderedSvg);
        if (errors.length) onError?.(errors);
      },
      onError: (message, errors) => {
        onError?.([{ code: 'WS_ERROR', category: 'websocket', message, line: 0, col: 0, severity: 'error' }, ...errors]);
      },
      onConnectionChange,
    });

    clientRef.current = client;
    return () => client.disconnect();
  }, [ws, debounce, reconnect]); // eslint-disable-line react-hooks/exhaustive-deps

  // Send code when it changes
  useEffect(() => {
    if (code !== undefined) clientRef.current?.send(code);
  }, [code]);

  return (
    <div
      className={className}
      style={style}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Hook for WebSocket Connection
// ─────────────────────────────────────────────────────────────────────────────

export interface UseIconoglottOptions {
  /** WebSocket URL for backend connection */
  url: string;
  /** Initial DSL code */
  initialCode?: string;
  /** Debounce delay in ms (default: 80) */
  debounce?: number;
  /** Auto-reconnect (default: true) */
  reconnect?: boolean;
}

export interface UseIconoglottReturn {
  /** Current rendered SVG */
  svg: string;
  /** Current parse errors */
  errors: ParseError[];
  /** Whether connected to backend */
  connected: boolean;
  /** Send DSL code to render */
  render: (code: string) => void;
  /** Disconnect from backend */
  disconnect: () => void;
}

/**
 * Hook for fine-grained control over WebSocket rendering.
 * 
 * @example
 * ```tsx
 * import { useIconoglott } from '@iconoglott/renderer/react'
 * 
 * function MyComponent() {
 *   const { svg, errors, connected, render } = useIconoglott({
 *     url: 'ws://localhost:8765/ws',
 *   })
 * 
 *   return (
 *     <div>
 *       <button onClick={() => render('canvas 400x300 fill #1a1a2e')}>
 *         Render
 *       </button>
 *       <span>{connected ? 'Connected' : 'Disconnected'}</span>
 *       {errors.map((e, i) => <p key={i} style={{ color: 'red' }}>{e.message}</p>)}
 *       <div dangerouslySetInnerHTML={{ __html: svg }} />
 *     </div>
 *   )
 * }
 * ```
 */
export function useIconoglott(options: UseIconoglottOptions): UseIconoglottReturn {
  const { url, initialCode, debounce = 80, reconnect = true } = options;
  
  const [svg, setSvg] = useState('');
  const [errors, setErrors] = useState<ParseError[]>([]);
  const [connected, setConnected] = useState(false);
  const clientRef = useRef<IconoglottClient | null>(null);

  const doRender = useCallback((code: string) => {
    clientRef.current?.send(code);
  }, []);

  const disconnect = useCallback(() => {
    clientRef.current?.disconnect();
    clientRef.current = null;
    setConnected(false);
  }, []);

  useEffect(() => {
    const client = createClient({
      url,
      debounce,
      reconnect,
      onRender: (renderedSvg, parseErrors) => {
        setSvg(renderedSvg);
        setErrors(parseErrors);
      },
      onError: (message, parseErrors) => {
        setErrors([{ code: 'WS_ERROR', category: 'websocket', message, line: 0, col: 0, severity: 'error' }, ...parseErrors]);
      },
      onConnectionChange: setConnected,
    });

    clientRef.current = client;

    // Render initial code if provided
    if (initialCode) client.send(initialCode);

    return () => client.disconnect();
  }, [url, debounce, reconnect, initialCode]);

  return { svg, errors, connected, render: doRender, disconnect };
}

// ─────────────────────────────────────────────────────────────────────────────
// Local Rendering (no backend required)
// ─────────────────────────────────────────────────────────────────────────────

export interface UseLocalRenderOptions {
  /** Initial DSL code */
  initialCode?: string;
}

export interface UseLocalRenderReturn {
  /** Current rendered SVG */
  svg: string;
  /** Current error message (if any) */
  error: string | null;
  /** Render DSL code locally */
  render: (code: string) => string | null;
}

/**
 * Hook for client-side rendering without a backend connection.
 * Uses the built-in TypeScript DSL interpreter.
 * 
 * @example
 * ```tsx
 * import { useLocalRender } from '@iconoglott/renderer/react'
 * 
 * function OfflineEditor() {
 *   const { svg, error, render } = useLocalRender()
 * 
 *   return (
 *     <div>
 *       <textarea onChange={e => render(e.target.value)} />
 *       {error && <p style={{ color: 'red' }}>{error}</p>}
 *       <div dangerouslySetInnerHTML={{ __html: svg }} />
 *     </div>
 *   )
 * }
 * ```
 */
export function useLocalRender(options: UseLocalRenderOptions = {}): UseLocalRenderReturn {
  const { initialCode } = options;
  
  const [svg, setSvg] = useState('');
  const [error, setError] = useState<string | null>(null);

  const doRender = useCallback((code: string) => {
    try {
      const result = render(code);
      setSvg(result);
      setError(null);
      return result;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      return null;
    }
  }, []);

  useEffect(() => {
    if (initialCode) doRender(initialCode);
  }, [initialCode, doRender]);

  return { svg, error, render: doRender };
}
