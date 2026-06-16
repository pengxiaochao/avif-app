<!--
  FormField.vue — 通用表单输入框组件
  =====================================
  这是一个可复用的"带标签的输入框"组件，页面里所有需要用户填写文字的地方都用它。

  包含三个视觉部分：
  ┌──────────────────────────────────────────┐
  │ 标签文字  （可选）                        │  ← label 区域
  │ ┌──────────────────────────────────────┐  │
  │ │  用户输入的文字...                   │  │  ← input 输入框
  │ └──────────────────────────────────────┘  │
  │ 提示说明文字（灰色小字）                  │  ← hint 区域（可选）
  └──────────────────────────────────────────┘

  使用示例：
    <FormField label="Bucket 名称" v-model="config.bucket" hint="填写 S3 的 Bucket 名" />
-->
<template>
  <!-- 外层容器，space-y-1 表示内部子元素之间有 4px 的垂直间距 -->
  <div class="space-y-1">
    <!-- 标签行：显示字段名称，如果 optional=true 则在右侧加"（可选）"提示 -->
    <label class="flex items-center gap-1.5 text-[12px] font-medium text-[#1d1d1f] dark:text-[#f2f2f7]">
      <!-- slot 是插槽：允许父组件传入自定义 HTML 内容；没传时默认显示 label 文字 -->
      <slot name="label">{{ label }}</slot>
      <!-- v-if 条件渲染：只有 optional 属性为 true 时才显示"（可选）"字样 -->
      <span v-if="optional" class="text-[11px] font-normal text-[#6e6e73] dark:text-[#aeaeb2]">（可选）</span>
    </label>

    <!--
      输入框：
      - v-bind="$attrs" 将父组件透传的属性（如 type="password", placeholder）全部应用到 input 上
      - :value="modelValue" 是单向绑定，将父组件传入的值显示在输入框中
      - @input 监听用户输入事件，每次按键后通过 $emit 通知父组件更新值（这是 Vue 双向绑定 v-model 的底层机制）
    -->
    <input
      v-bind="$attrs"
      class="w-full h-8 px-3 rounded-lg text-[13px]
             bg-[#f5f5f7] dark:bg-[#3a3a3c]
             border border-black/10 dark:border-white/10
             text-[#1d1d1f] dark:text-[#f2f2f7]
             placeholder:text-[#6e6e73] dark:placeholder:text-[#636366]
             focus:outline-none focus:ring-2 focus:ring-[#0071e3]/40 dark:focus:ring-[#0a84ff]/40
             transition-shadow"
      :value="modelValue"
      @input="$emit('update:modelValue', $event.target.value)"
    />

    <!-- 提示文字：只有传入 hint 属性时才显示，用于给用户提供填写说明 -->
    <p v-if="hint" class="text-[11px] text-[#6e6e73] dark:text-[#aeaeb2] leading-snug">{{ hint }}</p>
  </div>
</template>

<script setup>
/**
 * defineProps 声明这个组件接受哪些"从外部传入的参数"（称为 Props）
 *
 * - label:      输入框左上方的标签文字，如"Bucket 名称"
 * - modelValue: 输入框当前的值（配合 v-model 使用，这是 Vue 3 双向绑定的标准命名）
 * - optional:   是否显示"（可选）"标记，Boolean 类型（true/false）
 * - hint:       输入框下方的灰色提示小字
 */
defineProps({
  label: String,
  modelValue: String,
  optional: Boolean,
  hint: String,
})

/**
 * defineEmits 声明这个组件会向外发出哪些"事件"
 *
 * 'update:modelValue' 是 Vue 3 中 v-model 的标准事件名。
 * 当用户在输入框中输入内容时，组件会触发这个事件，
 * 父组件通过 v-model 监听到后自动更新对应的变量值。
 */
defineEmits(['update:modelValue'])
</script>
