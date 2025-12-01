/* tslint:disable */
/* eslint-disable */

/**
 * Standard canvas sizes (10-tier system)
 */
export enum CanvasSize {
  Nano = 16,
  Micro = 24,
  Tiny = 32,
  Small = 48,
  Medium = 64,
  Large = 96,
  XLarge = 128,
  Huge = 192,
  Massive = 256,
  Giant = 512,
}

/**
 * Compute best anchor points for an edge between two nodes
 * Returns {from: [x, y], to: [x, y]}
 */
export function compute_edge_anchors(from_cx: number, from_cy: number, from_w: number, from_h: number, to_cx: number, to_cy: number, to_w: number, to_h: number): any;

/**
 * Compute stable element ID from order, kind, and key properties
 */
export function compute_element_id(order: number, kind: string, key: any): string;

export function compute_path_bounds(d: string): any;

/**
 * Compute text bounding box accounting for anchor position
 * Returns [x, y, width, height]
 */
export function compute_text_bounds(x: number, y: number, content: string, font: string, size: number, anchor: string): any;

/**
 * Diff two scenes and return operations
 */
export function diff_scenes(old: any, _new: any): any;

/**
 * Compute FNV-1a hash of string data
 */
export function fnv1a_hash(data: string): string;

/**
 * Get all valid size names as array
 */
export function get_all_sizes(): any;

/**
 * Get size info as object: {name, width, height}
 */
export function get_size_info(name: string): any;

export function init(): void;

/**
 * Check if a size name is valid
 */
export function is_valid_size(name: string): boolean;

/**
 * Apply grid layout to nodes
 * Input: array of {id, w, h}
 * Output: array of {id, cx, cy}
 */
export function layout_grid(nodes: any, spacing: number): any;

/**
 * Apply hierarchical layout to nodes
 * Input: array of {id, w, h}
 * Output: array of {id, cx, cy}
 */
export function layout_hierarchical(nodes: any, direction: string, spacing: number): any;

/**
 * Measure text dimensions using font metrics
 * Returns {width, height, ascender, descender}
 */
export function measure_text(content: string, font: string, size: number): any;

/**
 * Parse DSL source and return AST as JSON
 */
export function parse(source: string): string;

/**
 * Parse and return errors as JSON (includes both parse and resolution errors)
 */
export function parse_with_errors(source: string): string;

/**
 * Render arrow marker definitions (call once per SVG if using edges)
 */
export function render_arrow_markers(color: string): string;

export function render_blur_filter(id: string, blur: number): string;

export function render_circle(cx: number, cy: number, r: number, style: any, transform?: string | null): string;

/**
 * Render a diamond shape (rotated rectangle for flowcharts)
 */
export function render_diamond(cx: number, cy: number, w: number, h: number, style: any, transform?: string | null): string;

/**
 * Render an edge (connector with optional arrow)
 */
export function render_edge(from_x: number, from_y: number, to_x: number, to_y: number, edge_style: string, arrow: string, label: string | null | undefined, stroke: string, stroke_width: number): string;

export function render_ellipse(cx: number, cy: number, rx: number, ry: number, style: any, transform?: string | null): string;

export function render_image(x: number, y: number, w: number, h: number, href: string, transform?: string | null): string;

export function render_line(x1: number, y1: number, x2: number, y2: number, stroke: string, stroke_width: number, transform?: string | null): string;

export function render_linear_gradient(id: string, from_color: string, to_color: string, angle: number): string;

/**
 * Render a graph node (shape + label)
 */
export function render_node(id: string, shape: string, cx: number, cy: number, w: number, h: number, label: string | null | undefined, style: any): string;

export function render_path(d: string, style: any, transform?: string | null): string;

export function render_polygon(points: any, style: any, transform?: string | null): string;

export function render_radial_gradient(id: string, from_color: string, to_color: string): string;

export function render_rect(x: number, y: number, w: number, h: number, rx: number, style: any, transform?: string | null): string;

/**
 * Render complete scene SVG using standardized size
 */
export function render_scene(size_name: string, background: string, defs: string, elements_svg: string): string;

export function render_shadow_filter(id: string, dx: number, dy: number, blur: number, color: string): string;

/**
 * Render a symbol definition (goes in <defs>)
 * content: inner SVG elements as string
 * viewbox: optional [x, y, width, height]
 */
export function render_symbol(id: string, content: string, viewbox: any): string;

export function render_text(x: number, y: number, content: string, font: string, size: number, weight: string, anchor: string, fill: string, transform?: string | null): string;

/**
 * Render a use element (references a symbol)
 */
export function render_use(href: string, x: number, y: number, width: any, height: any, style: any, transform?: string | null): string;

/**
 * Check if two scenes need any updates (fast path)
 */
export function scenes_equal(old: any, _new: any): boolean;

/**
 * Get pixel dimensions for a named size
 * Returns [width, height] or null if invalid
 */
export function size_to_pixels(name: string): any;

export function tokenize(source: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly init: () => void;
  readonly size_to_pixels: (a: number, b: number) => any;
  readonly is_valid_size: (a: number, b: number) => number;
  readonly get_all_sizes: () => any;
  readonly get_size_info: (a: number, b: number) => any;
  readonly fnv1a_hash: (a: number, b: number) => [number, number];
  readonly compute_element_id: (a: number, b: number, c: number, d: any) => [number, number];
  readonly render_rect: (a: number, b: number, c: number, d: number, e: number, f: any, g: number, h: number) => [number, number];
  readonly render_circle: (a: number, b: number, c: number, d: any, e: number, f: number) => [number, number];
  readonly render_ellipse: (a: number, b: number, c: number, d: number, e: any, f: number, g: number) => [number, number];
  readonly render_line: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
  readonly render_path: (a: number, b: number, c: any, d: number, e: number) => [number, number];
  readonly render_polygon: (a: any, b: any, c: number, d: number) => [number, number];
  readonly render_text: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => [number, number];
  readonly measure_text: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly compute_text_bounds: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => any;
  readonly render_image: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
  readonly render_linear_gradient: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly render_radial_gradient: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly render_shadow_filter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly render_blur_filter: (a: number, b: number, c: number) => [number, number];
  readonly diff_scenes: (a: any, b: any) => any;
  readonly scenes_equal: (a: any, b: any) => number;
  readonly render_scene: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
  readonly compute_path_bounds: (a: number, b: number) => any;
  readonly render_diamond: (a: number, b: number, c: number, d: number, e: any, f: number, g: number) => [number, number];
  readonly render_node: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: any) => [number, number];
  readonly render_edge: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => [number, number];
  readonly render_arrow_markers: (a: number, b: number) => [number, number];
  readonly compute_edge_anchors: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => any;
  readonly layout_hierarchical: (a: any, b: number, c: number, d: number) => any;
  readonly layout_grid: (a: any, b: number) => any;
  readonly render_symbol: (a: number, b: number, c: number, d: number, e: any) => [number, number];
  readonly render_use: (a: number, b: number, c: number, d: number, e: any, f: any, g: any, h: number, i: number) => [number, number];
  readonly tokenize: (a: number, b: number) => [number, number];
  readonly parse: (a: number, b: number) => [number, number];
  readonly parse_with_errors: (a: number, b: number) => [number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
