import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  // base: './' 确保生产构建使用相对路径，适配 Tauri 本地文件加载
  base: './',
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    // 输出到 dist/ 目录（tauri.conf.json 中 frontendDist 对应的路径）
    outDir: 'dist',
    emptyOutDir: true,
  },
})
