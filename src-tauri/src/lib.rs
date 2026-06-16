/// lib.rs — Tauri 应用主入口与初始化模块
///
/// 本文件负责：
///   1. 声明所有子模块（avif/commands/config/history/template/uploader）
///   2. 初始化 Tauri Builder，注册插件、状态、命令
///   3. 构建系统托盘菜单与图标
///   4. 注册/更新全局快捷键
///   5. 处理窗口关闭事件（隐藏而非退出）

// ── 子模块声明 ─────────────────────────────────────────────────────────────
// Rust 会在 src/ 目录下自动查找同名 .rs 文件
mod avif;      // AVIF 图片编码（encode_to_avif / is_image_ext）
mod commands;  // Tauri 命令（前端 invoke 调用的所有函数）
mod config;    // 应用配置结构体与磁盘持久化
mod history;   // 上传历史记录（内存 + JSON 文件）
mod template;  // 文件名/目录模板变量替换
mod uploader;  // AWS S3 上传客户端封装

use commands::{do_upload, read_clipboard};
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use std::str::FromStr;

/// 应用程序入口点（由 main.rs 调用）
///
/// `#[cfg_attr(mobile, tauri::mobile_entry_point)]` 宏仅在编译移动端时有效，
/// 桌面端此注解不产生任何影响。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    // 默认级别 info；开发时可设环境变量 RUST_LOG=debug 获得更详细输出
    // 示例：RUST_LOG=avif_image_lib=debug,info cargo tauri dev
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    log::info!("avif-image 启动，开始构建 Tauri 应用…");

    tauri::Builder::default()
        // ── 注册 Tauri 插件 ────────────────────────────────────────────────
        // shell：允许通过 window.__TAURI__.shell.open() 打开外部浏览器
        .plugin(tauri_plugin_shell::init())
        // notification：调用系统通知中心弹出通知
        .plugin(tauri_plugin_notification::init())
        // dialog：原生文件选择/保存对话框
        .plugin(tauri_plugin_dialog::init())
        // global_shortcut：即使窗口失焦也能响应全局快捷键
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // autostart：通过 macOS LaunchAgent 机制实现登录自动启动
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // ── 注册全局状态 ──────────────────────────────────────────────────
        // Mutex 保证多个 async 命令并发访问时的线程安全
        // Tauri 会通过 State<'_, T> 自动注入到命令函数中
        .manage(Mutex::new(config::Config::load()))    // 应用配置
        .manage(Mutex::new(history::History::load()))  // 上传历史
        // ── 注册前端可调用的命令 ──────────────────────────────────────────
        // 前端：await invoke("命令名", { 参数名: 值 })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,           // 读取完整配置对象
            commands::save_config,          // 保存配置并按需更新快捷键
            commands::upload_clipboard,     // 前端按钮触发的剪贴板上传
            commands::upload_file,          // 前端按钮触发的文件路径上传
            commands::upload_directory,     // 前端按钮触发的目录批量上传
            commands::get_history,          // 获取历史记录列表
            commands::clear_history,        // 清空历史记录
            commands::copy_to_clipboard,    // 将任意文本写入剪贴板
            register_shortcut,              // 动态注册/更换全局快捷键
            trigger_clipboard_upload,       // 前端手动触发剪贴板上传
        ])
        .setup(|app| {
            log::debug!("setup 回调开始，初始化托盘和快捷键…");

            // macOS 特有：隐藏 Dock 图标，使应用仅在菜单栏运行
            // Accessory 模式：不出现在 Dock 和 Cmd+Tab 切换器中
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // ── 构建托盘右键菜单 ──────────────────────────────────────────
            // MenuItem::new(app, 显示文字, 是否启用, 可选快捷键)
            let item_clipboard =
                MenuItem::new(app, "从剪贴板上传", true, None::<&str>)?;
            let item_file =
                MenuItem::new(app, "选择文件上传…", true, None::<&str>)?;
            let item_settings =
                MenuItem::new(app, "偏好设置…", true, None::<&str>)?;
            let item_history =
                MenuItem::new(app, "上传历史", true, None::<&str>)?;
            let item_quit = MenuItem::new(app, "退出", true, None::<&str>)?;

            // PredefinedMenuItem::separator 插入系统风格分隔线
            let menu = Menu::with_items(
                app,
                &[
                    &item_clipboard,
                    &item_file,
                    &PredefinedMenuItem::separator(app)?,
                    &item_settings,
                    &item_history,
                    &PredefinedMenuItem::separator(app)?,
                    &item_quit,
                ],
            )?;

            log::debug!("托盘菜单构建完成，共 {} 个可见项", 5);

            // ── 构建系统托盘图标 ──────────────────────────────────────────
            let tray_icon_image = build_tray_icon();

            TrayIconBuilder::new()
                .icon(tray_icon_image)
                // macOS：icon_as_template=true 让系统根据亮/暗模式自动反色
                .icon_as_template(true)
                .tooltip("avif-image")
                .menu(&menu)
                // 左键单击托盘图标时直接弹出菜单（符合 macOS 惯例）
                .show_menu_on_left_click(true)
                // 菜单事件回调：通过比较 id 区分被点击的菜单项
                .on_menu_event({
                    let app_handle = app.handle().clone();
                    move |_tray, event| {
                        let id = event.id.as_ref();
                        log::debug!("托盘菜单点击: id={id}");
                        match id {
                            id if id == item_quit.id().as_ref() => {
                                log::info!("用户选择退出");
                                app_handle.exit(0);
                            }
                            id if id == item_settings.id().as_ref() => {
                                log::debug!("打开偏好设置 (tab=s3)");
                                show_settings(&app_handle, "s3");
                            }
                            id if id == item_history.id().as_ref() => {
                                log::debug!("打开历史记录 (tab=history)");
                                show_settings(&app_handle, "history");
                            }
                            id if id == item_clipboard.id().as_ref() => {
                                log::debug!("托盘触发剪贴板上传");
                                let handle = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    trigger_upload_from_clipboard(&handle).await;
                                });
                            }
                            id if id == item_file.id().as_ref() => {
                                log::debug!("打开文件上传窗口 (tab=upload)");
                                show_settings(&app_handle, "upload");
                            }
                            _ => {
                                log::warn!("未处理的托盘菜单事件: id={id}");
                            }
                        }
                    }
                })
                // 托盘图标本身的点击事件（左键已由 show_menu_on_left_click 处理）
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // 左键单击：弹出菜单已由 show_menu_on_left_click 处理，此处为扩展预留
                        log::trace!("托盘图标左键点击（菜单已自动弹出）");
                    }
                })
                .build(app)?;

            log::debug!("系统托盘构建完成");

            // ── 从持久化配置注册全局快捷键 ───────────────────────────────
            // 克隆配置，避免长时间持有 Mutex 锁影响其他命令
            let cfg = app
                .state::<Mutex<config::Config>>()
                .lock()
                .expect("配置锁被污染")
                .clone();

            log::info!("从配置注册全局快捷键: '{}'", cfg.shortcut);
            register_global_shortcut(app.handle(), &cfg.shortcut);

            log::info!("应用初始化完成，等待用户操作");
            Ok(())
        })
        // 拦截窗口关闭事件：点击红色关闭按钮时隐藏窗口而非退出
        // 用户可通过托盘菜单「退出」彻底关闭程序
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                log::debug!("窗口关闭请求被拦截，改为隐藏（窗口: {}）", window.label());
                api.prevent_close(); // 阻止默认关闭行为
                window.hide().ok();  // 仅隐藏，不销毁
            }
        })
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}

// ── 全局快捷键注册 ────────────────────────────────────────────────────────────

/// 注册（或替换）全局快捷键
///
/// 流程：
///   1. 先注销所有已注册快捷键（防止重复注册或冲突）
///   2. 解析 shortcut_str（Tauri accelerator 格式，如 "CmdOrCtrl+Shift+U"）
///   3. 注册新快捷键，回调中触发剪贴板上传
///
/// # 参数
/// - `app`: Tauri 应用句柄，用于访问全局快捷键插件
/// - `shortcut_str`: 快捷键字符串，空字符串表示不注册
pub fn register_global_shortcut(app: &tauri::AppHandle, shortcut_str: &str) {
    log::debug!("register_global_shortcut 调用: shortcut='{shortcut_str}'");

    // 先清空所有快捷键，防止多次调用造成重复注册
    if let Err(e) = app.global_shortcut().unregister_all() {
        log::warn!("注销所有快捷键时出错（通常无害）: {e}");
    }

    if shortcut_str.is_empty() {
        log::info!("快捷键为空字符串，已注销所有快捷键");
        return;
    }

    let handle = app.clone();
    // 转换为 owned String，因为后面的 move 闭包要求 'static
    let shortcut_owned = shortcut_str.to_string();

    // 解析快捷键字符串
    let shortcut = match Shortcut::from_str(shortcut_str) {
        Ok(s) => s,
        Err(e) => {
            log::error!("快捷键 '{shortcut_str}' 格式无效，注册失败: {e}");
            return;
        }
    };

    // 注册按键回调：Pressed 状态（按下瞬间）触发上传
    if let Err(e) = app
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                log::info!("全局快捷键 '{shortcut_owned}' 触发，启动剪贴板上传");
                let h = handle.clone();
                tauri::async_runtime::spawn(async move {
                    trigger_upload_from_clipboard(&h).await;
                });
            }
        })
    {
        log::error!("注册快捷键 '{shortcut_str}' 失败: {e}");
    } else {
        log::info!("全局快捷键注册成功: '{shortcut_str}'");
    }
}

/// Tauri 命令：由前端调用，动态更新全局快捷键
///
/// 先校验格式，再注册到操作系统，最后持久化到配置文件。
/// 前端调用：`await invoke("register_shortcut", { shortcut: "Alt+Shift+U" })`
#[tauri::command]
async fn register_shortcut(
    shortcut: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<config::Config>>,
) -> Result<(), String> {
    log::info!("前端请求更新快捷键: '{shortcut}'");

    // 校验格式（Tauri 的 Shortcut::from_str 会验证 accelerator 语法）
    Shortcut::from_str(&shortcut)
        .map_err(|e| format!("快捷键格式无效 '{}': {}", shortcut, e))?;

    register_global_shortcut(&app, &shortcut);

    // 持久化新快捷键
    let mut cfg = state.lock().expect("配置锁被污染");
    cfg.shortcut = shortcut.clone();
    cfg.save().map_err(|e| {
        log::error!("持久化快捷键失败: {e}");
        e.to_string()
    })?;

    log::info!("快捷键已更新并保存: '{shortcut}'");
    Ok(())
}

/// Tauri 命令：由前端按钮调用，手动触发剪贴板上传
///
/// 与全局快捷键触发的行为完全一致。
#[tauri::command]
async fn trigger_clipboard_upload(app: tauri::AppHandle) {
    log::debug!("前端触发剪贴板上传命令（trigger_clipboard_upload）");
    trigger_upload_from_clipboard(&app).await;
}

// ── 剪贴板上传核心（托盘菜单 / 快捷键 / 前端均通过此函数执行）─────────────────

/// 从剪贴板读取内容并完整执行上传流程
///
/// 流程：
///   1. 读取配置
///   2. 在阻塞线程中读取剪贴板（arboard 需要 NSRunLoop，必须用 spawn_blocking）
///   3. 调用 do_upload（格式转换 + S3 上传）
///   4. 写入历史记录
///   5. 可选：自动复制链接到剪贴板
///   6. 通过 Tauri 事件通知前端（窗口隐藏时也能接收）
async fn trigger_upload_from_clipboard(app: &tauri::AppHandle) {
    log::info!("=== 剪贴板上传流程开始 ===");

    let cfg = app
        .state::<Mutex<config::Config>>()
        .lock()
        .expect("配置锁被污染")
        .clone();
    log::debug!("已读取配置: bucket={}", cfg.s3.bucket);

    // 读取剪贴板内容
    // arboard 在 macOS 上必须在有 NSRunLoop 的线程（主线程或 spawn_blocking 线程）调用
    let (data, stem, ext, is_image) = match read_clipboard() {
        Ok(v) => {
            log::debug!("剪贴板读取成功: stem={}, ext={}, is_image={}, size={}", v.1, v.2, v.3, v.0.len());
            v
        }
        Err(e) => {
            log::warn!("读取剪贴板失败，上传中止: {e}");
            app.emit("upload-err", format!("读取剪贴板失败: {}", e)).ok();
            show_notification(app, "上传失败", &format!("读取剪贴板失败: {}", e));
            return;
        }
    };

    // 通知前端：上传即将开始（可用于 UI 显示 loading 状态）
    log::debug!("发送 upload-start 事件: stem={stem}");
    app.emit("upload-start", &stem).ok();

    // 执行格式转换 + S3 上传
    log::debug!("调用 do_upload: stem={stem}, ext={ext}, is_image={is_image}");
    match do_upload(data, stem, ext, is_image, &cfg).await {
        Ok(result) => {
            log::info!("上传成功: filename={}, url={}, size={}", result.filename, result.url, result.size_bytes);

            // 写入历史记录
            {
                let history_state = app.state::<Mutex<history::History>>();
                let mut h = history_state.lock().expect("历史锁被污染");
                h.push(history::HistoryEntry {
                    timestamp: chrono::Local::now().timestamp(),
                    filename: result.filename.clone(),
                    key: result.key.clone(),
                    url: result.url.clone(),
                    size_bytes: result.size_bytes,
                });
                log::debug!("历史记录已写入: filename={}", result.filename);
            }

            // 可选：上传成功后自动将链接复制到剪贴板
            if cfg.upload.auto_copy {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(&result.link);
                    log::debug!("链接已自动复制到剪贴板: {}", result.link);
                } else {
                    log::warn!("自动复制失败：无法初始化剪贴板");
                }
            }

            // 通知前端：上传成功，携带完整结果对象
            app.emit("upload-ok", &result).ok();
            show_notification(
                app,
                "上传成功",
                &format!("链接已复制: {}", &result.url),
            );
            log::info!("=== 剪贴板上传流程完成 ===");
        }
        Err(e) => {
            log::error!("上传失败: {e}");
            app.emit("upload-err", e.to_string()).ok();
            show_notification(app, "上传失败", &e.to_string());
        }
    }
}

// ── 工具函数 ──────────────────────────────────────────────────────────────────

/// 显示（并聚焦）设置窗口，同时切换到指定标签页
///
/// 通过 JavaScript eval 调用前端暴露的 `window.switchTab(tab)` 函数。
fn show_settings(app: &tauri::AppHandle, tab: &str) {
    log::debug!("show_settings: tab={tab}");
    if let Some(window) = app.get_webview_window("main") {
        // 切换标签页（eval 在窗口隐藏时也能执行）
        let js = format!("window.switchTab && window.switchTab('{tab}')");
        let _ = window.eval(&js);
        // 显示并聚焦窗口
        let _ = window.show();
        let _ = window.set_focus();
        log::debug!("设置窗口已显示，切换到标签: {tab}");
    } else {
        log::warn!("找不到名为 'main' 的窗口");
    }
}

/// 通过 macOS 通知中心发送系统通知
///
/// 使用 tauri-plugin-notification 插件，通知会出现在通知中心。
fn show_notification(app: &tauri::AppHandle, title: &str, body: &str) {
    log::debug!("发送系统通知: title={title}, body={body}");
    use tauri_plugin_notification::NotificationExt;
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        log::warn!("系统通知发送失败: {e}");
    }
}

/// 加载托盘图标
///
/// 使用 include_bytes! 宏在编译时将图标文件嵌入到可执行文件中，
/// 确保在任何运行环境下都能找到图标资源。
fn build_tray_icon() -> Image<'static> {
    // 编译时嵌入 icons/tray-icon.png（路径相对于此源文件所在目录的父目录）
    let bytes = include_bytes!("../icons/tray-icon.png");
    log::debug!("加载托盘图标，大小: {} 字节", bytes.len());
    Image::from_bytes(bytes).expect("托盘图标加载失败，请确保 icons/tray-icon.png 存在")
}
