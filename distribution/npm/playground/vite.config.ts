import { defineConfig } from 'vite';

export default defineConfig({
  base: './',
  build: {
    outDir: 'dist',
    emptyDirBeforeWrite: true,
    target: 'esnext',
    assetsDir: 'assets',
  },
  server: {
    // HMR configuration for hot module replacement without full page refresh
    hmr: {
      overlay: true,
    },
    // Watch mode configuration
    watch: {
      // Watch for changes in the parent package as well
      ignored: ['!**/node_modules/@iconoglott/**'],
    },
  },
  // Optimize deps to include the local package
  optimizeDeps: {
    exclude: ['@iconoglott/renderer'],
  },
});
