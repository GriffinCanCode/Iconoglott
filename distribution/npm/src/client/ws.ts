import type { RenderResponse, ErrorResponse, ParseError, ServerMessage } from '../core/types';

export interface IconoglottClientOptions {
  /** WebSocket URL (e.g., 'ws://localhost:8765/ws') */
  url: string;
  /** Callback when SVG is rendered */
  onRender?: (svg: string, errors: ParseError[]) => void;
  /** Callback on fatal error */
  onError?: (message: string, errors: ParseError[]) => void;
  /** Callback on connection state change */
  onConnectionChange?: (connected: boolean) => void;
  /** Debounce delay in ms (default: 80) */
  debounce?: number;
  /** Auto-reconnect on disconnect (default: true) */
  reconnect?: boolean;
  /** Reconnect delay in ms (default: 2000) */
  reconnectDelay?: number;
}

export interface IconoglottClient {
  /** Send DSL code to render */
  send: (source: string) => void;
  /** Disconnect and cleanup */
  disconnect: () => void;
  /** Current connection state */
  readonly connected: boolean;
}

/**
 * Create a WebSocket client for connecting to the Iconoglott Python backend.
 * 
 * @example
 * ```ts
 * import { createClient } from '@iconoglott/renderer'
 * 
 * const client = createClient({
 *   url: 'ws://localhost:8765/ws',
 *   onRender: (svg, errors) => {
 *     document.getElementById('canvas').innerHTML = svg
 *     if (errors.length) console.warn('Parse errors:', errors)
 *   },
 *   onError: (msg) => console.error('Error:', msg),
 *   onConnectionChange: (connected) => {
 *     console.log(connected ? 'Connected' : 'Disconnected')
 *   },
 * })
 * 
 * // Send DSL to render (debounced automatically)
 * client.send(`canvas 400x300 fill #1a1a2e`)
 * 
 * // Cleanup
 * client.disconnect()
 * ```
 */
export function createClient(options: IconoglottClientOptions): IconoglottClient {
  const {
    url,
    onRender,
    onError,
    onConnectionChange,
    debounce = 80,
    reconnect = true,
    reconnectDelay = 2000,
  } = options;

  let ws: WebSocket | null = null;
  let shouldReconnect = reconnect;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let pendingSource: string | null = null;

  const connect = () => {
    ws = new WebSocket(url);

    ws.onopen = () => {
      onConnectionChange?.(true);
      // Send pending source if any
      if (pendingSource !== null) {
        sendNow(pendingSource);
        pendingSource = null;
      }
    };

    ws.onclose = () => {
      onConnectionChange?.(false);
      if (shouldReconnect) setTimeout(connect, reconnectDelay);
    };

    ws.onerror = () => {
      onError?.('WebSocket connection error', []);
    };

    ws.onmessage = (e) => {
      try {
        const msg: ServerMessage = JSON.parse(e.data);
        
        if (msg.type === 'render') {
          const { svg, errors } = msg as RenderResponse;
          onRender?.(svg, errors || []);
        } else if (msg.type === 'error') {
          const { message, errors } = msg as ErrorResponse;
          onError?.(message, errors || []);
        }
        // Ignore 'pong' messages
      } catch (err) {
        onError?.(`Failed to parse server message: ${err}`, []);
      }
    };
  };

  const sendNow = (source: string) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'source', payload: source }));
    }
  };

  const send = (source: string) => {
    if (debounceTimer) clearTimeout(debounceTimer);

    // If not connected, store for later
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      pendingSource = source;
      return;
    }

    debounceTimer = setTimeout(() => {
      sendNow(source);
      debounceTimer = null;
    }, debounce);
  };

  const disconnect = () => {
    shouldReconnect = false;
    if (debounceTimer) clearTimeout(debounceTimer);
    ws?.close();
    ws = null;
  };

  connect();

  return {
    send,
    disconnect,
    get connected() {
      return ws?.readyState === WebSocket.OPEN;
    },
  };
}
