import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import path from 'path';

export default defineConfig(({ mode }) => ({
  base: mode === 'production' ? '/captcha-royale/' : '/',
  plugins: [react(), wasm()],
  resolve: {
    alias: {
      'captcha-engine': path.resolve(__dirname, '../../packages/captcha-engine/pkg/captcha_engine'),
    },
  },
  build: {
    target: 'esnext',
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8787',
        ws: true,
      },
    },
  },
}));
