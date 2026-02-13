use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::domain::quiz_attempt::QuizAttempt;
use crate::models::domain::quiz_question::QuizQuestionType;
use crate::models::domain::{quiz::QuizStatus, Quiz, QuizQuestion, User};

// ============================================================================
// User DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct UserDto {
    pub username: String,
    pub email: String,
    pub full_name: String,
    #[graphql(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        UserDto {
            username: user.username,
            email: user.email,
            full_name: format!("{} {}", user.first_name, user.last_name),
            created_at: user.created_at,
        }
    }
}

// ============================================================================
// Quiz DTOs - Standard Response Format
// ============================================================================

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizDto {
    pub id: String,
    pub name: String,
    pub created_by_user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub question_count: i16,
    pub required_score: i16,
    pub attempt_limit: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    pub status: QuizStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questions: Option<Vec<QuizQuestion>>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

impl From<Quiz> for QuizDto {
    fn from(quiz: Quiz) -> Self {
        QuizDto {
            id: quiz.id,
            name: quiz.name,
            created_by_user_id: quiz.created_by_user_id,
            title: quiz.title,
            description: quiz.description,
            question_count: quiz.question_count,
            required_score: quiz.required_score,
            attempt_limit: quiz.attempt_limit,
            topic: quiz.topic,
            status: quiz.status,
            questions: quiz.questions,
            url: quiz.url,
            created_at: quiz.created_at,
            modified_at: quiz.modified_at,
        }
    }
}

// ============================================================================
// Quiz DTOs - LLM Optimized (String-based Enums)
// ============================================================================

/// LLM-optimized Quiz DTO with string-based enums for better LLM serialization
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuizDtoLLM {
    pub id: String,
    pub name: String,
    pub created_by_user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub question_count: i16,
    pub required_score: i16,
    pub attempt_limit: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Status as string for LLM processing
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questions: Option<Vec<QuizQuestionDtoLLM>>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

impl QuizDtoLLM {
    /// Convert from standard Quiz DTO to LLM-optimized format
    pub fn from_quiz_dto(quiz: QuizDto) -> Self {
        QuizDtoLLM {
            id: quiz.id,
            name: quiz.name,
            created_by_user_id: quiz.created_by_user_id,
            title: quiz.title,
            description: quiz.description,
            question_count: quiz.question_count,
            required_score: quiz.required_score,
            attempt_limit: quiz.attempt_limit,
            topic: quiz.topic,
            status: format!("{:?}", quiz.status),
            questions: quiz.questions.map(|qs| {
                qs.into_iter()
                    .map(QuizQuestionDtoLLM::from_quiz_question)
                    .collect()
            }),
            url: quiz.url,
            created_at: quiz.created_at.map(|dt| dt.to_rfc3339()),
            modified_at: quiz.modified_at.map(|dt| dt.to_rfc3339()),
        }
    }

    /// Convert from domain Quiz directly
    pub fn from_quiz(quiz: Quiz) -> Self {
        Self::from_quiz_dto(quiz.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuizQuestionDtoLLM {
    pub id: String,
    pub title: String,
    pub description: String,
    /// Question type as string for LLM processing
    pub question_type: String,
    pub options: Vec<QuizQuestionOptionDtoLLM>,
    pub option_count: i16,
    pub order: i16,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

impl QuizQuestionDtoLLM {
    pub fn from_quiz_question(question: QuizQuestion) -> Self {
        QuizQuestionDtoLLM {
            id: question.id,
            title: question.title,
            description: question.description,
            question_type: format!("{:?}", question.question_type),
            options: question
                .options
                .into_iter()
                .map(QuizQuestionOptionDtoLLM::from)
                .collect(),
            option_count: question.option_count,
            order: question.order,
            topic: question.topic,
            created_at: question.created_at.map(|dt| dt.to_rfc3339()),
            modified_at: question.modified_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuizQuestionOptionDtoLLM {
    pub id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl From<crate::models::domain::quiz_question::QuizQuestionOption> for QuizQuestionOptionDtoLLM {
    fn from(option: crate::models::domain::quiz_question::QuizQuestionOption) -> Self {
        QuizQuestionOptionDtoLLM {
            id: option.id,
            text: option.text,
            correct: Some(option.correct),
            explanation: Some(option.explanation),
        }
    }
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct ChatCompletionResponse {
    pub content: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct PaginationMetadata {
    pub offset: i64,
    pub limit: i64,
    pub total: i64,
}

// ============================================================================
// Quiz DTOs for Answer Visibility Control
// ============================================================================

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizQuestionOptionForTaking {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizQuestionForTaking {
    pub id: String,
    pub title: String,
    pub description: String,
    pub question_type: QuizQuestionType,
    pub options: Vec<QuizQuestionOptionForTaking>,
    pub option_count: i16,
    pub order: i16,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizForTaking {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub question_count: i16,
    pub required_score: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    pub status: QuizStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questions: Option<Vec<QuizQuestionForTaking>>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl QuizForTaking {
    pub fn from_quiz(quiz: Quiz) -> Self {
        let questions = quiz.questions.map(|qs| {
            qs.into_iter()
                .map(|q| QuizQuestionForTaking {
                    id: q.id,
                    title: q.title,
                    description: q.description,
                    question_type: q.question_type,
                    options: q
                        .options
                        .into_iter()
                        .map(|opt| QuizQuestionOptionForTaking {
                            id: opt.id,
                            text: opt.text,
                        })
                        .collect(),
                    option_count: q.option_count,
                    order: q.order,
                    topic: q.topic,
                    created_at: q.created_at,
                })
                .collect()
        });

        QuizForTaking {
            id: quiz.id,
            name: quiz.name,
            title: quiz.title,
            description: quiz.description,
            question_count: quiz.question_count,
            required_score: quiz.required_score,
            topic: quiz.topic,
            status: quiz.status,
            questions,
            url: quiz.url,
            created_at: quiz.created_at,
        }
    }
}

// ============================================================================
// Quiz Attempt DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizAttemptResponse {
    pub id: String,
    pub quiz_id: String,
    pub points_earned: i16,
    pub total_possible: i16,
    pub passed: bool,
    pub attempt_number: i16,
    pub submitted_at: DateTime<Utc>,
}

impl From<QuizAttempt> for QuizAttemptResponse {
    fn from(attempt: QuizAttempt) -> Self {
        QuizAttemptResponse {
            id: attempt.id,
            quiz_id: attempt.quiz_id,
            points_earned: attempt.points_earned,
            total_possible: attempt.total_possible,
            passed: attempt.passed,
            attempt_number: attempt.attempt_number,
            submitted_at: attempt.submitted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuestionAttemptDetail {
    pub question_id: String,
    pub user_selected_option_ids: Vec<String>,
    pub correct_option_ids: Vec<String>,
    pub is_correct: bool,
    pub points_earned: i16,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizAttemptReview {
    pub attempt: QuizAttemptResponse,
    pub quiz: QuizDto,
    pub question_results: Vec<QuestionAttemptDetail>,
}

#[derive(Debug, Serialize, SimpleObject)]
pub struct PaginatedQuizAttemptResponse {
    pub data: Vec<QuizAttemptResponse>,
    pub pagination: PaginationMetadata,
}

pub type PaginatedResponseQuizAttempt = PaginatedQuizAttemptResponse;

// ============================================================================
// Standard API Response Wrappers
// ============================================================================

/// Generic API response wrapper for single resource
#[derive(Debug, Serialize, SimpleObject)]
pub struct ApiResponse<T: async_graphql::OutputType> {
    pub data: T,
    pub message: String,
}

impl<T: async_graphql::OutputType> ApiResponse<T> {
    pub fn new(data: T, message: impl Into<String>) -> Self {
        Self {
            data,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct CreateQuizDraftResponseData {
    pub quiz: QuizDto,
    pub job_id: String,
}

#[derive(Debug, Serialize, SimpleObject)]
pub struct DeleteResponseData {
    pub message: String,
}

pub type ChatCompletionApiResponse = ApiResponse<ChatCompletionResponse>;

/// Generic API response wrapper for paginated resources
#[derive(Debug, Serialize, SimpleObject)]
pub struct PaginatedResponse<T: async_graphql::OutputType> {
    pub data: Vec<T>,
    pub pagination: PaginationMetadata,
}

impl<T: async_graphql::OutputType> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, pagination: PaginationMetadata) -> Self {
        Self { data, pagination }
    }
}

// ============================================================================
// Response Type Aliases
// ============================================================================

pub type CreateUserResponse = ApiResponse<UserDto>;
pub type UpdateUserResponse = ApiResponse<UserDto>;
pub type CreateQuizDraftResponse = ApiResponse<CreateQuizDraftResponseData>;
pub type DeleteUserResponse = ApiResponse<DeleteResponseData>;

/// Response for all paginated user queries
#[derive(Debug, Serialize, SimpleObject)]
pub struct PaginatedUserResponse {
    pub data: Vec<UserDto>,
    pub pagination: PaginationMetadata,
}

pub type PaginatedResponseUserDto = PaginatedUserResponse;

/// Response for all paginated quiz queries
#[derive(Debug, Serialize, SimpleObject)]
pub struct PaginatedQuizResponse {
    pub data: Vec<QuizDto>,
    pub pagination: PaginationMetadata,
}

pub type PaginatedResponseQuizDto = PaginatedQuizResponse;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_dto_full_name() {
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            github_id: None,
            role: crate::models::domain::user::UserRole::default(),
            created_at: Some(Utc::now()),
        };

        let dto: UserDto = user.into();
        assert_eq!(dto.full_name, "John Doe");
        assert_eq!(dto.username, "johndoe");
    }
}
