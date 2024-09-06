mod spotify_control;
mod tarkov_log_watcher;
use crate::tarkov_log_watcher::TarkovLogWatcher;
use std::thread;
use std::time::Duration;
use tauri::Manager;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tauri_plugin_autostart::MacosLauncher;
use tokio;
use tokio::runtime::Builder;
use tokio::time::sleep;
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

async fn start_loop() {
    let mut tarkov_log_watcher = TarkovLogWatcher::new();
    loop {
        println!("Watching logs");
        tarkov_log_watcher.watch_logs();
        thread::sleep(Duration::from_secs(5));
    }
}
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

fn main() {
    let quit = tauri::CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new().add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                if id.as_str() == "quit" {
                    std::process::exit(0);
                }
            }
            _ => {}
        })
        .setup(|app| {
            let app_handle = app.app_handle();
            tauri::async_runtime::spawn_blocking(move || {
                let mut tarkov_log_watcher = TarkovLogWatcher::new();
                loop {
                    println!("Watching logs");
                    tarkov_log_watcher.watch_logs();
                    thread::sleep(Duration::from_secs(5));
                }
            });
            // ..
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![spotify_play, spotify_pause])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
