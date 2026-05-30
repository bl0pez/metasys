#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod metasys_client;
mod queue_manager;
mod state;

use state::{AppState, Uma, UmaPoint, QueuedCommand};
use std::sync::Arc;
use tokio::sync::Notify;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
async fn get_umas(state: State<'_, Arc<AppState>>) -> Result<Vec<Uma>, String> {
    let umas = state.umas.lock().await;
    Ok(umas.clone())
}

#[tauri::command]
async fn add_uma(name: String, state: State<'_, Arc<AppState>>) -> Result<Uma, String> {
    let mut umas = state.umas.lock().await;
    let new_uma = Uma {
        id: Uuid::new_v4().to_string(),
        name,
        points: Vec::new(),
    };
    umas.push(new_uma.clone());
    state.save_to_disk().await;
    Ok(new_uma)
}

#[tauri::command]
async fn add_point(uma_id: String, name: String, reference: String, control_type: String, state: State<'_, Arc<AppState>>) -> Result<UmaPoint, String> {
    let mut umas = state.umas.lock().await;
    let new_point = UmaPoint {
        id: Uuid::new_v4().to_string(),
        name,
        reference,
        control_type,
    };
    
    if let Some(uma) = umas.iter_mut().find(|u| u.id == uma_id) {
        uma.points.push(new_point.clone());
        state.save_to_disk().await;
        Ok(new_point)
    } else {
        Err("UMA no encontrada".to_string())
    }
}

#[tauri::command]
async fn enqueue_command(uma_name: String, point_name: String, reference: String, action: String, state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let mut queue = state.queue.lock().await;
    let id = Uuid::new_v4().to_string();
    
    let cmd = QueuedCommand {
        id: id.clone(),
        uma_name,
        point_name,
        reference,
        action,
        retries: 0,
        status: "PENDING".to_string(),
    };
    
    queue.push(cmd);
    
    // Notificamos al worker de que hay comandos nuevos
    state.notify.notify_one();
    
    Ok(id)
}

#[tauri::command]
async fn get_queue_status(state: State<'_, Arc<AppState>>) -> Result<Vec<QueuedCommand>, String> {
    let queue = state.queue.lock().await;
    Ok(queue.clone())
}

#[tauri::command]
async fn remove_uma(id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut umas = state.umas.lock().await;
    umas.retain(|u| u.id != id);
    state.save_to_disk().await;
    Ok(())
}

#[tauri::command]
async fn remove_point(uma_id: String, point_id: String, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let mut umas = state.umas.lock().await;
    if let Some(uma) = umas.iter_mut().find(|u| u.id == uma_id) {
        uma.points.retain(|p| p.id != point_id);
    }
    state.save_to_disk().await;
    Ok(())
}

#[tauri::command]
async fn invoke_telemetry(payload: serde_json::Value, state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let token = state.metasys_client.get_valid_token().await.map_err(|e| e.to_string())?;
    let data = state.metasys_client.get_points_details(&token, payload).await.map_err(|e| e.to_string())?;
    Ok(data)
}

fn main() {
    let app_state = Arc::new(AppState::new());
    let worker_state = app_state.clone();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(app_state)
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                queue_manager::start_worker(worker_state).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_umas,
            add_uma,
            add_point,
            enqueue_command,
            get_queue_status,
            remove_uma,
            remove_point,
            invoke_telemetry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
