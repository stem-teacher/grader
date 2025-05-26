use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{info, warn};

mod config;
mod error;
mod handlers;
mod models;
mod services;

use config::Config;
use handlers::{submissions, grading, health};
use services::{database::DatabaseService, ai::AIService, storage::StorageService};

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<DatabaseService>,
    pub ai_service: Arc<AIService>,
    pub storage: Arc<StorageService>,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    
    // Initialize services
    let database = Arc::new(DatabaseService::new(&config.database_url).await?);
    let mut ai_service = AIService::new(config.clone())?;
    ai_service.initialize().await?;
    let ai_service = Arc::new(ai_service);
    let storage = Arc::new(StorageService::new(config.clone()).await?);

    // Initialize database schema
    database.initialize_schema().await?;

    let app_state = AppState {
        database,
        ai_service,
        storage,
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health::health_check))
        .route("/api/submissions", post(submissions::submit_exam))
        .route("/api/submissions/:code", get(submissions::get_submission))
        .route("/api/grading/:code", get(grading::get_grading_status))
        .route("/api/results/:code", get(grading::get_results))
        .route("/api/results/:code/pdf", get(grading::download_pdf))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
    info!("Server starting on {}", config.server_address);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
