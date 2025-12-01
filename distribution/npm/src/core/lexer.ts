import type { Token, TokenType } from './types';

type PatternEntry = [RegExp, TokenType | null];

const PATTERNS: PatternEntry[] = [
  [/^\/\/[^\n]*/, null], // Comments (skip)
  [/^\$[a-zA-Z_][a-zA-Z0-9_]*/, 'VAR'],
  [/^#[0-9a-fA-F]{3,8}\b/, 'COLOR'],
  [/^-?\d+\.?\d*[,x]-?\d+\.?\d*/, 'PAIR'],
  [/^"[^"]*"/, 'STRING'],
  [/^'[^']*'/, 'STRING'],
  [/^-?\d+\.?\d*/, 'NUMBER'],
  [/^\[/, 'LBRACKET'],
  [/^\]/, 'RBRACKET'],
  [/^->/, 'ARROW'],
  [/^:/, 'COLON'],
  [/^=/, 'EQUALS'],
  [/^[a-zA-Z_][a-zA-Z0-9_-]*/, 'IDENT'],
];

export class Lexer {
  private lines: string[];
  private indentStack: number[] = [0];

  constructor(source: string) {
    this.lines = source.split('\n');
  }

  *tokenize(): Generator<Token> {
    for (let lineno = 0; lineno < this.lines.length; lineno++) {
      const line = this.lines[lineno];
      const stripped = line.trimStart();

      if (!stripped || stripped.startsWith('//')) continue;

      const indent = line.length - stripped.length;
      yield* this.handleIndent(indent, lineno);
      yield* this.tokenizeLine(stripped, lineno);
      yield { type: 'NEWLINE', value: '\n', line: lineno, col: line.length };
    }

    while (this.indentStack.length > 1) {
      this.indentStack.pop();
      yield { type: 'DEDENT', value: '', line: this.lines.length - 1, col: 0 };
    }

    yield { type: 'EOF', value: '', line: this.lines.length - 1, col: 0 };
  }

  private *handleIndent(indent: number, line: number): Generator<Token> {
    const current = this.indentStack[this.indentStack.length - 1];
    if (indent > current) {
      this.indentStack.push(indent);
      yield { type: 'INDENT', value: '', line, col: 0 };
    } else {
      while (indent < this.indentStack[this.indentStack.length - 1]) {
        this.indentStack.pop();
        yield { type: 'DEDENT', value: '', line, col: 0 };
      }
    }
  }

  private *tokenizeLine(line: string, lineno: number): Generator<Token> {
    let pos = 0;
    while (pos < line.length) {
      if (/\s/.test(line[pos])) { pos++; continue; }

      let matched = false;
      for (const [pattern, ttype] of PATTERNS) {
        const match = line.slice(pos).match(pattern);
        if (match) {
          if (ttype !== null) {
            yield { type: ttype, value: this.parseValue(match[0], ttype), line: lineno, col: pos };
          }
          pos += match[0].length;
          matched = true;
          break;
        }
      }
      if (!matched) pos++;
    }
  }

  private parseValue(raw: string, ttype: TokenType): string | number | [number, number] {
    switch (ttype) {
      case 'NUMBER':
        return raw.includes('.') ? parseFloat(raw) : parseInt(raw, 10);
      case 'STRING':
        return raw.slice(1, -1);
      case 'PAIR': {
        const sep = raw.includes('x') ? 'x' : ',';
        const [a, b] = raw.split(sep);
        return [parseFloat(a), parseFloat(b)];
      }
      default:
        return raw;
    }
  }
}

