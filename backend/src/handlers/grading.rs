use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Serialize, Deserialize};

use crate::{
    models::{GradingStatus, GradingResults},
    services::pdf::PDFService,
    AppState,
};

#[derive(Debug, Serialize)]
pub struct GradingStatusResponse {
    pub status: GradingStatus,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ResultsResponse {
    pub results: Option<GradingResults>,
    pub status: GradingStatus,
}

pub async fn get_grading_status(
    State(state): State<AppState>,
    Path(submission_code): Path<String>,
) -> Result<Json<GradingStatusResponse>, StatusCode> {
    let submission = state
        .database
        .get_submission(&submission_code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let message = match &submission.grading_status {
        GradingStatus::Pending => "Grading is queued and will begin shortly".to_string(),
        GradingStatus::InProgress => "Your submission is currently being graded".to_string(),
        GradingStatus::Completed => "Grading completed successfully".to_string(),
        GradingStatus::Failed { error } => format!("Grading failed: {}", error),
    };

    Ok(Json(GradingStatusResponse {
        status: submission.grading_status,
        message,
    }))
}

pub async fn get_results(
    State(state): State<AppState>,
    Path(submission_code): Path<String>,
) -> Result<Json<ResultsResponse>, StatusCode> {
    let submission = state
        .database
        .get_submission(&submission_code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ResultsResponse {
        results: submission.results,
        status: submission.grading_status,
    }))
}

pub async fn download_pdf(
    State(state): State<AppState>,
    Path(submission_code): Path<String>,
) -> Result<Response, StatusCode> {
    let submission = state
        .database
        .get_submission(&submission_code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if submission.results.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let pdf_service = PDFService::new();
    let pdf_bytes = pdf_service
        .generate_results_pdf(&submission)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let filename = format!("HSC_Chemistry_Results_{}.pdf", submission_code);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        pdf_bytes,
    )
        .into_response())
}
