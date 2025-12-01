/**
 * Iconoglott Frontend Integration Demo
 * 
 * Uses the Canvas component from @iconoglott/renderer for easy integration.
 * Demonstrates both WebSocket and local rendering modes.
 */

import { Canvas, connectTo, CARD_PRESET, type CanvasFit } from '@iconoglott/renderer/canvas';

// ─────────────────────────────────────────────────────────────────────────────
// DOM Elements
// ─────────────────────────────────────────────────────────────────────────────

const editor = document.getElementById('editor') as HTMLTextAreaElement;
const canvasEl = document.getElementById('canvas') as HTMLElement;
const status = document.getElementById('status') as HTMLElement;
const runTestsBtn = document.getElementById('run-tests') as HTMLButtonElement;
const testResults = document.getElementById('test-results') as HTMLElement;

// ─────────────────────────────────────────────────────────────────────────────
// Canvas Component
// ─────────────────────────────────────────────────────────────────────────────

let iconoCanvas: Canvas | null = null;

function initCanvas() {
  // Use the new Canvas component with WebSocket connection
  iconoCanvas = connectTo(canvasEl, `ws://${location.host}/ws`, {
    fit: 'contain',
    background: 'transparent',
    onRender: () => {
      // Canvas handles SVG insertion automatically
    },
    onError: (msg) => {
      console.error('Render error:', msg);
    },
    onConnection: (connected) => {
      status.classList.toggle('connected', connected);
      status.textContent = connected ? 'Connected' : 'Disconnected';
      if (connected) iconoCanvas?.send(editor.value);
    },
  });
  
  // Editor input handler
  editor.addEventListener('input', () => {
    iconoCanvas?.send(editor.value);
  });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Suite
// ─────────────────────────────────────────────────────────────────────────────

interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
}

const tests: Array<{ name: string; fn: () => Promise<void> }> = [
  {
    name: 'Canvas Connection',
    async fn() {
      if (!iconoCanvas) throw new Error('Canvas not initialized');
      // Wait for connection
      await new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error('Connection timeout')), 3000);
        const check = setInterval(() => {
          if (status.textContent === 'Connected') {
            clearInterval(check);
            clearTimeout(timeout);
            resolve();
          }
        }, 100);
      });
    },
  },
  {
    name: 'Canvas Rendering',
    async fn() {
      if (!iconoCanvas) throw new Error('Canvas not initialized');
      iconoCanvas.send('canvas 100x100 fill #000\ncircle at 50,50 radius 20 fill #f00');
      await new Promise(resolve => setTimeout(resolve, 500));
      if (!iconoCanvas.getSvg().includes('<svg')) throw new Error('No SVG rendered');
    },
  },
  {
    name: 'Shape Types',
    async fn() {
      if (!iconoCanvas) throw new Error('Canvas not initialized');
      iconoCanvas.send(`canvas 200x150 fill #111
rect at 20,20 size 40x40 fill #f00
circle at 100,75 radius 25 fill #0f0
text "Test" at 150,75 font "Arial" 14 fill #fff`);
      await new Promise(resolve => setTimeout(resolve, 500));
      const svg = iconoCanvas.getSvg();
      if (!svg.includes('rect')) throw new Error('Missing rect');
      if (!svg.includes('circle') && !svg.includes('ellipse')) throw new Error('Missing circle');
    },
  },
  {
    name: 'Fit Modes',
    async fn() {
      if (!iconoCanvas) throw new Error('Canvas not initialized');
      // Test contain mode
      iconoCanvas.setFit('contain');
      await new Promise(resolve => setTimeout(resolve, 100));
      // Test cover mode
      iconoCanvas.setFit('cover');
      await new Promise(resolve => setTimeout(resolve, 100));
      // Back to contain
      iconoCanvas.setFit('contain');
    },
  },
];

function updateTestUI(results: TestResult[]) {
  testResults.innerHTML = results
    .map(r => `<div class="test-item ${r.passed ? 'pass' : r.error ? 'fail' : 'pending'}">${r.name}${r.error ? `: ${r.error}` : ''}</div>`)
    .join('');
}

async function runTests() {
  runTestsBtn.disabled = true;
  runTestsBtn.textContent = 'Running...';
  
  const results: TestResult[] = tests.map(t => ({ name: t.name, passed: false }));
  updateTestUI(results);
  
  for (let i = 0; i < tests.length; i++) {
    try {
      await tests[i].fn();
      results[i].passed = true;
    } catch (e) {
      results[i].passed = false;
      results[i].error = String(e);
    }
    updateTestUI(results);
  }
  
  runTestsBtn.disabled = false;
  runTestsBtn.textContent = 'Run Tests';
  
  const passed = results.filter(r => r.passed).length;
  console.log(`Tests: ${passed}/${results.length} passed`);
}

// ─────────────────────────────────────────────────────────────────────────────
// Initialize
// ─────────────────────────────────────────────────────────────────────────────

runTestsBtn.addEventListener('click', runTests);
initCanvas();
status.textContent = 'Connecting...';

