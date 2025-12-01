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

// ─────────────────────────────────────────────────────────────────────────────
// High-level API (WASM required)
// ─────────────────────────────────────────────────────────────────────────────

import { getWasm, tryGetWasm, initWasm } from './wasm/bridge';
import { renderShapeWasm, renderSceneWasm } from './core/wasm-renderer';
import type { Canvas } from './core/types';

/** Parsed AST node (from Rust parser) */
export interface AstNode {
  Scene?: AstNode[];
  Canvas?: { width: number; height: number; fill: string };
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
function astShapeToDict(shape: AstShape): Record<string, unknown> {
  return {
    kind: shape.kind,
    props: shape.props,
    style: {
      fill: shape.style.fill ?? shape.props.fill,
      stroke: shape.style.stroke,
      strokeWidth: shape.style.stroke_width,
      opacity: shape.style.opacity,
      corner: shape.style.corner,
      font: shape.style.font,
      fontSize: shape.style.font_size,
      fontWeight: shape.style.font_weight,
      textAnchor: shape.style.text_anchor,
    },
    transform: {
      translate: shape.transform.translate,
      rotate: shape.transform.rotate,
      scale: shape.transform.scale,
      origin: shape.transform.origin,
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
 *   canvas 400x300 fill #1a1a2e
 *   circle at 200,150 radius 50
 *     fill #e94560
 * `)
 * ```
 */
export function render(source: string): string {
  const wasm = getWasm();
  const { ast, errors } = parse(source);
  
  // Extract canvas and shapes from AST
  let canvas: Canvas = { width: 800, height: 600, fill: '#fff' };
  const shapes: Record<string, unknown>[] = [];
  
  const nodes = ast.Scene ?? [ast];
  for (const node of nodes) {
    if (node.Canvas) {
      canvas = node.Canvas;
    } else if (node.Shape) {
      shapes.push(astShapeToDict(node.Shape));
    }
  }
  
  // Render shapes with WASM
  const elementsSvg = shapes.map(s => renderShapeWasm(s as Parameters<typeof renderShapeWasm>[0], wasm)).join('');
  
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
