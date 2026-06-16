/// avif.rs — AVIF 图片编码模块
///
/// 本模块封装了将 `image::DynamicImage` 编码为 AVIF 格式的逻辑，
/// 使用 `ravif` crate 作为 AVIF 编码器（基于 libaom）。
///
/// AVIF（AV1 Image File Format）是目前最先进的图片格式之一：
/// - 相比 JPEG：同画质文件体积小约 50%
/// - 相比 PNG：支持有损压缩，体积更小
/// - 相比 WebP：更好的压缩率与色彩还原

use anyhow::Result;
use image::DynamicImage;
use ravif::{Encoder, Img};
use rgb::RGBA8;

/// 将 DynamicImage 编码为 AVIF 格式的字节流
///
/// # 参数
/// - `img`: 已解码的图片对象（支持任意色彩格式，内部会转换为 RGBA8）
/// - `quality`: 编码质量 1.0–100.0，越高越清晰，文件越大（推荐 70–90）
/// - `speed`: 编码速度 1–10，越小编码越慢但压缩率越高（推荐 6–8）
///
/// # 返回值
/// 成功时返回包含完整 AVIF 文件内容的字节向量，可直接写入文件或上传。
///
/// # 示例
/// ```rust
/// let img = image::open("photo.jpg").unwrap();
/// let avif_bytes = encode_to_avif(&img, 80.0, 6).unwrap();
/// ```
pub fn encode_to_avif(img: &DynamicImage, quality: f32, speed: u8) -> Result<Vec<u8>> {
    log::debug!(
        "encode_to_avif 开始: {}x{}, quality={quality}, speed={speed}",
        img.width(), img.height()
    );

    // 将任意格式图片统一转换为 RGBA8（每像素 4 字节：R/G/B/A）
    // to_rgba8() 会自动处理色彩空间转换（如 CMYK→RGB、灰度→RGB 等）
    let rgba = img.to_rgba8();
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;

    // 将 image crate 的像素格式转换为 ravif 所需的 rgb::RGBA8 格式
    let pixels: Vec<RGBA8> = rgba
        .pixels()
        .map(|p| RGBA8 {
            r: p[0], // 红色通道
            g: p[1], // 绿色通道
            b: p[2], // 蓝色通道
            a: p[3], // Alpha 透明度通道
        })
        .collect();

    log::debug!("像素转换完成，开始 AVIF 编码: {} 像素", pixels.len());

    // 使用 ravif 的 Encoder 进行 AVIF 编码
    // Img::new 构造一个借用像素数据的图像视图（零拷贝）
    let encoded = Encoder::new()
        .with_quality(quality)   // 设置质量参数
        .with_speed(speed)       // 设置编码速度
        .encode_rgba(Img::new(&pixels, width, height))?; // 执行编码

    let avif_size = encoded.avif_file.len();
    log::info!("AVIF 编码完成: 原始 RGBA 大小={} bytes, AVIF 大小={avif_size} bytes, 压缩率={:.1}%",
        pixels.len() * 4,
        (1.0 - avif_size as f32 / (pixels.len() * 4) as f32) * 100.0
    );

    // encoded.avif_file 是完整的 AVIF 文件字节（包含文件头、元数据等）
    Ok(encoded.avif_file)
}

/// 根据文件扩展名判断是否为可处理的图片格式
///
/// 支持的格式：jpg/jpeg/png/gif/bmp/tiff/tif/webp/ico
/// 这些格式都可以被 `image` crate 解码后转换为 AVIF。
///
/// # 参数
/// - `ext`: 不含点的文件扩展名，如 `"jpg"`、`"PNG"`（大小写不敏感）
///
/// # 返回值
/// - `true`: 该扩展名对应支持的图片格式
/// - `false`: 不支持（如 pdf、mp4 等非图片格式）
pub fn is_image_ext(ext: &str) -> bool {
    let result = matches!(
        ext.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "ico"
    );
    log::trace!("is_image_ext('{ext}') = {result}");
    result
}
