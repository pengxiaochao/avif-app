<!--
  TabShortcut.vue — 快捷键与系统设置标签页
  ===========================================
  提供两个系统级功能的配置：
  1. 全局快捷键：在任何应用中都能触发"粘贴上传"的组合键（如 ⌘⇧U）
  2. 开机自启：控制应用是否随操作系统开机自动启动

  快捷键录制逻辑：
  - 用户点击输入框 → 进入"录制模式"（输入框出现蓝色高亮）
  - 用户按下组合键 → 捕获按键事件，解析出修饰键 + 主键
  - 自动转换成 Tauri 格式（如 CmdOrCtrl+Shift+U）并保存
  - 展示时再转换成可读符号（如 ⌘⇧U）

  页面布局（两张卡片）：
  ┌──────────────────────────────────────────────┐
  │ 全局快捷键                                    │
  │  [⌘⇧U                         ] [清除]        │
  │  点击输入框后按下快捷键组合开始录制           │
  └──────────────────────────────────────────────┘
  ┌──────────────────────────────────────────────┐
  │ 系统启动                                      │
  │  ○ 开机自动启动                              │
  └──────────────────────────────────────────────┘
-->
<template>
  <div class="p-5 space-y-4">

    <!-- ── 第一张卡片：全局快捷键 ── -->
    <FormSection title="全局快捷键" desc="在任意应用中触发「粘贴上传」功能">
      <div class="space-y-3">
        <div class="flex gap-2">
          <!--
            快捷键录制输入框：
            - ref="shortcutInput"：获取此 DOM 元素的引用，以便调用 .blur() 等方法
            - readonly：用户不能直接输入文字，只能通过按键捕获
            - :value="displayShortcut"：显示转换后的可读符号（如 ⌘⇧U）
            - :class 动态绑定：录制中时添加蓝色发光边框
            - @click：点击时进入录制模式
            - @keydown.prevent：捕获键盘事件，.prevent 阻止浏览器默认行为（如空格滚动页面）
            - @blur：失去焦点时退出录制模式
          -->
          <input
            ref="shortcutInput"
            readonly
            :value="displayShortcut"
            placeholder="点击后按下快捷键组合…"
            class="flex-1 h-9 px-3 rounded-lg text-[13px]
                   bg-[#f5f5f7] dark:bg-[#3a3a3c]
                   border border-black/10 dark:border-white/10
                   text-[#1d1d1f] dark:text-[#f2f2f7]
                   placeholder:text-[#6e6e73] dark:placeholder:text-[#636366]
                   focus:outline-none focus:ring-2 focus:ring-[#0071e3]/40 dark:focus:ring-[#0a84ff]/40
                   cursor-pointer transition-shadow"
            :class="recording ? 'ring-2 ring-[#0071e3]/50 dark:ring-[#0a84ff]/50' : ''"
            @click="startRecording"
            @keydown.prevent="onKeyDown"
            @blur="recording = false"
          />
          <!-- 清除按钮：清空当前快捷键设置 -->
          <button
            class="px-3 h-9 rounded-lg text-[12.5px] font-medium
                   bg-white dark:bg-[#3a3a3c] hover:bg-black/5 dark:hover:bg-white/10
                   border border-black/10 dark:border-white/10
                   text-[#1d1d1f] dark:text-[#f2f2f7] transition-colors"
            @click="clearShortcut"
          >
            清除
          </button>
        </div>
        <!--
          状态提示文字：根据录制状态动态显示不同文字
          recording 为 true 时提示"正在录制"，否则提示操作方法
        -->
        <p class="text-[11px] text-[#6e6e73] dark:text-[#aeaeb2]">
          {{ recording ? '正在录制，请按下快捷键组合（如 ⌘⇧U）…' : '点击输入框后按下快捷键组合开始录制' }}
        </p>
      </div>
    </FormSection>

    <!-- ── 第二张卡片：开机启动控制 ── -->
    <FormSection title="系统启动" desc="控制应用随系统自动启动">
      <!--
        开机自启开关：
        v-model="autostartEnabled" 双向绑定到本地 ref
        @update:modelValue="toggleAutostart" 值变化时调用 Tauri 插件的 enable/disable API
      -->
      <ToggleSwitch
        label="开机自动启动"
        hint="应用将在登录时自动运行并出现在菜单栏"
        v-model="autostartEnabled"
        @update:modelValue="toggleAutostart"
      />
    </FormSection>
  </div>
</template>

<script setup>
// ref：创建单个值的响应式引用
// computed：创建根据其他响应式数据计算得出的派生值
// onMounted：组件挂载到 DOM 后执行的生命周期钩子
import { ref, computed, onMounted } from 'vue'
import FormSection from './FormSection.vue'
import ToggleSwitch from './ToggleSwitch.vue'

const props = defineProps({ config: Object })
const emit = defineEmits(['update:config'])

// ── 响应式状态定义 ──────────────────────────────────────────────────────────

// recording：是否处于快捷键录制模式（true=正在录制）
const recording = ref(false)

// autostartEnabled：开机自启是否开启（从 Tauri 插件读取初始值）
const autostartEnabled = ref(Boolean(props.config.autostart))

// shortcutInput：输入框的 DOM 元素引用，用于调用 .blur() 失去焦点
const shortcutInput = ref(null)

// 当前快捷键的 Tauri 格式字符串，如 "CmdOrCtrl+Shift+U"
const localShortcut = ref(props.config.shortcut ?? '')

const getAutostartPlugin = () =>
  window.__TAURI__?.autostart
  ?? window.__TAURI__?.['@tauri-apps/plugin-autostart']
  ?? window.__TAURI__?.['tauri-plugin-autostart']

/**
 * 计算属性：将 Tauri 格式的快捷键字符串转换为用户友好的可读符号
 *
 * 例如：
 * "CmdOrCtrl+Shift+U" → "⌘⇧U"
 * "Ctrl+Alt+S"        → "⌃⌥S"
 *
 * computed 的值会自动缓存，只有依赖的响应式数据（localShortcut）变化时才重新计算
 */
const displayShortcut = computed(() => {
  if (!localShortcut.value) return ''
  return localShortcut.value
    .replace('CmdOrCtrl', '⌘')   // macOS Command 键
    .replace('Command', '⌘')
    .replace('Ctrl', '⌃')        // Control 键
    .replace('Alt', '⌥')         // Option/Alt 键
    .replace('Option', '⌥')
    .replace('Shift', '⇧')       // Shift 键
    .replace(/\+/g, '')          // 去掉所有加号，符号紧挨在一起更美观
})

/**
 * 生命周期钩子：组件首次挂载到页面后执行
 * 这里用于从 Tauri autostart 插件读取当前开机自启的状态
 */
onMounted(async () => {
  try {
    // 先使用配置值，随后再以系统真实状态为准（若插件可用）
    autostartEnabled.value = Boolean(props.config.autostart)

    const plugin = getAutostartPlugin()
    if (plugin?.isEnabled) {
      const enabled = await plugin.isEnabled()
      autostartEnabled.value = enabled
      if (enabled !== Boolean(props.config.autostart)) {
        emit('update:config', { ...props.config, autostart: enabled })
      }
    }
  } catch {} // 忽略错误（如在非 Tauri 环境运行时）
})

// ── 快捷键录制相关方法 ───────────────────────────────────────────────────────

/** 点击输入框时进入录制模式 */
function startRecording() {
  recording.value = true
}

/**
 * 键盘按下事件处理：捕获用户按下的组合键，转换为 Tauri 格式的快捷键字符串
 *
 * @param {KeyboardEvent} e - 浏览器键盘事件对象
 *   e.metaKey  = Command 键（macOS）
 *   e.ctrlKey  = Ctrl 键
 *   e.altKey   = Alt/Option 键
 *   e.shiftKey = Shift 键
 *   e.key      = 主键名称，如 "u"、"Enter"、" "（空格）
 */
function onKeyDown(e) {
  if (!recording.value) return

  // 如果单独按下修饰键，不做处理（必须有主键才算有效快捷键）
  const modifiers = ['Control', 'Meta', 'Alt', 'Shift']
  if (modifiers.includes(e.key)) return

  // 按顺序收集修饰键（Tauri 要求的格式：修饰键 + 主键，用 + 连接）
  const parts = []
  if (e.metaKey)       parts.push('CmdOrCtrl')  // macOS ⌘ 映射为 CmdOrCtrl
  else if (e.ctrlKey)  parts.push('Ctrl')
  if (e.altKey)        parts.push('Alt')
  if (e.shiftKey)      parts.push('Shift')

  // 将主键名转换为 Tauri 格式
  let key = e.key.toUpperCase()
  if (key === ' ')           key = 'Space'
  else if (key === 'ESCAPE') key = 'Escape'
  else if (key === 'ENTER')  key = 'Return'
  else if (key === 'BACKSPACE') key = 'Backspace'

  // 不允许无修饰键的快捷键（防止占用普通按键）
  if (parts.length === 0) return
  parts.push(key)

  // 拼合成最终的快捷键字符串，如 "CmdOrCtrl+Shift+U"
  const newShortcut = parts.join('+')
  localShortcut.value = newShortcut

  // 录制完成，退出录制模式
  recording.value = false
  shortcutInput.value?.blur()  // 让输入框失去焦点，隐藏蓝色边框

  // 通知父组件更新快捷键配置
  emit('update:config', { ...props.config, shortcut: newShortcut })
}

/** 清除按钮：清空快捷键设置 */
function clearShortcut() {
  localShortcut.value = ''
  recording.value = false
  emit('update:config', { ...props.config, shortcut: '' })
}

/**
 * 开机自启切换：调用 Tauri autostart 插件的 API 实际控制系统级开机启动
 *
 * @param {boolean} enabled - 新的开关状态（true=开启，false=关闭）
 */
async function toggleAutostart(enabled) {
  // 与其他配置项一致，开关变化后先写回配置对象，交给父组件统一保存
  emit('update:config', { ...props.config, autostart: enabled })

  try {
    // 访问 Tauri autostart 插件（如果插件未安装或在浏览器环境下会跳过）
    const plugin = getAutostartPlugin()
    if (enabled) {
      await plugin?.enable?.()   // 将应用添加到系统登录启动项
    } else {
      await plugin?.disable?.()  // 从系统登录启动项移除
    }
  } catch (e) {
    console.warn('切换开机启动失败:', e)
  }
}
</script>
