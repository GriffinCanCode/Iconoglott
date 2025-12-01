/** Token types for lexical analysis */
export type TokenType =
  | 'IDENT' | 'NUMBER' | 'STRING' | 'COLOR' | 'VAR' | 'PAIR' | 'SIZE'
  | 'COLON' | 'EQUALS' | 'ARROW' | 'LBRACKET' | 'RBRACKET'
  | 'NEWLINE' | 'INDENT' | 'DEDENT' | 'EOF';

/** Standard canvas sizes (10-tier system) */
export type CanvasSize = 
  | 'nano'    // 16×16 - Favicons, tiny UI
  | 'micro'   // 24×24 - Small UI icons
  | 'tiny'    // 32×32 - Standard UI icons
  | 'small'   // 48×48 - Toolbar icons
  | 'medium'  // 64×64 - Medium icons
  | 'large'   // 96×96 - Large display icons
  | 'xlarge'  // 128×128 - Small app icons
  | 'huge'    // 192×192 - Touch/PWA icons
  | 'massive' // 256×256 - Medium app icons
  | 'giant';  // 512×512 - Large app icons

/** Size name to pixel dimensions mapping */
export const CANVAS_SIZES: Record<CanvasSize, { width: number; height: number }> = {
  nano:    { width: 16, height: 16 },
  micro:   { width: 24, height: 24 },
  tiny:    { width: 32, height: 32 },
  small:   { width: 48, height: 48 },
  medium:  { width: 64, height: 64 },
  large:   { width: 96, height: 96 },
  xlarge:  { width: 128, height: 128 },
  huge:    { width: 192, height: 192 },
  massive: { width: 256, height: 256 },
  giant:   { width: 512, height: 512 },
};

/** Get pixel dimensions for a canvas size */
export function getSizePixels(size: CanvasSize): { width: number; height: number } {
  return CANVAS_SIZES[size];
}

/** Check if a string is a valid canvas size */
export function isValidSize(name: string): name is CanvasSize {
  return name in CANVAS_SIZES;
}

/** All valid size names */
export const ALL_SIZES: CanvasSize[] = ['nano', 'micro', 'tiny', 'small', 'medium', 'large', 'xlarge', 'huge', 'massive', 'giant'];

export interface Token {
  type: TokenType;
  value: string | number | [number, number];
  line: number;
  col: number;
}

export interface Style {
  fill?: string;
  stroke?: string;
  strokeWidth: number;
  opacity: number;
  corner: number;
  font?: string;
  fontSize: number;
  fontWeight: string;
  textAnchor: string;
  shadow?: ShadowDef;
  gradient?: GradientDef;
}

export interface ShadowDef {
  x: number;
  y: number;
  blur: number;
  color: string;
}

export interface GradientDef {
  type: 'linear' | 'radial';
  from: string;
  to: string;
  angle: number;
}

export interface Transform {
  translate?: [number, number];
  rotate: number;
  scale?: [number, number];
  origin?: [number, number];
}

export interface Canvas {
  size: CanvasSize;
  fill: string;
  /** Computed width from size */
  readonly width: number;
  /** Computed height from size */
  readonly height: number;
}

export interface Shape {
  id?: string;
  kind: string;
  props: Record<string, unknown>;
  style: Style;
  transform: Transform;
  children: Shape[];
}

export interface Node {
  type: string;
  value?: Canvas | Shape | Record<string, unknown>;
  children: Node[];
}

export const defaultStyle = (): Style => ({
  strokeWidth: 1,
  opacity: 1,
  corner: 0,
  fontSize: 16,
  fontWeight: 'normal',
  textAnchor: 'start',
});

export const defaultTransform = (): Transform => ({ rotate: 0 });

export const defaultCanvas = (): Canvas => {
  const { width, height } = CANVAS_SIZES.medium;
  return { size: 'medium', fill: '#fff', width, height };
};

/** Create a canvas with specified size */
export const createCanvas = (size: CanvasSize, fill = '#fff'): Canvas => {
  const { width, height } = CANVAS_SIZES[size];
  return { size, fill, width, height };
};

// ─────────────────────────────────────────────────────────────────────────────
// Backend Protocol Types
// ─────────────────────────────────────────────────────────────────────────────

export type Severity = 'error' | 'warning' | 'info';

export interface ParseError {
  code: string;
  category: string;
  message: string;
  line: number;
  col: number;
  severity: Severity;
}

/** Server response for render success */
export interface RenderResponse {
  type: 'render';
  svg: string;
  errors: ParseError[];
}

/** Server response for fatal error */
export interface ErrorResponse {
  type: 'error';
  message: string;
  errors: ParseError[];
}

/** Pong response for keep-alive */
export interface PongResponse {
  type: 'pong';
}

export type ServerMessage = RenderResponse | ErrorResponse | PongResponse;

/** Client message to send DSL source */
export interface SourceMessage {
  type: 'source';
  payload: string;
}

/** Client message for keep-alive */
export interface PingMessage {
  type: 'ping';
}

export type ClientMessage = SourceMessage | PingMessage;
