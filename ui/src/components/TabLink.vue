<!--
  TabLink.vue — 链接格式设置标签页
  ===================================
  控制上传成功后写入剪贴板的链接格式。支持四种格式：
  - 纯 URL：直接复制图片地址，适合在浏览器地址栏使用
  - Markdown：复制 ![文件名](URL) 格式，适合 Markdown 写作工具（如 Typora、Obsidian）
  - HTML：复制 <img src="..."> 标签，适合直接写 HTML 代码
  - 自定义：用自定义模板，支持 {url} 和 {name} 变量

  页面布局：
  ┌──────────────────────────────────────────────┐
  │ 复制格式                                      │
  │  ○ 纯 URL        https://cdn.../abc.avif      │  ← 单选卡片（点击选中变蓝色边框）
  │  ● Markdown      ![abc](https://cdn.../...)   │  ← 当前选中（蓝色边框+浅蓝背景）
  │  ○ HTML          <img src="..." alt="...">    │
  │  ○ 自定义模板    使用自定义模板字符串...       │
  └──────────────────────────────────────────────┘
  （仅当选择"自定义"时出现）
  ┌──────────────────────────────────────────────┐
  │ 自定义模板                                    │
  │  [模板内容 输入框]                            │
  └──────────────────────────────────────────────┘
-->
<template>
  <div class="p-5 space-y-4">
    <FormSection title="复制格式" desc="上传成功后写入剪贴板的链接格式">
      <!--
        单选卡片列表：用 v-for 循环渲染四个格式选项
        每个选项是一个可点击的 label 标签，点击时更新 local.link_format
      -->
      <div class="space-y-2">
        <!--
          v-for 循环：遍历 options 数组，每项渲染一个选择卡片
          :key="opt.value" 给每个元素一个唯一标识，让 Vue 高效更新 DOM
        -->
        <label
          v-for="opt in options"
          :key="opt.value"
          class="flex items-start gap-3 p-3 rounded-xl cursor-pointer border transition-colors"
          :class="local.link_format === opt.value
            ? 'border-[#0071e3] dark:border-[#0a84ff] bg-[#0071e3]/5 dark:bg-[#0a84ff]/8'
            : 'border-black/8 dark:border-white/8 hover:border-black/15 dark:hover:border-white/15'"
          @click="local.link_format = opt.value"
        >
          <!--
            自定义圆形单选按钮（替代原生 <input type="radio">，方便自定义样式）：
            - 外层是圆形边框
            - 内层的小蓝点只有在此项被选中时才显示（v-if 条件渲染）
          -->
          <div class="mt-0.5 w-4 h-4 rounded-full border-2 flex items-center justify-center shrink-0 transition-colors"
            :class="local.link_format === opt.value
              ? 'border-[#0071e3] dark:border-[#0a84ff]'
              : 'border-[#6e6e73]/40 dark:border-[#aeaeb2]/40'">
            <!-- 选中时显示中心蓝点 -->
            <div v-if="local.link_format === opt.value"
              class="w-2 h-2 rounded-full bg-[#0071e3] dark:bg-[#0a84ff]" />
          </div>

          <!-- 格式说明文字 -->
          <div>
            <!-- 格式名称（如"Markdown"） -->
            <p class="text-[13px] font-medium text-[#1d1d1f] dark:text-[#f2f2f7]">{{ opt.label }}</p>
            <!-- 格式示例（灰色代码字体，展示实际输出效果） -->
            <code class="text-[11px] text-[#6e6e73] dark:text-[#aeaeb2] break-all leading-relaxed">{{ opt.preview }}</code>
          </div>
        </label>
      </div>
    </FormSection>

    <!--
      自定义模板输入卡片：
      只有当用户选择了"Custom"格式时才显示（v-if 条件渲染）
    -->
    <FormSection v-if="local.link_format === 'Custom'" title="自定义模板">
      <FormField
        label="模板内容"
        v-model="local.custom_link_template"
        placeholder="![{name}]({url})"
        hint="支持变量：{url} 完整 URL，{name} 文件名（不含扩展名）"
      />
    </FormSection>
  </div>
</template>

<script setup>
import { reactive, watch } from 'vue'
import FormSection from './FormSection.vue'
import FormField from './FormField.vue'

const props = defineProps({ config: Object })
const emit = defineEmits(['update:config'])

/**
 * 本地响应式数据：只关心链接相关的两个字段
 * 从 props.config.upload 中提取 link_format 和 custom_link_template
 */
const local = reactive({
  link_format: props.config.upload.link_format,
  custom_link_template: props.config.upload.custom_link_template,
})

/**
 * 四种格式选项的配置数据：
 * - value：存储到配置中的实际值（与 Rust 后端的枚举名一一对应）
 * - label：界面上显示的名称
 * - preview：展示给用户看的格式示例
 */
const options = [
  { value: 'Url',      label: '纯 URL',    preview: 'https://cdn.example.com/2026/05/07/a3f5c2b1.avif' },
  { value: 'Markdown', label: 'Markdown',  preview: '![a3f5c2b1](https://cdn.example.com/2026/05/07/a3f5c2b1.avif)' },
  { value: 'Html',     label: 'HTML',      preview: '<img src="https://cdn.example.com/..." alt="a3f5c2b1">' },
  { value: 'Custom',   label: '自定义模板', preview: '使用自定义模板字符串，支持 {url} 和 {name} 变量' },
]

/**
 * 监听本地数据变化，将更新后的 link_format 和 custom_link_template
 * 合并回父组件的 config.upload 对象中通知父组件
 */
watch(() => ({ ...local }), ({ link_format, custom_link_template }) => {
  emit('update:config', {
    ...props.config,
    upload: { ...props.config.upload, link_format, custom_link_template },
  })
}, { deep: true })

/**
 * 父组件配置更新时（如从 Rust 加载配置）同步到本地状态
 */
watch(() => props.config.upload, (u) => {
  local.link_format = u.link_format
  local.custom_link_template = u.custom_link_template
}, { deep: true })
</script>
