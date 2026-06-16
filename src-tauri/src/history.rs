/// history.rs — 上传历史记录模块
///
/// 本模块负责维护上传历史记录的内存状态和磁盘持久化。
///
/// 历史记录以 JSON 格式保存在：
/// - macOS: `~/Library/Application Support/upic-rs/history.json`
/// - Linux: `~/.config/upic-rs/history.json`
/// - Windows: `%APPDATA%\upic-rs\history.json`
///
/// 最多保留 200 条记录（超过时自动删除最旧的）。

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 单条上传历史记录
///
/// 每次成功上传后会创建一个 HistoryEntry 并添加到历史列表。
/// 字段都实现了 Serialize/Deserialize 以便 JSON 序列化和跨进程传输。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 上传完成的 Unix 时间戳（秒），用于列表排序和时间显示
    pub timestamp: i64,
    /// 上传后在 S3 上的文件名（含扩展名），如 `"a3f5c2b1.avif"`
    pub filename: String,
    /// 完整的 S3 对象 key，如 `"images/2026/05/07/a3f5c2b1.avif"`
    pub key: String,
    /// 最终可访问的完整 URL（含自定义域名或 S3 原始域名）
    pub url: String,
    /// 上传文件的字节大小（上传后的实际大小，非原始文件大小）
    pub size_bytes: u64,
}

/// 历史记录管理器
///
/// 持有内存中的记录列表和磁盘文件路径。
/// 通过 `Mutex<History>` 包裹后注册为 Tauri 全局状态，
/// 保证多个 async 命令并发访问时的线程安全。
pub struct History {
    /// 已加载的历史记录列表（最新在前）
    pub entries: Vec<HistoryEntry>,
    /// JSON 文件的磁盘路径（用于持久化读写）
    path: PathBuf,
}

impl History {
    /// 从磁盘加载历史记录
    ///
    /// 如果文件不存在或解析失败，返回空列表（不报错）。
    /// 这样即使历史文件损坏，应用也能正常启动。
    /// 会自动从旧版 `upic-rs` 目录迁移历史记录。
    pub fn load() -> Self {
        // 使用 dirs crate 获取平台相关的配置目录
        let path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("avif-image")
            .join("history.json");

        log::debug!("从磁盘加载历史记录: {:?}", path);

        // 迁移：如果新路径不存在但旧路径存在，从旧路径加载
        if !path.exists() {
            let legacy = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("upic-rs")
                .join("history.json");
            if legacy.exists() {
                log::info!("从旧版历史记录目录迁移: {:?} → {:?}", legacy, path);
                if let Ok(content) = std::fs::read_to_string(&legacy) {
                    let entries: Vec<HistoryEntry> = serde_json::from_str(&content).unwrap_or_default();
                    if let Some(dir) = path.parent() {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    if let Ok(json) = serde_json::to_string_pretty(&entries) {
                        let _ = std::fs::write(&path, json);
                    }
                    return Self { entries, path };
                }
            }
        }

        // 读取 JSON 文件并反序列化；任何错误（文件不存在、格式错误）都返回空列表
        let entries: Vec<HistoryEntry> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        log::info!("历史记录加载完成，共 {} 条", entries.len());

        Self { entries, path }
    }

    /// 在列表头部插入一条新记录，并持久化到磁盘
    ///
    /// 超过 200 条时自动截断（保留最新的 200 条）。
    /// 每次插入后都会立即写入磁盘，防止数据丢失。
    pub fn push(&mut self, entry: HistoryEntry) {
        log::debug!("新增历史记录: filename={}, url={}", entry.filename, entry.url);
        self.entries.insert(0, entry); // 插入到列表头部，最新记录在最前面
        self.entries.truncate(200);    // 保留最多 200 条
        if let Err(e) = self.save() {
            log::error!("保存历史记录到磁盘失败: {e}");
        }
    }

    /// 清空所有历史记录并持久化
    pub fn clear(&mut self) {
        log::info!("清空历史记录（共 {} 条）", self.entries.len());
        self.entries.clear();
        if let Err(e) = self.save() {
            log::error!("清空历史记录后保存失败: {e}");
        }
    }

    /// 将当前内存中的历史记录序列化为 JSON 并写入磁盘
    fn save(&self) -> Result<()> {
        // 确保父目录存在（首次运行时 upic-rs 目录可能不存在）
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // pretty 格式方便人工查看和调试
        let json = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(&self.path, json)?;
        log::debug!("历史记录已保存到磁盘: {} 条", self.entries.len());
        Ok(())
    }
}
