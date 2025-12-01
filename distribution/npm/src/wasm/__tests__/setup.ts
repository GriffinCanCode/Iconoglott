/**
 * Vitest setup for WASM tests
 * Provides the WASM binary to the module via Node.js fs
 */
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

// Patch global fetch to handle local WASM file loading
const originalFetch = globalThis.fetch;

globalThis.fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
  const url = input instanceof URL ? input : (input instanceof Request ? new URL(input.url) : new URL(input));
  
  if (url.protocol === 'file:' && url.pathname.endsWith('.wasm')) {
    const wasmPath = fileURLToPath(url);
    const wasmBuffer = readFileSync(wasmPath);
    return new Response(wasmBuffer, {
      status: 200,
      headers: { 'Content-Type': 'application/wasm' },
    });
  }
  
  return originalFetch(input, init);
};

