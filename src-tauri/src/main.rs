#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::VecDeque;
use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, ClipboardManager};
use tokio::sync::Mutex;
use warp::Filter;
use std::net::TcpListener;
use notify::{Watcher, RecursiveMode, Event};
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;
use warp::ws::{Message, WebSocket};
use std::env;

const MAX_RECENT_DIRS: usize = 5;
const DEFAULT_PORT: u16 = 8000;
const MAX_PORT_TRIES: u16 = 100;
const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "ServeLite";

// Server state
#[derive(Default)]
pub struct ServerState {
    server_handle: Option<tokio::task::JoinHandle<()>>,
    root_dir: Option<PathBuf>,
    recent_dirs: VecDeque<PathBuf>,
    current_port: u16,
    watcher: Option<notify::RecommendedWatcher>,
    reload_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            server_handle: None,
            root_dir: None,
            recent_dirs: VecDeque::with_capacity(MAX_RECENT_DIRS),
            current_port: DEFAULT_PORT,
            watcher: None,
            reload_tx: None,
        }
    }

    fn add_recent_dir(&mut self, path: PathBuf) {
        if let Some(pos) = self.recent_dirs.iter().position(|x| x == &path) {
            self.recent_dirs.remove(pos);
        }
        if self.recent_dirs.len() >= MAX_RECENT_DIRS {
            self.recent_dirs.pop_back();
        }
        self.recent_dirs.push_front(path);
    }
}

fn find_available_port(start_port: u16) -> Option<u16> {
    (start_port..start_port + MAX_PORT_TRIES)
        .find(|port| TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok())
}

async fn setup_live_reload(state: &mut ServerState, path: PathBuf) -> Result<(), String> {
    // Create a channel for reload notifications
    let (tx, _) = tokio::sync::broadcast::channel(100);
    state.reload_tx = Some(tx.clone());

    // Create file watcher
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res {
            if matches!(event.kind, notify::EventKind::Modify(_)) {
                let _ = tx.send(());
            }
        }
    }).map_err(|e| e.to_string())?;

    // Watch the directory
    watcher.watch(&path, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    state.watcher = Some(watcher);
    Ok(())
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
    state.add_recent_dir(path.clone());

    // Set up live reload
    setup_live_reload(&mut state, path.clone()).await?;

    // Create reload channel for this server instance
    let reload_tx = state.reload_tx.as_ref().unwrap().clone();

    // Create file serving route with CORS and live reload
    let files = warp::fs::dir(path);
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);

    // Create WebSocket route for live reload
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || reload_tx.clone()))
        .map(|ws: warp::ws::Ws, reload_tx| {
            ws.on_upgrade(move |socket| handle_ws_client(socket, reload_tx))
        });

    let routes = files.or(ws_route).with(cors);

    // Find available port
    let port = find_available_port(DEFAULT_PORT).ok_or("No available port found")?;
    state.current_port = port;

    // Create server
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async {
        tokio::signal::ctrl_c().await.ok();
    });

    // Start server in background
    let handle = tokio::spawn(server);
    state.server_handle = Some(handle);

    Ok(format!("Server started at http://localhost:{}", port))
}

async fn handle_ws_client(ws: WebSocket, reload_tx: tokio::sync::broadcast::Sender<()>) {
    let (mut tx, mut rx) = ws.split();
    let mut reload_rx = reload_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(()) = reload_rx.recv().await {
            if tx.send(Message::text("reload")).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = rx.next().await {}
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
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

fn create_tray_menu(state: &ServerState) -> SystemTrayMenu {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let start = CustomMenuItem::new("start".to_string(), "Start Server");
    let stop = CustomMenuItem::new("stop".to_string(), "Stop Server");
    let copy_url = CustomMenuItem::new("copy_url".to_string(), "Copy URL");
    
    let mut menu = SystemTrayMenu::new()
        .add_item(start)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(stop)
        .add_item(copy_url.clone())
        .add_native_item(SystemTrayMenuItem::Separator);

    // Add recent directories submenu
    if !state.recent_dirs.is_empty() {
        let mut recent_menu = SystemTrayMenu::new();
        for (idx, path) in state.recent_dirs.iter().enumerate() {
            let path_str = path.to_string_lossy().into_owned();
            recent_menu = recent_menu.add_item(CustomMenuItem::new(
                format!("recent_{}", idx),
                path_str
            ));
        }
        menu = menu.add_submenu(tauri::SystemTraySubmenu::new("Recent", recent_menu));
        menu = menu.add_native_item(SystemTrayMenuItem::Separator);
    }

    menu.add_item(quit)
}

fn handle_system_tray_event(app_handle: &AppHandle, event: SystemTrayEvent, state: Arc<Mutex<ServerState>>) {
    match event {
        SystemTrayEvent::LeftClick { .. } | SystemTrayEvent::RightClick { .. } => {
            let tray_handle = app_handle.tray_handle();
            let has_server = {
                let state = state.blocking_lock();
                state.server_handle.is_some()
            };
            tray_handle.get_item("start").set_enabled(!has_server).unwrap();
            tray_handle.get_item("stop").set_enabled(has_server).unwrap();
            tray_handle.get_item("copy_url").set_enabled(has_server).unwrap();
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            handle_menu_item(app_handle, &id, state);
        }
        _ => {}
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
                        match start_server_internal(state.clone(), path).await {
                            Ok(msg) => {
                                let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                                    .title("Success")
                                    .body(&msg)
                                    .show();
                                // Update tray menu with recent directories
                                let state = state.lock().await;
                                app_handle.tray_handle().set_menu(create_tray_menu(&state)).unwrap();
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
        "copy_url" => {
            let state = state.clone();
            tauri::async_runtime::block_on(async {
                let state = state.lock().await;
                if let Some(port) = state.server_handle.is_some().then(|| state.current_port) {
                    let url = format!("http://localhost:{}", port);
                    app_handle.clipboard_manager().write_text(url).unwrap();
                    let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                        .title("Success")
                        .body("URL copied to clipboard")
                        .show();
                }
            });
        }
        id if id.starts_with("recent_") => {
            if let Ok(idx) = id.strip_prefix("recent_").unwrap().parse::<usize>() {
                let app_handle = app_handle.clone();
                let state = state.clone();
                tauri::async_runtime::spawn(async move {
                    let path = {
                        let state = state.lock().await;
                        state.recent_dirs.get(idx).cloned()
                    };
                    if let Some(path) = path {
                        match start_server_internal(state.clone(), path).await {
                            Ok(msg) => {
                                let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                                    .title("Success")
                                    .body(&msg)
                                    .show();
                                // Update tray menu with recent directories
                                let state = state.lock().await;
                                app_handle.tray_handle().set_menu(create_tray_menu(&state)).unwrap();
                            }
                            Err(e) => {
                                let _ = tauri::api::notification::Notification::new(&app_handle.config().tauri.bundle.identifier)
                                    .title("Error")
                                    .body(&e)
                                    .show();
                            }
                        }
                    }
                });
            }
        }
        _ => {}
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--version" {
        println!("{} v{}", APP_NAME, VERSION);
        return;
    }

    let state = Arc::new(Mutex::new(ServerState::new()));
    let state_clone = Arc::clone(&state);
    let state_setup = Arc::clone(&state);

    tauri::Builder::default()
        .setup(move |app| {
            let tray_menu = create_tray_menu(&state_setup.blocking_lock());
            let tray = SystemTray::new().with_menu(tray_menu);
            app.manage(tray);
            Ok(())
        })
        .system_tray(SystemTray::new())
        .on_system_tray_event(move |app, event| {
            handle_system_tray_event(app, event, Arc::clone(&state_clone));
        })
        .manage(state)
        .run(tauri::generate_context!())
        .expect("error while running application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_server_lifecycle() {
        // Create server state
        let state = Arc::new(Mutex::new(ServerState::new()));
        
        // Create a temporary directory for testing
        let temp_dir = std::env::temp_dir().join("servelite_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        // Start server
        let result = start_server_internal(state.clone(), temp_dir.clone()).await;
        assert!(result.is_ok(), "Failed to start server: {:?}", result);
        
        // Verify server is running
        {
            let state = state.lock().await;
            assert!(state.server_handle.is_some(), "Server handle should exist");
            assert!(state.root_dir.is_some(), "Root directory should be set");
            assert_eq!(state.recent_dirs.len(), 1, "Should have one recent directory");
        }
        
        // Wait a bit for server to fully start
        sleep(Duration::from_secs(1)).await;
        
        // Stop server
        let result = stop_server_internal(state.clone()).await;
        assert!(result.is_ok(), "Failed to stop server: {:?}", result);
        
        // Verify server is stopped
        {
            let state = state.lock().await;
            assert!(state.server_handle.is_none(), "Server handle should be None");
        }
        
        // Clean up
        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_port_selection() {
        // Test with available port
        let port = find_available_port(DEFAULT_PORT);
        assert!(port.is_some(), "Should find an available port");
        assert!(port.unwrap() >= DEFAULT_PORT, "Port should be >= DEFAULT_PORT");
        
        // Create a listener to occupy the default port
        let _listener = TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT)).unwrap();
        
        // Test with occupied default port
        let port = find_available_port(DEFAULT_PORT);
        assert!(port.is_some(), "Should find an alternative port");
        assert!(port.unwrap() > DEFAULT_PORT, "Port should be > DEFAULT_PORT");
    }
}
