import { devtools } from '@tanstack/devtools-vite';
import { ViteImageOptimizer } from 'vite-plugin-image-optimizer';
import { tanstackRouter } from '@tanstack/router-plugin/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { defineConfig, loadEnv, type ProxyOptions } from 'vite';
import * as path from 'path';

// https://vite.dev/config/
export default defineConfig(async ({ command, mode }) => {
  // Match the plugin's Vite 8 build config without loading SWC's native addon.
  const reactPlugins =
    command === 'serve' ? (await import('@vitejs/plugin-react-swc')).default() : [];
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
    oxc: command === 'build'
      ? {
          jsx: {
            runtime: 'automatic' as const,
            importSource: 'react',
          },
        }
      : undefined,
    plugins: [
      devtools({
        removeDevtoolsOnBuild: true
      }),
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
      ...reactPlugins,
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
    build: {
      chunkSizeWarningLimit: 1000
    }
  };
});
