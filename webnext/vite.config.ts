import { paraglideVitePlugin } from '@inlang/paraglide-js'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'

// https://vite.dev/config/
export default defineConfig({
  plugins: [paraglideVitePlugin({ project: './project.inlang', outdir: './src/paraglide' }),react()],
})
