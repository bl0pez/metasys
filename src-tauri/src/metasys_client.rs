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
    pub schedule_url: String,
    pub token_cache: Arc<RwLock<Option<TokenCache>>>,
    pub auth_user: String,
    pub auth_pass: String,
}

impl MetasysClient {
    pub fn new(host: &str, user: &str, pass: &str) -> Self {
        // Ignoramos certificados inválidos por ser red interna
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Fallo al construir el cliente HTTP");

        Self {
            client,
            login_url: format!("https://{}/API/private-api/Authentication/LogIn", host),
            command_url: format!("https://{}/API/private-api/Command/SendCommands", host),
            telemetry_url: format!("https://{}/API/private-api/Equipment/GetPointsDetails", host),
            schedule_url: format!("https://{}/API/private-api/Schedule/EnableDisableSchedule", host),
            token_cache: Arc::new(RwLock::new(None)),
            auth_user: user.to_string(),
            auth_pass: pass.to_string(),
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
            "username": self.auth_user,
            "password": self.auth_pass,
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
        let clean_ref = reference.trim();
        let clean_val = value.trim();

        let payload = if clean_val == "RELEASE" {
            json!({
                "Commands": [
                    {
                        "Reference": clean_ref,
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
                        "Reference": clean_ref,
                        "EquipmentReference": "",
                        "Type": 1,
                        "Parameters": [clean_val]
                    }
                ]
            })
        };

        log::info!("Payload a enviar a Metasys: {}", serde_json::to_string(&payload).unwrap_or_default());

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

        if !res.status().is_success() {
            let status = res.status();
            let error_body = res.text().await.unwrap_or_else(|_| "No se pudo leer el body del error".to_string());
            return Err(format!("Metasys rechazó el comando ({}). Body: {}", status, error_body).into());
        }

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

    pub async fn set_schedule(&self, token: &str, payload: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        info!("Enviando payload de Schedule: {}", payload.to_string());
        
        let res = self.client
            .post(&self.schedule_url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await?;

        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            let mut cache = self.token_cache.write().await;
            *cache = None;
            return Err("Token rechazado por el servidor (401). Caché invalidado.".into());
        }

        if !res.status().is_success() {
            let status = res.status();
            let error_body = res.text().await.unwrap_or_else(|_| "No se pudo leer el body del error".to_string());
            return Err(format!("Metasys rechazó el comando de schedule ({}). Body: {}", status, error_body).into());
        }

        let txt = res.text().await?;
        Ok(txt)
    }
}
