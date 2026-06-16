/// commands.rs — Tauri 命令实现模块
///
/// 本模块包含所有可由前端通过 `window.__TAURI__.core.invoke()` 调用的命令函数。
/// 所有命令必须用 `#[tauri::command]` 注解，并在 lib.rs 的 invoke_handler 中注册。
///
/// # 命令列表
/// | 命令名 | 说明 |
/// |--------|------|
/// | `get_config` | 读取当前应用配置 |
/// | `save_config` | 保存配置并按需更新全局快捷键 |
/// | `upload_clipboard` | 从剪贴板上传（前端触发） |
/// | `upload_file` | 从指定文件路径上传 |
/// | `get_history` | 获取上传历史记录列表 |
/// | `clear_history` | 清空上传历史记录 |
/// | `copy_to_clipboard` | 将文本写入系统剪贴板 |
///
/// # 线程安全说明
/// 所有命令通过 `State<'_, Mutex<T>>` 访问共享状态，Tauri 会自动注入状态。
/// 长时间运行的 I/O 操作应使用 `tokio::task::spawn_blocking` 避免阻塞异步运行时。

use crate::{avif, config, history, template, uploader};
use arboard::Clipboard;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// 单文件最大允许大小：500MB
const MAX_FILE_SIZE: u64 = 500 * 1024 * 1024;

// ── 上传结果数据结构 ──────────────────────────────────────────────────────────

/// 单次上传操作的返回结果
///
/// 实现了 `Serialize` 以便通过 Tauri IPC 返回给前端（转换为 JSON 对象）。
#[derive(Debug, Serialize, Clone)]
pub struct UploadResult {
    /// 完整的访问 URL（含自定义域名前缀，如有）
    pub url: String,
    /// 格式化后的链接字符串（根据 link_format 配置，如 Markdown/HTML/URL）
    pub link: String,
    /// 最终文件名（含扩展名），如 `"a3f5c2b1.avif"`
    pub filename: String,
    /// S3 对象 key，如 `"images/2026/05/07/a3f5c2b1.avif"`
    pub key: String,
    /// 上传后文件的字节大小
    pub size_bytes: u64,
}

// ── 配置命令 ──────────────────────────────────────────────────────────────────

/// Tauri 命令：获取当前配置对象
///
/// 前端调用：`const cfg = await invoke("get_config")`
/// 返回完整的配置 JSON 对象，前端用于初始化表单。
#[tauri::command]
pub fn get_config(state: State<'_, Mutex<config::Config>>) -> config::Config {
    let cfg = state.lock().expect("配置锁被污染").clone();
    log::debug!("get_config: bucket={}", cfg.s3.bucket);
    cfg
}

/// Tauri 命令：保存配置到磁盘
///
/// 如果快捷键发生变化，会自动重新注册全局快捷键。
/// 前端调用：`await invoke("save_config", { cfg: configObject })`
///
/// # 错误
/// 返回字符串形式的错误消息（前端 catch 到的 error 内容）
#[tauri::command]
pub fn save_config(
    cfg: config::Config,
    state: State<'_, Mutex<config::Config>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    log::info!("save_config: 开始保存配置");

    // 获取当前快捷键，用于判断是否需要重新注册
    let old_shortcut = state.lock().expect("配置锁被污染").shortcut.clone();
    let new_shortcut = cfg.shortcut.clone();

    log::debug!("save_config: old_shortcut='{old_shortcut}', new_shortcut='{new_shortcut}'");

    // 先持久化到磁盘（如果失败直接返回错误，不更新内存状态）
    cfg.save().map_err(|e| {
        log::error!("保存配置到磁盘失败: {e}");
        e.to_string()
    })?;

    // 更新内存中的状态
    *state.lock().expect("配置锁被污染") = cfg;

    // 如果快捷键发生变化，重新注册全局快捷键
    if new_shortcut != old_shortcut {
        log::info!("快捷键已变更，重新注册: '{old_shortcut}' → '{new_shortcut}'");
        crate::register_global_shortcut(&app, &new_shortcut);
    }

    log::info!("配置保存成功");
    Ok(())
}

// ── 上传命令 ──────────────────────────────────────────────────────────────────

/// Tauri 命令：从剪贴板读取内容并上传
///
/// 此命令直接从前端调用，与 lib.rs 中的 `trigger_upload_from_clipboard` 不同：
/// - 此命令：前端调用，等待结果，直接返回 UploadResult
/// - trigger_upload_from_clipboard：后台触发，通过 Tauri Event 通知前端
///
/// 前端调用：`const result = await invoke("upload_clipboard")`
#[tauri::command]
pub async fn upload_clipboard(
    state: State<'_, Mutex<config::Config>>,
    history_state: State<'_, Mutex<history::History>>,
) -> Result<UploadResult, String> {
    log::info!("[upload_clipboard] 开始");

    // 克隆配置（Mutex 不能跨 await 持有，必须在 await 前释放锁）
    let cfg = state.lock().expect("配置锁被污染").clone();
    log::debug!("[upload_clipboard] 配置读取完成: bucket={}", cfg.s3.bucket);

    // arboard 在 macOS 必须在有 NSRunLoop 的线程运行
    // spawn_blocking 会在专用线程池中运行同步代码，不阻塞 tokio 运行时
    let (data, stem, ext, is_image) = tokio::task::spawn_blocking(read_clipboard)
        .await
        .map_err(|e| {
            let msg = format!("spawn_blocking panic: {e}");
            log::error!("[upload_clipboard] {msg}");
            msg
        })?
        .map_err(|e| {
            log::warn!("[upload_clipboard] 读取剪贴板失败: {e}");
            e.to_string()
        })?;

    log::debug!(
        "[upload_clipboard] 剪贴板内容: stem={stem}, ext={ext}, is_image={is_image}, size={}",
        data.len()
    );

    // 执行格式转换 + S3 上传
    let result = do_upload(data, stem, ext, is_image, &cfg)
        .await
        .map_err(|e| {
            log::error!("[upload_clipboard] 上传失败: {e}");
            e.to_string()
        })?;

    log::info!("[upload_clipboard] 上传成功: url={}", result.url);

    // 记录到历史
    record_history(&history_state, &result);

    // 自动复制链接到剪贴板
    if cfg.upload.auto_copy {
        let link = result.link.clone();
        let _ = tokio::task::spawn_blocking(move || set_clipboard(&link)).await;
        log::debug!("[upload_clipboard] 链接已自动复制: {}", result.link);
    }

    Ok(result)
}

/// Tauri 命令：从指定文件路径读取内容并上传
///
/// 前端调用：`const result = await invoke("upload_file", { path: "/Users/..." })`
#[tauri::command]
pub async fn upload_file(
    path: String,
    state: State<'_, Mutex<config::Config>>,
    history_state: State<'_, Mutex<history::History>>,
) -> Result<UploadResult, String> {
    log::info!("[upload_file] 开始: path={path}");

    let cfg = state.lock().expect("配置锁被污染").clone();
    let p = std::path::PathBuf::from(&path);

    // 校验文件大小，防止超大文件耗尽内存
    let metadata = std::fs::metadata(&p).map_err(|e| {
        log::error!("[upload_file] 获取文件元数据失败: path={path}, error={e}");
        format!("读取文件失败: {e}")
    })?;
    if metadata.len() > MAX_FILE_SIZE {
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        return Err(format!("文件过大 ({:.1}MB)，最大允许 50MB", size_mb));
    }

    // 读取文件内容（磁盘 I/O）
    let data = std::fs::read(&p).map_err(|e| {
        log::error!("[upload_file] 读取文件失败: path={path}, error={e}");
        format!("读取文件失败: {e}")
    })?;

    // 从路径中提取文件名（不含扩展名）和扩展名
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // 根据扩展名判断是否为图片（决定是否执行 AVIF 转换）
    let is_image = avif::is_image_ext(&ext);
    log::debug!(
        "[upload_file] 文件信息: stem={stem}, ext={ext}, is_image={is_image}, size={}",
        data.len()
    );

    let result = do_upload(data, stem, ext, is_image, &cfg)
        .await
        .map_err(|e| {
            log::error!("[upload_file] 上传失败: {e}");
            e.to_string()
        })?;

    log::info!("[upload_file] 上传成功: url={}", result.url);

    record_history(&history_state, &result);

    if cfg.upload.auto_copy {
        let link = result.link.clone();
        let _ = tokio::task::spawn_blocking(move || set_clipboard(&link)).await;
        log::debug!("[upload_file] 链接已自动复制: {}", result.link);
    }

    Ok(result)
}

// ── 历史记录命令 ──────────────────────────────────────────────────────────────

/// Tauri 命令：获取上传历史记录列表
///
/// 返回按时间倒序排列的历史记录数组（最新在前）。
/// 前端调用：`const entries = await invoke("get_history")`
#[tauri::command]
pub fn get_history(
    state: State<'_, Mutex<history::History>>,
) -> Vec<history::HistoryEntry> {
    let entries = state.lock().expect("历史锁被污染").entries.clone();
    log::debug!("get_history: 返回 {} 条记录", entries.len());
    entries
}

/// Tauri 命令：清空所有上传历史记录
///
/// 前端调用：`await invoke("clear_history")`
#[tauri::command]
pub fn clear_history(state: State<'_, Mutex<history::History>>) -> Result<(), String> {
    log::info!("clear_history: 清空历史记录");
    state.lock().expect("历史锁被污染").clear();
    Ok(())
}

/// Tauri 命令：将任意文本写入系统剪贴板
///
/// 前端调用：`await invoke("copy_to_clipboard", { text: "https://..." })`
#[tauri::command]
pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    log::debug!("copy_to_clipboard: 写入 {} 字符", text.len());
    set_clipboard(&text).map_err(|e| {
        log::error!("写入剪贴板失败: {e}");
        e.to_string()
    })
}

// ── 内部辅助函数 ──────────────────────────────────────────────────────────────

/// 从系统剪贴板读取内容，返回 (数据字节, 文件名stem, 扩展名, 是否为图片)
///
/// 读取优先级：
/// 1. 文件路径列表（如从 Finder 复制的文件）→ 读取第一个文件的内容
/// 2. 图片数据（如截图或从网页复制的图片）→ 编码为 PNG
/// 3. 如果都没有 → 返回错误
///
/// # 注意
/// arboard 在 macOS 上必须在有 NSRunLoop 的线程调用（通常是主线程或 spawn_blocking）
pub fn read_clipboard() -> anyhow::Result<(Vec<u8>, String, String, bool)> {
    log::debug!("read_clipboard: 初始化剪贴板");
    let mut cb = Clipboard::new()?;

    // ── 优先：尝试读取文件路径列表 ────────────────────────────────────────
    if let Ok(files) = cb.get().file_list() {
        log::debug!("剪贴板包含文件列表，共 {} 个", files.len());
        if let Some(path) = files.into_iter().next() {
            if path.is_file() {
                log::info!("从剪贴板文件路径读取: {:?}", path);
                let metadata = std::fs::metadata(&path)?;
                if metadata.len() > MAX_FILE_SIZE {
                    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                    anyhow::bail!("文件过大 ({:.1}MB)，最大允许 50MB", size_mb);
                }
                let data = std::fs::read(&path)?;
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("file")
                    .to_string();
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let is_image = avif::is_image_ext(&ext);
                log::debug!("文件读取成功: stem={stem}, ext={ext}, size={}", data.len());
                return Ok((data, stem, ext, is_image));
            }
        }
    }

    // ── 次选：尝试读取图片数据（截图、网页图片等）────────────────────────
    if let Ok(img_data) = cb.get_image() {
        let w = img_data.width as u32;
        let h = img_data.height as u32;
        log::debug!("剪贴板包含图片数据: {}x{}", w, h);

        let bytes = img_data.bytes.into_owned();

        // arboard 返回原始 RGBA 像素数据，需要用 image crate 编码为 PNG
        let img = image::RgbaImage::from_vec(w, h, bytes)
            .ok_or_else(|| anyhow::anyhow!("剪贴板图片数据无效（尺寸与字节数不匹配）"))?;

        let mut png: Vec<u8> = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(
                &mut std::io::Cursor::new(&mut png),
                image::ImageFormat::Png,
            )?;

        // 使用 UUID 前 8 位作为文件名，确保唯一性
        let stem = uuid::Uuid::new_v4().to_string()[..8].to_string();
        log::info!("剪贴板图片已编码为 PNG: stem={stem}, size={}", png.len());
        return Ok((png, stem, "png".to_string(), true));
    }

    // 剪贴板中无可用内容
    log::warn!("剪贴板中无文件或图片内容");
    anyhow::bail!("剪贴板中无文件路径或图片内容，请先复制一个文件或截图")
}

/// 上传核心逻辑：格式转换 + S3 上传 + URL 拼接
///
/// 此函数是整个上传流程的核心，被 `upload_clipboard`、`upload_file` 以及
/// lib.rs 中的 `trigger_upload_from_clipboard` 共同调用。
///
/// # 流程
/// 1. 校验配置完整性（bucket/access_key/secret_key/region）
/// 2. 渲染文件名/目录模板，构建 S3 key
/// 3. 如果是图片且开启了 AVIF 转换：在 spawn_blocking 中执行 AVIF 编码
/// 4. 调用 uploader::upload_to_s3 上传
/// 5. 拼接最终 URL（含自定义 CDN 域名）
/// 6. 格式化链接字符串（Markdown/HTML/URL）
pub async fn do_upload(
    raw: Vec<u8>,
    stem: String,
    ext: String,
    is_image: bool,
    cfg: &config::Config,
) -> anyhow::Result<UploadResult> {
    log::debug!(
        "do_upload: stem={stem}, ext={ext}, is_image={is_image}, raw_size={}",
        raw.len()
    );

    // ── 配置完整性校验 ────────────────────────────────────────────────────
    if cfg.s3.bucket.is_empty() {
        anyhow::bail!("S3 Bucket 未配置，请先在「S3 配置」填写 Bucket 名称");
    }
    if cfg.s3.access_key_id.is_empty() {
        anyhow::bail!("Access Key ID 未配置，请先在「S3 配置」填写");
    }
    if cfg.s3.secret_access_key.is_empty() {
        anyhow::bail!("Secret Access Key 未配置，请先在「S3 配置」填写");
    }
    if cfg.s3.region.is_empty() {
        anyhow::bail!("Region 未配置，请先在「S3 配置」填写");
    }

    // ── 渲染文件名和目录 ──────────────────────────────────────────────────
    let rendered_stem = template::render_filename(&cfg.upload.filename_format, &stem);

    // 如果是图片且开启了 AVIF 转换，最终扩展名强制为 avif
    let final_ext = if is_image && cfg.upload.convert_to_avif {
        "avif".to_string()
    } else {
        ext
    };

    let filename = if final_ext.is_empty() {
        rendered_stem.clone()
    } else {
        format!("{}.{}", rendered_stem, final_ext)
    };

    let dir = if cfg.upload.dir_format.is_empty() {
        String::new()
    } else {
        template::render_dir(&cfg.upload.dir_format)
    };

    let key = template::build_key(&cfg.upload.key_prefix, &dir, &filename);

    log::debug!("文件路径确定: filename={filename}, dir={dir}, key={key}");

    // ── AVIF 转换（如果需要）────────────────────────────────────────────
    let (data, ct) = if is_image && cfg.upload.convert_to_avif {
        log::info!("执行 AVIF 转换: quality={}, speed={}", cfg.upload.avif_quality, cfg.upload.avif_speed);
        let quality = cfg.upload.avif_quality;
        let speed = cfg.upload.avif_speed;

        // AVIF 编码是 CPU 密集型操作，放在 spawn_blocking 中避免阻塞异步运行时
        let bytes = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<u8>> {
            // 从原始字节解码为 DynamicImage（支持 JPEG/PNG/WebP 等所有常见格式）
            let img = image::load_from_memory(&raw)?;
            log::debug!("图片解码成功: {}x{}", img.width(), img.height());
            avif::encode_to_avif(&img, quality, speed)
        })
        .await
        .map_err(|e| anyhow::anyhow!("AVIF 编码 spawn_blocking panic: {e}"))??;

        log::info!("AVIF 转换完成，大小: {} bytes", bytes.len());
        (bytes, "image/avif".to_string())
    } else {
        // 不转换：推断原始文件的 Content-Type
        let ct = template::content_type_for(&filename).to_string();
        log::debug!("跳过 AVIF 转换，使用原始格式: content_type={ct}");
        (raw, ct)
    };

    let size = data.len() as u64;
    log::info!("开始 S3 上传: key={key}, content_type={ct}, size={size}");

    // ── 执行 S3 上传 ──────────────────────────────────────────────────────
    let raw_url = uploader::upload_to_s3(&cfg.s3, &key, data, &ct)
        .await
        .map_err(|e| {
            log::error!("S3 上传失败: {e}");
            e
        })?;

    log::debug!("S3 原始 URL: {raw_url}");

    // ── 拼接最终 URL（含自定义 CDN 域名）────────────────────────────────
    let url = if !cfg.upload.url_prefix.is_empty() {
        // 去掉 key 开头的斜杠，防止 URL 中出现双斜杠
        let clean = key.trim_start_matches('/');
        let final_url = format!(
            "{}/{}",
            cfg.upload.url_prefix.trim_end_matches('/'),
            clean
        );
        log::debug!("使用自定义域名前缀: {} → {}", cfg.upload.url_prefix, final_url);
        final_url
    } else {
        raw_url
    };

    // ── 格式化链接字符串 ──────────────────────────────────────────────────
    // link_name 用作 alt 文字（Markdown/HTML）或自定义模板中的 {name}
    let link_name = filename
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(&filename);
    let link = cfg.upload.link_format.format(&url, link_name, &cfg.upload.custom_link_template);

    log::info!("上传完成: url={url}, link={link}, size={size}");

    Ok(UploadResult {
        url,
        link,
        filename,
        key,
        size_bytes: size,
    })
}

// ── 私有辅助函数 ──────────────────────────────────────────────────────────────

/// 将上传结果记录到历史记录（需持有 Mutex 锁）
fn record_history(state: &Mutex<history::History>, r: &UploadResult) {
    log::debug!("record_history: filename={}", r.filename);
    let mut h = state.lock().expect("历史锁被污染");
    h.push(history::HistoryEntry {
        timestamp: chrono::Local::now().timestamp(),
        filename: r.filename.clone(),
        key: r.key.clone(),
        url: r.url.clone(),
        size_bytes: r.size_bytes,
    });
}

// ── 目录上传 ──────────────────────────────────────────────────────────────────

/// 批量上传结果汇总
///
/// 目录上传完成后返回此结构体，包含成功和失败的统计信息。
#[derive(Debug, Serialize, Clone)]
pub struct BatchUploadResult {
    /// 成功上传的文件列表（含 URL、文件名等完整信息）
    pub succeeded: Vec<UploadResult>,
    /// 上传失败的文件数量
    pub failed_count: usize,
    /// 失败文件的文件名列表（用于错误提示）
    pub failed_names: Vec<String>,
    /// 目录中全部文件总数（含跳过的隐藏文件之外的文件）
    pub total: usize,
}

/// 递归收集目录下所有文件路径（跳过 . 开头的隐藏文件和 macOS 元数据文件）
fn collect_files_recursive(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        // 按文件名排序，保证遍历顺序一致
        let mut sorted: Vec<_> = entries.flatten().collect();
        sorted.sort_by_key(|e| e.file_name());
        for entry in sorted {
            let path = entry.path();
            // 跳过隐藏文件（. 开头）
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    log::trace!("跳过隐藏文件: {:?}", path);
                    continue;
                }
            }
            if path.is_dir() {
                // 递归进入子目录
                files.extend(collect_files_recursive(&path));
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files
}

/// Tauri 命令：上传整个目录下的所有文件（递归）
///
/// 递归遍历指定目录（含子目录），对每个文件执行上传：
/// - 图片文件（jpg/png/gif/webp 等）：按配置决定是否转换为 AVIF
/// - 非图片文件（pdf/zip/tar 等）：直接上传原始内容，不做任何转换
/// - 隐藏文件（. 开头）和 macOS 元数据文件：自动跳过
///
/// 上传过程中遇到单个文件失败不会中止整批，失败信息收集在返回结果中。
/// 批量上传不触发 `auto_copy`（多文件场景下复制单个链接无意义）。
///
/// 前端调用：`const result = await invoke("upload_directory", { path: "/Users/..." })`
#[tauri::command]
pub async fn upload_directory(
    path: String,
    state: State<'_, Mutex<config::Config>>,
    history_state: State<'_, Mutex<history::History>>,
) -> Result<BatchUploadResult, String> {
    log::info!("[upload_directory] 开始: path={path}");
    let cfg = state.lock().expect("配置锁被污染").clone();
    let dir = std::path::PathBuf::from(&path);

    if !dir.is_dir() {
        return Err(format!("路径不是目录: {path}"));
    }

    let files = collect_files_recursive(&dir);
    let total = files.len();
    log::info!("[upload_directory] 目录扫描完成，共 {total} 个文件待上传");

    if total == 0 {
        return Ok(BatchUploadResult {
            succeeded: vec![],
            failed_count: 0,
            failed_names: vec![],
            total: 0,
        });
    }

    // 预读取所有文件并准备上传任务
    struct UploadTask {
        stem: String,
        ext: String,
        data: Vec<u8>,
        is_image: bool,
        display_name: String,
    }

    let mut tasks: Vec<UploadTask> = Vec::new();
    let mut skipped_names: Vec<String> = Vec::new();

    for file_path in &files {
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        let ext = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let display_name = format!("{}.{}", stem, ext);

        // 校验文件大小
        if let Ok(m) = std::fs::metadata(file_path) {
            if m.len() > MAX_FILE_SIZE {
                log::warn!("[upload_directory] 跳过过大文件: {display_name}");
                skipped_names.push(display_name);
                continue;
            }
        }

        match std::fs::read(file_path) {
            Ok(data) => {
                let is_image = avif::is_image_ext(&ext);
                tasks.push(UploadTask { stem, ext, data, is_image, display_name });
            }
            Err(e) => {
                log::error!("[upload_directory] 读取失败: {display_name}, {e}");
                skipped_names.push(display_name);
            }
        }
    }

    log::info!("[upload_directory] 预读取完成，{} 个文件待并发上传，{} 个跳过", tasks.len(), skipped_names.len());

    // 并发上传（最多 4 个并发）
    const CONCURRENCY: usize = 4;
    let mut succeeded = Vec::new();
    let mut failed_names = skipped_names;

    for chunk in tasks.chunks(CONCURRENCY) {
        let mut handles = Vec::new();
        for task in chunk {
            let cfg_clone = cfg.clone();
            let stem = task.stem.clone();
            let ext = task.ext.clone();
            let data = task.data.clone();
            let is_image = task.is_image;
            let display_name = task.display_name.clone();

            handles.push(tokio::spawn(async move {
                let result = do_upload(data, stem, ext, is_image, &cfg_clone).await;
                (display_name, result)
            }));
        }

        for handle in handles {
            match handle.await {
                Ok((display_name, Ok(result))) => {
                    log::info!("[upload_directory] 成功: {} → {}", display_name, result.url);
                    record_history(&history_state, &result);
                    succeeded.push(result);
                }
                Ok((display_name, Err(e))) => {
                    log::error!("[upload_directory] 上传失败: {display_name}, {e}");
                    failed_names.push(display_name);
                }
                Err(e) => {
                    log::error!("[upload_directory] 任务 panic: {e}");
                }
            }
        }
    }

    let failed_count = failed_names.len();
    log::info!(
        "[upload_directory] 全部完成: 成功={}, 失败={failed_count}, 总计={total}",
        succeeded.len()
    );

    Ok(BatchUploadResult {
        succeeded,
        failed_count,
        failed_names,
        total,
    })
}

/// 将文本写入系统剪贴板（同步版本，用于 spawn_blocking 或直接调用）
fn set_clipboard(text: &str) -> anyhow::Result<()> {
    log::debug!("set_clipboard: {} 字符", text.len());
    let mut cb = Clipboard::new()?;
    cb.set_text(text)?;
    Ok(())
}


