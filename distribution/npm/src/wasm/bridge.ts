/**
 * WASM Bridge - Loads and wraps the Rust WASM module
 * 
 * Provides type-safe access to the Rust rendering engine while
 * falling back to TypeScript implementation when WASM isn't available.
 */

// WASM module types (mirrors wasm-bindgen exports)
export interface WasmCore {
  // Hashing
  fnv1a_hash(data: string): string;
  compute_element_id(order: number, kind: string, keyJson: string): string;
  
  // Shape rendering
  render_rect(x: number, y: number, w: number, h: number, rx: number, styleJson: string, transform?: string): string;
  render_circle(cx: number, cy: number, r: number, styleJson: string, transform?: string): string;
  render_ellipse(cx: number, cy: number, rx: number, ry: number, styleJson: string, transform?: string): string;
  render_line(x1: number, y1: number, x2: number, y2: number, stroke: string, strokeWidth: number, transform?: string): string;
  render_path(d: string, styleJson: string, transform?: string): string;
  render_polygon(pointsJson: string, styleJson: string, transform?: string): string;
  render_text(x: number, y: number, content: string, font: string, size: number, weight: string, anchor: string, fill: string, transform?: string): string;
  render_image(x: number, y: number, w: number, h: number, href: string, transform?: string): string;
  
  // Definitions
  render_linear_gradient(id: string, fromColor: string, toColor: string, angle: number): string;
  render_radial_gradient(id: string, fromColor: string, toColor: string): string;
  render_shadow_filter(id: string, dx: number, dy: number, blur: number, color: string): string;
  render_blur_filter(id: string, blur: number): string;
  
  // Scene
  render_scene(width: number, height: number, background: string, defs: string, elementsSvg: string): string;
  diff_scenes(oldJson: string, newJson: string): string;
  scenes_equal(oldJson: string, newJson: string): boolean;
  
  // Path utilities
  compute_path_bounds(d: string): string;
}

let wasmModule: WasmCore | null = null;
let initPromise: Promise<WasmCore> | null = null;

/**
 * Initialize the WASM module
 * Returns a promise that resolves when WASM is ready
 */
export async function initWasm(): Promise<WasmCore> {
  if (wasmModule) return wasmModule;
  if (initPromise) return initPromise;
  
  initPromise = (async () => {
    try {
      // Dynamic import of WASM module (built by wasm-pack)
      const wasm = await import('./iconoglott_core.js');
      await wasm.default();
      wasmModule = wasm as unknown as WasmCore;
      return wasmModule;
    } catch (e) {
      console.warn('Failed to load WASM module, falling back to TypeScript:', e);
      throw e;
    }
  })();
  
  return initPromise;
}

/**
 * Check if WASM is available
 */
export function isWasmLoaded(): boolean {
  return wasmModule !== null;
}

/**
 * Get the WASM module (throws if not initialized)
 */
export function getWasm(): WasmCore {
  if (!wasmModule) throw new Error('WASM not initialized. Call initWasm() first.');
  return wasmModule;
}

/**
 * Try to get WASM module, returns null if not available
 */
export function tryGetWasm(): WasmCore | null {
  return wasmModule;
}

