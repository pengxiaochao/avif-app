/// template.rs — 文件名/目录路径模板渲染模块
///
/// 本模块负责将用户在设置界面配置的路径模板（包含 `{变量}` 占位符）
/// 替换为实际值，用于生成最终的 S3 存储路径。
///
/// # 支持的变量
///
/// | 变量 | 说明 | 示例 |
/// |------|------|------|
/// | `{Y}` | 4 位年份 | 2026 |
/// | `{m}` | 2 位月份 | 05 |
/// | `{d}` | 2 位日期 | 07 |
/// | `{H}` | 2 位小时（24h）| 14 |
/// | `{M}` | 2 位分钟 | 30 |
/// | `{S}` | 2 位秒数 | 59 |
/// | `{uuid}` | 完整 UUID | `550e8400-e29b...` |
/// | `{uuid8}` | UUID 前 8 位 | `550e8400` |
/// | `{name}` | 原始文件名（不含扩展名） | `photo` |
/// | `{timestamp}` | Unix 时间戳（秒） | `1746614400` |

use chrono::Local;
use uuid::Uuid;

/// 清理文件名中的危险字符，防止 S3 key 路径穿越
///
/// - 移除 `/` 和 `\`（路径分隔符）
/// - 移除 `..`（上级目录引用）
/// - 移除控制字符
fn sanitize_name(name: &str) -> String {
    name.replace('/', "_")
        .replace('\\', "_")
        .replace("..", "_")
        .chars()
        .filter(|c| !c.is_control())
        .collect()
}

/// 渲染文件名模板（不含扩展名）
///
/// 对每个 `{变量}` 占位符进行字符串替换，生成最终的文件名。
/// 注意：此函数不会添加扩展名，调用方需要自行追加。
///
/// # 参数
/// - `template`: 文件名模板，如 `"{uuid8}"` 或 `"{Y}{m}{d}_{name}"`
/// - `original_name`: 原始文件的 stem（不含扩展名），供 `{name}` 变量使用
///
/// # 示例
/// - `render_filename("{uuid8}", "photo")` → `"a3f5c2b1"`
/// - `render_filename("{Y}/{m}/{d}", "img")` → `"2026/05/07"`（注意：包含斜杠时不适合作文件名）
pub fn render_filename(template: &str, original_name: &str) -> String {
    let now = Local::now();
    // 每次调用生成一个全新的 UUID，确保文件名唯一性
    let uid = Uuid::new_v4().to_string();
    let uid8 = &uid[..8]; // 取前 8 位作为短 UUID

    // 对原始文件名进行清理：移除路径分隔符和特殊字符，防止 S3 key 路径穿越
    let safe_name = sanitize_name(original_name);

    let result = template
        .replace("{Y}", &now.format("%Y").to_string())         // 年
        .replace("{m}", &now.format("%m").to_string())         // 月
        .replace("{d}", &now.format("%d").to_string())         // 日
        .replace("{H}", &now.format("%H").to_string())         // 小时
        .replace("{M}", &now.format("%M").to_string())         // 分钟
        .replace("{S}", &now.format("%S").to_string())         // 秒
        .replace("{uuid}", &uid)                               // 完整 UUID
        .replace("{uuid8}", uid8)                              // UUID 前 8 位
        .replace("{name}", &safe_name)                         // 原始文件名（已清理）
        .replace("{timestamp}", &now.timestamp().to_string()); // Unix 时间戳

    log::trace!("render_filename: template='{template}', original_name='{original_name}' → '{result}'");
    result
}

/// 渲染目录路径模板，并去掉首尾的斜杠
///
/// 与 `render_filename` 类似，但不支持 `{uuid}` `{uuid8}` `{name}` 变量，
/// 因为目录名通常基于时间维度分组，不需要唯一性标识符。
///
/// # 示例
/// - `render_dir("{Y}/{m}/{d}")` → `"2026/05/07"`
/// - `render_dir("")` → `""` （空模板返回空字符串）
pub fn render_dir(template: &str) -> String {
    let now = Local::now();
    let result = template
        .replace("{Y}", &now.format("%Y").to_string())
        .replace("{m}", &now.format("%m").to_string())
        .replace("{d}", &now.format("%d").to_string())
        .replace("{H}", &now.format("%H").to_string())
        .replace("{M}", &now.format("%M").to_string())
        .replace("{S}", &now.format("%S").to_string())
        .replace("{timestamp}", &now.timestamp().to_string())
        .trim_matches('/') // 去掉首尾斜杠，保持 S3 key 格式正确
        .to_string();

    log::trace!("render_dir: template='{template}' → '{result}'");
    result
}

/// 拼接完整的 S3 存储 key
///
/// 将「路径前缀 / 日期目录 / 文件名」三段拼接成最终的 S3 key。
/// 空段会被自动跳过，避免出现多余的斜杠。
///
/// # 参数
/// - `key_prefix`: 固定路径前缀（如 `"images"`），末尾不含斜杠
/// - `dir`: 目录路径（如 `"2026/05/07"`），可为空
/// - `filename`: 完整文件名（含扩展名，如 `"a3f5c2b1.avif"`）
///
/// # 示例
/// ```
/// build_key("images", "2026/05/07", "a3f5c2b1.avif") → "images/2026/05/07/a3f5c2b1.avif"
/// build_key("", "2026/05/07", "photo.png")            → "2026/05/07/photo.png"
/// build_key("uploads", "", "file.avif")               → "uploads/file.avif"
/// ```
pub fn build_key(key_prefix: &str, dir: &str, filename: &str) -> String {
    let key = [key_prefix, dir, filename]
        .iter()
        .filter(|s| !s.is_empty()) // 过滤空段，防止多余斜杠
        .copied()
        .collect::<Vec<_>>()
        .join("/");

    // 清理最终 key：移除连续斜杠、路径穿越、首尾斜杠
    let key = key.replace("..", "").replace("//", "/");
    let key = key.trim_matches('/').to_string();

    log::debug!("build_key: prefix='{key_prefix}', dir='{dir}', filename='{filename}' → '{key}'");
    key
}

/// 根据文件扩展名推断 HTTP Content-Type
///
/// 上传到 S3 时需要设置正确的 Content-Type，
/// 浏览器和 CDN 会根据此字段决定如何处理/渲染文件。
/// 如果扩展名未知，返回通用二进制类型 `application/octet-stream`。
///
/// # 参数
/// - `filename`: 包含扩展名的完整文件名，如 `"photo.avif"` 或 `"document.pdf"`
pub fn content_type_for(filename: &str) -> &'static str {
    // rsplit('.').next() 取最后一个点之后的部分作为扩展名
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let ct = match ext.as_str() {
        "jpg" | "jpeg"  => "image/jpeg",
        "png"           => "image/png",
        "gif"           => "image/gif",
        "webp"          => "image/webp",
        "avif"          => "image/avif",    // AVIF 格式
        "bmp"           => "image/bmp",
        "tiff" | "tif"  => "image/tiff",
        "svg"           => "image/svg+xml",
        "ico"           => "image/x-icon",
        "pdf"           => "application/pdf",
        "mp4"           => "video/mp4",
        "mov"           => "video/quicktime",
        "mp3"           => "audio/mpeg",
        "txt"           => "text/plain",
        "html" | "htm"  => "text/html",
        "css"           => "text/css",
        "js"            => "application/javascript",
        "json"          => "application/json",
        _               => "application/octet-stream", // 未知类型
    };
    log::trace!("content_type_for('{filename}') = '{ct}'");
    ct
}
