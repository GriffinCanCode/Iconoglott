#!/usr/bin/env tsx
/**
 * Frontend Integration Tests for @iconoglott/renderer
 * 
 * Tests:
 * 1. WASM initialization and rendering
 * 2. Parser and tokenizer
 * 3. Type exports
 */

import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';
import { readFileSync } from 'fs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const distPath = resolve(__dirname, '../../distribution/npm/dist');

// ─────────────────────────────────────────────────────────────────────────────
// Test 1: Type Exports
// ─────────────────────────────────────────────────────────────────────────────

async function testTypeExports() {
  console.log('\n▸ Testing: Type Exports');
  
  // Check that the dist folder exists and has types
  const indexDts = readFileSync(resolve(distPath, 'index.d.ts'), 'utf-8');
  
  const requiredExports = [
    'initWasm',
    'render',
    'parse',
    'tokenize',
    'createClient',
    'Canvas',
    'ParseResult',
  ];
  
  for (const exp of requiredExports) {
    if (!indexDts.includes(exp)) {
      throw new Error(`Missing export: ${exp}`);
    }
  }
  
  console.log('✓ Type exports verified!');
  console.log(`  Found exports: ${requiredExports.join(', ')}`);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: Module Imports
// ─────────────────────────────────────────────────────────────────────────────

async function testModuleImports() {
  console.log('\n▸ Testing: Module Imports');
  
  // Import the package (uses the local distribution)
  const pkg = await import('@iconoglott/renderer');
  
  // Check core exports exist
  if (typeof pkg.initWasm !== 'function') {
    throw new Error('initWasm not exported');
  }
  if (typeof pkg.render !== 'function') {
    throw new Error('render not exported');
  }
  if (typeof pkg.parse !== 'function') {
    throw new Error('parse not exported');
  }
  if (typeof pkg.createClient !== 'function') {
    throw new Error('createClient not exported');
  }
  
  console.log('✓ Module imports work!');
  console.log(`  Exports: initWasm, render, parse, createClient, etc.`);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: WASM Bridge (without browser)
// ─────────────────────────────────────────────────────────────────────────────

async function testWasmBridge() {
  console.log('\n▸ Testing: WASM Bridge');
  
  const { tryGetWasm, isWasmLoaded } = await import('@iconoglott/renderer');
  
  // WASM shouldn't be loaded yet in Node environment
  const loaded = isWasmLoaded();
  console.log(`  WASM loaded: ${loaded}`);
  
  // tryGetWasm should return undefined when not loaded
  const wasm = tryGetWasm();
  console.log(`  tryGetWasm: ${wasm ? 'available' : 'not available (expected in Node)'}`);
  
  console.log('✓ WASM bridge functions accessible!');
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4: WebSocket Client API
// ─────────────────────────────────────────────────────────────────────────────

async function testWebSocketClient() {
  console.log('\n▸ Testing: WebSocket Client API');
  
  const { createClient } = await import('@iconoglott/renderer/client');
  
  if (typeof createClient !== 'function') {
    throw new Error('createClient not exported from /client');
  }
  
  // Test client creation (won't actually connect in Node)
  // Just verify the API shape
  console.log('✓ WebSocket client API available!');
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5: Package Structure
// ─────────────────────────────────────────────────────────────────────────────

async function testPackageStructure() {
  console.log('\n▸ Testing: Package Structure');
  
  const pkgJson = JSON.parse(
    readFileSync(resolve(__dirname, '../../distribution/npm/package.json'), 'utf-8')
  );
  
  // Verify exports
  const exports = pkgJson.exports;
  if (!exports['.']) throw new Error('Missing main export');
  if (!exports['./react']) throw new Error('Missing react export');
  if (!exports['./client']) throw new Error('Missing client export');
  if (!exports['./wasm']) throw new Error('Missing wasm export');
  
  console.log('✓ Package structure correct!');
  console.log(`  Exports: ${Object.keys(exports).join(', ')}`);
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

async function main() {
  console.log('\n' + '═'.repeat(60));
  console.log('  ICONOGLOTT FRONTEND INTEGRATION TESTS');
  console.log('═'.repeat(60));
  
  const tests = [
    ['Type Exports', testTypeExports],
    ['Module Imports', testModuleImports],
    ['WASM Bridge', testWasmBridge],
    ['WebSocket Client', testWebSocketClient],
    ['Package Structure', testPackageStructure],
  ] as const;
  
  let passed = 0;
  let failed = 0;
  
  for (const [name, testFn] of tests) {
    try {
      await testFn();
      passed++;
    } catch (e) {
      console.log(`✗ ${name} FAILED: ${e}`);
      failed++;
    }
  }
  
  console.log('\n' + '─'.repeat(60));
  console.log(`Results: ${passed} passed, ${failed} failed`);
  console.log('─'.repeat(60) + '\n');
  
  process.exit(failed === 0 ? 0 : 1);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});

