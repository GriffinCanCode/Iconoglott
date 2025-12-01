export * from './types';
export { Lexer } from './lexer';
export { Parser } from './parser';
export { SceneState, Interpreter } from './renderer';
export {
  fnv1a,
  computeId,
  indexScene,
  diff,
  RenderCache,
  type ContentHash,
  type ElementId,
  type IndexedElement,
  type SceneIndex,
  type IndexableScene,
  type DiffOp,
  type DiffResult,
} from './diff';

// Text metrics (WASM-accelerated)
export {
  measureTextWasm,
  computeTextBoundsWasm,
  type TextMetrics,
} from './wasm-renderer';

