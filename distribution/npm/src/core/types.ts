/** Token types for lexical analysis */
export type TokenType =
  | 'IDENT' | 'NUMBER' | 'STRING' | 'COLOR' | 'VAR' | 'PAIR'
  | 'COLON' | 'EQUALS' | 'ARROW' | 'LBRACKET' | 'RBRACKET'
  | 'NEWLINE' | 'INDENT' | 'DEDENT' | 'EOF';

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
  width: number;
  height: number;
  fill: string;
}

export interface Shape {
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

export const defaultCanvas = (): Canvas => ({ width: 800, height: 600, fill: '#fff' });

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
