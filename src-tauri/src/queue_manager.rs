use crate::state::AppState;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use log::{info, error};

pub async fn start_worker(app_state: Arc<AppState>) {
    let max_retries = 50;

    loop {
        let mut cmd_to_process = None;

        // Scope para el lock de la cola
        {
            let mut queue = app_state.queue.lock().await;
            if let Some(cmd) = queue.iter_mut().find(|c| c.status == "PENDING" || c.status == "RETRYING") {
                cmd.status = "PROCESSING".to_string();
                cmd_to_process = Some(cmd.clone());
            }
        }

        if let Some(cmd) = cmd_to_process {
            let reference = cmd.reference.clone();
            let action = cmd.action.clone();

            info!("Procesando comando para {}: {} [{}]", cmd.uma_name, cmd.point_name, action);
            
            let mut success = false;
            match app_state.metasys_client.get_valid_token().await {
                Ok(token) => {
                    match app_state.metasys_client.send_command(&token, &reference, &action).await {
                        Ok(res) => {
                            info!("Comando exitoso: {}", res);
                            success = true;
                        }
                        Err(e) => {
                            error!("Error enviando comando: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error obteniendo token: {}", e);
                }
            }

            let mut retries = cmd.retries;
            
            // Volver a tomar el lock para actualizar el estado final
            {
                let mut queue = app_state.queue.lock().await;
                if let Some(target) = queue.iter_mut().find(|c| c.id == cmd.id) {
                    if success {
                        target.status = "COMPLETED".to_string();
                    } else {
                        target.retries += 1;
                        retries = target.retries;
                        if target.retries >= max_retries {
                            target.status = "FAILED".to_string();
                        } else {
                            target.status = "RETRYING".to_string();
                        }
                    }
                }
            }

            // Backoff si falló
            if !success {
                let backoff_secs = std::cmp::min(5 * (1 << std::cmp::min(retries, 5)), 60);
                info!("Esperando {}s antes del siguiente reintento...", backoff_secs);
                sleep(Duration::from_secs(backoff_secs)).await;
            }
        } else {
            // Dormir si no hay nada en la cola
            sleep(Duration::from_secs(2)).await;
        }
    }
}
