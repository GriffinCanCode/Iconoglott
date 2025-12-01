/**
 * WASM-accelerated renderer
 * 
 * Uses Rust WASM for performance-critical operations:
 * - Shape SVG generation
 * - Scene diffing
 * - Content hashing
 * 
 * TypeScript handles:
 * - DSL parsing (lexer/parser)
 * - High-level scene API
 * - React bindings
 */

import type { Canvas, GradientDef, ShadowDef } from './types';
import { tryGetWasm, isWasmLoaded, type WasmCore } from '../wasm/bridge';

// ─────────────────────────────────────────────────────────────────────────────
// Style serialization for WASM
// ─────────────────────────────────────────────────────────────────────────────

interface WasmStyle {
  fill?: string;
  stroke?: string;
  stroke_width: number;
  opacity: number;
  corner: number;
  filter?: string;
}

function toWasmStyle(style: Record<string, unknown>): string {
  const ws: WasmStyle = {
    fill: style.fill as string | undefined,
    stroke: style.stroke as string | undefined,
    stroke_width: (style.strokeWidth as number) ?? 1,
    opacity: (style.opacity as number) ?? 1,
    corner: (style.corner as number) ?? 0,
    filter: style.filter as string | undefined,
  };
  return JSON.stringify(ws);
}

// ─────────────────────────────────────────────────────────────────────────────
// Shape Rendering (WASM-accelerated)
// ─────────────────────────────────────────────────────────────────────────────

export interface ShapeDict {
  id?: string;
  kind: string;
  props: Record<string, unknown>;
  style: Record<string, unknown>;
  transform: Record<string, unknown>;
  children: ShapeDict[];
}

function buildTransform(tf: Record<string, unknown>): string | undefined {
  const parts: string[] = [];
  if (tf.translate) {
    const [tx, ty] = tf.translate as [number, number];
    parts.push(`translate(${tx},${ty})`);
  }
  if (tf.rotate) {
    const origin = tf.origin as [number, number] | undefined;
    if (origin) parts.push(`rotate(${tf.rotate},${origin[0]},${origin[1]})`);
    else parts.push(`rotate(${tf.rotate})`);
  }
  if (tf.scale) {
    const [sx, sy] = tf.scale as [number, number];
    parts.push(`scale(${sx},${sy})`);
  }
  return parts.length ? parts.join(' ') : undefined;
}

/**
 * Render shape using WASM if available, otherwise falls back to TS
 */
export function renderShapeWasm(s: ShapeDict, wasm: WasmCore, offset: [number, number] = [0, 0]): string {
  const { kind, props, style, transform, children } = s;
  const at = (props.at as [number, number]) ?? [0, 0];
  const [x, y] = [at[0] + offset[0], at[1] + offset[1]];
  const tf = buildTransform(transform);

  switch (kind) {
    case 'rect': {
      const [w, h] = (props.size as [number, number]) ?? [100, 100];
      const corner = (style.corner as number) ?? 0;
      return wasm.render_rect(x, y, w, h, corner, toWasmStyle(style), tf);
    }
    case 'circle': {
      const r = (props.radius as number) ?? 50;
      return wasm.render_circle(x, y, r, toWasmStyle(style), tf);
    }
    case 'ellipse': {
      const [rx, ry] = (props.size as [number, number]) ?? [50, 30];
      return wasm.render_ellipse(x, y, rx, ry, toWasmStyle(style), tf);
    }
    case 'line': {
      const [x1, y1] = (props.from as [number, number]) ?? [0, 0];
      const [x2, y2] = (props.to as [number, number]) ?? [100, 100];
      const stroke = (style.stroke as string) ?? '#000';
      const width = (style.strokeWidth as number) ?? 1;
      return wasm.render_line(x1, y1, x2, y2, stroke, width, tf);
    }
    case 'path': {
      const d = (props.content as string) ?? (props.d as string) ?? '';
      return wasm.render_path(d, toWasmStyle(style), tf);
    }
    case 'polygon': {
      const points = (props.points as [number, number][]) ?? [];
      return wasm.render_polygon(JSON.stringify(points), toWasmStyle(style), tf);
    }
    case 'text': {
      const content = (props.content as string) ?? '';
      const font = (style.font as string) ?? 'system-ui';
      const size = (style.fontSize as number) ?? 16;
      const weight = (style.fontWeight as string) ?? 'normal';
      const anchor = (style.textAnchor as string) ?? 'start';
      const fill = (style.fill as string) ?? '#000';
      return wasm.render_text(x, y, content, font, size, weight, anchor, fill, tf);
    }
    case 'image': {
      const [w, h] = (props.size as [number, number]) ?? [100, 100];
      const href = (props.href as string) ?? '';
      return wasm.render_image(x, y, w, h, href, tf);
    }
    case 'group': {
      const inner = children.map(c => renderShapeWasm(c, wasm)).join('');
      return tf ? `<g transform="${tf}">${inner}</g>` : `<g>${inner}</g>`;
    }
    case 'layout': {
      const direction = (props.direction as string) ?? 'vertical';
      const gap = (props.gap as number) ?? 0;
      let inner = '';
      let layoutOffset = 0;

      for (const c of children) {
        const childOffset: [number, number] = direction === 'vertical'
          ? [x, y + layoutOffset]
          : [x + layoutOffset, y];
        inner += renderShapeWasm(c, wasm, childOffset);

        const size = c.props.size as [number, number] | undefined;
        const radius = c.props.radius as number | undefined;
        if (direction === 'vertical') {
          layoutOffset += (size ? size[1] : (radius ? radius * 2 : 40)) + gap;
        } else {
          layoutOffset += (size ? size[0] : (radius ? radius * 2 : 40)) + gap;
        }
      }
      return tf ? `<g transform="${tf}">${inner}</g>` : `<g>${inner}</g>`;
    }
    default:
      return '';
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Gradient/Filter Rendering
// ─────────────────────────────────────────────────────────────────────────────

export function renderGradientWasm(wasm: WasmCore, id: string, grad: GradientDef): string {
  if (grad.type === 'radial') {
    return wasm.render_radial_gradient(id, grad.from, grad.to);
  }
  return wasm.render_linear_gradient(id, grad.from, grad.to, grad.angle);
}

export function renderShadowWasm(wasm: WasmCore, id: string, shadow: ShadowDef): string {
  return wasm.render_shadow_filter(id, shadow.x, shadow.y, shadow.blur, shadow.color);
}

// ─────────────────────────────────────────────────────────────────────────────
// Scene Diffing (WASM-accelerated)
// ─────────────────────────────────────────────────────────────────────────────

export interface WasmDiffOp {
  type: 'none' | 'full_redraw' | 'add' | 'remove' | 'update' | 'move' | 'update_defs';
  id?: string;
  idx?: number;
  svg?: string;
  from_idx?: number;
  to_idx?: number;
}

export interface WasmDiffResult {
  ops: WasmDiffOp[];
  canvas_changed: boolean;
}

export interface WasmSceneInput {
  canvas: { size: string; fill: string };
  elements: Array<{ id: string; kind: string; svg: string }>;
  defs: string;
}

/**
 * Diff two scenes using WASM
 */
export function diffScenesWasm(wasm: WasmCore, old: WasmSceneInput, current: WasmSceneInput): WasmDiffResult {
  const result = wasm.diff_scenes(JSON.stringify(old), JSON.stringify(current));
  return JSON.parse(result) as WasmDiffResult;
}

/**
 * Check if scenes are equal (fast path)
 */
export function scenesEqualWasm(wasm: WasmCore, old: WasmSceneInput, current: WasmSceneInput): boolean {
  return wasm.scenes_equal(JSON.stringify(old), JSON.stringify(current));
}

// ─────────────────────────────────────────────────────────────────────────────
// Hashing (WASM-accelerated)
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Compute element ID using WASM FNV-1a
 */
export function computeIdWasm(wasm: WasmCore, order: number, kind: string, keyProps?: Record<string, unknown>): string {
  const keyJson = keyProps ? JSON.stringify(keyProps) : '{}';
  return wasm.compute_element_id(order, kind, keyJson);
}

/**
 * Compute content hash using WASM FNV-1a
 */
export function hashContentWasm(wasm: WasmCore, content: string): string {
  return wasm.fnv1a_hash(content);
}

// ─────────────────────────────────────────────────────────────────────────────
// Text Metrics (WASM-accelerated)
// ─────────────────────────────────────────────────────────────────────────────

export interface TextMetrics {
  width: number;
  height: number;
  ascender: number;
  descender: number;
}

/**
 * Measure text dimensions using bundled font metrics
 */
export function measureTextWasm(wasm: WasmCore, content: string, font: string, size: number): TextMetrics {
  return JSON.parse(wasm.measure_text(content, font, size)) as TextMetrics;
}

/**
 * Compute text bounding box accounting for anchor position
 * Returns [x, y, width, height]
 */
export function computeTextBoundsWasm(
  wasm: WasmCore,
  x: number,
  y: number,
  content: string,
  font: string,
  size: number,
  anchor: string
): [number, number, number, number] {
  return JSON.parse(wasm.compute_text_bounds(x, y, content, font, size, anchor)) as [number, number, number, number];
}

// ─────────────────────────────────────────────────────────────────────────────
// Full Scene Rendering
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Render complete scene SVG using WASM
 */
export function renderSceneWasm(
  wasm: WasmCore,
  canvas: Canvas,
  defs: string,
  elementsSvg: string
): string {
  return wasm.render_scene(canvas.size, canvas.fill, defs, elementsSvg);
}

