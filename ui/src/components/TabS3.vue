<!--
  TabS3.vue — S3 存储配置标签页
  ================================
  这个页面让用户填写 S3 兼容对象存储的连接信息，支持：
  - AWS S3（亚马逊标准对象存储）
  - Cloudflare R2（Cloudflare 的 S3 兼容存储，免流量费）
  - MinIO（私有化部署的开源对象存储）
  - 其他任何 S3 协议兼容的服务

  页面布局（从上到下两张卡片）：
  ┌────────────────────────────────────────────┐
  │ 存储服务                                    │
  │  [Bucket 名称]    [区域 Region]             │
  │  [Access Key ID]  [Secret Access Key]       │
  └────────────────────────────────────────────┘
  ┌────────────────────────────────────────────┐
  │ 高级选项                                    │
  │  [自定义 Endpoint URL]                      │
  │  ○ 使用路径风格 URL（MinIO 需要）           │
  └────────────────────────────────────────────┘
-->
<template>
  <!-- 页面整体：内边距 20px，子元素间距 16px -->
  <div class="p-5 space-y-4">

    <!-- ── 第一张卡片：基础连接信息 ── -->
    <FormSection title="存储服务" desc="S3 兼容存储的连接信息（支持 AWS、Cloudflare R2、MinIO 等）">
      <!--
        grid grid-cols-2：两列网格布局，让四个输入框排成 2×2 的矩阵
        gap-3：网格单元格之间的间距 12px
      -->
      <div class="grid grid-cols-2 gap-3">
        <!-- Bucket 名称：S3 存储桶的名字，类似"文件夹名" -->
        <FormField
          label="Bucket 名称"
          v-model="local.s3.bucket"
          placeholder="my-images-bucket"
          autocomplete="off"
        />
        <!-- 区域：S3 服务所在的数据中心区域，如 us-east-1（美东）、ap-east-1（亚太） -->
        <FormField
          label="区域 (Region)"
          v-model="local.s3.region"
          placeholder="us-east-1"
          autocomplete="off"
        />
        <!--
          Access Key ID：S3 访问凭证的公开部分（相当于"账号"）
          type="password" 让输入内容以圆点显示，防止被旁人看到
          autocomplete="new-password" 阻止浏览器自动填充密码
        -->
        <FormField
          label="Access Key ID"
          v-model="local.s3.access_key_id"
          placeholder="AKIAIOSFODNN7EXAMPLE"
          type="password"
          autocomplete="new-password"
        />
        <!-- Secret Access Key：S3 访问凭证的私密部分（相当于"密码"），必须保密 -->
        <FormField
          label="Secret Access Key"
          v-model="local.s3.secret_access_key"
          placeholder="wJalrXUtnFEMI/K7MDENG..."
          type="password"
          autocomplete="new-password"
        />
      </div>
    </FormSection>

    <!-- ── 第二张卡片：高级选项（MinIO / R2 等非标准服务需要） ── -->
    <FormSection title="高级选项" desc="自定义 Endpoint 和路径风格（MinIO / Cloudflare R2 等需要）">
      <div class="space-y-3">
        <!--
          自定义 Endpoint URL：当不使用 AWS 标准端点时，在这里填写自定义地址。
          :optional="true" 表示这个字段不是必填的，标签旁会显示"（可选）"
        -->
        <FormField
          label="自定义 Endpoint URL"
          v-model="local.s3.endpoint_url"
          placeholder="https://xxx.r2.cloudflarestorage.com"
          autocomplete="off"
          :optional="true"
        />
        <!--
          路径风格开关：
          - 关闭（默认）：URL 格式为 https://bucket.example.com/key（虚拟主机风格）
          - 开启：URL 格式为 https://example.com/bucket/key（路径风格）
          MinIO 私有部署通常需要开启此选项
        -->
        <ToggleSwitch
          label="使用路径风格 URL（Path-style）"
          hint="MinIO 通常需要此选项；AWS S3 和 Cloudflare R2 不需要"
          v-model="local.s3.path_style"
        />
      </div>
    </FormSection>
  </div>
</template>

<script setup>
// reactive：将普通对象变成"响应式"对象——对象属性变化时，Vue 会自动更新界面
// watch：监听某个值的变化，变化时执行回调函数
import { reactive, watch } from 'vue'
import FormSection from './FormSection.vue'
import FormField from './FormField.vue'
import ToggleSwitch from './ToggleSwitch.vue'

/**
 * Props（父组件传入的数据）：
 * - config：来自 App.vue 的完整配置对象，包含 s3、upload、shortcut 三个子对象
 */
const props = defineProps({ config: Object })

/**
 * 声明向父组件发出的事件：
 * - 'update:config'：当 S3 配置有变化时，通知 App.vue 更新 config
 */
const emit = defineEmits(['update:config'])

/**
 * 创建本地响应式副本（不直接修改 props）
 *
 * Vue 规定：不能直接修改从父组件传入的 props。
 * 所以我们把 props.config.s3 复制一份到 local.s3，
 * 用户修改的是本地副本，然后通过 watch + emit 通知父组件。
 *
 * { ...props.config.s3 } 是"展开运算符"，相当于把 s3 对象的所有属性复制过来。
 */
const local = reactive({
  s3: { ...props.config.s3 },
  upload: props.config.upload,
  shortcut: props.config.shortcut,
})

/**
 * 监听本地 s3 数据的变化，一旦有字段发生改变，就把新的 s3 值通过事件传给父组件
 *
 * { deep: true }：深度监听，对象内部的任何属性变化都会触发回调
 * () => ({ ...local.s3 })：每次都展开成新对象，确保 Vue 能检测到变化
 */
watch(() => ({ ...local.s3 }), (s3) => {
  // 触发 'update:config' 事件，把更新后的完整配置对象传给父组件 App.vue
  emit('update:config', { ...props.config, s3 })
}, { deep: true })

/**
 * 反向同步：当父组件（App.vue）的 config 被外部更新时（如 Rust 后端加载配置），
 * 同步更新本地的 local.s3，保证界面显示的数据始终与父组件一致
 */
watch(() => props.config.s3, (s3) => {
  Object.assign(local.s3, s3)
}, { deep: true })
</script>
