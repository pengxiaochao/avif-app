<!--
  ToggleSwitch.vue — macOS 风格开关组件
  ========================================
  模仿 macOS 系统设置中的拨动开关（Toggle），用于控制布尔类型（是/否）的设置项。

  视觉效果（开启状态）：
    ╭────────╮
    │    ●   │   绿色背景，小白圆圈滑到右侧
    ╰────────╯
  视觉效果（关闭状态）：
    ╭────────╮
    │ ●      │   灰色背景，小白圆圈在左侧
    ╰────────╯

  使用示例：
    <ToggleSwitch label="自动转换 AVIF" hint="开启后上传前自动转换" v-model="config.convert_to_avif" />
-->
<template>
  <!--
    外层 label 标签：点击整个区域（包括文字）都可以切换开关
    cursor-pointer：显示手形光标，提示用户可点击
  -->
  <label class="flex items-center gap-3 cursor-pointer group">
    <!--
      开关轨道（背景条）：
      - 宽 40px（w-10），高 24px（h-6），圆角形
      - modelValue 为 true 时显示绿色（#34c759，macOS 系统绿）
      - modelValue 为 false 时显示灰色
      - transition-colors：颜色切换时有过渡动画（200ms）
    -->
    <div
      class="relative w-10 h-6 rounded-full transition-colors duration-200"
      :class="modelValue ? 'bg-[#34c759]' : 'bg-[#d1d1d6] dark:bg-[#3a3a3c]'"
    >
      <!--
        滑块（小白圆圈）：
        - absolute 绝对定位，浮在轨道上方
        - transition-transform：位移动画（200ms），让滑块平滑滑动
        - modelValue 为 true 时向右平移 18px（translate-x-[18px]）
        - modelValue 为 false 时在左侧（translate-x-0.5，即 2px 边距）
      -->
      <div
        class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md transition-transform duration-200"
        :class="modelValue ? 'translate-x-[18px]' : 'translate-x-0.5'"
      />

      <!--
        实际的 HTML 复选框（隐藏不显示，但用于接收用户点击事件）
        - sr-only：在视觉上隐藏，但屏幕阅读器仍可访问（无障碍支持）
        - :checked 绑定当前值
        - @change 监听变化，通过 $emit 通知父组件更新
      -->
      <input type="checkbox" class="sr-only" :checked="modelValue" @change="$emit('update:modelValue', $event.target.checked)" />
    </div>

    <!-- 右侧文字区域 -->
    <div>
      <!-- 开关标签文字 -->
      <span class="text-[13px] text-[#1d1d1f] dark:text-[#f2f2f7]">{{ label }}</span>
      <!-- 提示文字：只有传入 hint 时才显示 -->
      <p v-if="hint" class="text-[11px] text-[#6e6e73] dark:text-[#aeaeb2] leading-snug">{{ hint }}</p>
    </div>
  </label>
</template>

<script setup>
/**
 * Props 说明：
 * - label:      开关右侧的主标签文字，如"自动转换 AVIF"
 * - hint:       标签下方的小号提示文字（可选）
 * - modelValue: 开关的当前状态，Boolean 类型（true=开, false=关）
 *               这是 Vue 3 中 v-model 的标准属性名
 */
defineProps({ label: String, hint: String, modelValue: Boolean })

/**
 * 声明此组件发出的事件：
 * 'update:modelValue' 是 v-model 的标准事件名，
 * 当用户点击开关时，组件发出此事件，父组件的变量会自动更新为新的布尔值
 */
defineEmits(['update:modelValue'])
</script>
