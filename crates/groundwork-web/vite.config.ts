import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig(({ command }) => ({
  // GitHub Pages serves from /<repo-name>/
  base: command === 'build' ? '/groundwork/' : '/',
  plugins: [wasm()],
  build: {
    target: 'esnext',
    outDir: 'dist',
  },
  server: {
    headers: {
      // Required for SharedArrayBuffer (future multi-threaded sim)
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
}));
