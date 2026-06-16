<!--
  TabUpload.vue — 上传设置标签页
  ================================
  控制文件上传时的各种行为：目录结构、文件命名、AVIF 转换参数等。

  页面布局（三张卡片从上到下）：

  ┌──────────────────────────────────────────────┐
  │ 文件路径                                      │
  │  [路径前缀]       [目录格式]                  │
  │  [文件名格式（支持 {Y} {uuid8} 等变量）]      │
  │  [自定义 CDN 域名前缀]                        │
  └──────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────┐
  │ AVIF 转换                                     │
  │  ○ 自动转换为 AVIF                            │
  │  ──────────── 编码质量 ────── [滑块]          │
  │  ──────────── 编码速度 ────── [滑块]          │
  └──────────────────────────────────────────────┘

  ┌──────────────────────────────────────────────┐
  │ 上传行为                                      │
  │  ○ 上传成功后自动复制链接                     │
  └──────────────────────────────────────────────┘
-->
<template>
  <div class="p-5 space-y-4">

    <!-- ── 第一张卡片：文件路径配置 ── -->
    <FormSection title="文件路径" desc="控制上传到 S3 后的目录结构和文件命名">
      <div class="grid grid-cols-2 gap-3">
        <!--
          路径前缀：上传到 S3 时，所有文件都放在这个前缀目录下。
          例如填 "images"，最终路径就是 images/2026/05/07/abc123.avif
        -->
        <FormField
          label="路径前缀"
          v-model="local.key_prefix"
          placeholder="images"
          autocomplete="off"
          :optional="true"
        />
        <!--
          目录格式：支持时间变量，自动生成按日期分组的目录。
          {Y} = 年（2026），{m} = 月（05），{d} = 日（07）
        -->
        <FormField
          label="目录格式"
          v-model="local.dir_format"
          placeholder="{Y}/{m}/{d}"
          autocomplete="off"
        />
        <!--
          文件名格式：col-span-2 让这个输入框占满整行（跨越两列）
          支持多种变量：{uuid8}=UUID前8位，{name}=原始文件名，{timestamp}=时间戳等
        -->
        <div class="col-span-2">
          <FormField
            label="文件名格式"
            v-model="local.filename_format"
            placeholder="{uuid8}"
            autocomplete="off"
            hint="可用变量：{Y} 年　{m} 月　{d} 日　{H} 时　{M} 分　{S} 秒　{uuid} UUID　{uuid8} UUID 前8位　{name} 原始文件名　{timestamp} 时间戳"
          />
        </div>
        <!--
          CDN 域名前缀：如果有自定义域名，生成的 URL 会使用此域名而非 S3 原始域名。
          例如填 "https://cdn.example.com"，生成 URL 为 https://cdn.example.com/2026/05/07/abc.avif
        -->
        <div class="col-span-2">
          <FormField
            label="自定义 CDN 域名前缀"
            v-model="local.url_prefix"
            placeholder="https://cdn.example.com"
            autocomplete="off"
            :optional="true"
          />
        </div>
      </div>
    </FormSection>

    <!-- ── 第二张卡片：AVIF 转换配置 ── -->
    <FormSection title="AVIF 转换" desc="将图片转换为 AVIF 格式，可大幅减小文件体积">
      <div class="space-y-4">
        <!-- AVIF 转换总开关 -->
        <ToggleSwitch
          label="上传前自动转换为 AVIF 格式"
          hint="AVIF 比 JPEG 体积更小，画质更好"
          v-model="local.convert_to_avif"
        />

        <!--
          v-if="local.convert_to_avif"：只有开关打开时才显示下面的滑块配置
          template 标签是 Vue 的透明容器，不会在 HTML 中渲染成实际元素
        -->
        <template v-if="local.convert_to_avif">

          <!-- 质量滑块（1-100，值越高画质越好但文件越大） -->
          <div class="space-y-1.5">
            <!-- 标题行：左侧文字，右侧显示当前数值 -->
            <div class="flex justify-between">
              <span class="text-[12px] font-medium text-[#1d1d1f] dark:text-[#f2f2f7]">编码质量</span>
              <!-- 蓝色数字实时显示当前滑块的值 -->
              <span class="text-[12px] font-semibold text-[#0071e3] dark:text-[#0a84ff]">{{ local.avif_quality }}</span>
            </div>
            <!--
              HTML 原生 range 滑块：
              - :value 绑定当前值（单向）
              - @input 监听拖动事件，Number() 将字符串值转为数字后存入 local
            -->
            <input
              type="range" min="1" max="100" step="1"
              class="w-full"
              :value="local.avif_quality"
              @input="local.avif_quality = Number($event.target.value)"
            />
            <!-- 滑块两端的标注说明 -->
            <div class="flex justify-between text-[11px] text-[#6e6e73] dark:text-[#aeaeb2]">
              <span>小文件（低质量）</span>
              <span>高质量（大文件）</span>
            </div>
          </div>

          <!-- 速度滑块（1-10，值越小编码越慢但压缩率越高） -->
          <div class="space-y-1.5">
            <div class="flex justify-between">
              <span class="text-[12px] font-medium text-[#1d1d1f] dark:text-[#f2f2f7]">编码速度</span>
              <span class="text-[12px] font-semibold text-[#0071e3] dark:text-[#0a84ff]">{{ local.avif_speed }}</span>
            </div>
            <input
              type="range" min="1" max="10" step="1"
              class="w-full"
              :value="local.avif_speed"
              @input="local.avif_speed = Number($event.target.value)"
            />
            <div class="flex justify-between text-[11px] text-[#6e6e73] dark:text-[#aeaeb2]">
              <span>慢（压缩率更高）</span>
              <span>快（压缩率低）</span>
            </div>
          </div>
        </template>
      </div>
    </FormSection>

    <!-- ── 第三张卡片：上传行为 ── -->
    <FormSection title="上传行为">
      <!--
        自动复制开关：上传成功后自动把格式化链接写入系统剪贴板，
        用户不需要手动点"复制"，非常方便在 Markdown 编辑器中粘贴使用
      -->
      <ToggleSwitch
        label="上传成功后自动复制链接"
        hint="根据「链接格式」设置自动将链接写入剪贴板"
        v-model="local.auto_copy"
      />
    </FormSection>
  </div>
</template>

<script setup>
import { reactive, watch } from 'vue'
import FormSection from './FormSection.vue'
import FormField from './FormField.vue'
import ToggleSwitch from './ToggleSwitch.vue'

const props = defineProps({ config: Object })
const emit = defineEmits(['update:config'])

/**
 * 把 props.config.upload 的所有字段展开复制到本地响应式对象。
 * { ...props.config.upload } 是 ES6 展开语法，等同于逐个复制每个字段。
 */
const local = reactive({ ...props.config.upload })

/**
 * 监听本地 upload 配置的任何变化（包括深层嵌套），
 * 变化时把完整更新后的配置通知给父组件 App.vue
 */
watch(() => ({ ...local }), (upload) => {
  emit('update:config', { ...props.config, upload })
}, { deep: true })

/**
 * 父组件（App.vue）在加载完 Rust 配置后会更新 props.config.upload，
 * 这里监听此变化，用 Object.assign 把新值合并到本地对象中，
 * 确保界面显示和后端数据保持同步
 */
watch(() => props.config.upload, (u) => {
  Object.assign(local, u)
}, { deep: true })
</script>
