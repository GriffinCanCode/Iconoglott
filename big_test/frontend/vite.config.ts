import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  server: {
    port: 3456,
    proxy: {
      '/ws': {
        target: 'ws://localhost:8765',
        ws: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
  resolve: {
    alias: {
      // Alias WASM module to source files
      'iconoglott_core': resolve(__dirname, '../../distribution/npm/src/wasm/iconoglott_core.js'),
    },
  },
  optimizeDeps: {
    exclude: ['@iconoglott/renderer'],
  },
  assetsInclude: ['**/*.wasm'],
});

