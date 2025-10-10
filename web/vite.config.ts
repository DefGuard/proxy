import { ViteImageOptimizer } from 'vite-plugin-image-optimizer';
import { tanstackRouter } from '@tanstack/router-plugin/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { defineConfig, loadEnv, type ProxyOptions } from 'vite';
import react from '@vitejs/plugin-react-swc';
import * as path from 'path';

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');

  const proxyOptions: Record<string, string | ProxyOptions> = {};
  const proxyUpdateServiceBase = env.VITE_UPDATE_BASE_URL;
  const proxyUpdateServiceTarget = env.UPDATE_TARGET_URL;

  if (
    mode === 'development' &&
    proxyUpdateServiceBase &&
    proxyUpdateServiceBase.length &&
    proxyUpdateServiceTarget &&
    proxyUpdateServiceTarget.length
  ) {
    proxyOptions['/update'] = {
      target: proxyUpdateServiceTarget,
      changeOrigin: true,
      secure: true,
      rewrite: (path) => path.replace(/^\/update/, ''),
    };
  }

  return {
    server: {
      port: 3002,
      strictPort: true,
      proxy: {
        '/api': {
          target: 'http://127.0.0.1:8080/',
          changeOrigin: true,
          secure: false,
        },
        ...proxyOptions
      },
    },
    plugins: [
      paraglideVitePlugin({
        project: './project.inlang',
        outdir: './src/paraglide',
        strategy: ['localStorage', 'preferredLanguage', 'baseLocale'],
      }),
      tanstackRouter({
        target: 'react',
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
      },
    },
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@use "@scssutils" as *;\n`,
        },
      },
    },
  };
});
