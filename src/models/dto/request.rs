use async_graphql::InputObject;
use serde::Deserialize;
use validator::Validate;

use crate::models::domain::{Quiz, QuizQuestion};

// static USERNAME_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
//     regex::Regex::new(r"^[a-zA-Z0-9_]+$")
//         .expect("USERNAME_REGEX is a valid regex pattern")
// });

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,

    #[validate(length(min = 1, max = 100))]
    pub last_name: String,

    #[validate(length(min = 3, max = 50))]
    // #[validate(regex(
    //     path = "*USERNAME_REGEX",
    //     message = "Username must be alphanumeric with underscores"
    // ))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct CreateQuizDraftRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    // #[validate(required)]
    pub question_count: i16,
    //
    // #[validate(required)]
    pub required_score: i16,
    //
    // #[validate(required)]
    pub attempt_limit: i16,

    #[validate(url)]
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
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

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
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

impl From<QuizQuestion> for QuizQuestionRequestDto {
    fn from(question: QuizQuestion) -> Self {
        let options = serde_json::to_string(&question.options)
            .unwrap_or_else(|_| "[]".to_string());

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
            created_at: question
                .created_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            modified_at: question
                .modified_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
        }
    }
}

impl From<Quiz> for QuizRequestDto {
    fn from(quiz: Quiz) -> Self {
        QuizRequestDto {
            id: quiz.id,
            name: quiz.name,
            created_by_user_id: quiz.created_by_user_id,
            title: quiz.title.unwrap_or_default(),
            description: quiz.description.unwrap_or_default(),
            question_count: quiz.question_count.to_string(),
            required_score: quiz.required_score.to_string(),
            attempt_limit: quiz.attempt_limit.to_string(),
            topic: quiz.topic.unwrap_or_default(),
            status: format!("{:?}", quiz.status),
            questions: quiz
                .questions
                .unwrap_or_default()
                .into_iter()
                .map(QuizQuestionRequestDto::from)
                .collect(),
            url: quiz.url,
            created_at: quiz
                .created_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            modified_at: quiz
                .modified_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
        }
    }
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
pub struct QuestionAnswerInput {
    pub question_id: String,              // UUID as string
    pub selected_option_ids: Vec<String>, // UUID strings
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct SubmitQuizAttemptInput {
    pub quiz_id: String,
    pub answers: Vec<QuestionAnswerInput>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_valid_create_user_request() {
        let request = CreateUserRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let request = CreateUserRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "invalid-email".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_username_too_short() {
        let request = CreateUserRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "ab".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(request.validate().is_err());
    }
}
