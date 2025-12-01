/* tslint:disable */
/* eslint-disable */

export class WasmStyle {
  free(): void;
  [Symbol.dispose](): void;
  constructor();
  set fill(value: string | null | undefined);
  set stroke(value: string | null | undefined);
  set stroke_width(value: number);
  set opacity(value: number);
  set corner(value: number);
  set filter(value: string | null | undefined);
}

/**
 * Compute stable element ID from order, kind, and key properties
 */
export function compute_element_id(order: number, kind: string, key_json: string): string;

export function compute_path_bounds(d: string): string;

/**
 * Diff two scenes and return JSON array of operations
 */
export function diff_scenes(old_json: string, new_json: string): string;

/**
 * Compute FNV-1a hash of string data
 */
export function fnv1a_hash(data: string): string;

export function init(): void;

/**
 * Parse DSL source and return AST as JSON
 */
export function parse(source: string): string;

/**
 * Parse and return errors as JSON
 */
export function parse_with_errors(source: string): string;

export function render_blur_filter(id: string, blur: number): string;

export function render_circle(cx: number, cy: number, r: number, style_json: string, transform?: string | null): string;

export function render_ellipse(cx: number, cy: number, rx: number, ry: number, style_json: string, transform?: string | null): string;

export function render_image(x: number, y: number, w: number, h: number, href: string, transform?: string | null): string;

export function render_line(x1: number, y1: number, x2: number, y2: number, stroke: string, stroke_width: number, transform?: string | null): string;

export function render_linear_gradient(id: string, from_color: string, to_color: string, angle: number): string;

export function render_path(d: string, style_json: string, transform?: string | null): string;

export function render_polygon(points_json: string, style_json: string, transform?: string | null): string;

export function render_radial_gradient(id: string, from_color: string, to_color: string): string;

export function render_rect(x: number, y: number, w: number, h: number, rx: number, style_json: string, transform?: string | null): string;

export function render_scene(width: number, height: number, background: string, defs: string, elements_svg: string): string;

export function render_shadow_filter(id: string, dx: number, dy: number, blur: number, color: string): string;

export function render_text(x: number, y: number, content: string, font: string, size: number, weight: string, anchor: string, fill: string, transform?: string | null): string;

/**
 * Check if two scenes need any updates (fast path)
 */
export function scenes_equal(old_json: string, new_json: string): boolean;

export function tokenize(source: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly init: () => void;
  readonly fnv1a_hash: (a: number, b: number) => [number, number];
  readonly compute_element_id: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly __wbg_wasmstyle_free: (a: number, b: number) => void;
  readonly wasmstyle_new: () => number;
  readonly wasmstyle_set_fill: (a: number, b: number, c: number) => void;
  readonly wasmstyle_set_stroke: (a: number, b: number, c: number) => void;
  readonly wasmstyle_set_stroke_width: (a: number, b: number) => void;
  readonly wasmstyle_set_opacity: (a: number, b: number) => void;
  readonly wasmstyle_set_corner: (a: number, b: number) => void;
  readonly wasmstyle_set_filter: (a: number, b: number, c: number) => void;
  readonly render_rect: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
  readonly render_circle: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly render_ellipse: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
  readonly render_line: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
  readonly render_path: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly render_polygon: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly render_text: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number) => [number, number];
  readonly render_image: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
  readonly render_linear_gradient: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly render_radial_gradient: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly render_shadow_filter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly render_blur_filter: (a: number, b: number, c: number) => [number, number];
  readonly diff_scenes: (a: number, b: number, c: number, d: number) => [number, number];
  readonly scenes_equal: (a: number, b: number, c: number, d: number) => number;
  readonly render_scene: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
  readonly compute_path_bounds: (a: number, b: number) => [number, number];
  readonly parse: (a: number, b: number) => [number, number];
  readonly parse_with_errors: (a: number, b: number) => [number, number];
  readonly tokenize: (a: number, b: number) => [number, number];
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
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
