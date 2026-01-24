use tauri::{AppHandle, Manager, Runtime};
use image::{Rgba, RgbaImage, GenericImage};
use rusttype::{Font, Scale, Point};
use std::sync::Mutex;

pub struct TrayState {
    pub font: Option<Font<'static>>,
    pub icon_cache: Option<RgbaImage>,
}

impl TrayState {
    pub fn new() -> Self {
        Self {
            font: None,
            icon_cache: None,
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
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial.ttf"
    ];

    for path in paths {
        if let Ok(data) = std::fs::read(path) {
            // 尝试标准加载
            if let Some(font) = Font::try_from_vec(data.clone()) {
                return Some(font);
            } 
            // 尝试集合索引（系统字体常见情况）
            // 如果是集合，只尝试几个索引
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
            
            if let Some(src_img) = RgbaImage::from_raw(icon_width, icon_height, icon_rgba.to_vec()) {
                 let resized = image::imageops::resize(&src_img, icon_size, icon_size, image::imageops::FilterType::Lanczos3);
                 state.icon_cache = Some(resized);
            }
        }
    }

    // 3. 创建托盘图标
    #[cfg(desktop)]
    {
        use tauri::menu::{Menu, MenuItem};
        use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton};

        let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
        let show_i = MenuItem::with_id(app, "show", "显示 Mua", true, None::<&str>)?;
        let pause_all_i = MenuItem::with_id(app, "pause_all", "暂停所有", true, None::<&str>)?;
        let resume_all_i = MenuItem::with_id(app, "resume_all", "恢复所有", true, None::<&str>)?;
        let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
        
        let menu = Menu::with_items(app, &[&show_i, &sep, &pause_all_i, &resume_all_i, &sep, &quit_i])?;

        let _ = TrayIconBuilder::with_id("tray")
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| {
                match event.id.as_ref() {
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
                }
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
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
pub async fn update_tray_icon_with_speed(app: AppHandle, dl_speed: u64, ul_speed: u64) -> Result<(), String> {
    let tray = match app.tray_by_id("tray") {
        Some(t) => t,
        None => return Ok(()),
    };

    let state_mutex = app.state::<Mutex<TrayState>>();
    let state_guard = state_mutex.lock().map_err(|_| "Failed to lock tray state".to_string())?;

    fn format_speed_compact(bytes: u64) -> String {
        if bytes == 0 { return "0 KB/s".to_string(); }
        let k = 1024.0;
        let m = k * 1024.0;
        if (bytes as f64) >= m {
            format!("{:.1} MB/s", (bytes as f64) / m)
        } else {
            format!("{:.0} KB/s", (bytes as f64) / k)
        }
    }
    
    let dl_text = format!("↓ {}", format_speed_compact(dl_speed));
    let ul_text = format!("↑ {}", format_speed_compact(ul_speed));

    let font = match &state_guard.font {
        Some(f) => f,
        None => {
            let _ = tray.set_title(Some(format!("{}  {}", dl_text, ul_text)));
            return Ok(());
        }
    };

    let scale_factor = 2; 
    let height = 22 * scale_factor; 
    let icon_size = 22 * scale_factor;
    let padding = 8 * scale_factor;
    let font_size = 10.0 * (scale_factor as f32); 
    
    let scale = Scale::uniform(font_size);
    
    let _v_metrics = font.v_metrics(scale);
    let glyphs_dl: Vec<_> = font.layout(&dl_text, scale, Point { x: 0.0, y: 0.0 }).collect();
    let glyphs_ul: Vec<_> = font.layout(&ul_text, scale, Point { x: 0.0, y: 0.0 }).collect();
    
    let width_dl = glyphs_dl.iter().map(|g| g.position().x + g.unpositioned().h_metrics().advance_width).next_back().unwrap_or(0.0);
    let width_ul = glyphs_ul.iter().map(|g| g.position().x + g.unpositioned().h_metrics().advance_width).next_back().unwrap_or(0.0);
    
    let text_width = width_dl.max(width_ul).ceil() as u32;
    let total_width = icon_size + padding + text_width + (4 * scale_factor); 

    let mut image = RgbaImage::new(total_width, height);

    if let Some(cached_icon) = &state_guard.icon_cache {
        let y_offset = (height - icon_size) / 2;
        let _ = image.copy_from(cached_icon, 2 * scale_factor, y_offset);
    }
    
    let text_x = icon_size + padding;
    let baseline_1 = 15;
    let baseline_2 = 36;
    
    let draw_text = |img: &mut RgbaImage, text: &str, x: u32, y: u32 | {
        for glyph in font.layout(text, scale, Point { x: x as f32, y: y as f32 }) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = (gx as i32 + bb.min.x) as u32;
                    let py = (gy as i32 + bb.min.y) as u32;
                    if px < img.width() && py < img.height() {
                        let alpha = (v * 255.0) as u8;
                        img.put_pixel(px, py, Rgba([255, 255, 255, alpha]));
                    }
                });
            }
        }
    };

    draw_text(&mut image, &ul_text, text_x, baseline_1); 
    draw_text(&mut image, &dl_text, text_x, baseline_2); 

    let tauri_img = tauri::image::Image::new_owned(image.into_vec(), total_width, height);
    let _ = tray.set_icon(Some(tauri_img));
    let _ = tray.set_title(None::<&str>);

    Ok(())
}
