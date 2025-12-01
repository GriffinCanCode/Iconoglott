/**
 * WASM Bridge - Loads and wraps the Rust WASM module
 * 
 * The Rust core is the SINGLE SOURCE OF TRUTH for:
 * - DSL lexing and parsing
 * - Shape rendering
 * - Scene diffing
 * - Content hashing
 * 
 * Uses serde-wasm-bindgen for direct JS<->Rust value conversion (no JSON overhead)
 */

// Style interface matching Rust WasmStyle
export interface WasmStyle {
  fill?: string;
  stroke?: string;
  stroke_width: number;
  opacity: number;
  corner: number;
  filter?: string;
}

// Diff operation types
export interface WasmDiffOp {
  type: 'none' | 'full_redraw' | 'add' | 'remove' | 'update' | 'move' | 'update_defs';
  id?: string;
  idx?: number;
  svg?: string;
  from_idx?: number;
  to_idx?: number;
}

export interface WasmDiffResult {
  ops: WasmDiffOp[];
  canvas_changed: boolean;
}

export interface WasmSceneInput {
  canvas: { size: string; fill: string };
  elements: Array<{ id: string; kind: string; svg: string }>;
  defs: string;
}

export interface TextMetrics {
  width: number;
  height: number;
  ascender: number;
  descender: number;
}

export interface SizeInfo {
  name: string;
  width: number;
  height: number;
}

// Graph layout types
export interface NodeInput {
  id: string;
  w: number;
  h: number;
}

export interface NodePosition {
  id: string;
  cx: number;
  cy: number;
}

export interface EdgeAnchors {
  from: [number, number];
  to: [number, number];
}

export interface EdgeAnchors {
  from: [number, number];
  to: [number, number];
}

export interface NodeLayoutInput {
  id: string;
  w: number;
  h: number;
}

export interface NodeLayoutOutput {
  id: string;
  cx: number;
  cy: number;
}

// WASM module types (mirrors wasm-bindgen exports)
export interface WasmCore {
  // DSL Processing (single source of truth)
  tokenize(source: string): string;  // Returns JSON array of tokens
  parse(source: string): string;  // Returns JSON AST
  parse_with_errors(source: string): string;  // Returns {ast, errors} JSON
  
  // Canvas Size System - returns native JS values via serde-wasm-bindgen
  size_to_pixels(name: string): [number, number] | null;
  is_valid_size(name: string): boolean;
  get_all_sizes(): string[];
  get_size_info(name: string): SizeInfo | null;
  
  // Hashing
  fnv1a_hash(data: string): string;
  compute_element_id(order: number, kind: string, key: Record<string, unknown>): string;
  
  // Shape rendering - accepts native JS objects for style/points
  render_rect(x: number, y: number, w: number, h: number, rx: number, style: WasmStyle, transform?: string): string;
  render_circle(cx: number, cy: number, r: number, style: WasmStyle, transform?: string): string;
  render_ellipse(cx: number, cy: number, rx: number, ry: number, style: WasmStyle, transform?: string): string;
  render_line(x1: number, y1: number, x2: number, y2: number, stroke: string, strokeWidth: number, transform?: string): string;
  render_path(d: string, style: WasmStyle, transform?: string): string;
  render_polygon(points: [number, number][], style: WasmStyle, transform?: string): string;
  render_text(x: number, y: number, content: string, font: string, size: number, weight: string, anchor: string, fill: string, transform?: string): string;
  render_image(x: number, y: number, w: number, h: number, href: string, transform?: string): string;
  
  // Graph/Flowchart rendering - native JS objects via serde-wasm-bindgen
  render_diamond(cx: number, cy: number, w: number, h: number, style: WasmStyle, transform?: string): string;
  render_node(id: string, shape: string, cx: number, cy: number, w: number, h: number, label: string | undefined, style: WasmStyle): string;
  render_edge(fromX: number, fromY: number, toX: number, toY: number, edgeStyle: string, arrow: string, label: string | undefined, stroke: string, strokeWidth: number): string;
  render_arrow_markers(color: string): string;
  compute_edge_anchors(fromCx: number, fromCy: number, fromW: number, fromH: number, toCx: number, toCy: number, toW: number, toH: number): EdgeAnchors;
  layout_hierarchical(nodes: NodeInput[], direction: string, spacing: number): NodePosition[];
  layout_grid(nodes: NodeInput[], spacing: number): NodePosition[];
  
  // Definitions
  render_linear_gradient(id: string, fromColor: string, toColor: string, angle: number): string;
  render_radial_gradient(id: string, fromColor: string, toColor: string): string;
  render_shadow_filter(id: string, dx: number, dy: number, blur: number, color: string): string;
  render_blur_filter(id: string, blur: number): string;
  
  // Scene - accepts/returns native JS objects
  render_scene(sizeName: string, background: string, defs: string, elementsSvg: string): string;
  diff_scenes(old: WasmSceneInput, current: WasmSceneInput): WasmDiffResult;
  scenes_equal(old: WasmSceneInput, current: WasmSceneInput): boolean;
  
  // Path utilities - returns native JS array
  compute_path_bounds(d: string): [number, number, number, number];
  
  // Text metrics - returns native JS object
  measure_text(content: string, font: string, size: number): TextMetrics;
  compute_text_bounds(x: number, y: number, content: string, font: string, size: number, anchor: string): [number, number, number, number];
  
  // Graph/Flowchart primitives - native JS objects
  render_diamond(cx: number, cy: number, w: number, h: number, style: WasmStyle, transform?: string): string;
  render_node(id: string, shape: string, cx: number, cy: number, w: number, h: number, label: string | null, style: WasmStyle): string;
  render_edge(fromX: number, fromY: number, toX: number, toY: number, edgeStyle: string, arrow: string, label: string | null, stroke: string, strokeWidth: number): string;
  render_arrow_markers(color: string): string;
  compute_edge_anchors(fromCx: number, fromCy: number, fromW: number, fromH: number, toCx: number, toCy: number, toW: number, toH: number): EdgeAnchors;
  layout_hierarchical(nodes: NodeLayoutInput[], direction: string, spacing: number): NodeLayoutOutput[];
  layout_grid(nodes: NodeLayoutInput[], spacing: number): NodeLayoutOutput[];
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

/** Check if WASM is available */
export function isWasmLoaded(): boolean {
  return wasmModule !== null;
}

/** Get the WASM module (throws if not initialized) */
export function getWasm(): WasmCore {
  if (!wasmModule) throw new Error('WASM not initialized. Call initWasm() first.');
  return wasmModule;
}

/** Try to get WASM module, returns null if not available */
export function tryGetWasm(): WasmCore | null {
  return wasmModule;
}
