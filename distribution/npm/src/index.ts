/**
 * Iconoglott - Visual DSL for SVG graphics
 * 
 * This package uses Rust/WASM as the SINGLE SOURCE OF TRUTH for:
 * - DSL lexing and parsing
 * - SVG rendering
 * - Scene diffing
 * 
 * TypeScript provides only:
 * - WASM bridge and type definitions
 * - React bindings
 * - WebSocket client
 */

// Re-export types
export * from './core/types';

// Re-export WASM bridge (required for all operations)
export { initWasm, isWasmLoaded, getWasm, tryGetWasm } from './wasm/bridge';
export type { WasmCore } from './wasm/bridge';

// Re-export WASM renderer utilities
export {
  renderShapeWasm,
  renderGradientWasm,
  renderShadowWasm,
  diffScenesWasm,
  scenesEqualWasm,
  computeIdWasm,
  hashContentWasm,
  renderSceneWasm,
} from './core/wasm-renderer';
export type { ShapeDict, WasmDiffOp, WasmDiffResult, WasmSceneInput } from './core/wasm-renderer';

// Re-export WebSocket client
export { createClient } from './client/ws';
export type { IconoglottClient, IconoglottClientOptions } from './client/ws';

// Re-export Canvas (for convenience - also available via /canvas)
export { 
  Canvas, 
  createContainCanvas, 
  createCoverCanvas, 
  createFillCanvas,
  renderTo, 
  connectTo,
  CARD_PRESET,
  BACKGROUND_PRESET,
  ICON_PRESET,
  THUMBNAIL_PRESET,
} from './canvas';
export type { CanvasOptions, CanvasFit } from './canvas';

// ─────────────────────────────────────────────────────────────────────────────
// High-level API (WASM required)
// ─────────────────────────────────────────────────────────────────────────────

import { getWasm, tryGetWasm, initWasm } from './wasm/bridge';
import { renderShapeWasm, renderSceneWasm, type ShapeDict } from './core/wasm-renderer';
import { CANVAS_SIZES, type Canvas, type CanvasSize } from './core/types';

/** Parsed AST node (from Rust parser) */
export interface AstNode {
  Scene?: AstNode[];
  Canvas?: { size: string; fill: string };
  Shape?: AstShape;
  Variable?: { name: string; value: unknown };
}

export interface AstShape {
  kind: string;
  props: Record<string, unknown>;
  style: AstStyle;
  shadow?: { x: number; y: number; blur: number; color: string };
  gradient?: { gtype: string; from: string; to: string; angle: number };
  transform: AstTransform;
  children: AstShape[];
}

export interface AstStyle {
  fill?: string;
  stroke?: string;
  stroke_width: number;
  opacity: number;
  corner: number;
  font?: string;
  font_size: number;
  font_weight: string;
  text_anchor: string;
}

export interface AstTransform {
  translate?: [number, number];
  rotate: number;
  scale?: [number, number];
  origin?: [number, number];
}

export interface ParseResult {
  ast: AstNode;
  errors: Array<{ message: string; line: number; col: number }>;
}

/**
 * Parse iconoglott DSL source to AST.
 * WASM must be initialized first.
 */
export function parse(source: string): ParseResult {
  const wasm = getWasm();
  const json = wasm.parse_with_errors(source);
  return JSON.parse(json) as ParseResult;
}

/**
 * Parse DSL and return just the AST (no errors).
 */
export function parseAst(source: string): AstNode {
  const wasm = getWasm();
  const json = wasm.parse(source);
  return JSON.parse(json) as AstNode;
}

/**
 * Tokenize DSL source.
 */
export function tokenize(source: string): unknown[] {
  const wasm = getWasm();
  const json = wasm.tokenize(source);
  return JSON.parse(json) as unknown[];
}

/**
 * Convert AST shape to internal dict format for rendering.
 */
function astShapeToDict(shape: AstShape): ShapeDict {
  return {
    kind: shape.kind,
    props: shape.props as Record<string, unknown>,
    style: {
      fill: (shape.style.fill ?? shape.props.fill) as string | undefined,
      stroke: shape.style.stroke ?? undefined,
      strokeWidth: shape.style.stroke_width,
      opacity: shape.style.opacity,
      corner: shape.style.corner,
      font: shape.style.font ?? undefined,
      fontSize: shape.style.font_size,
      fontWeight: shape.style.font_weight,
      textAnchor: shape.style.text_anchor,
    },
    transform: {
      translate: shape.transform.translate ?? undefined,
      rotate: shape.transform.rotate,
      scale: shape.transform.scale ?? undefined,
      origin: shape.transform.origin ?? undefined,
    },
    children: shape.children.map(astShapeToDict),
  };
}

/**
 * Render iconoglott DSL code to SVG string.
 * WASM must be initialized first via initWasm().
 * 
 * @example
 * ```ts
 * import { initWasm, render } from '@iconoglott/renderer'
 * 
 * await initWasm()
 * 
 * const svg = render(`
 *   canvas massive fill #1a1a2e
 *   circle at 200,150 radius 50
 *     fill #e94560
 * `)
 * ```
 */
export function render(source: string): string {
  const wasm = getWasm();
  const { ast } = parse(source);
  
  // Extract canvas and shapes from AST
  let canvas: Canvas = { size: 'medium', fill: '#fff', width: 64, height: 64 };
  const shapes: ShapeDict[] = [];
  
  const nodes = ast.Scene ?? [ast];
  for (const node of nodes) {
    if (node.Canvas) {
      // Map AstCanvas to Canvas with computed dimensions
      const astCanvas = node.Canvas;
      const dims = CANVAS_SIZES[astCanvas.size.toLowerCase() as CanvasSize] ?? { width: 64, height: 64 };
      canvas = { size: astCanvas.size.toLowerCase() as CanvasSize, fill: astCanvas.fill, ...dims };
    } else if (node.Shape) {
      shapes.push(astShapeToDict(node.Shape) as ShapeDict);
    }
  }
  
  // Render shapes with WASM
  const elementsSvg = shapes.map(s => renderShapeWasm(s, wasm)).join('');
  
  // Generate full SVG
  return renderSceneWasm(wasm, canvas, '', elementsSvg);
}

/**
 * Initialize WASM and render in one call.
 * Convenience function for simple use cases.
 */
export async function renderAsync(source: string): Promise<string> {
  await initWasm();
  return render(source);
}
