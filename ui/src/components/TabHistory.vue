<!--
  TabHistory.vue — 上传历史记录标签页
  ======================================
  展示所有历史上传记录，每条记录包含缩略图、文件名、文件大小和上传时间。
  用户可以：
  - 复制某条记录的 URL（点击复制图标按钮）
  - 在浏览器中打开图片（点击外链图标按钮）
  - 清空所有历史记录（点击右上角"清空"按钮）

  页面布局：
  ┌──────────────────────────────────────────────┐
  │ 上传历史          共 12 条记录   [清空]       │  ← 顶部工具栏
  ├──────────────────────────────────────────────┤
  │  [缩略图]  abc123.avif            [复制][打开]│
  │            23.4 KB · 2 分钟前                │
  │  [缩略图]  photo.avif             [复制][打开]│
  │            ...                               │
  │  （无记录时显示空状态占位图）                 │
  └──────────────────────────────────────────────┘

  数据加载时机：
  - onMounted：组件第一次显示时加载
  - onActivated：从其他 Tab 切换回来时重新加载（确保数据最新）
-->
<template>
  <!-- flex flex-col h-full：让整个页面占满父容器高度，顶部工具栏固定，列表可滚动 -->
  <div class="flex flex-col h-full">

    <!-- ── 顶部工具栏 ── -->
    <div class="flex items-center justify-between px-5 pt-5 pb-3">
      <div>
        <!-- 标题 -->
        <h2 class="text-[14px] font-semibold text-[#1d1d1f] dark:text-[#f2f2f7]">上传历史</h2>
        <!-- 记录数量提示：有记录时显示条数，没有时显示"暂无" -->
        <p class="text-[12px] text-[#6e6e73] dark:text-[#aeaeb2]">
          {{ entries.length > 0 ? `共 ${entries.length} 条记录` : '暂无上传记录' }}
        </p>
      </div>
      <!-- 清空按钮：只有有记录时才显示（v-if 条件渲染） -->
      <button
        v-if="entries.length > 0"
        class="px-3 h-7 rounded-lg text-[12px] font-medium
               text-red-500 dark:text-red-400
               hover:bg-red-50 dark:hover:bg-red-500/10
               border border-red-200 dark:border-red-500/30
               transition-colors"
        @click="clearHistory"
      >
        清空
      </button>
    </div>

    <!-- ── 历史记录列表区（可滚动） ── -->
    <div class="flex-1 overflow-y-auto px-5 pb-4 space-y-2">

      <!-- 空状态占位：没有历史记录时显示图标 + 提示文字 -->
      <div v-if="entries.length === 0" class="flex flex-col items-center justify-center h-40 gap-2">
        <!-- SVG 图片图标（内联 SVG，无需额外图片文件） -->
        <svg class="w-10 h-10 text-[#d1d1d6] dark:text-[#48484a]" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        <span class="text-[13px] text-[#6e6e73] dark:text-[#aeaeb2]">尚无上传记录</span>
      </div>

      <!--
        历史条目列表：
        v-for 遍历 entries 数组，每条记录渲染一个卡片
        :key="idx" 用索引作为唯一标识（这里用索引是因为记录没有唯一 ID）
        group 类：配合 group-hover 实现鼠标悬停时显示操作按钮的效果
      -->
      <div
        v-for="(entry, idx) in entries"
        :key="idx"
        class="flex items-center gap-3 p-3 rounded-xl
               bg-white dark:bg-[#2c2c2e]
               border border-black/[0.04] dark:border-white/[0.06]
               shadow-sm group"
      >
        <!-- 左侧缩略图区域（36×36px 正方形） -->
        <div class="w-9 h-9 rounded-lg bg-[#f5f5f7] dark:bg-[#3a3a3c] flex items-center justify-center shrink-0 overflow-hidden">
          <!--
            图片缩略图：只有当 URL 是图片格式时才尝试加载
            loading="lazy" 懒加载：不在视口内的图片不立即加载，提高性能
            @error：图片加载失败时（如 CORS 限制）隐藏图片元素，显示下方的文档图标
          -->
          <img
            v-if="isImageUrl(entry.url)"
            :src="entry.url"
            class="w-full h-full object-cover"
            loading="lazy"
            @error="$event.target.style.display='none'"
          />
          <!-- 非图片文件或图片加载失败时显示文档图标 -->
          <svg v-else class="w-4 h-4 text-[#6e6e73]" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </div>

        <!-- 中间文件信息（flex-1 占满剩余宽度，min-w-0 防止文字溢出） -->
        <div class="flex-1 min-w-0">
          <!-- 文件名：truncate 超长时显示省略号 -->
          <p class="text-[12.5px] font-medium text-[#1d1d1f] dark:text-[#f2f2f7] truncate">
            {{ entry.filename }}
          </p>
          <!-- 文件大小 + 上传时间：通过 formatSize 和 formatTime 函数格式化显示 -->
          <p class="text-[11px] text-[#6e6e73] dark:text-[#aeaeb2] truncate">
            {{ formatSize(entry.size_bytes) }} · {{ formatTime(entry.timestamp) }}
          </p>
        </div>

        <!--
          右侧操作按钮组：
          opacity-0 group-hover:opacity-100：鼠标悬停在整个条目上时才显示这两个按钮
          这是 Tailwind CSS 的 group/group-hover 机制，父元素加 group，子元素用 group-hover:
        -->
        <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
          <!-- 复制链接按钮 -->
          <button
            class="w-7 h-7 rounded-lg hover:bg-black/6 dark:hover:bg-white/10
                   flex items-center justify-center transition-colors"
            title="复制链接"
            @click="copyUrl(entry.url)"
          >
            <!-- 复制图标（两个叠加的矩形） -->
            <svg class="w-3.5 h-3.5 text-[#6e6e73] dark:text-[#aeaeb2]" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
          <!-- 在浏览器中打开按钮 -->
          <button
            class="w-7 h-7 rounded-lg hover:bg-black/6 dark:hover:bg-white/10
                   flex items-center justify-center transition-colors"
            title="在浏览器中打开"
            @click="openUrl(entry.url)"
          >
            <!-- 外链图标（矩形 + 右上角箭头） -->
            <svg class="w-3.5 h-3.5 text-[#6e6e73] dark:text-[#aeaeb2]" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
// ref：创建响应式引用
// onMounted：组件挂载后执行（第一次进入 Tab 时）
// onActivated：组件被 <keep-alive> 重新激活时执行（从其他 Tab 切换回来时）
import { ref, onMounted, onActivated } from 'vue'

// 此组件接受 config 但不使用（保持接口一致），也声明 update:config 事件（不触发）
defineProps({ config: Object })
defineEmits(['update:config'])

// ── 响应式状态 ────────────────────────────────────────────────────────────────

// entries：历史记录数组，每项包含 { filename, url, size_bytes, timestamp }
const entries = ref([])

/**
 * invoke 工具函数：兼容不同版本的 Tauri API 调用方式
 * 先尝试 Tauri v2 的 core.invoke，再尝试 Tauri v1 的直接 invoke，
 * 都不存在时（如浏览器开发环境）返回空数组（避免报错）
 */
const invoke = (...args) => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke(...args)
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke(...args)
  return Promise.resolve([])
}

// ── 数据操作方法 ───────────────────────────────────────────────────────────────

/**
 * 从 Rust 后端加载历史记录
 * 调用 Tauri 命令 "get_history"，Rust 从本地文件读取记录并返回
 * 按时间戳倒序排列，最新的在最前面
 */
async function loadHistory() {
  try {
    const data = await invoke('get_history')
    // 展开成新数组再排序（不直接修改原数组），b.timestamp - a.timestamp 表示降序
    entries.value = [...data].sort((a, b) => b.timestamp - a.timestamp)
  } catch (e) {
    console.error('加载历史记录失败:', e)
  }
}

/**
 * 清空所有历史记录
 * 调用 Tauri 命令 "clear_history"，Rust 清空本地存储文件
 * 成功后清空前端的 entries 数组，界面立即更新为空状态
 */
async function clearHistory() {
  try {
    await invoke('clear_history')
    entries.value = []
  } catch (e) {
    console.error('清空历史记录失败:', e)
  }
}

/**
 * 复制 URL 到剪贴板
 * 优先使用 Tauri 的原生剪贴板 API（跨平台、无需权限）
 * 降级使用浏览器 navigator.clipboard API
 *
 * @param {string} url - 要复制的图片 URL
 */
async function copyUrl(url) {
  try {
    // 调用 Rust 命令 "copy_to_clipboard"（在 commands.rs 中定义）
    await invoke('copy_to_clipboard', { text: url })
  } catch {
    // Rust 调用失败时降级到浏览器 API
    await navigator.clipboard?.writeText(url)
  }
}

/**
 * 在系统默认浏览器中打开 URL
 * 优先使用 Tauri shell 插件（调用系统浏览器），
 * 降级使用 window.open（在 WebView 内打开，体验较差）
 *
 * @param {string} url - 要打开的 URL
 */
function openUrl(url) {
  const s = window.__TAURI__?.shell ?? window.__TAURI__?.['@tauri-apps/plugin-shell']
  s?.open(url) ?? window.open(url, '_blank')
}

// ── 工具函数 ────────────────────────────────────────────────────────────────

/**
 * 判断 URL 是否是图片链接（用于决定显示缩略图还是文档图标）
 * 使用正则表达式匹配常见图片格式的文件扩展名
 * (\?.*)? 匹配可能存在的 URL 查询参数（如 ?v=1）
 */
function isImageUrl(url) {
  return /\.(avif|webp|png|jpg|jpeg|gif)(\?.*)?$/i.test(url)
}

/**
 * 将字节数格式化为人类可读的文件大小字符串
 * 例如：1234 → "1.2 KB"，1234567 → "1.18 MB"
 */
function formatSize(bytes) {
  if (!bytes) return '-'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1024 / 1024).toFixed(2) + ' MB'
}

/**
 * 将 Unix 时间戳（秒）格式化为相对时间或本地日期字符串
 * - 1 分钟内 → "刚刚"
 * - 1 小时内 → "X 分钟前"
 * - 24 小时内 → "X 小时前"
 * - 超过 24 小时 → "5月7日 14:30"
 *
 * @param {number} ts - Unix 时间戳（秒）
 */
function formatTime(ts) {
  if (!ts) return '-'
  const d = new Date(ts * 1000)  // 秒转毫秒
  const now = new Date()
  const diffMin = Math.floor((now - d) / 60000)  // 差多少分钟
  if (diffMin < 1)    return '刚刚'
  if (diffMin < 60)   return diffMin + ' 分钟前'
  if (diffMin < 1440) return Math.floor(diffMin / 60) + ' 小时前'
  // 超过24小时，显示本地化日期（月 日 时:分）
  return d.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

// ── 生命周期钩子 ─────────────────────────────────────────────────────────────

// 组件首次挂载时加载历史数据
onMounted(loadHistory)

// 从其他 Tab 切换回历史 Tab 时也重新加载（获取最新数据）
// onActivated 需要组件被包裹在 <keep-alive> 中才会触发
onActivated(loadHistory)
</script>
