use anyhow::Result;
use std::{path::Path, sync::Arc};
use tokio::fs;
use uuid::Uuid;

use crate::config::Config;

#[derive(Clone)]
pub struct StorageService {
    config: Arc<Config>,
}

impl StorageService {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        // Ensure local storage directory exists
        fs::create_dir_all(&config.storage_path).await?;
        
        Ok(Self { config })
    }

    pub async fn store_file(&self, content: &[u8], filename: &str) -> Result<String> {
        let file_id = Uuid::new_v4().to_string();
        let file_path = Path::new(&self.config.storage_path).join(&file_id);
        
        // Store locally
        fs::write(&file_path, content).await?;
        
        // Replicate to S3 if configured
        if let (Some(endpoint), Some(bucket)) = (&self.config.s3_endpoint, &self.config.s3_bucket) {
            if let Err(e) = self.replicate_to_s3(&file_id, content).await {
                tracing::warn!("Failed to replicate {} to S3: {}", file_id, e);
            }
        }
        
        Ok(file_id)
    }

    pub async fn get_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let file_path = Path::new(&self.config.storage_path).join(file_id);
        
        match fs::read(&file_path).await {
            Ok(content) => Ok(content),
            Err(_) => {
                // Try to retrieve from S3 if local file not found
                if self.config.s3_endpoint.is_some() {
                    self.retrieve_from_s3(file_id).await
                } else {
                    Err(anyhow::anyhow!("File not found: {}", file_id))
                }
            }
        }
    }

    async fn replicate_to_s3(&self, file_id: &str, content: &[u8]) -> Result<()> {
        // Implement S3 upload using reqwest
        let client = reqwest::Client::new();
        
        let endpoint = self.config.s3_endpoint.as_ref().unwrap();
        let bucket = self.config.s3_bucket.as_ref().unwrap();
        let url = format!("{}/{}/{}", endpoint, bucket, file_id);
        
        let response = client
            .put(&url)
            .header("Content-Type", "application/octet-stream")
            .body(content.to_vec())
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("S3 upload failed with status: {}", response.status()));
        }
        
        Ok(())
    }

    async fn retrieve_from_s3(&self, file_id: &str) -> Result<Vec<u8>> {
        let client = reqwest::Client::new();
        
        let endpoint = self.config.s3_endpoint.as_ref().unwrap();
        let bucket = self.config.s3_bucket.as_ref().unwrap();
        let url = format!("{}/{}/{}", endpoint, bucket, file_id);
        
        let response = client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("S3 retrieval failed with status: {}", response.status()));
        }
        
        let content = response.bytes().await?;
        
        // Cache locally for future requests
        let file_path = Path::new(&self.config.storage_path).join(file_id);
        if let Err(e) = fs::write(&file_path, &content).await {
            tracing::warn!("Failed to cache file locally: {}", e);
        }
        
        Ok(content.to_vec())
    }
}
