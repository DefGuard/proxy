import * as path from 'node:path';
import react from '@vitejs/plugin-react-swc';
import autoprefixer from 'autoprefixer';
import { defineConfig } from 'vite';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    strictPort: true,
    port: 3002,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080/',
        changeOrigin: true,
        secure: false,
      },
    },
  },
  resolve: {
    alias: {
      '@scss': path.resolve(__dirname, './src/shared/scss'),
      '@scssutils': path.resolve(__dirname, './src/shared/defguard-ui/scss/helpers'),
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@use "@scssutils" as *;\n`,
      },
    },
    postcss: {
      plugins: [autoprefixer],
    },
  },
});
