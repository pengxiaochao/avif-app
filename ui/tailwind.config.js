/** @type {import('tailwindcss').Config} */
export default {
  // 告诉 Tailwind 扫描哪些文件以生成样式（用于 PurgeCSS 去除未使用的类）
  content: [
    './index.html',
    './src/**/*.{vue,js,ts}',
  ],
  // 跟随系统暗色模式（通过 CSS media query 自动切换）
  darkMode: 'media',
  theme: {
    extend: {
      // macOS 配色方案
      colors: {
        apple: {
          blue: '#0071e3',
          'blue-dark': '#0a84ff',
          bg: '#f5f5f7',
          'bg-dark': '#1c1c1e',
          card: '#ffffff',
          'card-dark': '#2c2c2e',
          text: '#1d1d1f',
          'text-dark': '#f2f2f7',
          secondary: '#6e6e73',
          'secondary-dark': '#aeaeb2',
          separator: 'rgba(0,0,0,0.1)',
          'separator-dark': 'rgba(255,255,255,0.1)',
          danger: '#ff3b30',
          success: '#34c759',
          warning: '#ff9f0a',
        },
      },
      fontFamily: {
        // 使用 macOS 系统字体
        sans: ['-apple-system', 'BlinkMacSystemFont', '"SF Pro Text"', '"Helvetica Neue"', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
