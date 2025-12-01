export * from './types';

// Re-export specific types for convenience
export type {
  CanvasSize,
  Canvas,
  Shape,
  Style,
  Transform,
  Token,
  TokenType,
} from './types';

export {
  CANVAS_SIZES,
  ALL_SIZES,
  getSizePixels,
  isValidSize,
  createCanvas,
} from './types';

// Text metrics (WASM-accelerated)
export {
  measureTextWasm,
  computeTextBoundsWasm,
  type TextMetrics,
} from './wasm-renderer';

// DSL Processing - uses Rust WASM as single source of truth
// Use initWasm() first, then getWasm().tokenize() / getWasm().parse()
export { initWasm, getWasm, tryGetWasm, isWasmLoaded } from '../wasm/bridge';

