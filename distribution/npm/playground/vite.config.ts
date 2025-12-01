import { defineConfig } from 'vite';

export default defineConfig({
  base: './',
  build: {
    outDir: 'dist',
    emptyDirBeforeWrite: true,
    target: 'esnext',
    assetsDir: 'assets',
  },
});

