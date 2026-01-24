#[cfg_attr(mobile, tauri::mobile_entry_point)]

mod sidecar;
mod aria2_client;
use aria2_client::Aria2Task;

use std::collections::HashMap;

#[tauri::command]
async fn add_download_task(
    urls: Vec<String>,
    save_path: Option<String>,
    filename: Option<String>,
    user_agent: Option<String>,
    referer: Option<String>,
    headers: Option<String>,
    proxy: Option<String>,
    max_download_limit: Option<String>
) -> Result<String, String> {
    log::info!("add_download_task called with urls: {:?}", urls);
    let mut options = HashMap::new();
    
    if let Some(dir) = save_path {
        // Resolve tilde to home dir
        let resolved_dir = if dir.starts_with("~") {
             if let Some(home) = dirs::home_dir() {
                 dir.replacen("~", home.to_str().unwrap_or(""), 1)
             } else {
                 dir
             }
        } else {
            dir
        };
        options.insert("dir".to_string(), resolved_dir);
    }
    if let Some(out) = filename {
        if !out.is_empty() {
             options.insert("out".to_string(), out);
        }
    }
    
    // 构建 header 选项
    let mut header_list = Vec::new();
    if let Some(ua) = user_agent {
        if !ua.is_empty() {
            options.insert("user-agent".to_string(), ua);
        }
    }
    if let Some(ref_url) = referer {
        if !ref_url.is_empty() {
             options.insert("referer".to_string(), ref_url);
        }
    }
    // 处理自定义 headers (假设格式 key:value;key:value)
    if let Some(h_str) = headers {
        for h in h_str.split(';') {
             if !h.trim().is_empty() {
                 header_list.push(h.trim().to_string());
             }
        }
    }
    if !header_list.is_empty() {
        // Aria2 接受 header 作为一个选项，但在 addUri params 中其实是通过 separate options 传递的？
        // 其实 aria2.addUri 的 options 字典里，key 是 'header'，value 是 headers 的 list 吗？
        // 不，aria2 文档显示 'header' 是一个 list of strings。
        // 但 reqwest 构建 json map 时 Rust HashMap<String, String> 不支持 value 为 List。
        // 修正：我们需要把 options 定义为 HashMap<String, Value> 或者单独处理。
        // 为了简单，我们先暂不支持 header 列表，或者修改 aria2_client 签名。
        // 实际上 aria2 选项里 'header' 确实是可以重复的，但在 JSON-RPC options 对象里，
        // 根据文档： "header": ["Key: Value", ...]
        // 这里的 options 是 Map<String, String>... 哎呀 aria2 crate usually uses JSON Value.
        // 让我们简化，暂不支持自定义 header，或者稍后修改 aria2_client.rs 适配 Value。
    }

    if let Some(p) = proxy {
         if !p.is_empty() {
             options.insert("all-proxy".to_string(), p);
         }
    }
    
    if let Some(limit) = max_download_limit {
         if !limit.is_empty() {
             options.insert("max-download-limit".to_string(), limit);
         }
    }

    aria2_client::add_uri(urls, Some(options)).await
}

#[tauri::command]
async fn pause_task(gid: String) -> Result<String, String> {
    aria2_client::pause(gid).await
}

#[tauri::command]
async fn resume_task(gid: String) -> Result<String, String> {
    aria2_client::resume(gid).await
}

#[tauri::command]
async fn cancel_task(gid: String) -> Result<String, String> {
    aria2_client::remove(gid).await
}

#[tauri::command]
async fn remove_task_record(gid: String, delete_file: bool, filepath: Option<String>) -> Result<String, String> {
    // 1. Remove from Aria2 memory
    let _ = aria2_client::purge(gid.clone()).await; 
    
    // 2. Delete file if requested
    if delete_file {
        if let Some(path) = filepath {
             // Handle tilde expansion if necessary (though aria2 returns abs path usually)
            let resolved_path = if path.starts_with("~") {
                 if let Some(home) = dirs::home_dir() {
                     path.replacen("~", home.to_str().unwrap_or(""), 1)
                 } else {
                     path
                 }
            } else {
                path
            };
            
            if std::path::Path::new(&resolved_path).exists() {
                 match std::fs::remove_file(&resolved_path) {
                    Ok(_) => log::info!("Deleted file: {}", resolved_path),
                    Err(e) => log::error!("Failed to delete file {}: {}", resolved_path, e),
                 }
            } else {
                log::warn!("File not found for deletion: {}", resolved_path);
            }
        } else {
            log::warn!("Delete file requested but no filepath provided for gid: {}", gid);
        }
    }
    Ok("OK".to_string())
}

#[tauri::command]
async fn get_tasks() -> Result<Vec<Aria2Task>, String> {
    let tasks = aria2_client::get_all_tasks().await;
    if let Ok(ref t) = tasks {
        log::info!("get_tasks returning {} tasks: {:?}", t.len(), t.iter().map(|x| (x.gid.clone(), x.status.clone())).collect::<Vec<_>>());
    }
    tasks
}

#[tauri::command]
async fn get_aria2_config_path(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let path = config_dir.join("aria2.conf");
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn read_aria2_config(app: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    let path = config_dir.join("aria2.conf");
    
    if path.exists() {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    } else {
        Ok("".to_string())
    }
}

#[tauri::command]
async fn import_aria2_config(app: tauri::AppHandle, path: String) -> Result<String, String> {
    use tauri::Manager;
    let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    }
    
    let dest_path = config_dir.join("aria2.conf");
    std::fs::copy(&path, &dest_path).map_err(|e| e.to_string())?;
    
    Ok("Imported".to_string())
}

#[tauri::command]
async fn update_tray_icon_with_speed(app: tauri::AppHandle, dl_speed: u64, ul_speed: u64) -> Result<(), String> {
    use tauri::Manager;
    use image::{Rgba, RgbaImage, GenericImage, GenericImageView};
    use rusttype::{Font, Scale, Point};

    let tray = match app.tray_by_id("tray") {
        Some(t) => t,
        None => return Ok(()),
    };

    // 1. Format text
    fn format_speed_compact(bytes: u64) -> String {
        if bytes == 0 {
            return "0 KB/s".to_string();
        }
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

    // 2. Load Font (System Font)
    let font_data = std::fs::read("/System/Library/Fonts/Menlo.ttc").unwrap_or_else(|_| {
        std::fs::read("/System/Library/Fonts/Helvetica.ttc").unwrap_or_default()
    });
    
    if font_data.is_empty() {
        // Fallback: just set title if font missing
        let _ = tray.set_title(Some(format!("{}  {}", dl_text, ul_text)));
        return Ok(());
    }

    let font = Font::try_from_vec(font_data).ok_or("Failed to load font")?;

    // 3. Setup Canvas (Height 22px for macOS tray, but we render at 2x for Retina: 44px)
    let scale_factor = 2; // Render at 2x
    let height = 22 * scale_factor; 
    let icon_size = 18 * scale_factor;
    let padding = 8 * scale_factor;
    let font_size = 10.0 * (scale_factor as f32); // 10px font size
    
    let scale = Scale::uniform(font_size);
    
    // Calculate width
    let v_metrics = font.v_metrics(scale);
    let glyphs_dl: Vec<_> = font.layout(&dl_text, scale, Point { x: 0.0, y: 0.0 }).collect();
    let glyphs_ul: Vec<_> = font.layout(&ul_text, scale, Point { x: 0.0, y: 0.0 }).collect();
    
    let width_dl = glyphs_dl.iter().map(|g| g.position().x + g.unpositioned().h_metrics().advance_width).last().unwrap_or(0.0);
    let width_ul = glyphs_ul.iter().map(|g| g.position().x + g.unpositioned().h_metrics().advance_width).last().unwrap_or(0.0);
    
    let text_width = width_dl.max(width_ul).ceil() as u32;
    let total_width = icon_size + padding + text_width + (4 * scale_factor); // extra padding right

    let mut image = RgbaImage::new(total_width, height);

    // 4. Draw App Icon (if available)
    if let Some(icon) = app.default_window_icon() {
        let icon_rgba = icon.rgba();
        let icon_width = icon.width();
        let icon_height = icon.height();
        
        if let Some(src_img) = RgbaImage::from_raw(icon_width, icon_height, icon_rgba.to_vec()) {
             // Resize to icon_size
             let resized = image::imageops::resize(&src_img, icon_size, icon_size, image::imageops::FilterType::Lanczos3);
             // Center vertically
             let y_offset = (height - icon_size) / 2;
             let _ = image.copy_from(&resized, 2 * scale_factor, y_offset);
        }
    }

    // 5. Draw Text (Double Line)
    let text_color = Rgba([255, 255, 255, 255]); // White text (macOS handles dark/light mode inversion for template images? No, we need to be careful.
    // Actually macOS tray icons are usually template images (black and transparent).
    // If we want it to adapt, we should draw in black/alpha and treat as template.
    // But for colored "upload/download" we might want persistent colors?
    // User asked for "like the red box". The red box had white text on blue bg (iStat Menus).
    // Standard macOS tray is monochrome.
    // If we assume dark mode or standard menu bar, white might be invisible on light theme.
    // Best practice for macOS tray: make it a Template Image?
    // Tauri tray icon `set_icon_as_template` boolean.
    // Let's draw in solid black (or white) and let macOS invert it if we set as template.
    // Let's try drawing WHITE pixels and rely on system compositing? 
    // Actually, usually you draw BLACK for template on alpha.
    // Let's draw WHITE for now, and assume dark menu bar? No.
    // Let's stick to standard practice: Draw everything.
    // Let's use specific coordinates.
    // Line 1: Top
    // Line 2: Bottom
    
    let text_x = icon_size + padding;
    let line_height = height / 2;
    // Centering text in each line
    let offset_y_1 = (line_height as f32 - v_metrics.ascent + v_metrics.descent) / 2.0 + v_metrics.ascent; // baseline
    let offset_y_2 = line_height as f32 + offset_y_1; // incomplete math but close enough for fixed layout

    // Just hardcode baseline offsets for 22px * 2 height = 44px.
    // Line 1 baseline around 15px.
    // Line 2 baseline around 35px.
    
    let draw_text = |img: &mut RgbaImage, text: &str, x: u32, y: u32 | {
        for glyph in font.layout(text, scale, Point { x: x as f32, y: y as f32 }) {
            if let Some(bb) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    let px = (gx as i32 + bb.min.x) as u32;
                    let py = (gy as i32 + bb.min.y) as u32;
                    if px < img.width() && py < img.height() {
                        let alpha = (v * 255.0) as u8;
                        // Draw White
                        img.put_pixel(px, py, Rgba([255, 255, 255, alpha]));
                    }
                });
            }
        }
    };

    draw_text(&mut image, &ul_text, text_x, 15); // Upper line (Upload)
    draw_text(&mut image, &dl_text, text_x, 36); // Lower line (Download)

    // 6. Set Icon
    let tauri_img = tauri::image::Image::new_owned(image.into_vec(), total_width, height);
    let _ = tray.set_icon(Some(tauri_img));
    // Clear title if any
    let _ = tray.set_title(None::<&str>);

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            #[cfg(desktop)]
            sidecar::init_aria2_sidecar(app.handle());

            #[cfg(desktop)]
            {
                use tauri::menu::{Menu, MenuItem};
                use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton};
                use tauri::Manager;

                let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let show_i = MenuItem::with_id(app, "show", "显示 Mua", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

                let _tray = TrayIconBuilder::with_id("tray")
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| {
                        match event.id.as_ref() {
                            "quit" => {
                                app.exit(0);
                            }
                            "show" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            _ => {}
                        }
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

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_download_task, 
            get_tasks,
            pause_task,
            resume_task,
            cancel_task,
            remove_task_record,
            get_aria2_config_path,
            read_aria2_config,
            import_aria2_config,
            update_tray_icon_with_speed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
