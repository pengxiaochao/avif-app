# avif-image

一款运行在 macOS 菜单栏的轻量级图床工具，支持将剪贴板中的图片或文件一键转换为 AVIF 格式并上传到 S3 兼容对象存储（AWS S3、Cloudflare R2、MinIO 等），上传完成后自动将链接复制到剪贴板。

---

## 功能特性

| 功能 | 说明 |
|------|------|
| 🖼️ **AVIF 转换** | 上传图片前自动转换为 AVIF 格式，大幅降低文件体积 |
| ☁️ **S3 兼容上传** | 支持 AWS S3、Cloudflare R2、MinIO 等所有 S3 协议存储 |
| ⌨️ **全局快捷键** | 在任意应用内按下快捷键即可触发剪贴板上传 |
| 📋 **剪贴板感知** | 自动识别剪贴板中的图片数据或文件路径 |
| � **任意文件上传** | 支持上传 PDF、ZIP、TAR、视频等任意格式，非图片文件跳过 AVIF 转换 |
| 📂 **目录批量上传** | 选择整个目录，递归上传所有文件，最多 4 个并发，完成后自动跳转历史记录 |
| �🔗 **多种链接格式** | 支持纯 URL、Markdown、HTML 及完全自定义模板 |
| 📂 **文件名模板** | 支持日期、UUID、原始文件名等变量组合 |
| 📋 **历史记录** | 保存最近 200 条上传记录，支持一键复制和浏览器打开 |
| 🚀 **开机自启** | 支持配置登录时自动启动，常驻菜单栏 |
| 🌓 **暗色模式** | 托盘图标自动适配 macOS 亮色 / 暗色主题 |

---

## 设计思路

### 整体架构

项目使用 **Tauri v2** 构建，后端逻辑以 Rust 实现，前端界面基于 **Vite + Vue 3 + Tailwind CSS** 构建，二者通过 Tauri IPC 通信。

```
avif-image/
├── src-tauri/               # Rust 后端（Tauri 应用核心）
│   ├── src/
│   │   ├── lib.rs           # 应用启动、托盘菜单、全局快捷键
│   │   ├── commands.rs      # Tauri IPC 命令（前端调用入口）
│   │   ├── config.rs        # 配置结构体 + TOML 序列化
│   │   ├── avif.rs          # 图片→AVIF 编码（ravif）
│   │   ├── uploader.rs      # AWS S3 上传（aws-sdk-s3）
│   │   ├── template.rs      # 文件名 / 目录路径模板渲染
│   │   └── history.rs       # 上传历史持久化（JSON）
│   ├── icons/               # 应用图标
│   └── tauri.conf.json      # Tauri 构建配置
└── ui/                      # 前端（Vite + Vue 3 + Tailwind CSS）
    ├── index.html           # Vite 入口 HTML
    ├── vite.config.js       # Vite 构建配置
    ├── tailwind.config.js   # Tailwind 主题配置
    ├── package.json
    └── src/
        ├── main.js          # Vue 应用挂载
        ├── style.css        # Tailwind 基础样式 + 自定义变量
        ├── App.vue          # 根组件（标题栏 + 侧边栏 + 内容区）
        └── components/
            ├── TabS3.vue        # S3 存储配置
            ├── TabUpload.vue    # 上传与 AVIF 设置
            ├── TabLink.vue      # 链接格式选择
            ├── TabShortcut.vue  # 快捷键录制 + 开机启动
            ├── TabHistory.vue   # 上传历史记录
            ├── NavItem.vue      # 侧边栏导航项
            ├── FormSection.vue  # 卡片容器组件
            ├── FormField.vue    # 表单输入行
            ├── ToggleSwitch.vue # macOS 风格开关
            └── icons/           # SVG 图标组件
```

### 关键设计决策

**菜单栏应用模式**：通过 `tauri::ActivationPolicy::Accessory` 隐藏 Dock 图标，仅在菜单栏常驻，不干扰工作流。窗口关闭时隐藏而不退出。

**剪贴板处理**：优先读取剪贴板中的文件路径（复制文件后上传），次选图片像素数据（截图后上传）。由于 macOS 的 `NSRunLoop` 限制，剪贴板操作在 `spawn_blocking` 线程中执行。

**AVIF 编码**：使用 `ravif` 库对图片进行编码，质量（1-100）和速度（1-10）均可配置，在文件大小与编码速度之间取得平衡。

**配置持久化**：配置文件存储在 `~/Library/Application Support/avif-image/config.toml`，历史记录存储在同目录下的 `history.json`。

**快捷键处理**：前端使用 `e.code` 而非 `e.key` 来识别物理按键，解决了 macOS 上 Alt+字母 组合会产生特殊 Unicode 字符（如 `¨`、`∂`）导致快捷键无法识别的问题。

---

## 系统要求

- macOS 10.15 (Catalina) 或更高版本
- 仅支持 macOS（菜单栏应用特性依赖 macOS 平台）

---

## 打包与安装

### 前置依赖

```bash
# 1. 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 2. 安装 Tauri CLI v2
cargo install tauri-cli --version "^2"

# 3. 安装 Node.js（v18+ 推荐）
# 推荐使用 fnm 或 nvm 管理 Node 版本
node --version
npm --version
```

### 安装前端依赖

前端使用 Vite + Vue 3 + Tailwind CSS，**首次构建前**需安装依赖：

```bash
cd ui
npm install
```

### 开发模式运行

Tauri 会自动启动 Vite Dev Server 并打开带热更新的开发窗口：

```bash
# 在项目根目录
cargo tauri dev
```

> Vite Dev Server 监听 `http://localhost:5173`，前端代码修改后无需重启，页面自动热更新。

首次运行会编译所有 Rust 依赖（约需 3-10 分钟），后续增量编译较快。

### 构建发布版本

 Tauri 构建时会自动执行 `npm run build` 编译前端，无需手动操作：

```bash
# 在项目根目录，构建 .dmg 安装包（同时生成 .app 和 .dmg）
cargo tauri build
```

如果只想单独构建前端（调试 UI 用）：

```bash
cd ui
npm run build   # 产物输出到 ui/dist/
```

构建产物位于：
```
src-tauri/target/release/bundle/
├── dmg/          # avif-image_0.1.0_aarch64.dmg（Apple Silicon）
│                 # avif-image_0.1.0_x86_64.dmg（Intel）
└── macos/        # avif-image.app（未签名应用包）
```

### 安装到本机

**方式一：使用 .dmg（推荐）**

1. 双击打开 `.dmg` 文件
2. 将 `avif-image.app` 拖入 `/Applications` 文件夹
3. 首次运行可能需要在「系统设置 → 隐私与安全性」中允许运行未签名应用

**方式二：直接复制 .app**

```bash
cp -r src-tauri/target/release/bundle/macos/avif-image.app /Applications/
```

### 消除 macOS 安全警告（未签名应用）

```bash
# 移除隔离属性（首次运行前执行）
xattr -cr /Applications/avif-image.app
```

---

## 配置说明

### S3 配置

| 字段 | 说明 |
|------|------|
| Endpoint URL | 留空使用 AWS S3；填写自定义地址用于 R2/MinIO |
| Bucket | 存储桶名称 |
| Region | 区域，例如 `us-east-1` 或 `auto`（Cloudflare R2） |
| Access Key ID | 访问密钥 ID |
| Secret Access Key | 访问密钥 Secret |
| 路径风格 URL | MinIO 通常需要开启 |

### 上传设置

| 字段 | 默认值 | 说明 |
|------|--------|------|
| 路径前缀 | 空 | S3 Key 前缀，例如 `images` |
| 目录格式 | `{Y}/{m}/{d}` | 支持 `{Y}` `{m}` `{d}` `{H}` `{M}` `{S}` `{timestamp}` |
| 文件名格式 | `{uuid8}` | 支持 `{uuid}` `{uuid8}` `{name}` 及上述时间变量 |
| 自定义域名 | 空 | 替换 S3 原始地址，例如 `https://cdn.example.com` |
| 图片→AVIF | 开启 | 自动转换图片为 AVIF 格式 |
| AVIF 质量 | 80 | 1-100，越高越清晰 |
| AVIF 速度 | 8 | 1-10，越小编码越慢但质量越好 |

### 快捷键

点击快捷键输入框后，直接按下目标组合键即可录制。支持：

- 修饰键：`CmdOrCtrl`、`Alt`、`Shift`、`Super`
- 字母键、数字键、F1-F12 及方向键等

> **注意**：修复了 macOS 上 `Alt+Shift+字母` 组合键无法识别的问题。之前 macOS 会将 Alt+字母 转换为特殊 Unicode 字符（如 `¨`），现改用物理键码（`e.code`）识别。

### 开机自启

在「快捷键」标签页的「启动选项」区域，勾选「开机自动启动」即可。状态通过 macOS LaunchAgent 机制持久化，无需手动添加登录项。

---

## 使用流程

1. 启动应用后，avif-image 图标出现在菜单栏
2. 复制一张图片或文件（`⌘C`）
3. 按下全局快捷键（默认 `CmdOrCtrl+Shift+U`）或点击菜单栏图标选择「从剪贴板上传」
4. 应用自动将图片转为 AVIF 并上传到 S3
5. 上传完成后，链接自动复制到剪贴板，系统通知提示成功

### 上传任意文件

点击设置窗口侧边栏的「选择文件」按钮，可选择任意格式文件上传：
- **图片文件**（jpg/png/gif/webp/bmp/tiff）：按配置决定是否转换为 AVIF 后上传
- **非图片文件**（pdf/zip/tar/mp4 等）：直接以原始格式上传，不做任何转换

> **限制**：单个文件最大 500MB，超过限制的文件会被跳过。

### 批量目录上传

点击侧边栏「选择目录」按钮，选择一个本地目录后：
1. 应用递归遍历该目录下的所有文件（含子目录）
2. 自动跳过 `.DS_Store`、`.gitignore` 等隐藏文件（以 `.` 开头的文件）
3. 超过 500MB 的文件会自动跳过
4. 图片文件按配置执行 AVIF 转换，其他文件保持原格式
5. 最多 4 个文件并发上传，显著提升大批量上传速度
6. 单个文件上传失败不影响其他文件，失败信息会汇总在结果中
7. 全部完成后自动跳转到「历史记录」标签页，可查看所有上传结果

> **提示**：批量上传不会自动复制链接到剪贴板，请到「历史记录」页面单独复制所需链接。

---

## 依赖说明

### 前端技术栈

| 依赖 | 版本 | 用途 |
|------|------|------|
| `vite` | ^5 | 极速构建工具，开发时 HMR 热更新 |
| `vue` | ^3.4 | 响应式组件化 UI 框架 |
| `tailwindcss` | ^3.4 | 原子化 CSS，内置暗色模式支持 |
| `@vitejs/plugin-vue` | ^5 | Vite 的 Vue 单文件组件支持 |
| `autoprefixer` + `postcss` | — | CSS 兼容性处理 |

**前端优势**

- **macOS 原生质感**：颜色、圆角、阴影、字体均参照 Apple HIG 规范，支持系统级亮色/暗色自动切换。
- **组件化**：每个标签页独立为 Vue 单文件组件（SFC），逻辑与样式完全隔离，易于维护和扩展。
- **响应式状态**：使用 Vue 3 `reactive` / `ref`，配置修改实时同步，无需手动 DOM 操作。
- **极小体积**：Vite 生产构建会 Tree-shaking 未使用代码，Tailwind CSS 仅保留实际用到的类，最终产物远小于传统打包方案。
- **零运行时依赖**：前端不引入任何第三方 UI 库，所有组件均为自研，减少供应链风险。

### Rust 后端

| 依赖 | 用途 |
|------|------|
| `tauri` v2 | 跨平台桌面应用框架 |
| `tauri-plugin-global-shortcut` | 全局快捷键注册 |
| `tauri-plugin-autostart` | 开机自启（macOS LaunchAgent） |
| `tauri-plugin-notification` | 系统通知 |
| `tauri-plugin-dialog` | 文件选择对话框 |
| `tauri-plugin-shell` | 打开外部 URL |
| `ravif` | AVIF 图像编码 |
| `aws-sdk-s3` | AWS S3 兼容上传 |
| `arboard` | 跨平台剪贴板访问 |
| `tokio` | 异步运行时 |

---

## License

MIT
