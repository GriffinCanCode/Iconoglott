import { describe, it, expect } from 'vitest';
import * as fc from 'fast-check';
import { Parser } from '../parser';
import { Lexer } from '../lexer';
import type { Canvas, Shape } from '../types';

// Helper to parse source
function parse(source: string) {
  const tokens = new Lexer(source).tokenize();
  return new Parser(tokens).parse();
}

// Helper to get shape from AST
function getShape(source: string): Shape {
  const ast = parse(source);
  const node = ast.children.find(n => n.type === 'shape');
  return node?.value as Shape;
}

// Helper to get canvas from AST
function getCanvas(source: string): Canvas {
  const ast = parse(source);
  const node = ast.children.find(n => n.type === 'canvas');
  return node?.value as Canvas;
}

// ─────────────────────────────────────────────────────────────────────────────
// Canvas Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Canvas', () => {
  it('should parse basic canvas', () => {
    const canvas = getCanvas('canvas 800x600');
    expect(canvas.width).toBe(800);
    expect(canvas.height).toBe(600);
  });

  it('should parse canvas with fill', () => {
    const canvas = getCanvas('canvas 400x300 fill #1a1a2e');
    expect(canvas.fill).toBe('#1a1a2e');
  });

  it('should use default dimensions', () => {
    const canvas = getCanvas('canvas');
    expect(canvas.width).toBe(800);
    expect(canvas.height).toBe(600);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Shape Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Shapes', () => {
  describe('rect', () => {
    it('should parse basic rect', () => {
      const shape = getShape('rect at 10,20 size 100x50');
      expect(shape.kind).toBe('rect');
      expect(shape.props.at).toEqual([10, 20]);
      expect(shape.props.size).toEqual([100, 50]);
    });

    it('should parse rect with color', () => {
      const shape = getShape('rect at 0,0 size 50x50 #f00');
      expect(shape.props.fill).toBe('#f00');
    });
  });

  describe('circle', () => {
    it('should parse circle with radius keyword', () => {
      const shape = getShape('circle at 100,100 radius 50');
      expect(shape.kind).toBe('circle');
      expect(shape.props.radius).toBe(50);
    });

    it('should parse circle shorthand', () => {
      const shape = getShape('circle 100,100 50');
      expect(shape.props.at).toEqual([100, 100]);
      expect(shape.props.radius).toBe(50);
    });
  });

  describe('ellipse', () => {
    it('should parse ellipse', () => {
      const shape = getShape('ellipse at 100,100 radius 50,30');
      expect(shape.kind).toBe('ellipse');
      expect(shape.props.radius).toEqual([50, 30]);
    });
  });

  describe('line', () => {
    it('should parse line', () => {
      const shape = getShape('line from 0,0 to 100,100');
      expect(shape.kind).toBe('line');
      expect(shape.props.from).toEqual([0, 0]);
      expect(shape.props.to).toEqual([100, 100]);
    });
  });

  describe('path', () => {
    it('should parse path', () => {
      const shape = getShape('path d "M 0 0 L 100 100"');
      expect(shape.kind).toBe('path');
      expect(shape.props.d).toBe('M 0 0 L 100 100');
    });
  });

  describe('polygon', () => {
    it('should parse polygon', () => {
      const shape = getShape('polygon points [0,0 100,0 50,100]');
      expect(shape.kind).toBe('polygon');
      expect(shape.props.points).toEqual([[0, 0], [100, 0], [50, 100]]);
    });
  });

  describe('text', () => {
    it('should parse text', () => {
      const shape = getShape('text at 10,20 "Hello World"');
      expect(shape.kind).toBe('text');
      expect(shape.props.content).toBe('Hello World');
    });
  });

  describe('image', () => {
    it('should parse image', () => {
      const shape = getShape('image at 0,0 size 100x100 href "test.png"');
      expect(shape.kind).toBe('image');
      expect(shape.props.href).toBe('test.png');
    });
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Style Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Styles', () => {
  it('should parse fill', () => {
    const shape = getShape('rect at 0,0 size 100x50\n  fill #e94560');
    expect(shape.style.fill).toBe('#e94560');
  });

  it('should parse stroke with width', () => {
    const shape = getShape('rect at 0,0 size 100x50\n  stroke #000 2');
    expect(shape.style.stroke).toBe('#000');
    expect(shape.style.strokeWidth).toBe(2);
  });

  it('should parse opacity', () => {
    const shape = getShape('rect at 0,0 size 100x50\n  opacity 0.5');
    expect(shape.style.opacity).toBe(0.5);
  });

  it('should parse corner', () => {
    const shape = getShape('rect at 0,0 size 100x50\n  corner 8');
    expect(shape.style.corner).toBe(8);
  });

  it('should parse combined styles', () => {
    const shape = getShape(`rect at 0,0 size 100x50
  fill #f00
  stroke #000 2
  opacity 0.8
  corner 5`);
    expect(shape.style.fill).toBe('#f00');
    expect(shape.style.stroke).toBe('#000');
    expect(shape.style.strokeWidth).toBe(2);
    expect(shape.style.opacity).toBe(0.8);
    expect(shape.style.corner).toBe(5);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Text Style Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Text Styles', () => {
  it('should parse font', () => {
    const shape = getShape('text at 0,0 "Test"\n  font "Arial" 24');
    expect(shape.style.font).toBe('Arial');
    expect(shape.style.fontSize).toBe(24);
  });

  it('should parse bold', () => {
    const shape = getShape('text at 0,0 "Test"\n  bold');
    expect(shape.style.fontWeight).toBe('bold');
  });

  it('should parse center', () => {
    const shape = getShape('text at 0,0 "Test"\n  center');
    expect(shape.style.textAnchor).toBe('middle');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Transform Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Transforms', () => {
  it('should parse translate', () => {
    const shape = getShape('rect at 0,0 size 50x50\n  translate 10,20');
    expect(shape.transform.translate).toEqual([10, 20]);
  });

  it('should parse rotate', () => {
    const shape = getShape('rect at 0,0 size 50x50\n  rotate 45');
    expect(shape.transform.rotate).toBe(45);
  });

  it('should parse scale pair', () => {
    const shape = getShape('rect at 0,0 size 50x50\n  scale 1.5,2.0');
    expect(shape.transform.scale).toEqual([1.5, 2.0]);
  });

  it('should parse scale single', () => {
    const shape = getShape('rect at 0,0 size 50x50\n  scale 2');
    expect(shape.transform.scale).toEqual([2, 2]);
  });

  it('should parse origin', () => {
    const shape = getShape('rect at 0,0 size 50x50\n  origin 25,25');
    expect(shape.transform.origin).toEqual([25, 25]);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Gradient Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Gradients', () => {
  it('should parse linear gradient', () => {
    const shape = getShape('rect at 0,0 size 100x100\n  gradient linear #f00 #00f');
    expect(shape.style.gradient?.type).toBe('linear');
    expect(shape.style.gradient?.from).toBe('#f00');
    expect(shape.style.gradient?.to).toBe('#00f');
  });

  it('should parse radial gradient', () => {
    const shape = getShape('rect at 0,0 size 100x100\n  gradient radial #fff #000');
    expect(shape.style.gradient?.type).toBe('radial');
  });

  it('should parse gradient with angle', () => {
    const shape = getShape('rect at 0,0 size 100x100\n  gradient linear #f00 #00f 45');
    expect(shape.style.gradient?.angle).toBe(45);
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Shadow Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Shadows', () => {
  it('should parse shadow', () => {
    const shape = getShape('rect at 0,0 size 100x100\n  shadow 2,4 8 #0004');
    expect(shape.style.shadow?.x).toBe(2);
    expect(shape.style.shadow?.y).toBe(4);
    expect(shape.style.shadow?.blur).toBe(8);
    expect(shape.style.shadow?.color).toBe('#0004');
  });

  it('should parse default shadow', () => {
    const shape = getShape('rect at 0,0 size 100x100\n  shadow');
    expect(shape.style.shadow).toBeDefined();
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Variable Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Variables', () => {
  it('should parse variable assignment', () => {
    const ast = parse('$primary = #e94560');
    expect(ast.children[0].type).toBe('variable');
  });

  it('should resolve variable usage', () => {
    const shape = getShape('$color = #f00\nrect $color');
    expect(shape.props.fill).toBe('#f00');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Group Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Groups', () => {
  it('should parse group', () => {
    const shape = getShape('group "my-group"\n  rect at 0,0 size 50x50');
    expect(shape.kind).toBe('group');
    expect(shape.children.length).toBe(1);
  });

  it('should parse stack layout', () => {
    const shape = getShape('stack at 0,0 gap 10\n  rect size 50x30\n  rect size 50x30');
    expect(shape.kind).toBe('layout');
    expect(shape.props.direction).toBe('vertical');
    expect(shape.props.gap).toBe(10);
  });

  it('should parse row layout', () => {
    const shape = getShape('row at 0,0 gap 10\n  rect size 30x50\n  rect size 30x50');
    expect(shape.kind).toBe('layout');
    expect(shape.props.direction).toBe('horizontal');
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Property-based Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser property-based', () => {
  it('should roundtrip canvas dimensions', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 100, max: 2000 }),
        fc.integer({ min: 100, max: 2000 }),
        (w, h) => {
          const canvas = getCanvas(`canvas ${w}x${h}`);
          expect(canvas.width).toBe(w);
          expect(canvas.height).toBe(h);
        }
      ),
      { numRuns: 50 }
    );
  });

  it('should roundtrip rect properties', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: -500, max: 500 }),
        fc.integer({ min: -500, max: 500 }),
        fc.integer({ min: 1, max: 500 }),
        fc.integer({ min: 1, max: 500 }),
        (x, y, w, h) => {
          const shape = getShape(`rect at ${x},${y} size ${w}x${h}`);
          expect(shape.props.at).toEqual([x, y]);
          expect(shape.props.size).toEqual([w, h]);
        }
      ),
      { numRuns: 50 }
    );
  });

  it('should roundtrip circle properties', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: -500, max: 500 }),
        fc.integer({ min: -500, max: 500 }),
        fc.integer({ min: 1, max: 200 }),
        (cx, cy, r) => {
          const shape = getShape(`circle at ${cx},${cy} radius ${r}`);
          expect(shape.props.at).toEqual([cx, cy]);
          expect(shape.props.radius).toBe(r);
        }
      ),
      { numRuns: 50 }
    );
  });

  it('should roundtrip opacity values', () => {
    fc.assert(
      fc.property(
        fc.float({ min: 0, max: 1, noNaN: true }),
        (opacity) => {
          const rounded = Math.round(opacity * 100) / 100;
          const shape = getShape(`rect at 0,0 size 50x50\n  opacity ${rounded}`);
          expect(Math.abs(shape.style.opacity - rounded)).toBeLessThan(0.01);
        }
      ),
      { numRuns: 30 }
    );
  });

  it('should roundtrip rotation angles', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 360 }),
        (angle) => {
          const shape = getShape(`rect at 0,0 size 50x50\n  rotate ${angle}`);
          expect(shape.transform.rotate).toBe(angle);
        }
      ),
      { numRuns: 50 }
    );
  });

  it('should roundtrip polygon points', () => {
    fc.assert(
      fc.property(
        fc.array(
          fc.tuple(
            fc.integer({ min: 0, max: 200 }),
            fc.integer({ min: 0, max: 200 })
          ),
          { minLength: 3, maxLength: 10 }
        ),
        (points) => {
          const pointsStr = points.map(([x, y]) => `${x},${y}`).join(' ');
          const shape = getShape(`polygon points [${pointsStr}]`);
          expect(shape.props.points).toEqual(points);
        }
      ),
      { numRuns: 20 }
    );
  });

  it('should roundtrip text content', () => {
    const safeText = fc.string({ minLength: 1, maxLength: 50 })
      .filter(s => !s.includes('"') && !s.includes('\n'));

    fc.assert(
      fc.property(safeText, (content) => {
        const shape = getShape(`text at 0,0 "${content}"`);
        expect(shape.props.content).toBe(content);
      }),
      { numRuns: 30 }
    );
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Scene Integrity Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Parser - Scene Integrity', () => {
  it('should parse multiple shapes', () => {
    const ast = parse(`canvas 800x600
rect at 0,0 size 100x100
circle at 200,200 radius 50
text at 100,100 "Hello"`);
    expect(ast.children.length).toBe(4); // canvas + 3 shapes
  });

  it('should parse nested styles', () => {
    const shape = getShape(`rect at 0,0 size 100x100
  fill #f00
  stroke #000 2
  opacity 0.8
  corner 10
  shadow 2,4 8 #0004
  gradient linear #f00 #00f
  rotate 45
  translate 10,10`);
    expect(shape.style.fill).toBe('#f00');
    expect(shape.style.stroke).toBe('#000');
    expect(shape.style.shadow).toBeDefined();
    expect(shape.style.gradient).toBeDefined();
    expect(shape.transform.rotate).toBe(45);
    expect(shape.transform.translate).toEqual([10, 10]);
  });
});

