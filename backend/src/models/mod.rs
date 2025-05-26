use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub id: Uuid,
    pub submission_code: String,
    pub responses: ExamResponses,
    pub submitted_at: DateTime<Utc>,
    pub grading_status: GradingStatus,
    pub results: Option<GradingResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamResponses {
    pub multiple_choice: HashMap<String, String>,
    pub extended_response: HashMap<String, serde_json::Value>,
    pub time_taken_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GradingStatus {
    Pending,
    InProgress,
    Completed,
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingResults {
    pub total_score: f64,
    pub max_score: f64,
    pub section_scores: HashMap<String, SectionScore>,
    pub question_feedback: HashMap<String, QuestionFeedback>,
    pub overall_feedback: String,
    pub ai_provider_used: String,
    pub graded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionScore {
    pub score: f64,
    pub max_score: f64,
    pub feedback: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionFeedback {
    pub score: f64,
    pub max_score: f64,
    pub feedback: String,
    pub strengths: Vec<String>,
    pub improvements: Vec<String>,
    pub band_estimate: Option<String>,
}

// Multiple choice answer key from the HSC paper
pub const ANSWER_KEY: &[(&str, &str)] = &[
    ("q1", "D"), ("q2", "C"), ("q3", "B"), ("q4", "D"), ("q5", "D"),
    ("q6", "C"), ("q7", "A"), ("q8", "C"), ("q9", "C"), ("q10", "A"),
    ("q11", "C"), ("q12", "B"), ("q13", "A"), ("q14", "D"), ("q15", "B"),
    ("q16", "B"), ("q17", "A"), ("q18", "A"), ("q19", "B"), ("q20", "D"),
];
