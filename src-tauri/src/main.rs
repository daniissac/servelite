#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tokio::sync::Mutex;
use warp::Filter;

// Server state
#[derive(Default)]
pub struct ServerState {
    server_handle: Option<tokio::task::JoinHandle<()>>,
    root_dir: Option<PathBuf>,
}

async fn start_server_internal(state: Arc<Mutex<ServerState>>, path: PathBuf) -> Result<String, String> {
    let mut state = state.lock().await;
    
    // Stop existing server if running
    if let Some(handle) = state.server_handle.take() {
        handle.abort();
    }

    // Update root directory
    if !path.exists() {
        return Err("Directory does not exist".to_string());
    }
    state.root_dir = Some(path.clone());

    // Create file serving route with CORS
    let files = warp::fs::dir(path)
        .with(warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_headers(vec!["Content-Type"]));

    // Create server
    let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
    let (_, server) = warp::serve(files).bind_with_graceful_shutdown(addr, async {
        tokio::signal::ctrl_c().await.ok();
    });

    // Start server in background
    let handle = tokio::spawn(server);
    state.server_handle = Some(handle);

    Ok("Server started at http://localhost:8000".to_string())
}

async fn stop_server_internal(state: Arc<Mutex<ServerState>>) -> Result<(), String> {
    let mut state = state.lock().await;
    if let Some(handle) = state.server_handle.take() {
        handle.abort();
        Ok(())
    } else {
        Err("Server not running".to_string())
    }
}

fn handle_menu_item(app_handle: &AppHandle, id: &str, state: Arc<Mutex<ServerState>>) {
    match id {
        "quit" => {
            // Stop server before quitting
            let state = state.clone();
            tauri::async_runtime::block_on(async {
                let _ = stop_server_internal(state).await;
            });
            app_handle.exit(0);
        }
        "start" => {
            // Open directory selection dialog and start server
            let app_handle = app_handle.clone();
            let state = state.clone();
            tauri::api::dialog::FileDialogBuilder::new().pick_folder(move |path_buf| {
                if let Some(path) = path_buf {
                    let app_handle = app_handle.clone();
                    let state = state.clone();
                    tauri::async_runtime::spawn(async move {
                        match start_server_internal(state, path).await {
                            Ok(msg) => {
                                let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                                    .title("Success")
                                    .body(&msg)
                                    .show();
                            }
                            Err(e) => {
                                let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                                    .title("Error")
                                    .body(&e)
                                    .show();
                            }
                        }
                    });
                }
            });
        }
        "stop" => {
            // Stop server
            let app_handle = app_handle.clone();
            let state = state.clone();
            tauri::async_runtime::spawn(async move {
                match stop_server_internal(state).await {
                    Ok(_) => {
                        let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                            .title("Success")
                            .body("Server stopped")
                            .show();
                    }
                    Err(e) => {
                        let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                            .title("Error")
                            .body(&e)
                            .show();
                    }
                }
            });
        }
        _ => {}
    }
}

fn main() {
    // Create system tray menu
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let start = CustomMenuItem::new("start".to_string(), "Start Server");
    let stop = CustomMenuItem::new("stop".to_string(), "Stop Server");
    let tray_menu = SystemTrayMenu::new()
        .add_item(start)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(stop)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new()
        .with_menu(tray_menu)
        .with_tooltip("ServeLight");

    let server_state = Arc::new(Mutex::new(ServerState::default()));
    let server_state_clone = server_state.clone();

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(move |app_handle, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let tray_handle = app_handle.tray_handle();
                tray_handle.get_item("start").set_enabled(true).unwrap();
                tray_handle.get_item("stop").set_enabled(true).unwrap();
            }
            SystemTrayEvent::RightClick { .. } => {
                let tray_handle = app_handle.tray_handle();
                tray_handle.get_item("start").set_enabled(true).unwrap();
                tray_handle.get_item("stop").set_enabled(true).unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                handle_menu_item(app_handle, &id, server_state_clone.clone());
            }
            _ => {}
        })
        .manage(server_state)
        .setup(|app| {
            // Hide the main window
            if let Some(window) = app.get_window("main") {
                window.hide().unwrap();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
