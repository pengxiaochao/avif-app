<!--
  FormSection.vue — 通用表单区块卡片组件
  =========================================
  这是一个"白色圆角卡片"容器，用于把相关的表单字段分组展示。
  每个设置页面（S3配置、上传设置等）都由若干个 FormSection 卡片组成。

  视觉结构：
  ┌─────────────────────────────────────────────┐
  │  卡片标题（加粗）                            │  ← 标题栏（有 title 时才显示）
  │  卡片描述文字（灰色小字）                    │
  ├─────────────────────────────────────────────┤
  │                                             │
  │   <slot>  插槽内容（实际的表单字段）         │  ← 内容区
  │                                             │
  └─────────────────────────────────────────────┘

  使用示例：
    <FormSection title="存储服务" desc="填写 S3 连接信息">
      <FormField label="Bucket" v-model="..." />
    </FormSection>
-->
<template>
  <!--
    卡片容器：
    - bg-white / dark:bg-[#2c2c2e]：亮色模式白色背景，暗色模式深灰背景
    - rounded-2xl：大圆角，营造 macOS 风格卡片感
    - shadow-sm：轻微阴影，让卡片从背景中"浮起"
    - overflow-hidden：防止内容溢出圆角边界
  -->
  <div class="bg-white dark:bg-[#2c2c2e] rounded-2xl shadow-sm border border-black/[0.04] dark:border-white/[0.06] overflow-hidden">

    <!-- 卡片标题栏：只有传入了 title 或 desc 时才渲染此区域 -->
    <div v-if="title || desc" class="px-5 pt-4 pb-3 border-b border-black/[0.05] dark:border-white/[0.05]">
      <!-- 标题文字：较大、加粗 -->
      <h3 class="text-[14px] font-semibold text-[#1d1d1f] dark:text-[#f2f2f7]">{{ title }}</h3>
      <!-- 描述文字：只有传入 desc 时才显示，灰色小字，用于补充说明 -->
      <p v-if="desc" class="text-[12px] text-[#6e6e73] dark:text-[#aeaeb2] mt-0.5">{{ desc }}</p>
    </div>

    <!--
      内容区：放置具体的表单字段
      <slot /> 是 Vue 的默认插槽，父组件在 <FormSection> 标签内写的内容会渲染到这里
    -->
    <div class="px-5 py-4">
      <slot />
    </div>
  </div>
</template>

<script setup>
/**
 * 这个组件接受两个可选的字符串属性：
 * - title：卡片顶部的标题文字
 * - desc： 标题下方的灰色描述文字
 *
 * 两者都是可选的，不传就不显示标题栏。
 */
defineProps({ title: String, desc: String })
</script>
