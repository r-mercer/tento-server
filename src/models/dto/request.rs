use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use validator::Validate;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;

use crate::errors::{AppError, AppResult};
use crate::models::domain::quiz::QuizStatus;
use crate::models::domain::quiz_question::{QuizQuestionOption, QuizQuestionType};
use crate::models::domain::summary_document::SummaryDocument;
use crate::models::dto::quiz_dto::{QuizDto, QuizQuestionDto};

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateUserRequestDto {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,

    #[validate(length(min = 1, max = 100))]
    pub last_name: String,

    #[validate(length(min = 3, max = 50))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct UpdateUserRequestDto {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct QuizDraftDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    // #[validate(required)]
    pub question_count: i16,
    //
    // #[validate(required]
    pub required_score: i16,
    //
    // #[validate(required)]
    pub attempt_limit: i16,

    #[validate(url)]
    pub url: String,
}
impl QuizDraftDto {
    pub(crate) fn from_quiz(quiz: crate::models::domain::Quiz) -> QuizDraftDto {
        QuizDraftDto {
            name: quiz.name,
            question_count: quiz.question_count,
            required_score: quiz.required_score,
            attempt_limit: quiz.attempt_limit,
            url: quiz.url,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject, JsonSchema)]
pub struct GenerateQuizRequestDto {
    pub quiz_title: String,
    pub quiz_description: String,
    pub quiz_topic: String,
    pub quiz_questions: Vec<GenerateQuizQuestionRequestDto>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject, JsonSchema)]
pub struct GenerateQuizQuestionRequestDto {
    pub question_title: String,
    pub question_description: String,
    pub question_type: String,
    pub question_options: Vec<GenerateQuizQuestionOptionRequestDto>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject, JsonSchema)]
pub struct GenerateQuizQuestionOptionRequestDto {
    pub option_text: String,        // text to display
    pub option_correct: String,     // bool
    pub option_explanation: String, // explanation for why this option is correct or incorrect
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject, JsonSchema)]
#[graphql(rename_fields = "snake_case")]
pub struct QuizRequestDto {
    pub id: String,
    pub name: String,
    pub created_by_user_id: String,
    pub title: String,
    pub description: String,
    pub question_count: String,
    pub required_score: String,
    pub attempt_limit: String,
    pub topic: String,
    pub status: String,
    pub questions: Vec<QuizQuestionRequestDto>,
    pub url: String,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject, JsonSchema)]
pub struct QuizQuestionRequestDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub question_type: String,
    pub options: String,
    pub option_count: String,
    pub order: String,
    pub attempt_limit: String,
    pub topic: String,
    pub created_at: String,
    pub modified_at: String,
}

impl From<QuizQuestionDto> for QuizQuestionRequestDto {
    fn from(question: QuizQuestionDto) -> Self {
        let options = serde_json::to_string(&question.options).unwrap_or_else(|_| "[]".to_string());

        QuizQuestionRequestDto {
            id: question.id,
            title: question.title,
            description: question.description,
            question_type: format!("{:?}", question.question_type),
            options,
            option_count: question.option_count.to_string(),
            order: question.order.to_string(),
            attempt_limit: question.attempt_limit.to_string(),
            topic: question.topic,
            created_at: question.created_at.to_rfc3339(),
            modified_at: question.modified_at.to_rfc3339(),
        }
    }
}

impl TryFrom<QuizQuestionRequestDto> for QuizQuestionDto {
    type Error = AppError;

    fn try_from(dto: QuizQuestionRequestDto) -> Result<Self, Self::Error> {
        let options = parse_options_json(&dto.options)?;
        let created_at = parse_optional_datetime(&dto.created_at)?.unwrap_or_else(Utc::now);
        let modified_at = parse_optional_datetime(&dto.modified_at)?.unwrap_or_else(Utc::now);

        Ok(QuizQuestionDto {
            id: dto.id,
            title: dto.title,
            description: dto.description,
            question_type: parse_question_type(&dto.question_type)?,
            options,
            option_count: parse_i16_required(&dto.option_count, "option_count")?,
            order: parse_i16_required(&dto.order, "order")?,
            attempt_limit: parse_i16_required(&dto.attempt_limit, "attempt_limit")?,
            topic: dto.topic,
            created_at,
            modified_at,
        })
    }
}

impl From<QuizDto> for QuizRequestDto {
    fn from(quiz: QuizDto) -> Self {
        QuizRequestDto {
            id: quiz.id,
            name: quiz.name,
            created_by_user_id: quiz.created_by_user_id,
            title: quiz.title,
            description: quiz.description,
            question_count: quiz.question_count.to_string(),
            required_score: quiz.required_score.to_string(),
            attempt_limit: quiz.attempt_limit.to_string(),
            topic: quiz.topic,
            status: format!("{:?}", quiz.status).to_lowercase(),
            questions: quiz
                .questions
                .into_iter()
                .map(QuizQuestionRequestDto::from)
                .collect(),
            url: quiz.url,
            created_at: quiz.created_at.to_rfc3339(),
            modified_at: quiz.modified_at.to_rfc3339(),
        }
    }
}

impl TryFrom<QuizRequestDto> for QuizDto {
    type Error = AppError;

    fn try_from(dto: QuizRequestDto) -> Result<Self, Self::Error> {
        let created_at = parse_optional_datetime(&dto.created_at)?.unwrap_or_else(Utc::now);
        let modified_at = parse_optional_datetime(&dto.modified_at)?.unwrap_or_else(Utc::now);

        Ok(QuizDto {
            id: dto.id,
            name: dto.name,
            created_by_user_id: dto.created_by_user_id,
            title: dto.title,
            description: dto.description,
            question_count: parse_i16_required(&dto.question_count, "question_count")?,
            required_score: parse_i16_required(&dto.required_score, "required_score")?,
            attempt_limit: parse_i16_required(&dto.attempt_limit, "attempt_limit")?,
            topic: dto.topic,
            status: parse_quiz_status(&dto.status)?,
            questions: dto
                .questions
                .into_iter()
                .map(QuizQuestionDto::try_from)
                .collect::<Result<Vec<_>, AppError>>()?,
            url: dto.url,
            created_at,
            modified_at,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject, JsonSchema)]
#[graphql(rename_fields = "snake_case")]
pub struct SummaryDocumentRequestDto {
    pub id: String,
    pub quiz_id: String,
    pub url: String,
    pub content: String,
    pub created_at: String,
    pub modified_at: String,
}

impl From<SummaryDocument> for SummaryDocumentRequestDto {
    fn from(summary_document: SummaryDocument) -> Self {
        SummaryDocumentRequestDto {
            id: summary_document.id,
            quiz_id: summary_document.quiz_id,
            url: summary_document.url,
            content: summary_document.content,
            created_at: summary_document
                .created_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            modified_at: summary_document
                .modified_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
        }
    }
}

impl TryFrom<SummaryDocumentRequestDto> for SummaryDocument {
    type Error = AppError;

    fn try_from(dto: SummaryDocumentRequestDto) -> Result<Self, Self::Error> {
        Ok(SummaryDocument {
            id: dto.id,
            quiz_id: dto.quiz_id,
            url: dto.url,
            content: dto.content,
            created_at: parse_optional_datetime(&dto.created_at)?,
            modified_at: parse_optional_datetime(&dto.modified_at)?,
        })
    }
}

fn parse_i16_required(value: &str, field: &str) -> AppResult<i16> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::ValidationError(format!("{} is required", field)));
    }

    trimmed
        .parse::<i16>()
        .map_err(|e| AppError::ValidationError(format!("Invalid {}: {}", field, e)))
}

fn parse_optional_datetime(value: &str) -> AppResult<Option<DateTime<Utc>>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    DateTime::parse_from_rfc3339(trimmed)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(|e| AppError::ValidationError(format!("Invalid datetime: {}", e)))
}

fn parse_quiz_status(value: &str) -> AppResult<QuizStatus> {
    match value.trim().to_lowercase().as_str() {
        "draft" => Ok(QuizStatus::Draft),
        "pending" => Ok(QuizStatus::Pending),
        "ready" => Ok(QuizStatus::Ready),
        "complete" => Ok(QuizStatus::Complete),
        _ => Err(AppError::ValidationError(format!(
            "Invalid status: {}",
            value
        ))),
    }
}

fn parse_question_type(value: &str) -> AppResult<QuizQuestionType> {
    match value.trim().to_lowercase().as_str() {
        "single" => Ok(QuizQuestionType::Single),
        "multi" => Ok(QuizQuestionType::Multi),
        "bool" | "boolean" => Ok(QuizQuestionType::Bool),
        _ => Err(AppError::ValidationError(format!(
            "Invalid question_type: {}",
            value
        ))),
    }
}

fn parse_options_json(value: &str) -> AppResult<Vec<QuizQuestionOption>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::ValidationError(
            "Options JSON cannot be empty".to_string(),
        ));
    }

    let parsed: serde_json::Value = serde_json::from_str(trimmed)
        .or_else(|_| attempt_repair_options_json(trimmed))
        .map_err(|e| AppError::ValidationError(format!("Invalid options JSON: {}", e)))?;

    let items = parsed
        .as_array()
        .ok_or_else(|| AppError::ValidationError("Options JSON must be an array".to_string()))?;

    let mut options = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let obj = item.as_object().ok_or_else(|| {
            AppError::ValidationError(format!("Option at index {} must be an object", index))
        })?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::ValidationError(format!("Option {} missing id", index)))?
            .to_string();
        let text = obj
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::ValidationError(format!("Option {} missing text", index)))?
            .to_string();
        let correct = obj
            .get("correct")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let explanation = obj
            .get("explanation")
            .and_then(|v| v.as_str())
            .unwrap_or("No explanation provided.")
            .to_string();

        options.push(QuizQuestionOption {
            id,
            text,
            correct,
            explanation,
        });
    }

    Ok(options)
}

fn attempt_repair_options_json(value: &str) -> Result<serde_json::Value, serde_json::Error> {
    if value.starts_with('[') && !value.ends_with(']') {
        let repaired = format!("{}]", value);
        if let Ok(parsed) = serde_json::from_str(&repaired) {
            return Ok(parsed);
        }
    }

    if value.starts_with('{') && value.ends_with('}') {
        let wrapped = format!("[{}]", value);
        if let Ok(parsed) = serde_json::from_str(&wrapped) {
            return Ok(parsed);
        }
    }

    serde_json::from_str(value)
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct ChatCompletionRequest {
    #[validate(length(min = 1, max = 10000))]
    pub prompt: String,

    #[validate(length(min = 1, max = 100))]
    pub model: String,
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct PaginationParams {
    #[validate(range(min = 0))]
    pub offset: Option<i64>,

    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: Some(0),
            limit: Some(20),
        }
    }
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0)
    }

    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20).min(100)
    }
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct QuestionAnswerInput {
    pub question_id: String,              // UUID as string
    pub selected_option_ids: Vec<String>, // UUID strings
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct SubmitQuizAttemptInput {
    pub quiz_id: String,
    pub answers: Vec<QuestionAnswerInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct UpdateQuizQuestionOptionInput {
    pub id: String,
    pub text: Option<String>,
    pub correct: Option<bool>,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct UpdateQuizQuestionInput {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub options: Option<Vec<UpdateQuizQuestionOptionInput>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct UpdateQuizInput {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub questions: Option<Vec<UpdateQuizQuestionInput>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_valid_create_user_request() {
        let request = CreateUserRequestDto {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let request = CreateUserRequestDto {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "invalid-email".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_username_too_short() {
        let request = CreateUserRequestDto {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "ab".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(request.validate().is_err());
    }
}
