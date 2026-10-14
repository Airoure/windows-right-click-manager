#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod capture;
mod elevation;
mod journal;
mod model;
mod registry;
mod service;

use std::sync::{Arc, Mutex};
use tauri::Manager;
type Shared = Arc<Mutex<service::Service>>;

#[tauri::command]
async fn capture_menu(window: tauri::WebviewWindow) -> Result<capture::Capture, String> {
    window.minimize().map_err(|e| e.to_string())?;
    let result = tauri::async_runtime::spawn_blocking(capture::run_worker)
        .await
        .map_err(|e| e.to_string());
    let _ = window.unminimize();
    let _ = window.set_focus();
    result?
}

#[tauri::command]
async fn load_data(state: tauri::State<'_, Shared>) -> Result<service::AppData, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .lock()
            .map_err(|_| "操作状态异常，请重启应用。".to_owned())
            .map(|s| s.data())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn toggle_menu(
    state: tauri::State<'_, Shared>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .lock()
            .map_err(|_| "操作状态异常，请重启应用。".to_owned())?
            .toggle(&id, enabled)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn add_menu(state: tauri::State<'_, Shared>, menu: model::NewMenu) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .lock()
            .map_err(|_| "操作状态异常，请重启应用。".to_owned())?
            .create(menu)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn restore_operation(state: tauri::State<'_, Shared>, id: String) -> Result<(), String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        state
            .lock()
            .map_err(|_| "操作状态异常，请重启应用。".to_owned())?
            .restore(&id)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--capture-menu") {
        println!("{}", serde_json::to_string(&capture::observe()).unwrap());
        return;
    }
    if args.get(1).map(String::as_str) == Some("--elevated-helper") {
        if let Some(token) = args.get(2) {
            let _ = elevation::helper(token);
        }
        return;
    }
    if args.get(1).map(String::as_str) == Some("--scan") {
        println!("{}", serde_json::to_string(&registry::scan()).unwrap());
        return;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let path = app.path().app_local_data_dir()?;
            let journal = journal::Journal::new(&path).map_err(std::io::Error::other)?;
            app.manage(Arc::new(Mutex::new(service::Service { journal })));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_data,
            capture_menu,
            toggle_menu,
            add_menu,
            restore_operation
        ])
        .run(tauri::generate_context!())
        .expect("无法启动右键管理应用");
}
