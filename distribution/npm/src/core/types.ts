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
  animation?: AnimationState;
  children: Shape[];
}

// ─────────────────────────────────────────────────────────────────────────────
// Animation Types
// ─────────────────────────────────────────────────────────────────────────────

/** Easing/timing function for animations */
export type Easing = 
  | 'linear' | 'ease' | 'ease-in' | 'ease-out' | 'ease-in-out'
  | { cubicBezier: [number, number, number, number] }
  | { steps: [number, 'start' | 'end' | 'both' | 'none'] };

/** Animation playback direction */
export type AnimationDirection = 'normal' | 'reverse' | 'alternate' | 'alternate-reverse';

/** Animation fill mode */
export type AnimationFillMode = 'none' | 'forwards' | 'backwards' | 'both';

/** Animation iteration count */
export type AnimationIteration = number | 'infinite';

/** Duration in milliseconds */
export type Duration = number;

/** Animatable CSS properties */
export interface AnimatableProperty {
  opacity?: number;
  fill?: string;
  stroke?: string;
  strokeWidth?: number;
  transform?: string;
  translate?: [number, number];
  rotate?: number;
  scale?: [number, number];
  d?: string; // SVG path
}

/** Single keyframe step */
export interface KeyframeStep {
  offset: number; // 0-100%
  properties: AnimatableProperty;
}

/** Keyframes animation definition */
export interface Keyframes {
  name: string;
  steps: KeyframeStep[];
}

/** Animation reference applied to an element */
export interface Animation {
  name: string;
  duration: Duration;
  easing: Easing;
  delay: Duration;
  iteration: AnimationIteration;
  direction: AnimationDirection;
  fillMode: AnimationFillMode;
}

/** CSS transition definition */
export interface Transition {
  property: string;
  duration: Duration;
  easing: Easing;
  delay: Duration;
}

/** Animation state attached to a shape */
export interface AnimationState {
  animation?: Animation;
  transitions: Transition[];
}

/** Helper to create animation with defaults */
export const createAnimation = (
  name: string,
  duration: Duration = 300,
  easing: Easing = 'ease'
): Animation => ({
  name,
  duration,
  easing,
  delay: 0,
  iteration: 1,
  direction: 'normal',
  fillMode: 'none',
});

/** Helper to create transition with defaults */
export const createTransition = (
  property = 'all',
  duration: Duration = 300,
  easing: Easing = 'ease'
): Transition => ({
  property,
  duration,
  easing,
  delay: 0,
});

/** Convert animation to CSS string */
export const animationToCSS = (anim: Animation): string =>
  `${anim.name} ${anim.duration}ms ${typeof anim.easing === 'string' ? anim.easing : 
    'cubicBezier' in anim.easing ? `cubic-bezier(${anim.easing.cubicBezier.join(',')})` :
    `steps(${anim.easing.steps[0]}, ${anim.easing.steps[1]})`
  } ${anim.delay}ms ${anim.iteration === 'infinite' ? 'infinite' : anim.iteration} ${anim.direction} ${anim.fillMode}`;

/** Convert transition to CSS string */
export const transitionToCSS = (t: Transition): string =>
  `${t.property} ${t.duration}ms ${typeof t.easing === 'string' ? t.easing :
    'cubicBezier' in t.easing ? `cubic-bezier(${t.easing.cubicBezier.join(',')})` :
    `steps(${t.easing.steps[0]}, ${t.easing.steps[1]})`
  } ${t.delay}ms`;

/** Node shapes for graphs/flowcharts */
export type NodeShape = 'rect' | 'circle' | 'ellipse' | 'diamond';

/** Edge line styles */
export type EdgeStyle = 'straight' | 'curved' | 'orthogonal';

/** Arrow types for edges */
export type ArrowType = 'none' | 'forward' | 'backward' | 'both';

/** Graph layout algorithms */
export type GraphLayout = 'manual' | 'hierarchical' | 'grid' | 'tree' | 'force';

/** A node in a graph/flowchart */
export interface GraphNode {
  id: string;
  shape: NodeShape;
  label?: string;
  at?: [number, number];  // [cx, cy]
  size?: [number, number]; // [width, height]
  style?: Partial<Style>;
}

/** An edge/connector between nodes */
export interface GraphEdge {
  from: string;
  to: string;
  style: EdgeStyle;
  arrow: ArrowType;
  label?: string;
  stroke?: string;
  strokeWidth: number;
}

/** Graph container with layout */
export interface Graph {
  layout: GraphLayout;
  direction: 'vertical' | 'horizontal';
  spacing: number;
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface AstNode {
  type: string;
  value?: Canvas | Shape | Graph | Record<string, unknown>;
  children: AstNode[];
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
