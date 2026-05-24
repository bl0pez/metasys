use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use log::info;

#[derive(Clone)]
pub struct TokenCache {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}

pub struct MetasysClient {
    client: Client,
    pub login_url: String,
    pub command_url: String,
    pub telemetry_url: String,
    pub token_cache: Arc<RwLock<Option<TokenCache>>>,
}

impl MetasysClient {
    pub fn new() -> Self {
        // Ignoramos certificados inválidos por ser red interna
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Fallo al construir el cliente HTTP");

        Self {
            client,
            // Valores por defecto
            login_url: "https://10.6.18.102/API/private-api/Authentication/LogIn".to_string(),
            command_url: "https://10.6.18.102/API/private-api/Command/SendCommands".to_string(),
            telemetry_url: "https://10.6.18.102/API/private-api/Equipment/GetPointsDetails".to_string(),
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_valid_token(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        // 1. Revisar si hay un token válido en caché
        {
            let cache = self.token_cache.read().await;
            if let Some(token_data) = &*cache {
                // Dar un margen de seguridad de 5 minutos antes de expirar
                if token_data.expires_at > Utc::now() + chrono::Duration::minutes(5) {
                    return Ok(token_data.access_token.clone());
                }
            }
        }

        // 2. Si llegamos aquí, no hay token o expiró. Adquirimos lock de escritura.
        let mut cache = self.token_cache.write().await;
        
        // Doble chequeo por si otro thread lo actualizó mientras esperábamos el lock
        if let Some(token_data) = &*cache {
            if token_data.expires_at > Utc::now() + chrono::Duration::minutes(5) {
                return Ok(token_data.access_token.clone());
            }
        }

        info!("Token no válido o expirado. Iniciando proceso de Login en Metasys...");

        let payload = json!({
            "username": "operador",
            "password": "Hospital2020$",
            "domain": ""
        });

        let res = self.client
            .post(&self.login_url)
            .json(&payload)
            .send()
            .await?;

        res.error_for_status_ref()?;
        
        let data: Value = res.json().await?;
        
        let access_token = data.get("access_token")
            .and_then(|t| t.as_str())
            .ok_or("No se encontró access_token en la respuesta")?
            .to_string();
            
        let expires_str = data.get(".expires")
            .and_then(|e| e.as_str())
            .ok_or("No se encontró la fecha de expiración (.expires) en la respuesta")?;

        // Metasys devuelve la fecha como "2026-05-24T23:27:04+00:00" (RFC 3339)
        let expires_at = DateTime::parse_from_rfc3339(expires_str)?.with_timezone(&Utc);

        // Actualizamos el caché
        *cache = Some(TokenCache {
            access_token: access_token.clone(),
            expires_at,
        });

        info!("Login exitoso. Token en caché válido hasta: {}", expires_at);

        Ok(access_token)
    }

    pub async fn send_command(&self, token: &str, reference: &str, value: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let payload = if value == "RELEASE" {
            json!({
                "Commands": [
                    {
                        "Reference": reference,
                        "EquipmentReference": "",
                        "Type": 4,
                        "Parameters": []
                    }
                ]
            })
        } else {
            json!({
                "Commands": [
                    {
                        "Reference": reference,
                        "EquipmentReference": "",
                        "Type": 1,
                        "Parameters": [value]
                    }
                ]
            })
        };

        let res = self.client
            .post(&self.command_url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await?;

        // Si Metasys devuelve 401 Unauthorized, limpiamos el caché para forzar login en el próximo intento
        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            let mut cache = self.token_cache.write().await;
            *cache = None;
            return Err("Token rechazado por el servidor (401). Caché invalidado.".into());
        }

        res.error_for_status_ref()?;
        let txt = res.text().await?;
        Ok(txt)
    }

    pub async fn get_points_details(&self, token: &str, payload: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        info!("Enviando telemetría con payload: {}", payload.to_string());
        let res = self.client
            .post(&self.telemetry_url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await?;

        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            let mut cache = self.token_cache.write().await;
            *cache = None;
            return Err("Token rechazado por el servidor (401). Caché invalidado.".into());
        }

        res.error_for_status_ref()?;
        let txt = res.text().await?;
        // Comentado para no inundar si es muy largo, pero imprimimos los primeros 500 chars
        let log_txt = if txt.len() > 500 { &txt[..500] } else { &txt };
        info!("Respuesta Telemetría (parcial): {}", log_txt);
        Ok(txt)
    }
}
