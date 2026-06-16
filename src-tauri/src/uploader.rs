/// uploader.rs — AWS S3 上传模块
///
/// 本模块封装了与 Amazon S3（及兼容服务如 Cloudflare R2、MinIO）交互的逻辑。
/// 使用官方 `aws-sdk-s3` crate，支持：
/// - 标准 AWS S3（虚拟域名 URL 格式）
/// - 自定义 Endpoint（MinIO / Cloudflare R2 / 其他 S3 兼容存储）
/// - 路径风格 URL（MinIO 需要，`force_path_style = true`）

use anyhow::Result;
use aws_sdk_s3::{
    config::{BehaviorVersion, Credentials, Region},
    primitives::ByteStream,
    Client,
};

use crate::config::S3Config;

/// 将文件数据上传到 S3，返回该对象的直接访问 URL
///
/// # 参数
/// - `cfg`: S3 连接配置（endpoint/bucket/region/credentials）
/// - `key`: S3 对象 key，即存储路径，如 `"images/2026/05/07/photo.avif"`
/// - `data`: 文件内容字节数组
/// - `content_type`: HTTP Content-Type 头，如 `"image/avif"`
///
/// # 返回值
/// 成功时返回 S3 原始访问 URL（未含自定义 CDN 域名前缀）。
/// 调用方（`commands::do_upload`）负责替换为自定义域名。
///
/// # 错误
/// - 网络连接失败：DNS 解析失败、超时等
/// - S3 服务错误：权限不足、Bucket 不存在、Key 格式错误等
/// - 凭证错误：Access Key / Secret Key 无效
pub async fn upload_to_s3(
    cfg: &S3Config,
    key: &str,
    data: Vec<u8>,
    content_type: &str,
) -> Result<String> {
    log::debug!(
        "upload_to_s3: bucket={}, key={}, content_type={}, size={}",
        cfg.bucket, key, content_type, data.len()
    );

    // 构建临时凭证对象（静态凭证，不使用 IAM Role 或 STS）
    // 参数：access_key_id, secret_access_key, session_token, expiry, provider_name
    let creds = Credentials::new(
        &cfg.access_key_id,
        &cfg.secret_access_key,
        None::<String>, // session_token：普通 IAM 用户不需要
        None,           // expiry：静态凭证不过期
        "avif-image",   // provider_name：自定义标识符，用于日志/调试
    );

    // 构建 SDK 配置（使用 Builder 模式）
    let mut builder = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest()) // 使用最新行为版本
        .region(Region::new(cfg.region.clone()))
        .credentials_provider(creds);

    // 如果配置了自定义 Endpoint（MinIO/R2 等），覆盖默认的 AWS 域名
    if let Some(endpoint) = &cfg.endpoint_url {
        log::debug!("使用自定义 Endpoint: {endpoint}");
        builder = builder.endpoint_url(endpoint);
    }

    // 路径风格 URL：`http://minio:9000/bucket/key` 而非 `http://bucket.minio:9000/key`
    // MinIO 通常需要此选项；AWS S3 和 Cloudflare R2 不需要
    if cfg.path_style {
        log::debug!("启用路径风格 URL (force_path_style)");
        builder = builder.force_path_style(true);
    }

    let client = Client::from_conf(builder.build());

    log::debug!("开始执行 S3 PutObject 请求…");

    // 执行 PutObject 操作，将文件内容上传到指定 key
    // ByteStream::from(data) 将 Vec<u8> 转换为 SDK 所需的流格式
    client
        .put_object()
        .bucket(&cfg.bucket)
        .key(key)
        .body(ByteStream::from(data))
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| {
            // 详细记录 SDK 错误信息，方便排查问题
            log::error!("S3 PutObject 失败（详细）: {:#?}", e);

            // 将 SDK 错误转换为人类可读的字符串
            let msg = match &e {
                aws_sdk_s3::error::SdkError::DispatchFailure(d) => {
                    // 网络层错误（DNS 解析失败、连接超时、TLS 证书错误等）
                    let src = d.as_connector_error()
                        .map(|c| format!("{c}"))
                        .unwrap_or_else(|| format!("{d:?}"));
                    format!("网络连接失败: {src}")
                }
                aws_sdk_s3::error::SdkError::ServiceError(se) => {
                    // S3 服务返回的错误（403 权限、404 Bucket 不存在等）
                    format!("S3 服务错误: {}", se.err())
                }
                other => format!("上传失败: {other}"),
            };
            anyhow::anyhow!("{}", msg)
        })?;

    // 根据配置构建 S3 对象的原始访问 URL
    let url = s3_url(cfg, key);
    log::info!("S3 上传成功: key={key}, url={url}");
    Ok(url)
}

/// 构建 S3 对象的原始访问 URL（不含自定义 CDN 域名前缀）
///
/// 根据是否配置了自定义 Endpoint 选择不同的 URL 格式：
/// - 有自定义 Endpoint（MinIO/R2）：`{endpoint}/{bucket}/{key}`
/// - 标准 AWS S3：`https://{bucket}.s3.{region}.amazonaws.com/{key}`
fn s3_url(cfg: &S3Config, key: &str) -> String {
    // 去掉 key 开头的斜杠，避免 URL 中出现双斜杠
    let clean = key.trim_start_matches('/');

    if let Some(ep) = &cfg.endpoint_url {
        // 自定义 Endpoint：使用路径风格 URL（bucket 作为路径的一部分）
        let url = format!("{}/{}/{}", ep.trim_end_matches('/'), cfg.bucket, clean);
        log::debug!("生成自定义 Endpoint URL: {url}");
        return url;
    }

    // 标准 AWS S3：使用虚拟托管风格 URL（bucket 作为子域名）
    let url = format!(
        "https://{}.s3.{}.amazonaws.com/{}",
        cfg.bucket, cfg.region, clean
    );
    log::debug!("生成 AWS S3 URL: {url}");
    url
}
