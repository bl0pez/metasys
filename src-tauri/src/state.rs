use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fs;
use std::path::Path;
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::metasys_client::MetasysClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetasysConfig {
    pub host: String,
    pub user: String,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub metasys: MetasysConfig,
    pub umas: Vec<Uma>,
}

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
    pub notify: Arc<tokio::sync::Notify>,
}

impl AppState {
    pub fn new() -> Self {
        let config_path = "metasys_config.json";
        
        let config = if Path::new(config_path).exists() {
            let data = fs::read_to_string(config_path).expect("No se pudo leer el config.json");
            serde_json::from_str::<AppConfig>(&data).unwrap_or_else(|_| Self::default_config())
        } else {
            let cfg = Self::default_config();
            fs::write(config_path, serde_json::to_string_pretty(&cfg).unwrap()).ok();
            cfg
        };

        Self {
            umas: Arc::new(Mutex::new(config.umas)),
            queue: Arc::new(Mutex::new(Vec::new())),
            metasys_client: Arc::new(MetasysClient::new(&config.metasys.host, &config.metasys.user, &config.metasys.pass)),
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    fn default_config() -> AppConfig {
        let mut initial_umas = Vec::new();
        
        for i in 1..=9 {
            initial_umas.push(Uma {
                id: Uuid::new_v4().to_string(),
                name: format!("Pabellón {} (UMA 2.{})", i, i),
                points: vec![
                    UmaPoint {
                        id: Uuid::new_v4().to_string(),
                        name: "Habilitación UMA".to_string(),
                        reference: if i >= 7 {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.UNITEN-MODE", i)
                        } else {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.ENABLE UMA", i)
                        },
                        control_type: "toggle".to_string(),
                    },
                    UmaPoint {
                        id: Uuid::new_v4().to_string(),
                        name: "Válvula de Frío".to_string(),
                        reference: if i >= 7 {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.CLG-O", i)
                        } else {
                            format!("HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-{}.CLG SL-O", i)
                        },
                        control_type: "analog".to_string(),
                    },
                    UmaPoint {
                        id: Uuid::new_v4().to_string(),
                        name: "Válvula de Calor".to_string(),
                        reference: if i >= 7 {
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

        // Recuperación Parto (UMA 2.20) - Similar a UMA 2.1
        initial_umas.push(Uma {
            id: Uuid::new_v4().to_string(),
            name: "Recuperación Parto (UMA 2.20)".to_string(),
            points: vec![
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Habilitación UMA".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-20.ENABLE UMA".to_string(),
                    control_type: "toggle".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Válvula de Frío".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-20.CLG SL-O".to_string(),
                    control_type: "analog".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Válvula de Calor".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-20.HTG SL-O".to_string(),
                    control_type: "analog".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Comando Ventilador".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-20.SF-C".to_string(),
                    control_type: "toggle".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Setpoint T° (Pabellón)".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-2-20.ZNT-SP".to_string(),
                    control_type: "temperature".to_string(),
                }
            ],
        });

        // Farmacia Cerrada (UMA 1.91) - Similar a UMA 2.9
        initial_umas.push(Uma {
            id: Uuid::new_v4().to_string(),
            name: "Farmacia Cerrada (UMA 1.91)".to_string(),
            points: vec![
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Habilitación UMA".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-1-91.UNITEN-MODE".to_string(),
                    control_type: "toggle".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Válvula de Frío".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-1-91.CLG-O".to_string(),
                    control_type: "analog".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Válvula de Calor".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-1-91.HTG-O".to_string(),
                    control_type: "analog".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Comando Ventilador".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-1-91.SF-C".to_string(),
                    control_type: "toggle".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Setpoint T° (Pabellón)".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/Bacnet MSTP.UMA-1-91.ZNT-SP".to_string(),
                    control_type: "temperature".to_string(),
                }
            ],
        });

        // Grupo de Luces Edificio CAE (Piso 1)
        initial_umas.push(Uma {
            id: Uuid::new_v4().to_string(),
            name: "Luces Edificio CAE (Piso 1)".to_string(),
            points: vec![
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Pasillo Sur 50% (Hall Espera)".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/N2 Trunk.LUCES P-1B Y STATUS.Binary Outputs.BO-2".to_string(),
                    control_type: "toggle".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Pasillo Norte 50% (Hall Espera)".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/N2 Trunk.LUCES P-1B Y STATUS.Binary Outputs.BO-3".to_string(),
                    control_type: "toggle".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Pasillo Circulación Interior Recepción CAE".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/N2 Trunk.LUCES P-1B Y STATUS.Binary Outputs.BO-4".to_string(),
                    control_type: "toggle".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Hall Espera CAE Oriente".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/N2 Trunk.LUCES P-1B Y STATUS.Binary Outputs.BO-5".to_string(),
                    control_type: "toggle".to_string(),
                },
                UmaPoint {
                    id: Uuid::new_v4().to_string(),
                    name: "Hall Acceso CAE".to_string(),
                    reference: "HDA-Metasys10:NAE-HArica/N2 Trunk.LUCES P-1B Y STATUS.Binary Outputs.BO-6".to_string(),
                    control_type: "toggle".to_string(),
                }
            ],
        });

        AppConfig {
            metasys: MetasysConfig {
                host: "10.6.18.102".to_string(),
                user: "operador".to_string(),
                pass: "Hospital2020$".to_string(),
            },
            umas: initial_umas,
        }
    }

    pub async fn save_to_disk(&self) {
        let umas = self.umas.lock().await.clone();
        // Preservamos las credenciales al guardar, las volvemos a sacar del cliente
        let config = AppConfig {
            metasys: MetasysConfig {
                host: self.metasys_client.login_url.split("/API").next().unwrap_or("https://10.6.18.102").replace("https://", ""),
                user: self.metasys_client.auth_user.clone(),
                pass: self.metasys_client.auth_pass.clone(),
            },
            umas,
        };
        fs::write("metasys_config.json", serde_json::to_string_pretty(&config).unwrap()).ok();
    }
}
