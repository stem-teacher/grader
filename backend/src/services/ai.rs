use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tokio::fs;

use crate::{config::Config, models::QuestionFeedback};

#[derive(Clone)]
pub struct AIService {
    client: Client,
    config: Arc<Config>,
    marking_guidelines: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_completion_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

impl AIService {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let client = Client::new();
        Ok(Self {
            client,
            config,
            marking_guidelines: String::new(), // Will be loaded in initialize
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.marking_guidelines = fs::read_to_string("marking-guidelines/review-prompt.md").await?;
        Ok(())
    }

    pub async fn grade_extended_responses(
        &self,
        responses: &HashMap<String, Value>,
    ) -> Result<Vec<QuestionFeedback>> {
        let mut results = Vec::new();

        for (question_id, response) in responses {
            let feedback = self.grade_single_response(question_id, response).await?;
            results.push(feedback);
        }

        Ok(results)
    }

    async fn grade_single_response(
        &self,
        question_id: &str,
        response: &Value,
    ) -> Result<QuestionFeedback> {
        // Try OpenAI first, fallback to Gemini if it fails
        match self.grade_with_openai(question_id, response).await {
            Ok(feedback) => Ok(feedback),
            Err(e) => {
                tracing::warn!("OpenAI grading failed for {}: {}. Trying Gemini.", question_id, e);
                self.grade_with_gemini(question_id, response).await
            }
        }
    }

    async fn grade_with_openai(
        &self,
        question_id: &str,
        response: &Value,
    ) -> Result<QuestionFeedback> {
        let prompt = self.create_grading_prompt(question_id, response);

        let request = OpenAIRequest {
            model: "o1-mini".to_string(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: self.marking_guidelines.clone(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            max_completion_tokens: 2000,
            temperature: 0.1,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.config.openai_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let openai_response: OpenAIResponse = response.json().await?;
        let content = &openai_response.choices[0].message.content;

        self.parse_ai_feedback(content, "OpenAI o1-mini")
    }

    async fn grade_with_gemini(
        &self,
        question_id: &str,
        response: &Value,
    ) -> Result<QuestionFeedback> {
        let prompt = format!(
            "{}\n\nQuestion ID: {}\nStudent Response: {}\n\nPlease provide detailed feedback.",
            self.marking_guidelines,
            question_id,
            serde_json::to_string_pretty(response)?
        );

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: prompt }],
            }],
            generation_config: GeminiGenerationConfig {
                temperature: 0.1,
                max_output_tokens: 2000,
            },
        };

        let response = self
            .client
            .post(&format!(
                "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-exp:generateContent?key={}",
                self.config.gemini_api_key
            ))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let gemini_response: Value = response.json().await?;
        let content = gemini_response["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("");

        self.parse_ai_feedback(content, "Gemini 2.5 Pro")
    }

    fn create_grading_prompt(&self, question_id: &str, response: &Value) -> String {
        format!(
            "Grade the following HSC Chemistry response:\n\n\
            Question ID: {}\n\
            Student Response: {}\n\n\
            Please provide:\n\
            1. A score out of the maximum marks for this question\n\
            2. Specific feedback on strengths and areas for improvement\n\
            3. Band estimate if applicable\n\
            4. Suggestions for improvement\n\n\
            Format your response as JSON with the following structure:\n\
            {{\n\
              \"score\": <number>,\n\
              \"max_score\": <number>,\n\
              \"feedback\": \"<detailed feedback>\",\n\
              \"strengths\": [\"<strength1>\", \"<strength2>\"],\n\
              \"improvements\": [\"<improvement1>\", \"<improvement2>\"],\n\
              \"band_estimate\": \"<band>\"\n\
            }}",
            question_id,
            serde_json::to_string_pretty(response).unwrap_or_default()
        )
    }

    fn parse_ai_feedback(&self, content: &str, provider: &str) -> Result<QuestionFeedback> {
        // Extract JSON from AI response
        let json_start = content.find('{').unwrap_or(0);
        let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_str = &content[json_start..json_end];

        let feedback_data: Value = serde_json::from_str(json_str)
            .or_else(|_| {
                // Fallback parsing if JSON is malformed
                Ok(json!({
                    "score": 0.0,
                    "max_score": 1.0,
                    "feedback": content,
                    "strengths": [],
                    "improvements": ["Response could not be parsed properly"],
                    "band_estimate": "Unable to determine"
                }))
            })?;

        Ok(QuestionFeedback {
            score: feedback_data["score"].as_f64().unwrap_or(0.0),
            max_score: feedback_data["max_score"].as_f64().unwrap_or(1.0),
            feedback: feedback_data["feedback"].as_str().unwrap_or(content).to_string(),
            strengths: feedback_data["strengths"]
                .as_array()
                .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
                .unwrap_or_default(),
            improvements: feedback_data["improvements"]
                .as_array()
                .map(|arr| arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect())
                .unwrap_or_default(),
            band_estimate: feedback_data["band_estimate"].as_str().map(|s| s.to_string()),
        })
    }
}
