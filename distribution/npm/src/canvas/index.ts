/**
 * Iconoglott Canvas - Pre-configured viewport for easy web integration
 * 
 * @example
 * ```ts
 * import { Canvas } from '@iconoglott/renderer/canvas'
 * 
 * // Create canvas with auto-fit sizing
 * const canvas = new Canvas('#container', { fit: 'contain' })
 * canvas.render(`
 *   canvas 400x300 fill #0a0f1a
 *   circle at 200,150 radius 50 fill #3b82f6
 * `)
 * 
 * // Or with WebSocket for live updates
 * const liveCanvas = new Canvas('#preview', {
 *   fit: 'cover',
 *   ws: 'ws://localhost:8765/ws'
 * })
 * liveCanvas.send(code) // Sends to backend
 * ```
 */

import { createClient, type IconoglottClient } from '../client/ws';

/** Sizing modes for the canvas */
export type CanvasFit = 
  | 'contain'  // Fit entire SVG inside container (letterbox)
  | 'cover'    // Fill container, crop overflow
  | 'fill'     // Stretch to fill (may distort)
  | 'none'     // Native SVG size
  | 'scale-down'; // Like contain, but never scale up

/** Canvas configuration options */
export interface CanvasOptions {
  /** How SVG fits in container (default: 'contain') */
  fit?: CanvasFit;
  /** Background color (default: transparent) */
  background?: string;
  /** Border radius in px (default: 0) */
  borderRadius?: number;
  /** Padding in px (default: 0) */
  padding?: number;
  /** WebSocket URL for live rendering (optional) */
  ws?: string;
  /** Debounce delay for WS in ms (default: 80) */
  debounce?: number;
  /** Callback when render completes */
  onRender?: (svg: string) => void;
  /** Callback on errors */
  onError?: (message: string) => void;
  /** Callback when WS connection changes */
  onConnection?: (connected: boolean) => void;
}

/** CSS for different fit modes */
const FIT_STYLES: Record<CanvasFit, string> = {
  contain: 'object-fit: contain; width: 100%; height: 100%;',
  cover: 'object-fit: cover; width: 100%; height: 100%;',
  fill: 'width: 100%; height: 100%;',
  none: '',
  'scale-down': 'object-fit: scale-down; width: 100%; height: 100%;',
};

/**
 * Canvas class for rendering Iconoglott DSL in web interfaces.
 * 
 * Provides a pre-configured viewport with:
 * - Automatic sizing and scaling
 * - Optional WebSocket connection for live updates
 * - Responsive container fitting
 */
export class Canvas {
  private container: HTMLElement;
  private wrapper: HTMLElement;
  private options: Required<Omit<CanvasOptions, 'ws' | 'onRender' | 'onError' | 'onConnection'>> & 
    Pick<CanvasOptions, 'ws' | 'onRender' | 'onError' | 'onConnection'>;
  private client: IconoglottClient | null = null;
  private currentSvg = '';

  /**
   * Create a new Canvas instance.
   * @param target - CSS selector or HTMLElement
   * @param options - Configuration options
   */
  constructor(target: string | HTMLElement, options: CanvasOptions = {}) {
    this.container = typeof target === 'string' 
      ? document.querySelector(target) as HTMLElement
      : target;
    
    if (!this.container) {
      throw new Error(`Canvas: target element not found: ${target}`);
    }

    this.options = {
      fit: options.fit ?? 'contain',
      background: options.background ?? 'transparent',
      borderRadius: options.borderRadius ?? 0,
      padding: options.padding ?? 0,
      debounce: options.debounce ?? 80,
      ws: options.ws,
      onRender: options.onRender,
      onError: options.onError,
      onConnection: options.onConnection,
    };

    // Create wrapper element
    this.wrapper = document.createElement('div');
    this.applyStyles();
    this.container.appendChild(this.wrapper);

    // Connect WebSocket if URL provided
    if (this.options.ws) {
      this.connect(this.options.ws);
    }
  }

  /** Apply CSS styles to wrapper */
  private applyStyles(): void {
    const { fit, background, borderRadius, padding } = this.options;
    
    this.wrapper.style.cssText = `
      width: 100%;
      height: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      background: ${background};
      border-radius: ${borderRadius}px;
      padding: ${padding}px;
      box-sizing: border-box;
      overflow: hidden;
    `;

    // Style SVG when it's inserted
    this.wrapper.setAttribute('data-fit', fit);
  }

  /** Style the SVG element based on fit mode */
  private styleSvg(svg: SVGElement): void {
    const { fit } = this.options;
    
    // Reset styles
    svg.style.cssText = '';
    
    switch (fit) {
      case 'contain':
        svg.style.maxWidth = '100%';
        svg.style.maxHeight = '100%';
        svg.style.width = 'auto';
        svg.style.height = 'auto';
        break;
      case 'cover':
        svg.style.width = '100%';
        svg.style.height = '100%';
        svg.style.objectFit = 'cover';
        svg.removeAttribute('width');
        svg.removeAttribute('height');
        break;
      case 'fill':
        svg.style.width = '100%';
        svg.style.height = '100%';
        svg.removeAttribute('width');
        svg.removeAttribute('height');
        break;
      case 'scale-down':
        svg.style.maxWidth = '100%';
        svg.style.maxHeight = '100%';
        break;
      case 'none':
      default:
        // Keep native size
        break;
    }
  }

  /** Connect to WebSocket backend */
  connect(url: string): void {
    if (this.client) {
      this.client.disconnect();
    }

    this.client = createClient({
      url,
      debounce: this.options.debounce,
      onRender: (svg) => {
        this.setSvg(svg);
        this.options.onRender?.(svg);
      },
      onError: (message) => {
        this.options.onError?.(message);
      },
      onConnectionChange: (connected) => {
        this.options.onConnection?.(connected);
      },
    });
  }

  /** Disconnect from WebSocket */
  disconnect(): void {
    this.client?.disconnect();
    this.client = null;
  }

  /** Send DSL code to WebSocket backend for rendering */
  send(code: string): void {
    if (!this.client) {
      throw new Error('Canvas: No WebSocket connection. Call connect() or provide ws option.');
    }
    this.client.send(code);
  }

  /** Set SVG content directly (bypasses WebSocket) */
  setSvg(svg: string): void {
    this.currentSvg = svg;
    this.wrapper.innerHTML = svg;
    
    // Apply fit styles to inserted SVG
    const svgEl = this.wrapper.querySelector('svg');
    if (svgEl) {
      this.styleSvg(svgEl);
    }
  }

  /** Render DSL code locally (requires WASM to be initialized) */
  async render(code: string): Promise<string> {
    const { initWasm, render } = await import('../index');
    await initWasm();
    const svg = render(code);
    this.setSvg(svg);
    return svg;
  }

  /** Get current SVG content */
  getSvg(): string {
    return this.currentSvg;
  }

  /** Update options */
  setOptions(options: Partial<CanvasOptions>): void {
    Object.assign(this.options, options);
    this.applyStyles();
    
    // Re-style existing SVG if present
    const svgEl = this.wrapper.querySelector('svg');
    if (svgEl) {
      this.styleSvg(svgEl);
    }
  }

  /** Change fit mode */
  setFit(fit: CanvasFit): void {
    this.setOptions({ fit });
  }

  /** Clear canvas */
  clear(): void {
    this.currentSvg = '';
    this.wrapper.innerHTML = '';
  }

  /** Destroy canvas instance */
  destroy(): void {
    this.disconnect();
    this.wrapper.remove();
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Factory Functions
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Create a canvas that fits inside its container (letterbox style).
 * Best for maintaining aspect ratio.
 */
export const createContainCanvas = (target: string | HTMLElement, options?: Omit<CanvasOptions, 'fit'>) =>
  new Canvas(target, { ...options, fit: 'contain' });

/**
 * Create a canvas that covers its container (may crop).
 * Best for backgrounds and full-bleed graphics.
 */
export const createCoverCanvas = (target: string | HTMLElement, options?: Omit<CanvasOptions, 'fit'>) =>
  new Canvas(target, { ...options, fit: 'cover' });

/**
 * Create a canvas that fills its container (may distort).
 * Best when you need exact container matching.
 */
export const createFillCanvas = (target: string | HTMLElement, options?: Omit<CanvasOptions, 'fit'>) =>
  new Canvas(target, { ...options, fit: 'fill' });

// ─────────────────────────────────────────────────────────────────────────────
// Preset Configurations
// ─────────────────────────────────────────────────────────────────────────────

/** Preset for card-style display */
export const CARD_PRESET: CanvasOptions = {
  fit: 'contain',
  background: '#0a0f1a',
  borderRadius: 12,
  padding: 16,
};

/** Preset for full-screen backgrounds */
export const BACKGROUND_PRESET: CanvasOptions = {
  fit: 'cover',
  background: 'transparent',
  borderRadius: 0,
  padding: 0,
};

/** Preset for icon display */
export const ICON_PRESET: CanvasOptions = {
  fit: 'contain',
  background: 'transparent',
  borderRadius: 0,
  padding: 4,
};

/** Preset for thumbnail previews */
export const THUMBNAIL_PRESET: CanvasOptions = {
  fit: 'cover',
  background: '#1a1a2e',
  borderRadius: 8,
  padding: 0,
};

// ─────────────────────────────────────────────────────────────────────────────
// Quick Render Function
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Quick one-liner to render DSL into an element.
 * 
 * @example
 * ```ts
 * import { renderTo } from '@iconoglott/renderer/canvas'
 * 
 * renderTo('#preview', `
 *   canvas 200x200 fill #111
 *   circle at 100,100 radius 50 fill #f00
 * `)
 * ```
 */
export async function renderTo(
  target: string | HTMLElement, 
  code: string, 
  options?: CanvasOptions
): Promise<Canvas> {
  const canvas = new Canvas(target, options);
  await canvas.render(code);
  return canvas;
}

/**
 * Quick one-liner with WebSocket connection.
 * 
 * @example
 * ```ts
 * import { connectTo } from '@iconoglott/renderer/canvas'
 * 
 * const canvas = connectTo('#preview', 'ws://localhost:8765/ws')
 * canvas.send('canvas 200x200 fill #111')
 * ```
 */
export function connectTo(
  target: string | HTMLElement, 
  wsUrl: string, 
  options?: Omit<CanvasOptions, 'ws'>
): Canvas {
  return new Canvas(target, { ...options, ws: wsUrl });
}

