use axum::{extract::{State, Path}, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

use crate::{
    models::{Submission, ExamResponses, GradingStatus, SectionScore, GradingResults, ANSWER_KEY},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct SubmitExamRequest {
    pub submission_code: String,
    pub responses: ExamResponses,
}

#[derive(Debug, Serialize)]
pub struct SubmitExamResponse {
    pub success: bool,
    pub submission_id: Uuid,
    pub message: String,
}

pub async fn submit_exam(
    State(state): State<AppState>,
    Json(request): Json<SubmitExamRequest>,
) -> Result<Json<SubmitExamResponse>, StatusCode> {
    // Validate submission code format
    if !is_valid_submission_code(&request.submission_code) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if submission already exists
    if state.database.submission_exists(&request.submission_code).await.unwrap_or(false) {
        return Err(StatusCode::CONFLICT);
    }

    let submission = Submission {
        id: Uuid::new_v4(),
        submission_code: request.submission_code,
        responses: request.responses,
        submitted_at: Utc::now(),
        grading_status: GradingStatus::Pending,
        results: None,
    };

    // Store submission
    match state.database.store_submission(&submission).await {
        Ok(_) => {
            // Trigger async grading
            let grading_state = state.clone();
            let submission_code = submission.submission_code.clone();
            tokio::spawn(async move {
                if let Err(e) = process_grading(grading_state, submission_code).await {
                    tracing::error!("Grading failed: {}", e);
                }
            });

            Ok(Json(SubmitExamResponse {
                success: true,
                submission_id: submission.id,
                message: "Submission received and queued for grading".to_string(),
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_submission(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<Submission>, StatusCode> {
    match state.database.get_submission(&code).await {
        Ok(Some(submission)) => Ok(Json(submission)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn process_grading(state: AppState, submission_code: String) -> anyhow::Result<()> {
    // Update status to in progress
    state.database.update_grading_status(&submission_code, GradingStatus::InProgress).await?;

    // Get submission
    let submission = state.database.get_submission(&submission_code).await?
        .ok_or_else(|| anyhow::anyhow!("Submission not found"))?;

    // Grade multiple choice automatically
    let mc_score = grade_multiple_choice(&submission.responses.multiple_choice);
    
    // Grade extended responses with AI
    let ai_results = state.ai_service.grade_extended_responses(&submission.responses.extended_response).await?;

    // Combine results
    let total_score = mc_score.score + ai_results.iter().map(|r| r.score).sum::<f64>();
    let max_score = mc_score.max_score + ai_results.iter().map(|r| r.max_score).sum::<f64>();

    let grading_results = GradingResults {
        total_score,
        max_score,
        section_scores: create_section_scores(&mc_score, &ai_results),
        question_feedback: create_question_feedback(&mc_score, &ai_results),
        overall_feedback: generate_overall_feedback(total_score, max_score),
        ai_provider_used: ai_results.first().map(|r| "AI Service".to_string()).unwrap_or_default(),
        graded_at: Utc::now(),
    };

    // Store results
    state.database.store_grading_results(&submission_code, &grading_results).await?;
    state.database.update_grading_status(&submission_code, GradingStatus::Completed).await?;

    Ok(())
}

fn is_valid_submission_code(code: &str) -> bool {
    code.len() >= 10 && code.contains("-")
}

fn grade_multiple_choice(responses: &HashMap<String, String>) -> SectionScore {
    let mut correct = 0;
    let total = ANSWER_KEY.len();

    for (question, correct_answer) in ANSWER_KEY {
        if let Some(student_answer) = responses.get(*question) {
            if student_answer == correct_answer {
                correct += 1;
            }
        }
    }

    SectionScore {
        score: correct as f64,
        max_score: total as f64,
        feedback: format!("Multiple choice: {}/{} correct", correct, total),
    }
}

fn create_section_scores(mc_score: &SectionScore, ai_results: &[crate::models::QuestionFeedback]) -> HashMap<String, SectionScore> {
    let mut sections = HashMap::new();
    
    sections.insert("Section I - Multiple Choice".to_string(), mc_score.clone());
    
    let extended_score = ai_results.iter().map(|r| r.score).sum::<f64>();
    let extended_max = ai_results.iter().map(|r| r.max_score).sum::<f64>();
    
    sections.insert("Section II - Extended Response".to_string(), SectionScore {
        score: extended_score,
        max_score: extended_max,
        feedback: format!("Extended response: {:.1}/{:.1}", extended_score, extended_max),
    });
    
    sections
}

fn create_question_feedback(mc_score: &SectionScore, ai_results: &[crate::models::QuestionFeedback]) -> HashMap<String, crate::models::QuestionFeedback> {
    let mut feedback = HashMap::new();
    
    // Add MC feedback as a single entry
    feedback.insert("Multiple Choice".to_string(), crate::models::QuestionFeedback {
        score: mc_score.score,
        max_score: mc_score.max_score,
        feedback: mc_score.feedback.clone(),
        strengths: vec![],
        improvements: vec![],
        band_estimate: None,
    });
    
    // Add individual AI feedback
    for (i, result) in ai_results.iter().enumerate() {
        feedback.insert(format!("Question {}", i + 21), result.clone());
    }
    
    feedback
}

fn generate_overall_feedback(total_score: f64, max_score: f64) -> String {
    let percentage = (total_score / max_score) * 100.0;
    
    match percentage {
        p if p >= 90.0 => "Excellent performance demonstrating comprehensive understanding of chemistry concepts.".to_string(),
        p if p >= 80.0 => "Strong performance with good understanding of most concepts.".to_string(),
        p if p >= 70.0 => "Good performance with solid understanding of key concepts.".to_string(),
        p if p >= 60.0 => "Satisfactory performance with basic understanding demonstrated.".to_string(),
        _ => "Performance indicates need for additional study and practice.".to_string(),
    }
}
