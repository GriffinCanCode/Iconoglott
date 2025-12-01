/**
 * Iconoglott Playground
 * Real-time DSL rendering using @iconoglott/renderer WebSocket client
 */

import { createClient } from '@iconoglott/renderer/client';
import type { ParseError } from '@iconoglott/renderer';

// Semantic classification for syntax highlighting (presentation layer only)
const KEYWORDS = new Set(['canvas', 'group', 'stack', 'row', 'graph', 'node', 'edge', 'symbol', 'use', '@keyframes']);
const SHAPES = new Set(['rect', 'circle', 'ellipse', 'line', 'path', 'polygon', 'text', 'image', 'arc', 'curve', 'diamond']);
const PROPERTIES = new Set([
  'at', 'size', 'radius', 'from', 'to', 'fill', 'stroke', 'opacity', 'corner',
  'shadow', 'gradient', 'blur', 'font', 'bold', 'italic', 'center', 'end', 'middle',
  'translate', 'rotate', 'scale', 'origin', 'width', 'height', 'gap', 'vertical', 'horizontal',
  'linear', 'radial', 'd', 'points', 'href', 'label', 'shape', 'spacing', 'curved', 'straight',
  'orthogonal', 'hierarchical', 'force', 'grid', 'tree', 'manual', 'justify', 'align', 'wrap',
  'start', 'smooth', 'sharp', 'closed', 'padding', 'anchor', 'auto', 'viewbox',
  // Animation properties
  'animate', 'transition', 'delay', 'infinite', 'alternate', 'forwards', 'backwards', 'both',
  'ease', 'ease-in', 'ease-out', 'ease-in-out', 'normal', 'reverse', 'alternate-reverse'
]);

// ═══════════════════════════════════════════════════════════
// SHOWCASE EXAMPLES
// ═══════════════════════════════════════════════════════════

const EXAMPLES = {
  shapes: `canvas 160x100 fill #0a0f1a
circle at 40,50 radius 25 fill #f97316
rect at 75,30 size 40,40 fill #a855f7 corner 6
polygon points [(130,25) (150,50) (130,75) (110,50)] fill #22d3ee`,

  graphs: `canvas 160x100 fill #0a0f1a
graph at 20,20 size 120,60 hierarchical
  node "A" shape rect fill #3b82f6
  node "B" shape rect fill #10b981
  node "C" shape rect fill #f59e0b
  edge A -> B
  edge A -> C`,

  animations: `canvas 160x100 fill #0a0f1a
@keyframes pulse
  0% opacity 0.5 scale 0.9,0.9
  50% opacity 1 scale 1.1,1.1
  100% opacity 0.5 scale 0.9,0.9
circle at 50,50 radius 20
  gradient radial #f97316 #0a0f1a
  animate pulse 2s ease-in-out infinite
circle at 110,50 radius 15
  fill none stroke #a855f7 3
  animate pulse 2s ease-in-out infinite delay 500ms`
};

const DEMO_CODE = `canvas 400x250 fill #0a0f1a

@keyframes pulse
  0% opacity 0.6 scale 0.95,0.95
  50% opacity 1 scale 1.05,1.05
  100% opacity 0.6 scale 0.95,0.95

// Orbs
circle at 320,50 radius 30
  gradient radial #10b981 #0a0f1a
  animate pulse 3s ease-in-out infinite

circle at 350,70 radius 18
  gradient radial #3b82f6 #0a0f1a
  animate pulse 2.5s ease-in-out infinite delay 300ms

// Shapes showcase
rect at 40,80 size 60,60 fill #f97316 corner 8
circle at 140,110 radius 30 fill #a855f7
polygon points [(210,80) (250,130) (170,130)] fill #22d3ee

// Labels
text "rect" at 55,160 font "Space Grotesk" 10 fill #64748b
text "circle" at 120,160 font "Space Grotesk" 10 fill #64748b
text "polygon" at 185,160 font "Space Grotesk" 10 fill #64748b

// Bottom bar
rect at 20,200 size 360,30 fill #161b22 corner 6
text "canvas → shapes → SVG" at 120,220 font "Space Grotesk" 11 fill #8b949e`;

const DEFAULT_CODE = `// Iconoglott — Visual DSL for Vector Graphics
// Demo: Animations & Transitions

canvas giant fill #0a0f1a

// Color palette
$accent = #10b981
$blue = #3b82f6
$purple = #8b5cf6
$amber = #f59e0b
$cyan = #06b6d4

// ═══════════════════════════════════════════════
// TITLE
// ═══════════════════════════════════════════════

text "Iconoglott" at 20,30
  font "Space Grotesk" 22
  fill #fff
  bold

text "CSS Animations & Transitions" at 20,50
  font "Space Grotesk" 11
  fill #64748b

// ═══════════════════════════════════════════════
// KEYFRAMES — Define reusable animations
// ═══════════════════════════════════════════════

@keyframes pulse
  0%
    opacity 0.4
    scale 0.9,0.9
  50%
    opacity 1
    scale 1.1,1.1
  100%
    opacity 0.4
    scale 0.9,0.9

@keyframes spin
  0%
    rotate 0
  100%
    rotate 360

@keyframes fade-in
  0%
    opacity 0
  100%
    opacity 1

@keyframes glow
  0%
    opacity 0.3
  50%
    opacity 0.8
  100%
    opacity 0.3

// ═══════════════════════════════════════════════
// ANIMATED SHAPES
// ═══════════════════════════════════════════════

// Pulsing orb
circle at 100,140 radius 30
  gradient radial $accent #0a0f1a
  animate pulse 2s ease-in-out infinite

// Spinning star
polygon points [(80,250) (90,280) (120,280) (95,300) (105,330) (80,310) (55,330) (65,300) (40,280) (70,280)]
  fill $amber
  animate spin 4s linear infinite

// Fading text
text "Hello, World!" at 200,140
  font "Space Grotesk" 18
  fill #fff
  animate fade-in 1s ease-out forwards

// Glowing ring
circle at 350,250 radius 40
  fill none
  stroke $purple 4
  animate glow 2s ease-in-out infinite

circle at 350,250 radius 30
  fill none
  stroke $blue 2
  animate glow 2s ease-in-out infinite
  delay 500ms

// ═══════════════════════════════════════════════
// TRANSITIONS — Smooth state changes
// ═══════════════════════════════════════════════

rect at 200,300 size 80,40
  fill $blue
  corner 8
  transition opacity 300ms ease
  transition fill 500ms ease-out

text "Hover Me" at 210,325
  font "Space Grotesk" 12
  fill #fff

// ═══════════════════════════════════════════════
// COMPONENT SYMBOLS WITH ANIMATION
// ═══════════════════════════════════════════════

symbol "dot"
  circle 8,8 8
    gradient radial $cyan #0a0f1a

// Animated dot grid
use "dot" at 400,120
  animate pulse 1.5s ease infinite
use "dot" at 440,120
  animate pulse 1.5s ease infinite delay 200ms
use "dot" at 480,120
  animate pulse 1.5s ease infinite delay 400ms
use "dot" at 420,150
  animate pulse 1.5s ease infinite delay 300ms
use "dot" at 460,150
  animate pulse 1.5s ease infinite delay 500ms

// Labels
text "Pulse" at 85,190
  font "Space Grotesk" 9
  fill #64748b

text "Spin" at 70,380
  font "Space Grotesk" 9
  fill #64748b

text "Glow" at 335,320
  font "Space Grotesk" 9
  fill #64748b

text "Dots" at 430,190
  font "Space Grotesk" 9
  fill #64748b

// Background ambience
circle at 480,400 radius 60
  gradient radial #3b82f620 #0a0f1a
  animate glow 3s ease-in-out infinite`;

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

      // @keyframes directive
      const keyframesMatch = remaining.match(/^(@keyframes)\b/);
      if (keyframesMatch) {
        tokens.push({ type: 'keyword', value: keyframesMatch[1] });
        remaining = remaining.slice(keyframesMatch[1].length);
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

      // Percentage pair (50%,50% or 50%x50%)
      const percentPairMatch = remaining.match(/^(-?\d+\.?\d*%[,x]-?\d+\.?\d*%)/);
      if (percentPairMatch) {
        tokens.push({ type: 'number', value: percentPairMatch[1] });
        remaining = remaining.slice(percentPairMatch[1].length);
        continue;
      }

      // Pair (100,200 or 100x200)
      const pairMatch = remaining.match(/^(-?\d+\.?\d*[,x]-?\d+\.?\d*)/);
      if (pairMatch) {
        tokens.push({ type: 'number', value: pairMatch[1] });
        remaining = remaining.slice(pairMatch[1].length);
        continue;
      }

      // Percentage (50%)
      const percentMatch = remaining.match(/^(-?\d+\.?\d*%)/);
      if (percentMatch) {
        tokens.push({ type: 'number', value: percentMatch[1] });
        remaining = remaining.slice(percentMatch[1].length);
        continue;
      }

      // Duration (500ms, 1s, 2.5s)
      const durationMatch = remaining.match(/^(\d+\.?\d*)(ms|s)\b/);
      if (durationMatch) {
        tokens.push({ type: 'number', value: durationMatch[0] });
        remaining = remaining.slice(durationMatch[0].length);
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

// ═══════════════════════════════════════════════════════════
// SHOWCASE CONTROLLER
// ═══════════════════════════════════════════════════════════

class ShowcaseController {
  private showcase = document.getElementById('showcase')!;
  private features = document.getElementById('features')!;
  private editorSection = document.getElementById('editorSection')!;
  private showcaseDemo = document.getElementById('showcaseDemo')!;
  private wsClient: ReturnType<typeof createClient> | null = null;
  private activeExample = 'shapes';

  constructor() {
    this.init();
  }

  private init() {
    // Set up WebSocket for showcase rendering
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    this.wsClient = createClient({
      url: `${proto}//${location.host}/ws`,
      debounce: 50,
      onRender: (svg) => this.handleShowcaseRender(svg),
      onConnectionChange: (connected) => {
        if (connected) this.renderShowcase();
      },
    });

    // Navigation buttons
    document.getElementById('featuresBtn')?.addEventListener('click', () => {
      this.features.scrollIntoView({ behavior: 'smooth' });
    });

    document.getElementById('tryBtn')?.addEventListener('click', () => {
      this.showEditor();
    });

    document.getElementById('backBtn')?.addEventListener('click', () => {
      this.showcase.scrollIntoView({ behavior: 'smooth' });
    });

    // Example cards
    document.querySelectorAll('.example-card').forEach(card => {
      card.addEventListener('click', () => {
        const example = (card as HTMLElement).dataset.example!;
        this.setActiveExample(example);
      });
    });

    // Render initial showcase
    this.renderShowcase();
    this.renderExamplePreviews();
  }

  private handleShowcaseRender(svg: string) {
    this.showcaseDemo.innerHTML = svg;
  }

  private renderShowcase() {
    this.wsClient?.send(DEMO_CODE);
  }

  private renderExamplePreviews() {
    // Render static SVGs for example cards (inline simple previews)
    const shapesPreview = `<svg viewBox="0 0 160 100" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect width="160" height="100" fill="#0a0f1a"/>
      <circle cx="40" cy="50" r="25" fill="#f97316"/>
      <rect x="75" y="30" width="40" height="40" rx="6" fill="#a855f7"/>
      <polygon points="130,25 150,50 130,75 110,50" fill="#22d3ee"/>
    </svg>`;

    const graphsPreview = `<svg viewBox="0 0 160 100" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect width="160" height="100" fill="#0a0f1a"/>
      <rect x="60" y="10" width="40" height="24" rx="4" fill="#3b82f6"/>
      <text x="80" y="27" font-size="10" fill="#fff" text-anchor="middle">A</text>
      <rect x="20" y="60" width="40" height="24" rx="4" fill="#10b981"/>
      <text x="40" y="77" font-size="10" fill="#fff" text-anchor="middle">B</text>
      <rect x="100" y="60" width="40" height="24" rx="4" fill="#f59e0b"/>
      <text x="120" y="77" font-size="10" fill="#fff" text-anchor="middle">C</text>
      <line x1="70" y1="34" x2="45" y2="60" stroke="#64748b" stroke-width="2"/>
      <line x1="90" y1="34" x2="115" y2="60" stroke="#64748b" stroke-width="2"/>
    </svg>`;

    const animPreview = `<svg viewBox="0 0 160 100" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect width="160" height="100" fill="#0a0f1a"/>
      <defs>
        <radialGradient id="g1"><stop offset="0%" stop-color="#f97316"/><stop offset="100%" stop-color="#0a0f1a"/></radialGradient>
      </defs>
      <circle cx="50" cy="50" r="20" fill="url(#g1)">
        <animate attributeName="opacity" values="0.6;1;0.6" dur="2s" repeatCount="indefinite"/>
      </circle>
      <circle cx="110" cy="50" r="15" fill="none" stroke="#a855f7" stroke-width="3">
        <animate attributeName="opacity" values="0.6;1;0.6" dur="2s" begin="0.5s" repeatCount="indefinite"/>
      </circle>
    </svg>`;

    document.getElementById('exampleShapes')!.innerHTML = shapesPreview;
    document.getElementById('exampleGraphs')!.innerHTML = graphsPreview;
    document.getElementById('exampleAnimations')!.innerHTML = animPreview;
  }

  private setActiveExample(example: string) {
    this.activeExample = example;
    document.querySelectorAll('.example-card').forEach(card => {
      card.classList.toggle('active', (card as HTMLElement).dataset.example === example);
    });
    // Load example into editor when clicking
    const code = EXAMPLES[example as keyof typeof EXAMPLES];
    if (code) this.wsClient?.send(code);
  }

  private showEditor() {
    this.editorSection.scrollIntoView({ behavior: 'smooth' });
  }
}

// Store instances for HMR
let showcase = new ShowcaseController();
let playground = new IconoglottPlayground();

// HMR support - preserve editor state across hot reloads
// @ts-ignore - Vite HMR types
if (import.meta.hot) {
  // @ts-ignore - Vite HMR types
  import.meta.hot.accept(() => {
    const editor = document.getElementById('editor') as HTMLTextAreaElement;
    const currentValue = editor?.value;
    const currentPos = editor?.selectionStart;
    
    // Recreate instances
    showcase = new ShowcaseController();
    playground = new IconoglottPlayground();
    
    // Restore state after a tick
    if (currentValue && editor) {
      requestAnimationFrame(() => {
        editor.value = currentValue;
        editor.selectionStart = editor.selectionEnd = currentPos ?? 0;
        editor.dispatchEvent(new Event('input'));
      });
    }
  });
}

