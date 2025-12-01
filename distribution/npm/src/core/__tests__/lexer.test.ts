import { describe, it, expect } from 'vitest';
import * as fc from 'fast-check';
import { Lexer } from '../lexer';

// ─────────────────────────────────────────────────────────────────────────────
// Basic Lexer Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Lexer', () => {
  describe('basics', () => {
    it('should tokenize empty input', () => {
      const tokens = [...new Lexer('').tokenize()];
      expect(tokens[tokens.length - 1].type).toBe('EOF');
    });

    it('should skip comments', () => {
      const tokens = [...new Lexer('// comment').tokenize()];
      expect(tokens[tokens.length - 1].type).toBe('EOF');
    });

    it('should skip whitespace-only lines', () => {
      const tokens = [...new Lexer('   \t  ').tokenize()];
      expect(tokens[tokens.length - 1].type).toBe('EOF');
    });
  });

  describe('identifiers', () => {
    it('should tokenize simple identifier', () => {
      const tokens = [...new Lexer('canvas').tokenize()];
      expect(tokens[0].type).toBe('IDENT');
      expect(tokens[0].value).toBe('canvas');
    });

    it('should tokenize identifier with numbers', () => {
      const tokens = [...new Lexer('layer1').tokenize()];
      expect(tokens[0].type).toBe('IDENT');
      expect(tokens[0].value).toBe('layer1');
    });

    it('should tokenize identifier with hyphen', () => {
      const tokens = [...new Lexer('my-shape').tokenize()];
      expect(tokens[0].type).toBe('IDENT');
      expect(tokens[0].value).toBe('my-shape');
    });
  });

  describe('numbers', () => {
    it('should tokenize integer', () => {
      const tokens = [...new Lexer('100').tokenize()];
      expect(tokens[0].type).toBe('NUMBER');
      expect(tokens[0].value).toBe(100);
    });

    it('should tokenize negative integer', () => {
      const tokens = [...new Lexer('-50').tokenize()];
      expect(tokens[0].type).toBe('NUMBER');
      expect(tokens[0].value).toBe(-50);
    });

    it('should tokenize float', () => {
      const tokens = [...new Lexer('3.14').tokenize()];
      expect(tokens[0].type).toBe('NUMBER');
      expect(tokens[0].value).toBe(3.14);
    });
  });

  describe('pairs', () => {
    it('should tokenize comma-separated pair', () => {
      const tokens = [...new Lexer('100,200').tokenize()];
      expect(tokens[0].type).toBe('PAIR');
      expect(tokens[0].value).toEqual([100, 200]);
    });

    it('should tokenize x-separated pair', () => {
      const tokens = [...new Lexer('800x600').tokenize()];
      expect(tokens[0].type).toBe('PAIR');
      expect(tokens[0].value).toEqual([800, 600]);
    });

    it('should tokenize float pairs', () => {
      const tokens = [...new Lexer('10.5,20.5').tokenize()];
      expect(tokens[0].type).toBe('PAIR');
      expect(tokens[0].value).toEqual([10.5, 20.5]);
    });

    it('should tokenize negative pairs', () => {
      const tokens = [...new Lexer('-10,20').tokenize()];
      expect(tokens[0].type).toBe('PAIR');
      expect(tokens[0].value).toEqual([-10, 20]);
    });
  });

  describe('colors', () => {
    it('should tokenize short hex', () => {
      const tokens = [...new Lexer('#fff').tokenize()];
      expect(tokens[0].type).toBe('COLOR');
      expect(tokens[0].value).toBe('#fff');
    });

    it('should tokenize long hex', () => {
      const tokens = [...new Lexer('#e94560').tokenize()];
      expect(tokens[0].type).toBe('COLOR');
      expect(tokens[0].value).toBe('#e94560');
    });

    it('should tokenize hex with alpha', () => {
      const tokens = [...new Lexer('#00000080').tokenize()];
      expect(tokens[0].type).toBe('COLOR');
      expect(tokens[0].value).toBe('#00000080');
    });
  });

  describe('strings', () => {
    it('should tokenize double-quoted string', () => {
      const tokens = [...new Lexer('"Hello World"').tokenize()];
      expect(tokens[0].type).toBe('STRING');
      expect(tokens[0].value).toBe('Hello World');
    });

    it('should tokenize single-quoted string', () => {
      const tokens = [...new Lexer("'Hello World'").tokenize()];
      expect(tokens[0].type).toBe('STRING');
      expect(tokens[0].value).toBe('Hello World');
    });

    it('should tokenize empty string', () => {
      const tokens = [...new Lexer('""').tokenize()];
      expect(tokens[0].type).toBe('STRING');
      expect(tokens[0].value).toBe('');
    });
  });

  describe('variables', () => {
    it('should tokenize variable', () => {
      const tokens = [...new Lexer('$primary').tokenize()];
      expect(tokens[0].type).toBe('VAR');
      expect(tokens[0].value).toBe('$primary');
    });
  });

  describe('operators', () => {
    it('should tokenize equals', () => {
      const tokens = [...new Lexer('=').tokenize()];
      expect(tokens[0].type).toBe('EQUALS');
    });

    it('should tokenize brackets', () => {
      const tokens = [...new Lexer('[]').tokenize()];
      expect(tokens[0].type).toBe('LBRACKET');
      expect(tokens[1].type).toBe('RBRACKET');
    });

    it('should tokenize arrow', () => {
      const tokens = [...new Lexer('->').tokenize()];
      expect(tokens[0].type).toBe('ARROW');
    });
  });

  describe('indentation', () => {
    it('should emit INDENT on increase', () => {
      const tokens = [...new Lexer('parent\n  child').tokenize()];
      const types = tokens.map(t => t.type);
      expect(types).toContain('INDENT');
    });

    it('should emit DEDENT on decrease', () => {
      const tokens = [...new Lexer('parent\n  child\nsibling').tokenize()];
      const types = tokens.map(t => t.type);
      expect(types).toContain('DEDENT');
    });

    it('should handle multiple indent levels', () => {
      const tokens = [...new Lexer('l0\n  l1\n    l2\nback').tokenize()];
      const types = tokens.map(t => t.type);
      const indents = types.filter(t => t === 'INDENT').length;
      const dedents = types.filter(t => t === 'DEDENT').length;
      expect(indents).toBe(2);
      expect(dedents).toBe(2);
    });
  });

  describe('complex input', () => {
    it('should tokenize canvas with size', () => {
      const tokens = [...new Lexer('canvas 800x600').tokenize()];
      expect(tokens[0].type).toBe('IDENT');
      expect(tokens[0].value).toBe('canvas');
      expect(tokens[1].type).toBe('PAIR');
      expect(tokens[1].value).toEqual([800, 600]);
    });

    it('should tokenize variable assignment', () => {
      const tokens = [...new Lexer('$primary = #e94560').tokenize()];
      expect(tokens[0].type).toBe('VAR');
      expect(tokens[1].type).toBe('EQUALS');
      expect(tokens[2].type).toBe('COLOR');
    });

    it('should tokenize shape with properties', () => {
      const tokens = [...new Lexer('rect at 10,20 size 100x50').tokenize()];
      const types = tokens.map(t => t.type);
      expect(types[0]).toBe('IDENT');
      expect(types[1]).toBe('IDENT');
      expect(types[2]).toBe('PAIR');
      expect(types[3]).toBe('IDENT');
      expect(types[4]).toBe('PAIR');
    });
  });
});

// ─────────────────────────────────────────────────────────────────────────────
// Property-based Tests
// ─────────────────────────────────────────────────────────────────────────────

describe('Lexer property-based', () => {
  it('should roundtrip integers', () => {
    fc.assert(
      fc.property(fc.integer({ min: -10000, max: 10000 }), (n) => {
        const tokens = [...new Lexer(String(n)).tokenize()];
        expect(tokens[0].type).toBe('NUMBER');
        expect(tokens[0].value).toBe(n);
      }),
      { numRuns: 100 }
    );
  });

  it('should roundtrip positive integer pairs', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 1000 }),
        fc.integer({ min: 0, max: 1000 }),
        (x, y) => {
          const tokens = [...new Lexer(`${x},${y}`).tokenize()];
          expect(tokens[0].type).toBe('PAIR');
          expect(tokens[0].value).toEqual([x, y]);
        }
      ),
      { numRuns: 100 }
    );
  });

  it('should tokenize valid identifiers', () => {
    const identArb = fc.stringOf(
      fc.constantFrom(...'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_'),
      { minLength: 1, maxLength: 20 }
    ).filter(s => /^[a-zA-Z_]/.test(s));

    fc.assert(
      fc.property(identArb, (ident) => {
        const tokens = [...new Lexer(ident).tokenize()];
        expect(tokens[0].type).toBe('IDENT');
        expect(tokens[0].value).toBe(ident);
      }),
      { numRuns: 50 }
    );
  });

  it('should tokenize valid short hex colors', () => {
    const hexDigit = fc.constantFrom(...'0123456789abcdefABCDEF');
    const shortHex = fc.tuple(hexDigit, hexDigit, hexDigit)
      .map(([a, b, c]) => `#${a}${b}${c}`);

    fc.assert(
      fc.property(shortHex, (color) => {
        const tokens = [...new Lexer(color).tokenize()];
        expect(tokens[0].type).toBe('COLOR');
        expect(tokens[0].value).toBe(color);
      }),
      { numRuns: 50 }
    );
  });

  it('should tokenize valid long hex colors', () => {
    const hexDigit = fc.constantFrom(...'0123456789abcdefABCDEF');
    const longHex = fc.tuple(hexDigit, hexDigit, hexDigit, hexDigit, hexDigit, hexDigit)
      .map(digits => `#${digits.join('')}`);

    fc.assert(
      fc.property(longHex, (color) => {
        const tokens = [...new Lexer(color).tokenize()];
        expect(tokens[0].type).toBe('COLOR');
        expect(tokens[0].value).toBe(color);
      }),
      { numRuns: 50 }
    );
  });

  it('should preserve string content', () => {
    const safeString = fc.string({ minLength: 0, maxLength: 50 })
      .filter(s => !s.includes('"') && !s.includes('\n'));

    fc.assert(
      fc.property(safeString, (content) => {
        const tokens = [...new Lexer(`"${content}"`).tokenize()];
        expect(tokens[0].type).toBe('STRING');
        expect(tokens[0].value).toBe(content);
      }),
      { numRuns: 50 }
    );
  });
});

