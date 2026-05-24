use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::metasys_client::MetasysClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmaPoint {
    pub id: String,
    pub name: String,
    pub reference: String,
    pub control_type: String, // "toggle" o "analog"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uma {
    pub id: String,
    pub name: String,
    pub points: Vec<UmaPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueuedCommand {
    pub id: String,
    pub uma_name: String,
    pub point_name: String,
    pub reference: String,
    pub action: String, // "1", "0", "45", etc.
    pub retries: u32,
    pub status: String,
}

pub struct AppState {
    pub umas: Arc<Mutex<Vec<Uma>>>,
    pub queue: Arc<Mutex<Vec<QueuedCommand>>>,
    pub metasys_client: Arc<MetasysClient>,
}

impl AppState {
    pub fn new() -> Self {
        let mut initial_umas = Vec::new();
        
        for i in 1..=9 {
            initial_umas.push(Uma {
                id: Uuid::new_v4().to_string(),
                name: format!("Pabellón {} (UMA 2.{})", i, i),
                points: vec![
                    UmaPoint {
                        id: Uuid::new_v4().to_string(),
                        name: "Habilitación UMA".to_string(),
                        reference: if i >= 8 {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.UNITEN-MODE", i)
                        } else {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.ENABLE UMA", i)
                        },
                        control_type: "toggle".to_string(),
                    },
                    UmaPoint {
                        id: Uuid::new_v4().to_string(),
                        name: "Válvula de Frío".to_string(),
                        reference: if i >= 8 {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.CLG-O", i)
                        } else {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.CLG SL-O", i)
                        },
                        control_type: "analog".to_string(),
                    },
                    UmaPoint {
                        id: Uuid::new_v4().to_string(),
                        name: "Válvula de Calor".to_string(),
                        reference: if i >= 8 {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.HTG-O", i)
                        } else {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.HTG SL-O", i)
                        },
                        control_type: "analog".to_string(),
                    },
                    UmaPoint {
                        id: Uuid::new_v4().to_string(),
                        name: "Comando Ventilador".to_string(),
                        reference: format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.SF-C", i),
                        control_type: "toggle".to_string(),
                    },
                    UmaPoint {
                        id: Uuid::new_v4().to_string(),
                        name: "Setpoint T° (Pabellón)".to_string(),
                        reference: format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.ZNT-SP", i),
                        control_type: "temperature".to_string(),
                    }
                ],
            });
        }

        Self {
            umas: Arc::new(Mutex::new(initial_umas)),
            queue: Arc::new(Mutex::new(Vec::new())),
            metasys_client: Arc::new(MetasysClient::new()),
        }
    }
}
