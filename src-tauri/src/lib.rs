// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use sysinfo::Networks;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, Position, Size, WindowEvent, image::Image, Wry};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use serde::Serialize;
use lazy_static::lazy_static;
use tauri_plugin_store::{StoreExt, Builder as StoreBuilder};

lazy_static! {
    static ref SELECTED_INTERFACES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

#[derive(Clone, Serialize)]
struct NetworkSpeed {
    interface: String,
    rx_formatted: String,
    tx_formatted: String,
    rx_bytes: u64,
    tx_bytes: u64,
}

fn format_speed(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    
    let value = bytes as f64;
    if value >= GB {
        format!("{:.2} GB/s", value / GB)
    } else if value >= MB {
        format!("{:.2} MB/s", value / MB)
    } else if value >= KB {
        format!("{:.2} KB/s", value / KB)
    } else {
        format!("{} B/s", bytes)
    }
}

fn generate_tray_icon(is_up: bool) -> Vec<u8> {
    const W: u32 = 32;
    const H: u32 = 32;
    let mut rgba = vec![0u8; (W * H * 4) as usize];
    // Colors based on App.css: Green #39ff8f [57, 255, 143], Blue #38bdf8 [56, 189, 248]
    let color = if is_up { [56, 189, 248, 255] } else { [57, 255, 143, 255] };

    for y in 0..H {
        for x in 0..W {
            let i = ((y * W + x) * 4) as usize;
            let mut draw = false;

            // Draw a simple 16px wide arrow centered in 32px canvas
            let mx = x as i32 - 16;
            let my = if is_up { y as i32 - 8 } else { 24 - y as i32 }; // Flip Y for down

            if (mx.abs() <= 2 && my >= 0 && my <= 14) || // Shaft
               (my >= 0 && my <= 8 && mx.abs() <= 8 - my) { // Arrowhead
                draw = true;
            }

            if draw { rgba[i] = color[0]; rgba[i+1] = color[1]; rgba[i+2] = color[2]; rgba[i+3] = color[3]; }
        }
    }
    rgba
}

fn generate_idle_icon() -> Vec<u8> {
    const W: u32 = 32;
    const H: u32 = 32;
    let mut rgba = vec![0u8; (W * H * 4) as usize];
    
    for y in 0..H {
        for x in 0..W {
            let i = ((y * W + x) * 4) as usize;
            // Draw a small semi-transparent grey dot for idle
            let dx = x as i32 - 16;
            let dy = y as i32 - 16;
            if dx * dx + dy * dy <= 36 { // 6px radius
                rgba[i] = 150; rgba[i+1] = 150; rgba[i+2] = 150; rgba[i+3] = 200;
            }
        }
    }
    rgba
}

fn open_settings_window(app_handle: &AppHandle<Wry>) {
    if let Some(settings_window) = app_handle.get_webview_window("settings") {
        let _ = settings_window.set_focus();
        return;
    }

    let _settings_window = tauri::WebviewWindowBuilder::new(
        app_handle,
        "settings", // Unique label for the settings window
        tauri::WebviewUrl::App("settings.html".into()) // Path to new settings HTML
    )
    .title("BitFlow Settings")
    .inner_size(400.0, 500.0)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .build()
    .expect("Failed to create settings window");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_all_network_interfaces() -> Vec<String> {
    let networks = sysinfo::Networks::new_with_refreshed_list();
    networks.iter().map(|(name, _)| name.clone()).collect()
}

const STORE_FILE_NAME: &str = ".settings.dat";
const SELECTED_INTERFACES_KEY: &str = "selected_interfaces";

#[tauri::command]
async fn save_selected_interfaces(app: AppHandle, selected: Vec<String>) -> Result<(), String> {
    let store = app.store(STORE_FILE_NAME.to_string()).map_err(|e| e.to_string())?;
    store.set(SELECTED_INTERFACES_KEY.to_string(), serde_json::to_value(&selected).unwrap());
    store.save().map_err(|e: tauri_plugin_store::Error| e.to_string())?;

    // Update global static
    let mut global_selected = SELECTED_INTERFACES.lock().unwrap();
    *global_selected = selected;

    Ok(())
}

#[tauri::command]
async fn load_selected_interfaces(app: AppHandle) -> Result<Vec<String>, String> {
    let store = app.store(STORE_FILE_NAME.to_string()).map_err(|e| e.to_string())?;
    
    if let Some(value) = store.get(SELECTED_INTERFACES_KEY.to_string()) {
        let loaded: Vec<String> = serde_json::from_value(value.clone()).map_err(|e: serde_json::Error| e.to_string())?;
        // Update global static on load
        let mut global_selected = SELECTED_INTERFACES.lock().unwrap();
        *global_selected = loaded.clone();
        Ok(loaded)
    } else {
        Ok(Vec::new()) // Return empty if nothing saved
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            
            // Set initial window size to be small like a widget
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(Size::Logical(LogicalSize { width: 300.0, height: 450.0 }));
                let _ = window.set_minimizable(false);
                let _ = window.set_maximizable(false);

                // Position window at bottom right on startup
                if let Ok(Some(monitor)) = window.current_monitor() {
                    let screen_size = monitor.size();
                    if let Ok(window_size) = window.outer_size() {
                        let x = screen_size.width as i32 - window_size.width as i32 - 20;
                        let y = screen_size.height as i32 - window_size.height as i32 - 40;
                        let _ = window.set_position(Position::Physical(PhysicalPosition { x, y }));
                    }
                }

                let w_clone = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let _ = w_clone.hide();
                        api.prevent_close();
                    }
                });
            }

            // --- System Tray Setup ---
            let toggle_i = MenuItem::with_id(app, "toggle", "Hide", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?; // NEW
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_i, &settings_i, &quit_i])?; // Add settings_i to menu

            let toggle_i_clone = toggle_i.clone();

            let tray = TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("BitFlow")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app: &AppHandle, event| {
                    match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let new_title = if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                    "Show"
                                } else {
                                    // Position window at bottom right
                                    if let Ok(Some(monitor)) = window.current_monitor() {
                                        let screen_size = monitor.size();
                                        if let Ok(window_size) = window.outer_size() {
                                            let x = screen_size.width as i32 - window_size.width as i32 - 20;
                                            let y = screen_size.height as i32 - window_size.height as i32 - 40;
                                            let _ = window.set_position(Position::Physical(PhysicalPosition { x, y }));
                                        }
                                    }
                                    
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    "Hide"
                                };
                                let _ = toggle_i_clone.set_text(new_title);
                            }
                        },
                        "settings" => open_settings_window(app), // NEW
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |tray: &TrayIcon, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let new_title = if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                                "Show"
                            } else {
                                // Position window at bottom right
                                if let Ok(Some(monitor)) = window.current_monitor() {
                                    let screen_size = monitor.size();
                                    if let Ok(window_size) = window.outer_size() {
                                        let x = screen_size.width as i32 - window_size.width as i32 - 20;
                                        let y = screen_size.height as i32 - window_size.height as i32 - 40;
                                        let _ = window.set_position(Position::Physical(PhysicalPosition { x, y }));
                                    }
                                }

                                let _ = window.show();
                                let _ = window.set_focus();
                                "Hide"
                            };
                            let _ = toggle_i.set_text(new_title);
                        }
                    }
                })
                .build(app)?;

            let tray_handle = tray.clone();
            let icon_up_data = generate_tray_icon(true);
            let icon_down_data = generate_tray_icon(false);
            let icon_idle_data = generate_idle_icon();

            // Initial load of settings on startup
            {
                if let Ok(store) = handle.store(STORE_FILE_NAME.to_string()).map_err(|e| e.to_string()) {
                    if let Some(value) = store.get(SELECTED_INTERFACES_KEY.to_string()) {
                        if let Ok(loaded) = serde_json::from_value::<Vec<String>>(value.clone()) {
                            let mut global_selected = SELECTED_INTERFACES.lock().unwrap();
                            *global_selected = loaded;
                        }
                    }
                }
            }

            // --- Background Thread for Network Monitoring ---
            thread::spawn(move || {
                let mut networks = Networks::new_with_refreshed_list();
                let mut step = 0;

                loop {
                    thread::sleep(Duration::from_secs(1));
                    networks.refresh();
                    
                    let mut payloads = Vec::new();
                    let mut total_rx = 0;
                    let mut total_tx = 0;

                    let global_selected_interfaces_lock = SELECTED_INTERFACES.lock().unwrap();
                    let selected_interfaces_is_empty = global_selected_interfaces_lock.is_empty();

                    for (interface_name, data) in &networks {
                        let name = interface_name.to_lowercase();

                        // Check if this interface is selected, or if no interfaces are specifically selected (monitor all)
                        let is_selected = selected_interfaces_is_empty || global_selected_interfaces_lock.contains(interface_name);

                        // Filter logic:
                        // 1. Must look like a physical interface (Ethernet, WiFi, wlan, eth, en)
                        // 2. Must NOT look like a virtual interface (vEthernet, VirtualBox, docker, etc.)
                        // 3. Must be active (receiving or transmitting data)
                        let is_physical = name.contains("ethernet") || name.contains("wifi") || name.contains("wi-fi") || name.starts_with("en") || name.starts_with("eth") || name.starts_with("wlan");
                        let is_virtual = name.contains("virtual") || name.contains("vethernet") || name.contains("wsl") || name.contains("docker") || name.contains("loopback");

                        if is_selected && is_physical && !is_virtual { // Add is_selected to the condition
                            total_rx += data.received();
                            total_tx += data.transmitted();

                            payloads.push(NetworkSpeed {
                                interface: interface_name.clone(),
                                rx_formatted: format_speed(data.received()),
                                tx_formatted: format_speed(data.transmitted()),
                                rx_bytes: data.received(),
                                tx_bytes: data.transmitted(),
                            });
                        }
                    }

                    // Always emit data, even if empty or zero speed, to update the UI
                    let _ = handle.emit("network-speed", &payloads);

                    // Update Tray Tooltip
                    let tooltip = if total_rx > 0 || total_tx > 0 {
                        format!("▼ {} | ▲ {}", format_speed(total_rx), format_speed(total_tx))
                    } else {
                        "BitFlow".to_string()
                    };
                    let _ = tray_handle.set_tooltip(Some(&tooltip));

                    // Update Tray Icon Animation
                    let icon_data = if total_rx > 0 && total_tx > 0 {
                        // Both active: Toggle every second
                        if step % 2 == 0 { &icon_down_data } else { &icon_up_data }
                    } else if total_rx > 0 {
                        &icon_down_data
                    } else if total_tx > 0 {
                        &icon_up_data
                    } else {
                        &icon_idle_data
                    };
                    
                    // Construct Image from the static byte vectors available in this thread
                    let icon = Image::new(icon_data, 32, 32);
                    let _ = tray_handle.set_icon(Some(icon));
                    step += 1;
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(StoreBuilder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_all_network_interfaces,
            save_selected_interfaces,
            load_selected_interfaces
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
