#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod spotify_control;
mod tarkov_log_watcher;
use crate::tarkov_log_watcher::TarkovLogWatcher;
use log::{debug, error, info, LevelFilter};
use std::time::Duration;
use std::{env, thread};
use tauri::{Manager, WindowBuilder, WindowUrl};
use tauri::{SystemTray, SystemTrayEvent, SystemTrayMenu};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::LogTarget;

#[tauri::command]
fn spotify_play() {
    let mut spotify = spotify_control::SpotifyControls::new();
    spotify.get_spotify_hwnd();
    spotify.play();
}
#[tauri::command]
async fn spotify_pause() {
    let mut spotify = spotify_control::SpotifyControls::new();
    spotify.get_spotify_hwnd();
    spotify.pause();
}
#[tauri::command]
fn create_new_window(app: tauri::AppHandle, label: String, title: String) -> Option<tauri::Window> {
    info!("window creating");
    let new_window = WindowBuilder::new(
        &app,
        label,                               // ウィンドウのラベル (ユニークな識別子)
        WindowUrl::App("index.html".into()), // ロードするページ
    )
    .title(title) // ウィンドウのタイトル
    .visible(true)
    .resizable(true) // サイズ変更可能
    .minimizable(false)
    .maximizable(false)
    .inner_size(450.0, 300.0) // ウィンドウサイズ
    .build();

    match new_window {
        Ok(window) => {
            info!("window created");
            Some(window)
        }
        Err(e) => {
            error!("failed create window: {:?}", e);
            None
        }
    }
}
fn main() {
    let settings = tauri::CustomMenuItem::new("settings".to_string(), "EFTify  Settings");
    let quit = tauri::CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new().add_item(settings).add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);
    let mut ctx = tauri::generate_context!();
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let builder = tauri::Builder::default();

    builder
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_theme::init(ctx.config_mut()))
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "settings" => {
                    if create_new_window(
                        app.clone(),
                        "settings".to_string(),
                        "EFTify Settings".to_string(),
                    )
                    .is_none()
                    {
                        let window = app.get_window("settings").unwrap();

                        window.set_focus().unwrap();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .setup(|_| {
            tauri::async_runtime::spawn_blocking(move || {
                let mut tarkov_log_watcher = TarkovLogWatcher::new();
                loop {
                    let _ = tarkov_log_watcher.watch_logs();
                    thread::sleep(Duration::from_secs(3));
                }
            });
            // ..
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![spotify_play, spotify_pause])
        .build(ctx)
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
