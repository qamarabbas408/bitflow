// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use sysinfo::Networks;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, Position, Size, WindowEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use serde::Serialize;

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

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_i, &quit_i])?;

            let toggle_i_clone = toggle_i.clone();

            let _tray = TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().unwrap().clone())
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
                        }
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

            // --- Background Thread for Network Monitoring ---
            thread::spawn(move || {
                let mut networks = Networks::new_with_refreshed_list();
                loop {
                    thread::sleep(Duration::from_secs(1));
                    networks.refresh();
                    
                    let mut payloads = Vec::new();
                    for (interface_name, data) in &networks {
                        let name = interface_name.to_lowercase();
                        // Filter logic:
                        // 1. Must look like a physical interface (Ethernet, WiFi, wlan, eth, en)
                        // 2. Must NOT look like a virtual interface (vEthernet, VirtualBox, docker, etc.)
                        // 3. Must be active (receiving or transmitting data)
                        let is_physical = name.contains("ethernet") || name.contains("wifi") || name.contains("wi-fi") || name.starts_with("en") || name.starts_with("eth") || name.starts_with("wlan");
                        let is_virtual = name.contains("virtual") || name.contains("vethernet") || name.contains("wsl") || name.contains("docker") || name.contains("loopback");
                        let is_active = data.received() > 0 || data.transmitted() > 0;

                        if is_physical && !is_virtual && is_active {
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
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
