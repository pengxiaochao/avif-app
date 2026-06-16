/// config.rs — 应用配置结构体与持久化模块
///
/// 本模块定义了应用的所有配置项，并实现从磁盘加载和保存的功能。
/// 配置以 TOML 格式保存在系统配置目录：
/// - macOS: `~/Library/Application Support/upic-rs/config.toml`
/// - Linux: `~/.config/upic-rs/config.toml`
/// - Windows: `%APPDATA%\upic-rs\config.toml`
///
/// 所有结构体都实现了 `Serialize`/`Deserialize`（用于 TOML/JSON），
/// `Clone`（用于线程安全地在 Mutex 外复制），
/// `Default`（提供合理的默认值）。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── 链接格式枚举 ──────────────────────────────────────────────────────────────

/// 上传成功后复制到剪贴板的链接格式
///
/// 根据使用场景选择合适的格式：
/// - `Url`：直接 URL，通用
/// - `Markdown`：`![alt](url)` 格式，适合 Markdown 编辑器
/// - `Html`：`<img src="..." alt="...">` 格式，适合 HTML 开发
/// - `Custom`：用户自定义模板，支持 `{url}` 和 `{name}` 变量
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LinkFormat {
    Url,
    Markdown,
    Html,
    Custom,
}

impl Default for LinkFormat {
    fn default() -> Self {
        // 默认使用纯 URL 格式，兼容性最好
        LinkFormat::Url
    }
}

impl LinkFormat {
    /// 返回格式的人类可读标签（用于旧代码兼容，前端直接用枚举字符串）
    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        match self {
            LinkFormat::Url => "URL",
            LinkFormat::Markdown => "Markdown",
            LinkFormat::Html => "HTML",
            LinkFormat::Custom => "自定义",
        }
    }

    /// 根据 URL 和文件名生成格式化的链接字符串
    ///
    /// # 参数
    /// - `url`: 完整的图片访问 URL
    /// - `name`: 文件名（不含扩展名），用作 alt 文字
    /// - `custom_tpl`: 自定义模板字符串（`Custom` 格式时使用）
    pub fn format(&self, url: &str, name: &str, custom_tpl: &str) -> String {
        match self {
            LinkFormat::Url => url.to_string(),
            LinkFormat::Markdown => format!("![{}]({})", name, url),
            LinkFormat::Html => format!(r#"<img src="{}" alt="{}">"#, url, name),
            // 将模板中的 {url} 和 {name} 替换为实际值
            LinkFormat::Custom => custom_tpl.replace("{url}", url).replace("{name}", name),
        }
    }
}

// ── S3 存储配置 ───────────────────────────────────────────────────────────────

/// S3 连接与认证配置
///
/// 支持标准 AWS S3 和所有 S3 兼容存储服务（如 Cloudflare R2、MinIO 等）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    /// S3 Bucket 名称（必填）
    pub bucket: String,
    /// AWS 区域代码，如 `"us-east-1"`, `"ap-northeast-1"`
    pub region: String,
    /// IAM 用户的 Access Key ID（必填）
    pub access_key_id: String,
    /// IAM 用户的 Secret Access Key（必填，敏感信息）
    pub secret_access_key: String,
    /// 自定义 Endpoint URL，仅 MinIO/R2 等非 AWS 服务需要
    /// 示例：`"https://your-account.r2.cloudflarestorage.com"`
    pub endpoint_url: Option<String>,
    /// 是否强制使用路径风格 URL（Path-style）
    /// MinIO 通常需要此选项；AWS S3 和 Cloudflare R2 不需要
    #[serde(default)]
    pub path_style: bool,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            region: "us-east-1".to_string(), // AWS 默认区域
            access_key_id: String::new(),
            secret_access_key: String::new(),
            endpoint_url: None,
            path_style: false,
        }
    }
}

// ── 上传行为配置 ──────────────────────────────────────────────────────────────

/// 控制上传行为的配置项
///
/// 包括文件路径模板、AVIF 转换参数、链接格式等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    /// 所有上传文件的路径前缀，如 `"images"`（末尾不加斜杠）
    #[serde(default)]
    pub key_prefix: String,

    /// 子目录路径模板，支持时间变量（见 template.rs）
    /// 默认：`"{Y}/{m}/{d}"`（按年/月/日分目录）
    #[serde(default = "default_dir_format")]
    pub dir_format: String,

    /// 文件名模板（不含扩展名），支持时间和 UUID 变量
    /// 默认：`"{uuid8}"`（使用 UUID 前 8 位保证唯一性）
    #[serde(default = "default_filename_format")]
    pub filename_format: String,

    /// 自定义 CDN 域名前缀，留空则使用 S3 原始域名
    /// 示例：`"https://cdn.example.com"`
    #[serde(default)]
    pub url_prefix: String,

    /// 是否在上传前将图片自动转换为 AVIF 格式（默认开启）
    #[serde(default = "bool_true")]
    pub convert_to_avif: bool,

    /// AVIF 编码质量（1–100），越高越清晰文件越大（默认 80）
    #[serde(default = "default_quality")]
    pub avif_quality: f32,

    /// AVIF 编码速度（1–10），越小越慢但压缩率越高（默认 8）
    #[serde(default = "default_speed")]
    pub avif_speed: u8,

    /// 上传成功后复制到剪贴板的链接格式
    #[serde(default)]
    pub link_format: LinkFormat,

    /// 自定义链接模板（`link_format = Custom` 时使用）
    /// 可用变量：`{url}` 完整 URL，`{name}` 文件名（不含扩展名）
    #[serde(default = "default_custom_tpl")]
    pub custom_link_template: String,

    /// 上传成功后是否自动将链接复制到剪贴板（默认开启）
    #[serde(default = "bool_true")]
    pub auto_copy: bool,
}

// 配置默认值函数（serde 的 default 特性要求函数形式）
fn default_dir_format() -> String {
    "{Y}/{m}/{d}".to_string()
}
fn default_filename_format() -> String {
    "{uuid8}".to_string()
}
fn default_quality() -> f32 {
    80.0
}
fn default_speed() -> u8 {
    8
}
fn bool_true() -> bool {
    true
}
fn default_custom_tpl() -> String {
    "![{name}]({url})".to_string()
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            key_prefix: String::new(),
            dir_format: default_dir_format(),
            filename_format: default_filename_format(),
            url_prefix: String::new(),
            convert_to_avif: true,
            avif_quality: default_quality(),
            avif_speed: default_speed(),
            link_format: LinkFormat::default(),
            custom_link_template: default_custom_tpl(),
            auto_copy: true,
        }
    }
}

// ── 顶层配置结构体 ────────────────────────────────────────────────────────────

/// 完整的应用配置
///
/// 通过 `Mutex<Config>` 注册为 Tauri 全局状态，
/// 在所有命令函数中通过 `State<'_, Mutex<Config>>` 参数注入访问。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// S3 存储服务连接配置
    #[serde(default)]
    pub s3: S3Config,
    /// 上传行为与格式配置
    #[serde(default)]
    pub upload: UploadConfig,
    /// 全局快捷键，如 `"CmdOrCtrl+Shift+U"`，空字符串表示禁用
    #[serde(default = "default_shortcut")]
    pub shortcut: String,
    /// 开机自动启动应用
    #[serde(default)]
    pub autostart: bool,
}

fn default_shortcut() -> String {
    "CmdOrCtrl+Shift+U".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            s3: S3Config::default(),
            upload: UploadConfig::default(),
            shortcut: default_shortcut(),
            autostart: false,
        }
    }
}

impl Config {
    /// 返回配置文件的磁盘路径
    pub fn config_path() -> PathBuf {
        // dirs::config_dir() 返回平台相关的配置目录
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("avif-image")
            .join("config.toml")
    }

    /// 返回旧版配置文件路径（upic-rs），用于迁移
    fn legacy_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("upic-rs")
            .join("config.toml")
    }

    /// 从磁盘加载配置文件
    ///
    /// 如果文件不存在或解析失败，返回默认配置（不报错）。
    /// 使用 `#[serde(default)]` 可以容忍配置文件中缺少某些字段的情况。
    /// 会自动从旧版 `upic-rs` 目录迁移配置。
    pub fn load() -> Self {
        let path = Self::config_path();
        log::debug!("加载配置文件: {:?}", path);

        // 迁移：如果新路径不存在但旧路径存在，从旧路径加载
        if !path.exists() {
            let legacy = Self::legacy_config_path();
            if legacy.exists() {
                log::info!("从旧版配置目录迁移: {:?} → {:?}", legacy, path);
                if let Ok(content) = std::fs::read_to_string(&legacy) {
                    if let Ok(cfg) = toml::from_str::<Self>(&content) {
                        // 保存到新路径
                        if let Err(e) = cfg.save() {
                            log::warn!("迁移后保存配置失败: {e}");
                        }
                        return cfg;
                    }
                }
            }
        }

        let Ok(content) = std::fs::read_to_string(&path) else {
            log::info!("配置文件不存在，使用默认配置: {:?}", path);
            return Self::default();
        };

        match toml::from_str(&content) {
            Ok(cfg) => {
                log::info!("配置文件加载成功: {:?}", path);
                cfg
            }
            Err(e) => {
                log::error!("配置文件解析失败，使用默认配置: {e}");
                Self::default()
            }
        }
    }

    /// 将当前配置序列化为 TOML 并写入磁盘
    ///
    /// 会自动创建父目录（如果不存在）。
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        log::debug!("保存配置文件: {:?}", path);

        // 确保父目录存在
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }

        let content = toml::to_string_pretty(self).context("序列化配置失败")?;
        std::fs::write(&path, content).context("写入配置文件失败")?;

        log::info!("配置文件保存成功: {:?}", path);
        Ok(())
    }

    /// 调试用：打印配置文件路径和内容到 stdout
    #[allow(dead_code)]
    pub fn debug_print() {
        let path = Self::config_path();
        println!("配置文件路径: {}", path.display());
        match std::fs::read_to_string(&path) {
            Ok(content) => println!("配置文件内容:\n{}", content),
            Err(e) => println!("无法读取配置文件: {}", e),
        }
    }

    /// 单元测试：验证配置文件路径在不同平台上是否正确
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn test_config_path() {
        let path = Self::config_path();
        println!("测试配置文件路径: {}", path.display());
        #[cfg(target_os = "windows")]
        assert!(path.ends_with("AppData\\Roaming\\upic-rs\\config.toml"));
        #[cfg(target_os = "macos")]
        assert!(path.ends_with("Library/Application Support/upic-rs/config.toml"));
        #[cfg(target_os = "linux")]
        assert!(path.ends_with(".config/upic-rs/config.toml"));
    }
}