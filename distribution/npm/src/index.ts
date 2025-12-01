import { Lexer } from './core/lexer';
import { Parser } from './core/parser';
import { Interpreter, SceneState } from './core/renderer';

// Re-export types
export * from './core/types';
export { Lexer, Parser, Interpreter, SceneState };

// Re-export WebSocket client
export { createClient } from './client/ws';
export type { IconoglottClient, IconoglottClientOptions } from './client/ws';

/**
 * Render iconoglott DSL code to SVG string (client-side, no backend needed).
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
