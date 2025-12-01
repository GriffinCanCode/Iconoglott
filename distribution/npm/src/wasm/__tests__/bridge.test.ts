import { describe, it, expect, beforeAll } from 'vitest';
import {
  initWasm,
  isWasmLoaded,
  getWasm,
  tryGetWasm,
  WasmCore,
  WasmStyle,
  WasmSceneInput,
} from '../bridge';

const defaultStyle: WasmStyle = {
  fill: '#ff0000',
  stroke: '#000000',
  stroke_width: 1,
  opacity: 1,
  corner: 0,
};

describe('WASM Bridge', () => {
  let wasm: WasmCore;

  beforeAll(async () => {
    wasm = await initWasm();
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Module Initialization
  // ─────────────────────────────────────────────────────────────────────────
  describe('Module Initialization', () => {
    it('initializes WASM module', async () => {
      const result = await initWasm();
      expect(result).toBeDefined();
    });

    it('returns cached module on subsequent calls', async () => {
      const first = await initWasm();
      const second = await initWasm();
      expect(first).toBe(second);
    });

    it('isWasmLoaded returns true after init', () => {
      expect(isWasmLoaded()).toBe(true);
    });

    it('getWasm returns module after init', () => {
      expect(getWasm()).toBeDefined();
    });

    it('tryGetWasm returns module after init', () => {
      expect(tryGetWasm()).toBeDefined();
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // DSL Processing
  // ─────────────────────────────────────────────────────────────────────────
  describe('DSL Processing', () => {
    it('tokenize returns JSON tokens', () => {
      const result = wasm.tokenize('canvas medium');
      expect(result).toContain('canvas');
      const parsed = JSON.parse(result);
      expect(Array.isArray(parsed)).toBe(true);
    });

    it('parse returns JSON AST', () => {
      const result = wasm.parse('canvas medium\nrect 10 10 50 50');
      const ast = JSON.parse(result);
      expect(ast).toBeDefined();
    });

    it('parse_with_errors returns ast and errors', () => {
      const result = wasm.parse_with_errors('canvas medium');
      const parsed = JSON.parse(result);
      expect(parsed).toHaveProperty('ast');
      expect(parsed).toHaveProperty('errors');
    });

    it('parse_with_errors captures syntax errors', () => {
      const result = wasm.parse_with_errors('canvas !!!invalid');
      const parsed = JSON.parse(result);
      expect(parsed.errors.length).toBeGreaterThan(0);
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Canvas Size System
  // ─────────────────────────────────────────────────────────────────────────
  describe('Canvas Size System', () => {
    it('size_to_pixels returns dimensions for valid size', () => {
      const result = wasm.size_to_pixels('medium');
      expect(result).toEqual([64, 64]);
    });

    it('size_to_pixels returns null for invalid size', () => {
      const result = wasm.size_to_pixels('nonexistent');
      expect(result).toBeNull();
    });

    it('is_valid_size returns true for valid sizes', () => {
      expect(wasm.is_valid_size('nano')).toBe(true);
      expect(wasm.is_valid_size('micro')).toBe(true);
      expect(wasm.is_valid_size('tiny')).toBe(true);
      expect(wasm.is_valid_size('small')).toBe(true);
      expect(wasm.is_valid_size('medium')).toBe(true);
      expect(wasm.is_valid_size('large')).toBe(true);
      expect(wasm.is_valid_size('xlarge')).toBe(true);
      expect(wasm.is_valid_size('huge')).toBe(true);
      expect(wasm.is_valid_size('massive')).toBe(true);
      expect(wasm.is_valid_size('giant')).toBe(true);
    });

    it('is_valid_size returns false for invalid sizes', () => {
      expect(wasm.is_valid_size('invalid')).toBe(false);
      expect(wasm.is_valid_size('')).toBe(false);
    });

    it('get_all_sizes returns array of size names', () => {
      const sizes = wasm.get_all_sizes();
      expect(Array.isArray(sizes)).toBe(true);
      expect(sizes).toContain('medium');
      expect(sizes.length).toBe(10);
    });

    it('get_size_info returns size details', () => {
      const info = wasm.get_size_info('large');
      expect(info).toEqual({ name: 'large', width: 96, height: 96 });
    });

    it('get_size_info returns null for invalid size', () => {
      const info = wasm.get_size_info('invalid');
      expect(info).toBeNull();
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Hashing
  // ─────────────────────────────────────────────────────────────────────────
  describe('Hashing', () => {
    it('fnv1a_hash returns consistent hash', () => {
      const hash1 = wasm.fnv1a_hash('test');
      const hash2 = wasm.fnv1a_hash('test');
      expect(hash1).toBe(hash2);
    });

    it('fnv1a_hash returns different hashes for different inputs', () => {
      const hash1 = wasm.fnv1a_hash('test1');
      const hash2 = wasm.fnv1a_hash('test2');
      expect(hash1).not.toBe(hash2);
    });

    it('compute_element_id generates stable IDs', () => {
      const id1 = wasm.compute_element_id(0, 'rect', { x: 10, y: 20 });
      const id2 = wasm.compute_element_id(0, 'rect', { x: 10, y: 20 });
      expect(id1).toBe(id2);
    });

    it('compute_element_id differs by order', () => {
      const id1 = wasm.compute_element_id(0, 'rect', { x: 10 });
      const id2 = wasm.compute_element_id(1, 'rect', { x: 10 });
      expect(id1).not.toBe(id2);
    });

    it('compute_element_id differs by kind', () => {
      const id1 = wasm.compute_element_id(0, 'rect', { x: 10 });
      const id2 = wasm.compute_element_id(0, 'circle', { x: 10 });
      expect(id1).not.toBe(id2);
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Shape Rendering
  // ─────────────────────────────────────────────────────────────────────────
  describe('Shape Rendering', () => {
    it('render_rect produces valid SVG rect', () => {
      const svg = wasm.render_rect(10, 20, 100, 50, 5, defaultStyle);
      expect(svg).toContain('<rect');
      expect(svg).toContain('x="10"');
      expect(svg).toContain('y="20"');
      expect(svg).toContain('width="100"');
      expect(svg).toContain('height="50"');
      expect(svg).toContain('rx="5"');
    });

    it('render_rect applies transform', () => {
      const svg = wasm.render_rect(0, 0, 10, 10, 0, defaultStyle, 'rotate(45)');
      expect(svg).toContain('transform="rotate(45)"');
    });

    it('render_circle produces valid SVG circle', () => {
      const svg = wasm.render_circle(50, 50, 25, defaultStyle);
      expect(svg).toContain('<circle');
      expect(svg).toContain('cx="50"');
      expect(svg).toContain('cy="50"');
      expect(svg).toContain('r="25"');
    });

    it('render_ellipse produces valid SVG ellipse', () => {
      const svg = wasm.render_ellipse(50, 50, 30, 20, defaultStyle);
      expect(svg).toContain('<ellipse');
      expect(svg).toContain('cx="50"');
      expect(svg).toContain('cy="50"');
      expect(svg).toContain('rx="30"');
      expect(svg).toContain('ry="20"');
    });

    it('render_line produces valid SVG line', () => {
      const svg = wasm.render_line(0, 0, 100, 100, '#000', 2);
      expect(svg).toContain('<line');
      expect(svg).toContain('x1="0"');
      expect(svg).toContain('y1="0"');
      expect(svg).toContain('x2="100"');
      expect(svg).toContain('y2="100"');
    });

    it('render_path produces valid SVG path', () => {
      const svg = wasm.render_path('M0 0 L100 100', defaultStyle);
      expect(svg).toContain('<path');
      expect(svg).toContain('d="M0 0 L100 100"');
    });

    it('render_polygon produces valid SVG polygon', () => {
      const points: [number, number][] = [[0, 0], [50, 0], [25, 50]];
      const svg = wasm.render_polygon(points, defaultStyle);
      expect(svg).toContain('<polygon');
      expect(svg).toContain('points=');
    });

    it('render_text produces valid SVG text', () => {
      const svg = wasm.render_text(10, 20, 'Hello', 'Arial', 16, 'normal', 'start', '#000');
      expect(svg).toContain('<text');
      expect(svg).toContain('Hello');
      expect(svg).toContain('x="10"');
      expect(svg).toContain('y="20"');
    });

    it('render_image produces valid SVG image', () => {
      const svg = wasm.render_image(0, 0, 100, 100, 'data:image/png;base64,abc');
      expect(svg).toContain('<image');
      expect(svg).toContain('href="data:image/png;base64,abc"');
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Graph/Flowchart Rendering
  // ─────────────────────────────────────────────────────────────────────────
  describe('Graph/Flowchart Rendering', () => {
    it('render_diamond produces valid diamond shape', () => {
      const svg = wasm.render_diamond(50, 50, 40, 30, defaultStyle);
      expect(svg).toContain('<polygon');
    });

    it('render_node produces node with label', () => {
      const svg = wasm.render_node('node1', 'rect', 50, 50, 80, 40, 'Label', defaultStyle);
      expect(svg).toContain('<g');
      expect(svg).toContain('Label');
    });

    it('render_node works without label', () => {
      const svg = wasm.render_node('node2', 'rect', 50, 50, 80, 40, null, defaultStyle);
      expect(svg).toContain('<g');
    });

    it('render_edge produces edge SVG', () => {
      const svg = wasm.render_edge(0, 0, 100, 100, 'solid', 'end', null, '#000', 2);
      expect(svg).toContain('<path');
    });

    it('render_edge with label', () => {
      const svg = wasm.render_edge(0, 0, 100, 100, 'solid', 'end', 'connects', '#000', 2);
      expect(svg).toContain('connects');
    });

    it('render_arrow_markers produces defs content', () => {
      const svg = wasm.render_arrow_markers('#000');
      expect(svg).toContain('<marker');
    });

    it('compute_edge_anchors returns anchor points', () => {
      const anchors = wasm.compute_edge_anchors(50, 50, 80, 40, 150, 50, 80, 40);
      expect(anchors).toHaveProperty('from');
      expect(anchors).toHaveProperty('to');
      expect(anchors.from).toHaveLength(2);
      expect(anchors.to).toHaveLength(2);
    });

    it('layout_hierarchical positions nodes', () => {
      const nodes = [
        { id: 'a', w: 80, h: 40 },
        { id: 'b', w: 80, h: 40 },
      ];
      const result = wasm.layout_hierarchical(nodes, 'TB', 50);
      expect(Array.isArray(result)).toBe(true);
      expect(result).toHaveLength(2);
      expect(result[0]).toHaveProperty('id');
      expect(result[0]).toHaveProperty('cx');
      expect(result[0]).toHaveProperty('cy');
    });

    it('layout_grid positions nodes', () => {
      const nodes = [
        { id: 'a', w: 80, h: 40 },
        { id: 'b', w: 80, h: 40 },
        { id: 'c', w: 80, h: 40 },
      ];
      const result = wasm.layout_grid(nodes, 20);
      expect(Array.isArray(result)).toBe(true);
      expect(result).toHaveLength(3);
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Definitions (Gradients & Filters)
  // ─────────────────────────────────────────────────────────────────────────
  describe('Definitions', () => {
    it('render_linear_gradient produces valid gradient', () => {
      const svg = wasm.render_linear_gradient('grad1', '#ff0000', '#0000ff', 45);
      expect(svg).toContain('<linearGradient');
      expect(svg).toContain('id="grad1"');
    });

    it('render_radial_gradient produces valid gradient', () => {
      const svg = wasm.render_radial_gradient('rgrad1', '#ff0000', '#0000ff');
      expect(svg).toContain('<radialGradient');
      expect(svg).toContain('id="rgrad1"');
    });

    it('render_shadow_filter produces valid filter', () => {
      const svg = wasm.render_shadow_filter('shadow1', 2, 2, 4, 'rgba(0,0,0,0.5)');
      expect(svg).toContain('<filter');
      expect(svg).toContain('id="shadow1"');
    });

    it('render_blur_filter produces valid filter', () => {
      const svg = wasm.render_blur_filter('blur1', 5);
      expect(svg).toContain('<filter');
      expect(svg).toContain('id="blur1"');
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Scene Management
  // ─────────────────────────────────────────────────────────────────────────
  describe('Scene Management', () => {
    it('render_scene produces complete SVG', () => {
      const svg = wasm.render_scene('medium', '#ffffff', '', '<rect x="0" y="0" width="10" height="10"/>');
      expect(svg).toContain('<svg');
      expect(svg).toContain('</svg>');
      expect(svg).toContain('width="64"');
      expect(svg).toContain('height="64"');
    });

    it('render_scene includes defs', () => {
      const defs = wasm.render_linear_gradient('g1', '#f00', '#00f', 0);
      const svg = wasm.render_scene('small', '#fff', defs, '');
      expect(svg).toContain('<defs>');
      expect(svg).toContain('linearGradient');
    });

    it('scenes_equal returns true for identical scenes', () => {
      const scene: WasmSceneInput = {
        canvas: { size: 'medium', fill: '#fff' },
        elements: [{ id: 'e1', kind: 'rect', svg: '<rect/>' }],
        defs: '',
      };
      expect(wasm.scenes_equal(scene, scene)).toBe(true);
    });

    it('scenes_equal returns false for different scenes', () => {
      const scene1: WasmSceneInput = {
        canvas: { size: 'medium', fill: '#fff' },
        elements: [{ id: 'e1', kind: 'rect', svg: '<rect/>' }],
        defs: '',
      };
      const scene2: WasmSceneInput = {
        canvas: { size: 'medium', fill: '#fff' },
        elements: [{ id: 'e2', kind: 'circle', svg: '<circle/>' }],
        defs: '',
      };
      expect(wasm.scenes_equal(scene1, scene2)).toBe(false);
    });

    it('diff_scenes returns ops for changes', () => {
      const oldScene: WasmSceneInput = {
        canvas: { size: 'medium', fill: '#fff' },
        elements: [],
        defs: '',
      };
      const newScene: WasmSceneInput = {
        canvas: { size: 'medium', fill: '#fff' },
        elements: [{ id: 'e1', kind: 'rect', svg: '<rect/>' }],
        defs: '',
      };
      const result = wasm.diff_scenes(oldScene, newScene);
      expect(result).toHaveProperty('ops');
      expect(result).toHaveProperty('canvas_changed');
      expect(Array.isArray(result.ops)).toBe(true);
    });

    it('diff_scenes detects canvas change', () => {
      const oldScene: WasmSceneInput = {
        canvas: { size: 'medium', fill: '#fff' },
        elements: [],
        defs: '',
      };
      const newScene: WasmSceneInput = {
        canvas: { size: 'large', fill: '#fff' },
        elements: [],
        defs: '',
      };
      const result = wasm.diff_scenes(oldScene, newScene);
      expect(result.canvas_changed).toBe(true);
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Path Utilities
  // ─────────────────────────────────────────────────────────────────────────
  describe('Path Utilities', () => {
    it('compute_path_bounds returns bounding box', () => {
      const bounds = wasm.compute_path_bounds('M0 0 L100 100');
      expect(bounds).toHaveLength(4);
      const [x, y, w, h] = bounds;
      expect(x).toBe(0);
      expect(y).toBe(0);
      expect(w).toBe(100);
      expect(h).toBe(100);
    });

    it('compute_path_bounds handles complex paths', () => {
      const bounds = wasm.compute_path_bounds('M10 10 L50 10 L50 50 L10 50 Z');
      expect(bounds).toHaveLength(4);
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Text Metrics
  // ─────────────────────────────────────────────────────────────────────────
  describe('Text Metrics', () => {
    it('measure_text returns dimensions', () => {
      const metrics = wasm.measure_text('Hello World', 'Arial', 16);
      expect(metrics).toHaveProperty('width');
      expect(metrics).toHaveProperty('height');
      expect(metrics).toHaveProperty('ascender');
      expect(metrics).toHaveProperty('descender');
      expect(metrics.width).toBeGreaterThan(0);
      expect(metrics.height).toBeGreaterThan(0);
    });

    it('measure_text scales with font size', () => {
      const small = wasm.measure_text('Test', 'Arial', 12);
      const large = wasm.measure_text('Test', 'Arial', 24);
      expect(large.width).toBeGreaterThan(small.width);
      expect(large.height).toBeGreaterThan(small.height);
    });

    it('compute_text_bounds returns position and size', () => {
      const bounds = wasm.compute_text_bounds(10, 20, 'Hello', 'Arial', 16, 'start');
      expect(bounds).toHaveLength(4);
      const [x, y, w, h] = bounds;
      expect(typeof x).toBe('number');
      expect(typeof y).toBe('number');
      expect(w).toBeGreaterThan(0);
      expect(h).toBeGreaterThan(0);
    });

    it('compute_text_bounds respects anchor', () => {
      const start = wasm.compute_text_bounds(100, 50, 'Test', 'Arial', 16, 'start');
      const middle = wasm.compute_text_bounds(100, 50, 'Test', 'Arial', 16, 'middle');
      const end = wasm.compute_text_bounds(100, 50, 'Test', 'Arial', 16, 'end');
      // Start anchor: x should be at or near 100
      // Middle anchor: x should be less (shifted left)
      // End anchor: x should be even less (shifted more left)
      expect(start[0]).toBeGreaterThanOrEqual(middle[0]);
      expect(middle[0]).toBeGreaterThanOrEqual(end[0]);
    });
  });

  // ─────────────────────────────────────────────────────────────────────────
  // Style Application
  // ─────────────────────────────────────────────────────────────────────────
  describe('Style Application', () => {
    it('applies fill style', () => {
      const style: WasmStyle = { ...defaultStyle, fill: '#00ff00' };
      const svg = wasm.render_rect(0, 0, 10, 10, 0, style);
      expect(svg).toContain('fill="#00ff00"');
    });

    it('applies stroke style', () => {
      const style: WasmStyle = { ...defaultStyle, stroke: '#0000ff', stroke_width: 3 };
      const svg = wasm.render_rect(0, 0, 10, 10, 0, style);
      expect(svg).toContain('stroke="#0000ff"');
      expect(svg).toContain('stroke-width="3"');
    });

    it('applies opacity', () => {
      const style: WasmStyle = { ...defaultStyle, opacity: 0.5 };
      const svg = wasm.render_rect(0, 0, 10, 10, 0, style);
      expect(svg).toContain('opacity="0.5"');
    });

    it('applies filter reference', () => {
      const style: WasmStyle = { ...defaultStyle, filter: 'shadow1' };
      const svg = wasm.render_rect(0, 0, 10, 10, 0, style);
      expect(svg).toContain('filter="url(#shadow1)"');
    });

    it('applies gradient fill reference', () => {
      const style: WasmStyle = { ...defaultStyle, fill: 'url(#grad1)' };
      const svg = wasm.render_rect(0, 0, 10, 10, 0, style);
      expect(svg).toContain('fill="url(#grad1)"');
    });
  });
});

