import { ViteImageOptimizer } from 'vite-plugin-image-optimizer';
import {tanstackRouter} from "@tanstack/router-plugin/vite";
import { paraglideVitePlugin } from '@inlang/paraglide-js'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import * as path from 'path';

// https://vite.dev/config/
export default defineConfig({
  server: {
    port: 3002,
    strictPort: true,
  },
  plugins: [
    paraglideVitePlugin({
      project: './project.inlang',
      outdir: './src/paraglide',
      strategy: ['localStorage', 'preferredLanguage', 'baseLocale'],
    }),
    tanstackRouter({
      target: "react",
      autoCodeSplitting: true,
    }),
    ViteImageOptimizer({
      test: /\.(jpe?g|png|gif|tiff|webp|avif)$/i,
    }),
    react(),
  ],
  resolve: {
    alias: {
      '@scssutils': path.resolve(__dirname, './src/shared/defguard-ui/scss/global'),
    }
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@use "@scssutils" as *;\n`,
      }
    }
  }
})
