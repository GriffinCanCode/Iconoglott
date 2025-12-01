/**
 * Iconoglott Playground
 * Real-time DSL rendering using @iconoglott/renderer WebSocket client
 */

import { createClient } from '@iconoglott/renderer/client';
import type { ParseError } from '@iconoglott/renderer';

// Semantic classification for syntax highlighting (presentation layer only)
const KEYWORDS = new Set(['canvas', 'group', 'stack', 'row', 'graph', 'node', 'edge']);
const SHAPES = new Set(['rect', 'circle', 'ellipse', 'line', 'path', 'polygon', 'text', 'image', 'arc', 'curve', 'diamond']);
const PROPERTIES = new Set([
  'at', 'size', 'radius', 'from', 'to', 'fill', 'stroke', 'opacity', 'corner',
  'shadow', 'gradient', 'blur', 'font', 'bold', 'italic', 'center', 'end', 'middle',
  'translate', 'rotate', 'scale', 'origin', 'width', 'height', 'gap', 'vertical', 'horizontal',
  'linear', 'radial', 'd', 'points', 'href', 'label', 'shape', 'spacing', 'curved', 'straight',
  'orthogonal', 'hierarchical', 'force', 'grid', 'tree', 'manual', 'justify', 'align', 'wrap',
  'start', 'smooth', 'sharp', 'closed', 'padding', 'anchor', 'auto'
]);

const DEFAULT_CODE = `// Iconoglott — Visual DSL for Vector Graphics
// Flowchart demo using manual node positioning

canvas giant fill #0a0f1a

// Color palette
$accent = #10b981
$blue = #3b82f6
$purple = #8b5cf6
$amber = #f59e0b
$cyan = #06b6d4
$text = #e2e8f0

// ═══════════════════════════════════════════════
// TITLE
// ═══════════════════════════════════════════════

text "Iconoglott" at 20,30
  font "Space Grotesk" 22
  fill #fff
  bold

text "DSL → SVG Pipeline" at 20,50
  font "Space Grotesk" 11
  fill #64748b

// ═══════════════════════════════════════════════
// FLOWCHART - Manual layout with exact positions
// ═══════════════════════════════════════════════

graph manual
  node "Input" at 256,90 size 80x32
    shape rect
    fill $blue
    label "DSL"
  
  node "Lexer" at 256,155 size 80x32
    shape rect
    fill $purple
    label "Lexer"
  
  node "Parser" at 256,220 size 80x32
    shape rect
    fill $purple
    label "Parser"
  
  node "AST" at 256,290 size 80x36
    shape diamond
    fill $accent
    label "AST"
  
  node "Render" at 256,365 size 80x32
    shape rect
    fill $amber
    label "Render"
  
  node "SVG" at 256,430 size 80x32
    shape rect
    fill $cyan
    label "SVG"
  
  edge "Input" -> "Lexer" curved $text 1.5
  edge "Lexer" -> "Parser" curved $text 1.5
  edge "Parser" -> "AST" curved $text 1.5
  edge "AST" -> "Render" curved $text 1.5
  edge "Render" -> "SVG" curved $text 1.5

// Decorative accents
circle at 450,60 radius 18
  gradient radial #10b981 #0a0f1a
  opacity 0.6

circle at 470,80 radius 10
  gradient radial #3b82f6 #0a0f1a
  opacity 0.4`;

class IconoglottPlayground {
  private editor: HTMLTextAreaElement;
  private highlight: HTMLElement;
  private canvas: HTMLElement;
  private status: HTMLElement;
  private lineNumbers: HTMLElement;
  private errorPanel: HTMLElement;
  private errorCount: HTMLElement;
  private errors: ParseError[] = [];

  constructor() {
    this.editor = document.getElementById('editor') as HTMLTextAreaElement;
    this.highlight = document.getElementById('highlight')!;
    this.canvas = document.getElementById('canvas')!;
    this.status = document.getElementById('status')!;
    this.lineNumbers = document.getElementById('lineNumbers')!;
    this.errorPanel = document.getElementById('errorPanel')!;
    this.errorCount = document.getElementById('errorCount')!;
    this.init();
  }

  private init() {
    // Set up WebSocket client using the package
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const client = createClient({
      url: `${proto}//${location.host}/ws`,
      debounce: 80,
      onRender: (svg, errors) => {
        this.canvas.innerHTML = svg;
        this.errors = errors;
        this.updateErrors();
        this.updateLineNumbers();
      },
      onError: (message, errors) => {
        console.error('Render error:', message);
        this.errors = errors;
        this.updateErrors();
        this.updateLineNumbers();
      },
      onConnectionChange: (connected) => {
        this.status.classList.toggle('connected', connected);
        if (connected && this.editor.value) {
          client.send(this.editor.value);
        }
      },
    });

    // Editor events
    this.editor.addEventListener('input', () => {
      this.updateHighlight();
      this.updateLineNumbers();
      client.send(this.editor.value);
    });
    this.editor.addEventListener('scroll', () => this.syncScroll());
    this.editor.addEventListener('keydown', (e) => this.handleKey(e));

    // Set default content
    this.editor.value = DEFAULT_CODE;
    this.updateHighlight();
    this.updateLineNumbers();
  }

  private handleKey(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      e.preventDefault();
      const start = this.editor.selectionStart;
      const end = this.editor.selectionEnd;
      this.editor.value = this.editor.value.substring(0, start) + '  ' + this.editor.value.substring(end);
      this.editor.selectionStart = this.editor.selectionEnd = start + 2;
      this.editor.dispatchEvent(new Event('input'));
    }
  }

  private syncScroll() {
    this.highlight.scrollTop = this.editor.scrollTop;
    this.highlight.scrollLeft = this.editor.scrollLeft;
    this.lineNumbers.scrollTop = this.editor.scrollTop;
  }

  private updateLineNumbers() {
    const lines = this.editor.value.split('\n').length;
    const errorLines = new Set(this.errors.map(e => e.line));
    this.lineNumbers.innerHTML = Array.from({ length: lines }, (_, i) => {
      const hasError = errorLines.has(i);
      return `<div class="line${hasError ? ' error' : ''}">${i + 1}</div>`;
    }).join('');
  }

  private updateHighlight() {
    this.highlight.innerHTML = this.highlightCode(this.editor.value);
  }

  private highlightCode(code: string): string {
    return code.split('\n').map(line => this.highlightLine(line)).join('\n');
  }

  private highlightLine(line: string): string {
    // Handle comments first
    const commentIdx = line.indexOf('//');
    if (commentIdx !== -1) {
      const before = line.substring(0, commentIdx);
      const comment = line.substring(commentIdx);
      return this.highlightTokens(before) + `<span class="comment">${this.escapeHtml(comment)}</span>`;
    }
    return this.highlightTokens(line);
  }

  private highlightTokens(line: string): string {
    const tokens: { type: string; value: string }[] = [];
    let remaining = line;

    while (remaining.length > 0) {
      // Whitespace
      const wsMatch = remaining.match(/^(\s+)/);
      if (wsMatch) {
        tokens.push({ type: 'ws', value: wsMatch[1] });
        remaining = remaining.slice(wsMatch[1].length);
        continue;
      }

      // Variable
      const varMatch = remaining.match(/^(\$[a-zA-Z_][a-zA-Z0-9_]*)/);
      if (varMatch) {
        tokens.push({ type: 'variable', value: varMatch[1] });
        remaining = remaining.slice(varMatch[1].length);
        continue;
      }

      // Color
      const colorMatch = remaining.match(/^(#[0-9a-fA-F]{3,8})\b/);
      if (colorMatch) {
        tokens.push({ type: 'color', value: colorMatch[1] });
        remaining = remaining.slice(colorMatch[1].length);
        continue;
      }

      // String
      const strMatch = remaining.match(/^("[^"]*"|'[^']*')/);
      if (strMatch) {
        tokens.push({ type: 'string', value: strMatch[1] });
        remaining = remaining.slice(strMatch[1].length);
        continue;
      }

      // Pair (100,200 or 100x200)
      const pairMatch = remaining.match(/^(-?\d+\.?\d*[,x]-?\d+\.?\d*)/);
      if (pairMatch) {
        tokens.push({ type: 'number', value: pairMatch[1] });
        remaining = remaining.slice(pairMatch[1].length);
        continue;
      }

      // Number
      const numMatch = remaining.match(/^(-?\d+\.?\d*)/);
      if (numMatch) {
        tokens.push({ type: 'number', value: numMatch[1] });
        remaining = remaining.slice(numMatch[1].length);
        continue;
      }

      // Identifier
      const identMatch = remaining.match(/^([a-zA-Z_][a-zA-Z0-9_-]*)/);
      if (identMatch) {
        const word = identMatch[1];
        let type = 'ident';
        if (KEYWORDS.has(word)) type = 'keyword';
        else if (SHAPES.has(word)) type = 'shape';
        else if (PROPERTIES.has(word)) type = 'property';
        tokens.push({ type, value: word });
        remaining = remaining.slice(word.length);
        continue;
      }

      // Operators and brackets
      const opMatch = remaining.match(/^([=:\->\[\]])/);
      if (opMatch) {
        tokens.push({ type: 'punct', value: opMatch[1] });
        remaining = remaining.slice(opMatch[1].length);
        continue;
      }

      // Fallback
      tokens.push({ type: 'plain', value: remaining[0] });
      remaining = remaining.slice(1);
    }

    return tokens.map(t => `<span class="${t.type}">${this.escapeHtml(t.value)}</span>`).join('');
  }

  private escapeHtml(str: string): string {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  private updateErrors() {
    const count = this.errors.length;
    this.errorCount.textContent = `${count} error${count !== 1 ? 's' : ''}`;
    this.errorCount.classList.toggle('visible', count > 0);

    if (count > 0) {
      this.errorPanel.innerHTML = this.errors.map(e =>
        `<div class="error-item" data-line="${e.line}">
          <span class="line-ref">L${e.line + 1}:</span>${this.escapeHtml(e.message)}
        </div>`
      ).join('');
      this.errorPanel.classList.add('visible');

      this.errorPanel.querySelectorAll('.error-item').forEach(el => {
        el.addEventListener('click', () => {
          const line = parseInt((el as HTMLElement).dataset.line!);
          this.goToLine(line);
        });
      });
    } else {
      this.errorPanel.classList.remove('visible');
      this.errorPanel.innerHTML = '';
    }
  }

  private goToLine(lineNum: number) {
    const lines = this.editor.value.split('\n');
    let pos = 0;
    for (let i = 0; i < lineNum && i < lines.length; i++) {
      pos += lines[i].length + 1;
    }
    this.editor.focus();
    this.editor.selectionStart = this.editor.selectionEnd = pos;
    const lineHeight = parseFloat(getComputedStyle(this.editor).lineHeight);
    this.editor.scrollTop = lineNum * lineHeight - this.editor.clientHeight / 2;
  }
}

new IconoglottPlayground();

