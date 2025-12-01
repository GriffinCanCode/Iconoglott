import { Lexer } from './core/lexer';
import { Parser } from './core/parser';
import { Interpreter, SceneState } from './core/renderer';
import { tryGetWasm } from './wasm/bridge';
import { renderShapeWasm, renderSceneWasm } from './core/wasm-renderer';

// Re-export types
export * from './core/types';
export { Lexer, Parser, Interpreter, SceneState };

// Re-export WASM bridge
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

/**
 * Render iconoglott DSL code to SVG string (client-side, no backend needed).
 * Uses TypeScript-only rendering. For WASM-accelerated rendering, use renderWasm().
 * 
 * @example
 * ```ts
 * import { render } from '@iconoglott/renderer'
 * 
 * const svg = render(`
 *   canvas 400x300 fill #1a1a2e
 *   circle at 200,150 radius 50
 *     fill #e94560
 * `)
 * 
 * document.getElementById('canvas').innerHTML = svg
 * ```
 */
export function render(source: string): string {
  const interpreter = new Interpreter();
  return interpreter.eval(source, Lexer, Parser).toSvg();
}

/**
 * Parse iconoglott DSL and return scene state for further manipulation.
 */
export function parse(source: string): SceneState {
  const interpreter = new Interpreter();
  return interpreter.eval(source, Lexer, Parser);
}

/**
 * Render iconoglott DSL using WASM-accelerated renderer.
 * Falls back to TypeScript if WASM isn't loaded.
 * 
 * @example
 * ```ts
 * import { initWasm, renderWasm } from '@iconoglott/renderer'
 * 
 * // Initialize WASM once at startup
 * await initWasm()
 * 
 * // Then render with WASM acceleration
 * const svg = renderWasm(`
 *   canvas 400x300 fill #1a1a2e
 *   circle at 200,150 radius 50
 *     fill #e94560
 * `)
 * ```
 */
export function renderWasm(source: string): string {
  const wasm = tryGetWasm();
  if (!wasm) return render(source); // Fallback to TypeScript
  
  // Parse with TypeScript (DSL parsing stays in TS)
  const interpreter = new Interpreter();
  const state = interpreter.eval(source, Lexer, Parser);
  
  // Render shapes with WASM
  const elementsSvg = state.shapes.map(s => renderShapeWasm(s, wasm)).join('');
  
  // Render full scene with WASM
  return renderSceneWasm(wasm, state.canvas, state.defs.join(''), elementsSvg);
}
