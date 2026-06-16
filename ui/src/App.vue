<template>
  <!--
    App.vue — 根组件（整个应用的顶层容器）
    =========================================
    这里定义了应用的整体布局框架，采用 macOS 原生应用风格设计：

    整体布局（纵向三层结构）：
    ┌──────────────────────────────────────────────────────┐
    │  标题栏（40px，可拖拽移动窗口）                       │
    ├────────────┬─────────────────────────────────────────┤
    │            │                                         │
    │  侧边栏    │         内容区（动态 Tab 组件）          │
    │  (180px)   │                                         │
    │            │                                         │
    ├────────────┴─────────────────────────────────────────┤
    │  底部状态栏（48px，状态文字 + 保存配置按钮）          │
    └──────────────────────────────────────────────────────┘

    Tailwind CSS 说明（对新手）：
    - flex：启用 Flexbox 弹性布局，子元素横向排列
    - flex-col：让子元素纵向排列（覆盖默认横向）
    - h-screen：高度等于屏幕高度（100vh）
    - dark:xxx：当系统开启暗色模式时应用这个样式
    - bg-[#f5f5f7]：自定义背景色（macOS 浅灰色）
  -->
  <div class="flex flex-col h-screen bg-[#f5f5f7] dark:bg-[#1c1c1e] overflow-hidden select-none">

    <!-- ── 标题栏（40px，可拖拽移动窗口） ────────────────────────────────── -->
    <!--
      data-tauri-drag-region：Tauri 特有属性，标记此区域可拖拽移动窗口
      backdrop-blur-sm：毛玻璃效果，与 /80 透明度配合使用
    -->
    <div
      class="flex items-center justify-between px-4 h-10 shrink-0
             bg-[#f5f5f7]/80 dark:bg-[#1c1c1e]/80 backdrop-blur-sm
             border-b border-black/[0.06] dark:border-white/[0.06]"
      data-tauri-drag-region
    >
      <!--
        左侧空白区：预留给 macOS 的"红黄绿"交通灯按钮（关闭/最小化/全屏）
        宽度 70px 是根据 macOS 交通灯实际宽度估算的
      -->
      <div class="w-[70px]" data-tauri-drag-region />

      <!-- 居中应用标题 -->
      <span
        class="text-[13px] font-semibold text-[#1d1d1f] dark:text-[#f2f2f7] tracking-tight"
        data-tauri-drag-region
      >
        AVIF Uploader
      </span>

      <!-- 右侧等宽占位（保持标题居中） -->
      <div class="w-[70px]" data-tauri-drag-region />
    </div>

    <!-- ── 主体区域（侧边栏 + 内容区，横向排列） ──────────────────────────── -->
    <!--
      flex-1：占满剩余垂直空间（在标题栏和状态栏之间）
      overflow-hidden：防止内容溢出
    -->
    <div class="flex flex-1 overflow-hidden">

      <!-- ── 左侧边栏（固定 180px 宽） ── -->
      <aside class="w-[180px] shrink-0 flex flex-col border-r border-black/[0.06] dark:border-white/[0.06]
                    bg-[#f5f5f7] dark:bg-[#1c1c1e]">

        <!-- 导航菜单区（可滚动，防止菜单项过多时溢出） -->
        <nav class="flex-1 overflow-y-auto py-2 px-2">
          <!--
            v-for 循环渲染导航菜单项：
            - navItems 是下方 script 中定义的菜单配置数组
            - :key="item.id" 给每个菜单项唯一标识，Vue 用来优化 DOM 更新
            - :active="activeTab === item.id" 判断是否高亮（当前选中的 Tab）
            - @click="activeTab = item.id" 点击时切换当前 Tab
          -->
          <NavItem
            v-for="item in navItems"
            :key="item.id"
            :label="item.label"
            :icon="item.icon"
            :active="activeTab === item.id"
            @click="activeTab = item.id"
          />
        </nav>

        <!-- 侧边栏底部：三个上传操作按钮（固定在侧栏底部） -->
        <div class="p-3 space-y-2 border-t border-black/[0.06] dark:border-white/[0.06]">

          <!--
            粘贴上传按钮（蓝色主按钮）：
            - :disabled="uploading" 上传中时禁用按钮，防止重复点击
            - 按钮文字在上传中/空闲状态间切换：三元运算符 ? :
          -->
          <button
            class="w-full flex items-center justify-center gap-1.5 h-8 rounded-lg
                   bg-[#0071e3] dark:bg-[#0a84ff] hover:opacity-90 active:opacity-80
                   text-white text-[12.5px] font-medium transition-opacity"
            :disabled="uploading"
            @click="doClipboardUpload"
          >
            <IconClipboard class="w-3.5 h-3.5" />
            <!-- 上传中显示"上传中…"，否则显示"粘贴上传" -->
            <span>{{ uploading ? '上传中…' : '粘贴上传' }}</span>
          </button>

          <!-- 选择文件上传按钮（次级白色按钮） -->
          <button
            class="w-full flex items-center justify-center gap-1.5 h-8 rounded-lg
                   bg-white dark:bg-[#3a3a3c] hover:bg-black/5 dark:hover:bg-white/10
                   active:bg-black/10 dark:active:bg-white/15
                   text-[#1d1d1f] dark:text-[#f2f2f7] text-[12.5px] font-medium
                   border border-black/10 dark:border-white/10 transition-colors
                   disabled:opacity-40 disabled:cursor-not-allowed"
            :disabled="uploading"
            @click="doFileUpload"
          >
            <IconFolder class="w-3.5 h-3.5" />
            <span>选择文件</span>
          </button>

          <!-- 选择目录上传按钮（批量上传整个文件夹） -->
          <button
            class="w-full flex items-center justify-center gap-1.5 h-8 rounded-lg
                   bg-white dark:bg-[#3a3a3c] hover:bg-black/5 dark:hover:bg-white/10
                   active:bg-black/10 dark:active:bg-white/15
                   text-[#1d1d1f] dark:text-[#f2f2f7] text-[12.5px] font-medium
                   border border-black/10 dark:border-white/10 transition-colors
                   disabled:opacity-40 disabled:cursor-not-allowed"
            :disabled="uploading"
            @click="doDirectoryUpload"
          >
            <!-- 内联 SVG 文件夹图标 -->
            <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V7z" />
            </svg>
            <span>选择目录</span>
          </button>
        </div>
      </aside>

      <!-- ── 右侧内容区（动态 Tab 组件） ── -->
      <!--
        <component :is="currentTabComponent"> 是 Vue 的"动态组件"语法。
        根据 currentTabComponent 计算属性返回的不同组件（TabS3/TabUpload/...），
        动态渲染对应的 Tab 页面内容——相当于一个"组件切换器"。

        :config="config" 将配置对象传入子组件（子组件通过 props 接收）
        @update:config="onConfigUpdate" 监听子组件发出的配置更新事件
      -->
      <main class="flex-1 overflow-y-auto">
        <component :is="currentTabComponent" :config="config" @update:config="onConfigUpdate" />
      </main>
    </div>

    <!-- ── 底部状态栏（固定 48px 高） ──────────────────────────────────── -->
    <footer class="flex items-center justify-between px-4 h-12 shrink-0
                   border-t border-black/[0.06] dark:border-white/[0.06]
                   bg-[#f5f5f7] dark:bg-[#1c1c1e]">
      <!--
        状态文字：显示当前操作状态，如"就绪"、"正在上传…"、"上传成功：xxx.avif"
        truncate：文字过长时末尾显示省略号，max-w-[55%] 防止文字覆盖右侧按钮
      -->
      <span class="text-[12px] text-[#6e6e73] dark:text-[#aeaeb2] truncate max-w-[55%]">
        {{ statusText }}
      </span>
      <!-- 保存配置按钮：将当前所有设置持久化到磁盘 -->
      <button
        class="px-4 h-7 rounded-lg bg-[#0071e3] dark:bg-[#0a84ff] hover:opacity-90 active:opacity-80
               text-white text-[12.5px] font-medium transition-opacity"
        @click="saveConfig"
      >
        保存配置
      </button>
    </footer>

    <!--
      Toast 轻提示（类似手机 App 底部弹出的小通知条）：
      <Transition name="toast"> 是 Vue 内置的过渡动画组件，
      配合下方 <style> 中的 .toast-enter/leave 类实现淡入淡出效果。

      v-if="toast.visible" 控制显示/隐藏
      :class 根据 toast.type 决定背景色（error=红色，info=深色）
      pointer-events-none：Toast 不接收鼠标点击，防止遮挡背后的按钮
    -->
    <Transition name="toast">
      <div
        v-if="toast.visible"
        class="fixed bottom-16 left-1/2 -translate-x-1/2 px-4 py-2 rounded-xl
               text-white text-[13px] font-medium shadow-lg z-50
               pointer-events-none"
        :class="toast.type === 'error' ? 'bg-red-500' : 'bg-[#1d1d1f] dark:bg-[#3a3a3c]'"
      >
        {{ toast.message }}
      </div>
    </Transition>
  </div>
</template>

<script setup>
/**
 * App.vue <script setup> — 根组件的逻辑层
 * ==========================================
 *
 * Vue 3 使用 <script setup> 语法（Composition API）。
 * 与旧版 Options API 不同，这里没有 data()、methods、mounted 等固定分区，
 * 而是自由地声明响应式状态、计算属性和函数，代码更灵活直观。
 *
 * 关键概念速览：
 * - ref(初始值)     ：创建一个响应式变量。变量值变化时，界面自动更新。
 *                     在 script 中用 变量.value 访问/修改；在 template 中直接写变量名。
 * - computed(函数)  ：创建"派生值"——根据其他响应式变量计算出的值，自动缓存。
 * - onMounted(函数) ：组件挂载到 DOM 后执行一次（类似其他框架的"初始化"）。
 * - shallowRef      ：与 ref 类似，但不对对象内部属性做深层响应式处理（性能更好）。
 */
import { ref, computed, onMounted, shallowRef } from 'vue'

// ── 导入子组件 ──────────────────────────────────────────────────────────────
import NavItem from './components/NavItem.vue'
import TabS3 from './components/TabS3.vue'
import TabUpload from './components/TabUpload.vue'
import TabLink from './components/TabLink.vue'
import TabShortcut from './components/TabShortcut.vue'
import TabHistory from './components/TabHistory.vue'
import IconClipboard from './components/icons/IconClipboard.vue'
import IconFolder from './components/icons/IconFolder.vue'

// ── Tauri API 访问工具函数（兼容开发模式） ──────────────────────────────────

/**
 * 获取 Tauri 核心 API 对象（Tauri v1 / v2 双版本兼容）
 * 在浏览器开发环境（npm run dev）中 window.__TAURI__ 不存在，返回 undefined 也不会报错
 */
const tauri = () => window.__TAURI__?.core ?? window.__TAURI__

/**
 * invoke — 调用 Rust 后端命令的核心函数
 *
 * Tauri 的核心机制：前端通过 invoke() 向 Rust 发送命令请求，
 * Rust 处理后返回结果。整个过程类似调用一个异步函数，但实际执行发生在 Rust 端。
 *
 * @param {string} cmd  - Rust 命令名称，对应 commands.rs 中 #[tauri::command] 函数名
 * @param {object} args - 传给 Rust 函数的参数（自动序列化为 JSON）
 * @returns {Promise}   - Rust 函数的返回值（自动从 JSON 反序列化）
 *
 * 使用示例：
 *   const config = await invoke('get_config')        // 无参调用
 *   await invoke('save_config', { cfg: config })      // 带参调用
 */
const invoke = (...args) => {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke(...args)
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke(...args)
  // 在浏览器开发环境中，打印警告并返回空对象（防止报错）
  console.warn('[invoke mock]', ...args)
  return Promise.resolve({})
}

/**
 * openFileDialog — 调用系统原生文件选择对话框
 * 让用户选择任意单个文件，返回所选文件的本地路径字符串
 * 用户取消选择时返回 null
 */
const openFileDialog = () => {
  const d = window.__TAURI__?.dialog ?? window.__TAURI__?.['@tauri-apps/plugin-dialog']
  return d?.open({ multiple: false })
    ?? Promise.resolve(null)
}

/**
 * openDirectoryDialog — 调用系统原生文件夹选择对话框
 * 让用户选择一个目录，返回目录的本地路径字符串
 * { directory: true } 参数告知对话框只允许选择目录而非文件
 */
const openDirectoryDialog = () => {
  const d = window.__TAURI__?.dialog ?? window.__TAURI__?.['@tauri-apps/plugin-dialog']
  return d?.open({ directory: true, multiple: false })
    ?? Promise.resolve(null)
}

/**
 * listenEvent — 订阅来自 Rust 后端的事件
 *
 * Tauri 支持 Rust → 前端的单向"事件推送"机制：
 * Rust 代码调用 app.emit("event-name", payload) 发出事件，
 * 前端通过 listenEvent("event-name", callback) 注册监听器接收通知。
 *
 * 这种机制适用于长时间运行的操作（如上传进度），Rust 可以主动通知前端更新界面。
 *
 * @param {string}   event - 事件名称（与 Rust emit 的名称一致）
 * @param {function} cb    - 收到事件时调用的回调函数，参数 ev.payload 是事件附带的数据
 */
const listenEvent = (event, cb) => {
  const e = window.__TAURI__?.event ?? window.__TAURI__?.['@tauri-apps/api/event']
  return e?.listen(event, cb) ?? Promise.resolve(() => {})
}

// ── 导航菜单数据定义 ─────────────────────────────────────────────────────────

/**
 * navItems：侧边栏导航菜单的配置数组
 * 每项定义了一个 Tab 的：id（内部标识）、label（显示文字）、icon（图标名）
 * NavItem 组件会根据 icon 字段渲染对应的 SVG 图标
 */
const navItems = [
  { id: 's3',       label: 'S3 配置',  icon: 'server'   },
  { id: 'upload',   label: '上传设置', icon: 'upload'   },
  { id: 'link',     label: '链接格式', icon: 'link'     },
  { id: 'shortcut', label: '快捷键',   icon: 'keyboard' },
  { id: 'history',  label: '历史记录', icon: 'history'  },
]

// ── 响应式状态定义 ────────────────────────────────────────────────────────────

/** 当前激活的 Tab ID，初始为 's3'（S3配置页） */
const activeTab = ref('s3')

/** 是否正在上传（true 时禁用上传按钮，防止重复触发） */
const uploading = ref(false)

/** 底部状态栏的文字（如"就绪"、"正在上传…"、"上传成功：xxx.avif"） */
const statusText = ref('就绪')

/** Toast 轻提示的状态：visible 控制显示/隐藏，message 是文字，type 影响背景色 */
const toast = ref({ visible: false, message: '', type: 'info' })

/**
 * Tab 组件映射表：将 Tab ID 映射到对应的 Vue 组件
 * 配合 computed 实现动态渲染，切换 activeTab 时自动切换显示的组件
 */
const tabComponentMap = {
  s3: TabS3,
  upload: TabUpload,
  link: TabLink,
  shortcut: TabShortcut,
  history: TabHistory,
}

/**
 * 计算属性：根据当前 activeTab 返回对应的组件
 * 例如 activeTab.value 为 'upload' 时，返回 TabUpload 组件
 * template 中的 <component :is="currentTabComponent"> 会渲染这个组件
 */
const currentTabComponent = computed(() => tabComponentMap[activeTab.value])

/**
 * 全局配置对象（响应式）：
 * 这是整个应用的"唯一数据源"，所有 Tab 组件都通过 :config 属性接收它。
 * 初始值是默认值，onMounted 时会从 Rust 后端加载真实配置覆盖这些默认值。
 *
 * 结构说明：
 * - s3：S3 连接信息（bucket、region、key、secret 等）
 * - upload：上传行为设置（路径格式、AVIF 转换参数等）
 * - shortcut：全局快捷键字符串（Tauri 格式，如 "CmdOrCtrl+Shift+U"）
 * - autostart：是否开机自启
 */
const config = ref({
  s3: {
    bucket: '', region: 'us-east-1', access_key_id: '',
    secret_access_key: '', endpoint_url: '', path_style: false,
  },
  upload: {
    key_prefix: '', dir_format: '{Y}/{m}/{d}', filename_format: '{uuid8}',
    url_prefix: '', convert_to_avif: true, avif_quality: 80, avif_speed: 8,
    link_format: 'Url', custom_link_template: '![{name}]({url})', auto_copy: true,
  },
  shortcut: 'CmdOrCtrl+Shift+U',
  autostart: false,
})

// ── 生命周期钩子 ─────────────────────────────────────────────────────────────

/**
 * onMounted：组件第一次渲染到页面后执行（整个应用启动时执行一次）
 * 在这里完成三件事：
 * 1. 从 Rust 后端加载持久化的配置
 * 2. 注册 Rust 事件监听器（接收上传进度通知）
 * 3. 暴露全局函数供 Rust 调用（用于快捷键触发时切换 Tab）
 */
onMounted(async () => {
  // ① 加载配置：调用 Rust 命令 "get_config"，用真实配置覆盖默认值
  try {
    const cfg = await invoke('get_config')
    config.value = cfg   // 更新响应式数据，界面自动刷新为真实配置
  } catch (e) {
    showToast('加载配置失败: ' + e, 'error')
  }

  // ② 订阅 Rust 事件：Rust 在执行全局快捷键触发的上传时，会 emit 这些事件通知前端
  //    前端收到事件后更新界面状态

  // "upload-start" 事件：Rust 开始上传时发出
  await listenEvent('upload-start', () => {
    statusText.value = '正在上传…'
    uploading.value = true
  })

  // "upload-ok" 事件：Rust 上传成功时发出，payload 包含文件名等信息
  await listenEvent('upload-ok', (ev) => {
    uploading.value = false
    statusText.value = '上传成功：' + (ev.payload?.filename ?? '')
    showToast('已复制链接 ' + (ev.payload?.filename ?? ''), 'info')
    // 上传成功后历史记录已自动更新，切换到历史 Tab 时会重新加载
  })

  // "upload-err" 事件：Rust 上传失败时发出，payload 是错误信息字符串
  await listenEvent('upload-err', (ev) => {
    uploading.value = false
    statusText.value = '上传失败'
    showToast(ev.payload ?? '上传失败', 'error')
  })

  // ③ 暴露全局函数：挂载到 window 对象上，让 Rust 可以通过 WebView 的 eval() 调用
  //    例如：Rust 在快捷键触发后可以调用 window.switchTab('history') 切换到历史页
  window.switchTab = (id) => { activeTab.value = id }
})

// ── 事件处理方法 ──────────────────────────────────────────────────────────────

/**
 * onConfigUpdate：子组件（各 Tab 页）配置变更时的回调
 * 各 Tab 组件通过 emit('update:config', newCfg) 触发此函数，
 * 这里统一更新根组件的 config，保持全局状态同步。
 *
 * @param {object} newCfg - 子组件传来的最新完整配置对象
 */
function onConfigUpdate(newCfg) {
  config.value = newCfg
}

/**
 * saveConfig：保存配置到磁盘
 * 调用 Rust 命令 "save_config"，Rust 将配置序列化为 JSON 并写入应用数据目录。
 * 同时如果快捷键有变化，Rust 会自动重新注册全局快捷键。
 */
async function saveConfig() {
  try {
    // { cfg: config.value } 是传给 Rust 的参数，Rust 命令签名为 save_config(cfg: Config)
    await invoke('save_config', { cfg: config.value })
    showToast('配置已保存', 'info')
    statusText.value = '配置已保存'
  } catch (e) {
    showToast('保存失败: ' + e, 'error')
  }
}

/**
 * doClipboardUpload：粘贴上传
 * 读取系统剪贴板中的图片，转换为 AVIF 后上传到 S3。
 * 上传过程完全在 Rust 端执行，前端只负责触发和显示结果。
 */
async function doClipboardUpload() {
  if (uploading.value) return   // 防止重复上传
  uploading.value = true
  statusText.value = '正在上传剪贴板内容…'
  try {
    // invoke 是异步的，await 等待 Rust 处理完成后继续执行
    // result 是 Rust UploadResult 结构体，包含 filename、url、link 等字段
    const result = await invoke('upload_clipboard')
    statusText.value = '上传成功：' + result.filename
    showToast('✓ 已复制链接', 'info')
  } catch (e) {
    statusText.value = '上传失败'
    showToast(String(e), 'error')
  } finally {
    // finally 块无论成功失败都会执行，确保 uploading 状态被重置
    uploading.value = false
  }
}

/**
 * doFileUpload：选择文件上传
 * 先弹出系统文件选择对话框让用户选文件，再将路径传给 Rust 执行上传。
 */
async function doFileUpload() {
  if (uploading.value) return
  // 调用 Tauri 对话框 API 弹出文件选择窗口，用户取消时返回 null
  const filePath = await openFileDialog()
  if (!filePath) return   // 用户取消了，直接返回不做任何事

  uploading.value = true
  statusText.value = '正在上传文件…'
  try {
    // 将文件路径传给 Rust，Rust 读取文件、转换格式、上传到 S3
    const result = await invoke('upload_file', { path: filePath })
    statusText.value = '上传成功：' + result.filename
    showToast('✓ 已复制链接', 'info')
  } catch (e) {
    statusText.value = '上传失败'
    showToast(String(e), 'error')
  } finally {
    uploading.value = false
  }
}

/**
 * doDirectoryUpload：批量上传目录
 * 选择一个文件夹，Rust 会递归扫描其中的所有图片文件并批量上传。
 * 上传完成后自动跳转到历史记录 Tab 查看结果。
 */
async function doDirectoryUpload() {
  if (uploading.value) return
  // 弹出目录选择对话框
  const dirPath = await openDirectoryDialog()
  if (!dirPath) return

  uploading.value = true
  statusText.value = '正在扫描目录…'
  try {
    // Rust 返回的结果包含 succeeded（成功列表）和 failed_count（失败数量）
    const result = await invoke('upload_directory', { path: dirPath })
    const ok = result.succeeded.length    // 成功上传的文件数
    const fail = result.failed_count      // 上传失败的文件数

    if (fail === 0) {
      statusText.value = `目录上传完成：${ok} 个文件`
      showToast(`✓ 已上传 ${ok} 个文件`, 'info')
    } else {
      statusText.value = `目录上传完成：${ok} 成功，${fail} 失败`
      showToast(`上传完成：${ok} 成功，${fail} 失败`, 'error')
    }

    // 有成功上传的文件时，自动切换到历史记录 Tab
    if (ok > 0) activeTab.value = 'history'
  } catch (e) {
    statusText.value = '目录上传失败'
    showToast(String(e), 'error')
  } finally {
    uploading.value = false
  }
}

// ── Toast 提示工具函数 ────────────────────────────────────────────────────────

/** 用于存储 Toast 定时器，clearTimeout 防止多次调用时 Toast 提前消失 */
let toastTimer = null

/**
 * showToast：显示底部轻提示条
 * Toast 会在 2.8 秒后自动消失
 *
 * @param {string} message - 提示文字内容
 * @param {string} type    - 类型：'info'（深色背景）或 'error'（红色背景）
 */
function showToast(message, type = 'info') {
  toast.value = { visible: true, message, type }
  // 清除上一个定时器（防止多次调用时之前的定时器提前隐藏新的 Toast）
  clearTimeout(toastTimer)
  // 2800ms 后自动隐藏 Toast
  toastTimer = setTimeout(() => { toast.value.visible = false }, 2800)
}
</script>

<style scoped>
/*
  Toast 提示条的进出场动画
  ========================
  Vue <Transition name="toast"> 组件在元素出现/消失时，会自动给元素添加
  以下 CSS 类。我们在这里定义各阶段的样式，Vue 负责在正确时机添加/移除这些类。

  动画阶段：
  - .toast-enter-active / .toast-leave-active：分别在进场/离场期间持续存在，
    用于声明 transition（过渡时间和缓动函数）
  - .toast-enter-from / .toast-leave-to：分别是进场开始和离场结束的"极端状态"
    （不可见 + 向下偏移），配合上方的 transition 产生滑入/滑出效果
*/

/* 进/离场过渡动画（0.25秒，ease 缓动） */
.toast-enter-active, .toast-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}

/* 进场起始 & 离场结束：完全透明 + 向下偏移 8px（产生从下滑入/向下滑出的视觉效果） */
.toast-enter-from, .toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(8px);
}
</style>
