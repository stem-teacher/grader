use anyhow::Result;
use surrealdb::{engine::local::RocksDb, Surreal};
use uuid::Uuid;

use crate::models::{Submission, GradingResults, GradingStatus};

#[derive(Clone)]
pub struct DatabaseService {
    db: Surreal<surrealdb::engine::local::Db>,
}

impl DatabaseService {
    pub async fn new(database_url: &str) -> Result<Self> {
        let db = if database_url == "memory" {
            Surreal::new::<surrealdb::engine::local::Mem>(()).await?
        } else {
            Surreal::new::<RocksDb>(database_url).await?
        };

        db.use_ns("hsc_chemistry").use_db("exams").await?;

        Ok(Self { db })
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        // Define submission table
        self.db
            .query("DEFINE TABLE submissions SCHEMAFULL")
            .await?;

        self.db
            .query("DEFINE FIELD id ON submissions TYPE uuid")
            .await?;

        self.db
            .query("DEFINE FIELD submission_code ON submissions TYPE string")
            .await?;

        self.db
            .query("DEFINE FIELD responses ON submissions TYPE object")
            .await?;

        self.db
            .query("DEFINE FIELD submitted_at ON submissions TYPE datetime")
            .await?;

        self.db
            .query("DEFINE FIELD grading_status ON submissions TYPE string")
            .await?;

        self.db
            .query("DEFINE FIELD results ON submissions TYPE option<object>")
            .await?;

        // Create unique index on submission_code
        self.db
            .query("DEFINE INDEX submission_code_idx ON submissions FIELDS submission_code UNIQUE")
            .await?;

        Ok(())
    }

    pub async fn store_submission(&self, submission: &Submission) -> Result<()> {
        self.db
            .create(("submissions", submission.id.to_string()))
            .content(submission)
            .await?;
        Ok(())
    }

    pub async fn get_submission(&self, submission_code: &str) -> Result<Option<Submission>> {
        let mut result = self.db
            .query("SELECT * FROM submissions WHERE submission_code = $code")
            .bind(("code", submission_code))
            .await?;

        let submissions: Vec<Submission> = result.take(0)?;
        Ok(submissions.into_iter().next())
    }

    pub async fn submission_exists(&self, submission_code: &str) -> Result<bool> {
        let submission = self.get_submission(submission_code).await?;
        Ok(submission.is_some())
    }

    pub async fn update_grading_status(
        &self,
        submission_code: &str,
        status: GradingStatus,
    ) -> Result<()> {
        self.db
            .query("UPDATE submissions SET grading_status = $status WHERE submission_code = $code")
            .bind(("status", serde_json::to_string(&status)?))
            .bind(("code", submission_code))
            .await?;
        Ok(())
    }

    pub async fn store_grading_results(
        &self,
        submission_code: &str,
        results: &GradingResults,
    ) -> Result<()> {
        self.db
            .query("UPDATE submissions SET results = $results WHERE submission_code = $code")
            .bind(("results", results))
            .bind(("code", submission_code))
            .await?;
        Ok(())
    }

    pub async fn health_check(&self) -> Result<()> {
        // Simple health check - try to query the database
        let _: Vec<surrealdb::sql::Value> = self.db
            .query("SELECT 1 as health_check")
            .await?
            .take(0)?;
        Ok(())
    }
}
