<!--
  NavItem.vue — 左侧导航栏单个菜单项组件
  =========================================
  这是侧边栏导航菜单中的每一个条目（如"S3 配置"、"上传设置"等）。

  视觉效果：
  ┌──────────────────────┐
  │  🔌  S3 配置          │  ← 未选中：透明背景，深色文字
  └──────────────────────┘
  ┌──────────────────────┐
  │  📤  上传设置         │  ← 选中：蓝色背景，白色文字
  └──────────────────────┘

  使用示例：
    <NavItem label="S3 配置" icon="server" :active="true" @click="activeTab = 's3'" />
-->
<template>
  <!--
    按钮容器：
    - 当 active 为 true（此项被选中）时：蓝色背景 + 白色文字
    - 当 active 为 false（未选中）时：透明背景 + 深色文字，鼠标悬停时有淡灰背景
    - cursor-default 使用默认箭头光标（而不是手形），模仿原生 macOS 应用风格
    - @click 点击时向父组件发出 'click' 事件，父组件据此切换 activeTab
  -->
  <button
    class="flex items-center gap-2 w-full px-2.5 py-1.5 rounded-lg text-left
           text-[13px] transition-colors duration-100 cursor-default"
    :class="active
      ? 'bg-[#0071e3] dark:bg-[#0a84ff] text-white'
      : 'text-[#1d1d1f] dark:text-[#f2f2f7] hover:bg-black/6 dark:hover:bg-white/8'"
    @click="$emit('click')"
  >
    <!--
      动态图标：
      <component :is="iconComponent"> 是 Vue 的动态组件语法。
      iconComponent 是一个计算属性，根据 icon 字符串（如 "server"）查找对应的图标组件并渲染。
      这样就不用写 v-if/v-else 判断每种图标了。
    -->
    <component :is="iconComponent" class="w-3.5 h-3.5 shrink-0" />
    <!-- 菜单文字标签 -->
    <span>{{ label }}</span>
  </button>
</template>

<script setup>
import { computed } from 'vue'
// 引入所有可用的图标组件
import IconServer from './icons/IconServer.vue'
import IconUpload from './icons/IconUpload.vue'
import IconLink from './icons/IconLink.vue'
import IconKeyboard from './icons/IconKeyboard.vue'
import IconHistory from './icons/IconHistory.vue'

/**
 * Props 说明：
 * - label：显示在按钮上的文字，如"S3 配置"
 * - icon：图标名称字符串，如 "server"、"upload"，对应下方 iconMap 中的键
 * - active：是否高亮（蓝色背景），由父组件根据当前选中的 Tab 控制
 */
const props = defineProps({
  label: String,
  icon: String,
  active: Boolean,
})

// 声明此组件会向外发出的事件：点击时通知父组件
defineEmits(['click'])

/**
 * 图标映射表：将字符串名称映射到实际的 Vue 图标组件
 * 这样父组件只需传 icon="server"，不用关心具体用哪个组件
 */
const iconMap = {
  server:   IconServer,
  upload:   IconUpload,
  link:     IconLink,
  keyboard: IconKeyboard,
  history:  IconHistory,
}

/**
 * computed（计算属性）：根据 icon 字符串动态返回对应的组件
 * 每当 props.icon 变化时，Vue 会自动重新计算这个值
 */
const iconComponent = computed(() => iconMap[props.icon])
</script>
