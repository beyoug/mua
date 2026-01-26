use image::{GenericImage, Rgba, RgbaImage};
use rusttype::{Font, Point, Scale};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};

pub struct TrayState {
    pub font: Option<Font<'static>>,
    pub icon_cache: Option<RgbaImage>,
    pub last_update_key: Option<String>,
}

impl TrayState {
    pub fn new() -> Self {
        Self {
            font: None,
            icon_cache: None,
            last_update_key: None,
        }
    }
}

impl Default for TrayState {
    fn default() -> Self {
        Self::new()
    }
}

fn load_system_font() -> Option<Font<'static>> {
    let paths = [
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/Menlo.ttc",
        "/Library/Fonts/Arial.ttf",
    ];

    for path in paths {
        if let Ok(data) = std::fs::read(path) {
            // 对于 Helvetica.ttc (Collection)，通常 Index 1 是 Bold
            // 我们优先尝试加载 Bold
            if path.contains("Helvetica.ttc") {
                if let Some(font) = Font::try_from_vec_and_index(data.clone(), 1) {
                    return Some(font);
                }
            }

            // 尝试标准加载 (Index 0 or single font)
            if let Some(font) = Font::try_from_vec(data.clone()) {
                return Some(font);
            }

            // 尝试其他索引
            for i in 0..4 {
                if let Some(font) = Font::try_from_vec_and_index(data.clone(), i) {
                    return Some(font);
                }
            }
        }
    }
    None
}

pub fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();

    // 1. 初始化状态
    app.manage(Mutex::new(TrayState::default()));

    // 2. 加载资源到状态
    if let Ok(state) = handle.state::<Mutex<TrayState>>().lock() {
        let mut state = state; // MutexGuard

        // 加载字体
        if let Some(font) = load_system_font() {
            state.font = Some(font);
        }

        // 缓存调整大小后的图标
        if let Some(icon) = app.default_window_icon() {
            let icon_rgba = icon.rgba();
            let icon_width = icon.width();
            let icon_height = icon.height();

            let scale_factor = 2;
            let icon_size = 22 * scale_factor;

            if let Some(src_img) = RgbaImage::from_raw(icon_width, icon_height, icon_rgba.to_vec())
            {
                let resized = image::imageops::resize(
                    &src_img,
                    icon_size,
                    icon_size,
                    image::imageops::FilterType::Lanczos3,
                );
                state.icon_cache = Some(resized);
            }
        }
    }

    // 3. 创建托盘图标
    #[cfg(desktop)]
    {
        use tauri::menu::{Menu, MenuItem};
        use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};

        let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
        let show_i = MenuItem::with_id(app, "show", "显示 Mua", true, None::<&str>)?;
        let pause_all_i = MenuItem::with_id(app, "pause_all", "暂停所有", true, None::<&str>)?;
        let resume_all_i = MenuItem::with_id(app, "resume_all", "恢复所有", true, None::<&str>)?;
        let sep = tauri::menu::PredefinedMenuItem::separator(app)?;

        let menu = Menu::with_items(
            app,
            &[&show_i, &sep, &pause_all_i, &resume_all_i, &sep, &quit_i],
        )?;

        let _ = TrayIconBuilder::with_id("tray")
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "quit" => app.exit(0),
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "pause_all" => {
                    tauri::async_runtime::spawn(async {
                        let _ = crate::aria2_client::pause_all().await;
                    });
                }
                "resume_all" => {
                    tauri::async_runtime::spawn(async {
                        let _ = crate::aria2_client::unpause_all().await;
                    });
                }
                _ => {}
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            })
            .build(app)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn update_tray_icon_with_speed(
    app: AppHandle,
    dl_speed: u64,
    ul_speed: u64,
) -> Result<(), String> {
    let tray = match app.tray_by_id("tray") {
        Some(t) => t,
        None => return Ok(()),
    };

    let state_mutex = app.state::<Mutex<TrayState>>();
    let mut state_guard = state_mutex
        .lock()
        .map_err(|_| "Failed to lock tray state".to_string())?;

    // 格式化速度文本，右对齐，紧凑格式
    // 例: "123KB/s", "5.2MB/s", "2KB/s", "0KB/s"
    fn format_speed_fixed(bytes: u64) -> String {
        if bytes == 0 {
            return "0KB/s".to_string();
        }

        let k = 1024.0;
        let m = k * 1024.0;
        let g = m * 1024.0;

        let val = bytes as f64;
        let (num, unit) = if val >= g {
            (val / g, "GB/s")
        } else if val >= m {
            (val / m, "MB/s")
        } else {
            (val / k, "KB/s")
        };

        // 核心逻辑: 保持3位有效数字，但去除末尾多余的0
        // 不需要空格 (根据用户示例 "123KB/s")
        let s = if num >= 100.0 {
            format!("{:.0}", num) // 123
        } else if num >= 10.0 {
            format!("{:.1}", num) // 12.3
        } else {
            format!("{:.2}", num) // 1.23
        };

        // 去除末尾的 .00 或 .0 (e.g. "2.00" -> "2", "5.20" -> "5.2")
        let clean_s = s.trim_end_matches('0').trim_end_matches('.');

        format!("{}{}", clean_s, unit)
    }

    // 移除前缀，只显示纯数字
    let dl_text = format_speed_fixed(dl_speed);
    let ul_text = format_speed_fixed(ul_speed);

    // 生成去重 Key
    let update_key = format!("{}|{}", dl_text, ul_text);

    // 检查是否需要更新
    if let Some(last_key) = &state_guard.last_update_key {
        if last_key == &update_key {
            return Ok(());
        }
    }

    // 更新缓存
    state_guard.last_update_key = Some(update_key);

    let font = match &state_guard.font {
        Some(f) => f,
        None => {
            let title = format!("{}  {}", dl_text, ul_text);
            let _ = tray.set_title(Some(&title));
            return Ok(());
        }
    };

    // 关键修复: 关闭 Template 模式以允许显示自定义颜色（白色）
    let _ = tray.set_icon_as_template(false);

    // 使用 2x (Retina) 尺寸
    let scale_factor = 2;
    let height = 22 * scale_factor; // 44px
    let icon_size = 22 * scale_factor;
    // 减小间距以减少视觉空白 (8 -> 4 -> 2)
    // 之前 8*2=16px 太大，现在 2*2=4px 比较紧凑
    let padding = 2 * scale_factor;

    // 调整字体大小: 稍微调大一点 11.0 * 2 = 22px (极限双行大小)
    let font_size = 11.0 * (scale_factor as f32);
    let scale = Scale::uniform(font_size);

    // 计算固定宽度 (基于最宽可能字符串 "8.88 GB/s")
    // 这样无论当前网速如何，画布宽度永远恒定，防止图标左右晃动
    // 注意: 虽然 Helvetica 是变宽字体，我们通过右对齐来保证视觉稳定
    let max_text = "8.88 GB/s";
    let glyphs_max: Vec<_> = font
        .layout(max_text, scale, Point { x: 0.0, y: 0.0 })
        .collect();
    let max_text_width_f = glyphs_max
        .iter()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .next_back()
        .unwrap_or(0.0);

    let text_area_width = max_text_width_f.ceil() as u32;
    let total_width = icon_size + padding + text_area_width + (2 * scale_factor);

    let mut image = RgbaImage::new(total_width, height);

    // 1. 绘制左侧图标
    if let Some(cached_icon) = &state_guard.icon_cache {
        let y_offset = (height - icon_size) / 2;
        let _ = image.copy_from(cached_icon, 0, y_offset);
    }

    // 文本区域起始X坐标
    let text_area_start_x = icon_size + padding;

    // 2. 调整基线 (Baseline)
    // 字体变大后(22px)，上一次调整(15/37)导致文字太靠上
    // 理论计算: 22px 行高，CapHeight约15px，居中基线应在 18-19px 左右
    // 本次下移至 18 / 40
    let baseline_1 = 18;
    let baseline_2 = 40;

    // 右对齐绘制函数
    let draw_text_right_aligned = |img: &mut RgbaImage, text: &str, y: u32| {
        // 1. 计算当前文本宽度
        let glyphs: Vec<_> = font.layout(text, scale, Point { x: 0.0, y: 0.0 }).collect();
        let current_width = glyphs
            .iter()
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .next_back()
            .unwrap_or(0.0);

        // 2. 计算右对齐的起始 X 坐标
        // X = 区域起始 + (区域总宽 - 当前文字宽)
        let x_pos = text_area_start_x as f32 + (text_area_width as f32 - current_width);

        for glyph in font.layout(
            text,
            scale,
            Point {
                x: x_pos,
                y: y as f32,
            },
        ) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = (gx as i32 + bb.min.x) as u32;
                    let py = (gy as i32 + bb.min.y) as u32;
                    if px < img.width() && py < img.height() {
                        // 线性增强 Alpha，让文字更黑更实
                        let alpha = (v * 255.0 * 1.2).min(255.0) as u8;
                        if alpha > 10 {
                            // 自定义颜色：白色 (255, 255, 255)
                            img.put_pixel(px, py, Rgba([255, 255, 255, alpha]));
                        }
                    }
                });
            }
        }
    };

    draw_text_right_aligned(&mut image, &ul_text, baseline_1);
    draw_text_right_aligned(&mut image, &dl_text, baseline_2);

    let tauri_img = tauri::image::Image::new_owned(image.into_vec(), total_width, height);

    // 必须清除 Title，否则会挤在一起
    let _ = tray.set_title(None::<&str>);
    let _ = tray.set_icon(Some(tauri_img));

    Ok(())
}
