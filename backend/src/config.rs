use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
    pub openai_api_key: String,
    pub gemini_api_key: String,
    pub storage_path: String,
    pub s3_endpoint: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            server_address: env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "memory".to_string()),
            openai_api_key: env::var("OPENAI_API_KEY")?,
            gemini_api_key: env::var("GEMINI_API_KEY")?,
            storage_path: env::var("STORAGE_PATH").unwrap_or_else(|_| "./storage".to_string()),
            s3_endpoint: env::var("S3_ENDPOINT").ok(),
            s3_bucket: env::var("S3_BUCKET").ok(),
            s3_access_key: env::var("S3_ACCESS_KEY").ok(),
            s3_secret_key: env::var("S3_SECRET_KEY").ok(),
        })
    }
}
