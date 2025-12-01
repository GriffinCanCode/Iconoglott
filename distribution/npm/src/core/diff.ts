/**
 * Incremental scene diffing with stable element IDs
 * 
 * Uses content-addressed hashing + ID-based reconciliation for O(n) diffing
 * with minimal SVG regeneration. Mirrors Rust implementation for cross-platform parity.
 */

import type { Shape, Style, Transform } from './types';

// ─────────────────────────────────────────────────────────────────────────────
// Hashing
// ─────────────────────────────────────────────────────────────────────────────

const FNV_OFFSET = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;

/** FNV-1a hash function for stable identity */
export function fnv1a(data: string): bigint {
  let hash = FNV_OFFSET;
  for (let i = 0; i < data.length; i++) {
    hash ^= BigInt(data.charCodeAt(i));
    hash = BigInt.asUintN(64, hash * FNV_PRIME);
  }
  return hash;
}

/** Content hash for element equality */
export type ContentHash = bigint;

/** Stable element identity */
export type ElementId = bigint;

/** Compute stable ID from creation order and kind */
export function computeId(order: number, kind: string, keyProps?: Record<string, unknown>): ElementId {
  let key = `${order}:${kind}`;
  if (keyProps) {
    const sorted = Object.keys(keyProps).sort();
    for (const k of sorted) key += `:${k}=${JSON.stringify(keyProps[k])}`;
  }
  return fnv1a(key);
}

// ─────────────────────────────────────────────────────────────────────────────
// Indexed Elements
// ─────────────────────────────────────────────────────────────────────────────

export interface IndexedElement {
  id: ElementId;
  hash: ContentHash;
  kind: string;
  index: number;
  svg: string;
}

/** Extract key properties for identity computation */
function keyProps(shape: Shape): Record<string, unknown> {
  const { kind, props } = shape;
  switch (kind) {
    case 'rect': return { at: props.at, size: props.size };
    case 'circle': return { at: props.at, radius: props.radius };
    case 'ellipse': return { at: props.at, size: props.size };
    case 'line': return { from: props.from, to: props.to };
    case 'path': return { d: props.d ?? props.content };
    case 'polygon': return { points: props.points };
    case 'text': return { at: props.at, content: props.content };
    case 'image': return { href: props.href };
    default: return {};
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Diff Operations
// ─────────────────────────────────────────────────────────────────────────────

export type DiffOp =
  | { type: 'none' }
  | { type: 'full_redraw' }
  | { type: 'add'; id: string; idx: number; svg: string }
  | { type: 'remove'; id: string; idx: number }
  | { type: 'update'; id: string; idx: number; attrs: [string, string][]; svg?: string }
  | { type: 'move'; id: string; from: number; to: number }
  | { type: 'update_defs'; svg: string };

// ─────────────────────────────────────────────────────────────────────────────
// Scene Indexing
// ─────────────────────────────────────────────────────────────────────────────

export interface SceneIndex {
  elements: IndexedElement[];
  idMap: Map<string, number>;
}

export interface IndexableScene {
  canvas: { width: number; height: number; fill: string };
  shapes: ShapeDict[];
  defs: string[];
}

interface ShapeDict {
  kind: string;
  props: Record<string, unknown>;
  style: Record<string, unknown>;
  transform: Record<string, unknown>;
  children: ShapeDict[];
}

/** Index a scene for O(1) lookups */
export function indexScene(scene: IndexableScene, renderShape: (s: ShapeDict) => string): SceneIndex {
  const elements: IndexedElement[] = [];
  const idMap = new Map<string, number>();

  for (let i = 0; i < scene.shapes.length; i++) {
    const shape = scene.shapes[i];
    const svg = renderShape(shape);
    const id = computeId(i, shape.kind, shape.props as Record<string, unknown>);
    const hash = fnv1a(svg);
    const idStr = id.toString(16);

    elements.push({ id, hash, kind: shape.kind, index: i, svg });
    idMap.set(idStr, i);
  }

  return { elements, idMap };
}

// ─────────────────────────────────────────────────────────────────────────────
// Diffing
// ─────────────────────────────────────────────────────────────────────────────

export interface DiffResult {
  ops: DiffOp[];
  canvasChanged: boolean;
}

/** Diff two indexed scenes */
export function diff(old: SceneIndex, oldScene: IndexableScene, newScene: IndexableScene, renderShape: (s: ShapeDict) => string): DiffResult {
  // Canvas change = full redraw
  if (
    oldScene.canvas.width !== newScene.canvas.width ||
    oldScene.canvas.height !== newScene.canvas.height ||
    oldScene.canvas.fill !== newScene.canvas.fill
  ) {
    return { ops: [{ type: 'full_redraw' }], canvasChanged: true };
  }

  // Fast path: empty scenes
  if (old.elements.length === 0 && newScene.shapes.length === 0) {
    return { ops: [], canvasChanged: false };
  }

  const ops: DiffOp[] = [];
  const matched = new Array(old.elements.length).fill(false);

  // Pass 1: Match new elements to old
  for (let newIdx = 0; newIdx < newScene.shapes.length; newIdx++) {
    const shape = newScene.shapes[newIdx];
    const newId = computeId(newIdx, shape.kind, shape.props as Record<string, unknown>);
    const newIdStr = newId.toString(16);
    const svg = renderShape(shape);
    const newHash = fnv1a(svg);

    const oldIdx = old.idMap.get(newIdStr);

    if (oldIdx !== undefined) {
      matched[oldIdx] = true;
      const oldEl = old.elements[oldIdx];

      // Content changed
      if (oldEl.hash !== newHash) {
        const attrs = diffAttrs(oldScene.shapes[oldIdx], shape);
        ops.push({
          type: 'update',
          id: newIdStr,
          idx: newIdx,
          attrs,
          svg: attrs.length > 3 ? svg : undefined,
        });
      }

      // Position changed
      if (oldIdx !== newIdx) {
        ops.push({ type: 'move', id: newIdStr, from: oldIdx, to: newIdx });
      }
    } else {
      // New element
      ops.push({ type: 'add', id: newIdStr, idx: newIdx, svg });
    }
  }

  // Pass 2: Remove unmatched old elements (reverse for stable indices)
  for (let i = matched.length - 1; i >= 0; i--) {
    if (!matched[i]) {
      const oldEl = old.elements[i];
      ops.push({ type: 'remove', id: oldEl.id.toString(16), idx: i });
    }
  }

  // Check defs changes
  const oldDefs = oldScene.defs.join('');
  const newDefs = newScene.defs.join('');
  if (oldDefs !== newDefs) {
    ops.push({ type: 'update_defs', svg: newDefs });
  }

  return { ops, canvasChanged: false };
}

// ─────────────────────────────────────────────────────────────────────────────
// Attribute Diffing
// ─────────────────────────────────────────────────────────────────────────────

function diffAttrs(old: ShapeDict, newShape: ShapeDict): [string, string][] {
  const changes: [string, string][] = [];

  // Props diff
  for (const [k, v] of Object.entries(newShape.props)) {
    if (JSON.stringify(old.props[k]) !== JSON.stringify(v)) {
      changes.push([propToAttr(k), formatValue(v)]);
    }
  }

  // Style diff
  for (const [k, v] of Object.entries(newShape.style)) {
    if (JSON.stringify(old.style[k]) !== JSON.stringify(v)) {
      changes.push([styleToAttr(k), formatValue(v)]);
    }
  }

  // Transform diff
  if (JSON.stringify(old.transform) !== JSON.stringify(newShape.transform)) {
    changes.push(['transform', buildTransform(newShape.transform)]);
  }

  return changes;
}

function propToAttr(prop: string): string {
  switch (prop) {
    case 'at': return 'x';
    case 'size': return 'width';
    case 'radius': return 'r';
    case 'content': return 'textContent';
    default: return prop;
  }
}

function styleToAttr(style: string): string {
  switch (style) {
    case 'strokeWidth': return 'stroke-width';
    case 'fontSize': return 'font-size';
    case 'fontWeight': return 'font-weight';
    case 'textAnchor': return 'text-anchor';
    default: return style;
  }
}

function formatValue(v: unknown): string {
  if (Array.isArray(v)) return v.join(',');
  return String(v ?? '');
}

function buildTransform(tf: Record<string, unknown>): string {
  const parts: string[] = [];
  if (tf.translate) {
    const [tx, ty] = tf.translate as [number, number];
    parts.push(`translate(${tx},${ty})`);
  }
  if (tf.rotate) {
    parts.push(`rotate(${tf.rotate})`);
  }
  if (tf.scale) {
    const [sx, sy] = tf.scale as [number, number];
    parts.push(`scale(${sx},${sy})`);
  }
  return parts.join(' ');
}

// ─────────────────────────────────────────────────────────────────────────────
// Render Cache
// ─────────────────────────────────────────────────────────────────────────────

interface CacheEntry {
  svg: string;
  hits: number;
}

/** SVG fragment memoization cache */
export class RenderCache {
  private entries = new Map<string, CacheEntry>();
  private maxSize: number;

  constructor(maxSize = 1024) {
    this.maxSize = maxSize;
  }

  get(hash: ContentHash): string | undefined {
    const key = hash.toString(16);
    const entry = this.entries.get(key);
    if (entry) {
      entry.hits++;
      return entry.svg;
    }
    return undefined;
  }

  set(hash: ContentHash, svg: string): void {
    if (this.entries.size >= this.maxSize) this.evictLru();
    this.entries.set(hash.toString(16), { svg, hits: 1 });
  }

  getOrCompute(hash: ContentHash, compute: () => string): string {
    const cached = this.get(hash);
    if (cached !== undefined) return cached;
    const svg = compute();
    this.set(hash, svg);
    return svg;
  }

  private evictLru(): void {
    let minKey: string | undefined;
    let minHits = Infinity;
    for (const [key, entry] of this.entries) {
      if (entry.hits < minHits) {
        minHits = entry.hits;
        minKey = key;
      }
    }
    if (minKey) this.entries.delete(minKey);
  }

  clear(): void {
    this.entries.clear();
  }

  get size(): number {
    return this.entries.size;
  }
}

