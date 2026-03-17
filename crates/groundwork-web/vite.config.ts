import { defineConfig, type Plugin } from 'vite';
import { readFileSync, existsSync } from 'fs';
import path from 'path';
import wasm from 'vite-plugin-wasm';

/** Dev-only: serve wiki markdown + images from the repo root wiki/ directory. */
function wikiDevPlugin(): Plugin {
  const wikiRoot = path.resolve(__dirname, '../../wiki');
  return {
    name: 'wiki-dev',
    apply: 'serve',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        // Match /wiki/*.md and /wiki/images/**
        if (req.url?.startsWith('/wiki/') && (req.url.endsWith('.md') || req.url.includes('/images/'))) {
          const relPath = req.url.replace('/wiki/', '');
          const filePath = path.join(wikiRoot, relPath);
          if (existsSync(filePath)) {
            const ext = path.extname(filePath);
            const types: Record<string, string> = { '.md': 'text/markdown', '.png': 'image/png', '.jpg': 'image/jpeg', '.svg': 'image/svg+xml' };
            res.setHeader('Content-Type', types[ext] || 'application/octet-stream');
            res.end(readFileSync(filePath));
            return;
          }
        }
        next();
      });
    },
  };
}

export default defineConfig(({ command }) => ({
  // GitHub Pages serves from /<repo-name>/
  base: command === 'build' ? '/groundwork/' : '/',
  plugins: [wasm(), wikiDevPlugin()],
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
