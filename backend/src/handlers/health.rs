use axum::{extract::State, Json};
use serde::{Serialize, Deserialize};
use chrono::Utc;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub services: ServiceHealth,
}

#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub database: String,
    pub ai_services: String,
    pub storage: String,
}

pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let database_status = match state.database.health_check().await {
        Ok(_) => "healthy".to_string(),
        Err(e) => format!("unhealthy: {}", e),
    };

    let ai_status = "healthy".to_string(); // Could add actual AI service ping
    let storage_status = "healthy".to_string(); // Could add storage check

    Json(HealthResponse {
        status: if database_status == "healthy" { "healthy".to_string() } else { "degraded".to_string() },
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services: ServiceHealth {
            database: database_status,
            ai_services: ai_status,
            storage: storage_status,
        },
    })
}
