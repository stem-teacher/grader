use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{GradingResults, Submission};

pub struct PDFService;

impl PDFService {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_results_pdf(&self, submission: &Submission) -> Result<Vec<u8>> {
        let results = submission.results.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No grading results available"))?;

        let html_content = self.generate_html_report(submission, results)?;
        
        // Use wkhtmltopdf to convert HTML to PDF
        let pdf_bytes = self.html_to_pdf(&html_content).await?;
        
        Ok(pdf_bytes)
    }

    fn generate_html_report(&self, submission: &Submission, results: &GradingResults) -> Result<String> {
        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>HSC Chemistry Examination Results</title>
    <style>
        body {{
            font-family: 'Times New Roman', serif;
            margin: 40px;
            line-height: 1.6;
        }}
        .header {{
            text-align: center;
            border-bottom: 2px solid #333;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }}
        .score-summary {{
            background-color: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 30px;
        }}
        .question-feedback {{
            margin-bottom: 25px;
            padding: 15px;
            border-left: 4px solid #007bff;
            background-color: #f8f9fa;
        }}
        .score {{
            font-weight: bold;
            color: #28a745;
        }}
        .improvements {{
            color: #dc3545;
        }}
        .strengths {{
            color: #28a745;
        }}
        .footer {{
            margin-top: 40px;
            text-align: center;
            font-size: 12px;
            color: #666;
        }}
        @media print {{
            body {{ margin: 20px; }}
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>NSW HSC Chemistry Examination Results</h1>
        <p><strong>Submission Code:</strong> {}</p>
        <p><strong>Submitted:</strong> {}</p>
        <p><strong>Graded:</strong> {}</p>
    </div>

    <div class="score-summary">
        <h2>Overall Performance</h2>
        <p class="score">Total Score: {:.1}/{:.1} ({:.1}%)</p>
        <p><strong>Time Taken:</strong> {:.1} minutes</p>
        <p><strong>AI Grading Provider:</strong> {}</p>
    </div>

    <div class="overall-feedback">
        <h2>Overall Feedback</h2>
        <p>{}</p>
    </div>

    <div class="section-scores">
        <h2>Section Performance</h2>
        {}
    </div>

    <div class="question-by-question">
        <h2>Question-by-Question Feedback</h2>
        {}
    </div>

    <div class="footer">
        <p>This report was generated automatically using AI-powered grading technology.</p>
        <p>Generated on: {}</p>
    </div>
</body>
</html>
        "#,
        submission.submission_code,
        submission.submitted_at.format("%Y-%m-%d %H:%M:%S UTC"),
        results.graded_at.format("%Y-%m-%d %H:%M:%S UTC"),
        results.total_score,
        results.max_score,
        (results.total_score / results.max_score * 100.0),
        submission.responses.time_taken_minutes,
        results.ai_provider_used,
        results.overall_feedback,
        self.generate_section_scores_html(&results.section_scores),
        self.generate_question_feedback_html(&results.question_feedback),
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        Ok(html)
    }

    fn generate_section_scores_html(&self, section_scores: &HashMap<String, crate::models::SectionScore>) -> String {
        section_scores
            .iter()
            .map(|(section, score)| {
                format!(
                    r#"
                    <div class="section-score">
                        <h3>{}</h3>
                        <p class="score">Score: {:.1}/{:.1}</p>
                        <p>{}</p>
                    </div>
                    "#,
                    section, score.score, score.max_score, score.feedback
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_question_feedback_html(&self, question_feedback: &HashMap<String, crate::models::QuestionFeedback>) -> String {
        question_feedback
            .iter()
            .map(|(question, feedback)| {
                let strengths_html = if !feedback.strengths.is_empty() {
                    format!(
                        "<div class='strengths'><strong>Strengths:</strong><ul>{}</ul></div>",
                        feedback.strengths.iter().map(|s| format!("<li>{}</li>", s)).collect::<Vec<_>>().join("")
                    )
                } else {
                    String::new()
                };

                let improvements_html = if !feedback.improvements.is_empty() {
                    format!(
                        "<div class='improvements'><strong>Areas for Improvement:</strong><ul>{}</ul></div>",
                        feedback.improvements.iter().map(|i| format!("<li>{}</li>", i)).collect::<Vec<_>>().join("")
                    )
                } else {
                    String::new()
                };

                let band_html = if let Some(band) = &feedback.band_estimate {
                    format!("<p><strong>Band Estimate:</strong> {}</p>", band)
                } else {
                    String::new()
                };

                format!(
                    r#"
                    <div class="question-feedback">
                        <h3>Question {}</h3>
                        <p class="score">Score: {:.1}/{:.1}</p>
                        {}
                        <div class="feedback-content">
                            <p>{}</p>
                            {}
                            {}
                        </div>
                    </div>
                    "#,
                    question,
                    feedback.score,
                    feedback.max_score,
                    band_html,
                    feedback.feedback,
                    strengths_html,
                    improvements_html
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    async fn html_to_pdf(&self, html: &str) -> Result<Vec<u8>> {
        use std::process::Command;
        use tokio::fs;

        // Create temporary file for HTML
        let temp_html = format!("/tmp/{}.html", Uuid::new_v4());
        let temp_pdf = format!("/tmp/{}.pdf", Uuid::new_v4());

        fs::write(&temp_html, html).await?;

        // Use wkhtmltopdf to convert HTML to PDF
        let output = Command::new("wkhtmltopdf")
            .arg("--page-size")
            .arg("A4")
            .arg("--margin-top")
            .arg("0.75in")
            .arg("--margin-right")
            .arg("0.75in")
            .arg("--margin-bottom")
            .arg("0.75in")
            .arg("--margin-left")
            .arg("0.75in")
            .arg("--enable-local-file-access")
            .arg(&temp_html)
            .arg(&temp_pdf)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("PDF conversion failed: {}", error));
        }

        let pdf_bytes = fs::read(&temp_pdf).await?;

        // Clean up temporary files
        let _ = fs::remove_file(&temp_html).await;
        let _ = fs::remove_file(&temp_pdf).await;

        Ok(pdf_bytes)
    }
}
