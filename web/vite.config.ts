import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import autoprefixer from 'autoprefixer';
import * as path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    strictPort: true,
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080/',
        changeOrigin: true,
      },
    },
  },
  resolve: {
    alias: {
      '@scssutils': path.resolve(__dirname, '/src/shared/scss/helpers'),
    },
  },
  css: {
    postcss: {
      plugins: [autoprefixer],
    },
  },
});
