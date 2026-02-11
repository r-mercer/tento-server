use async_graphql::InputObject;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

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

// ============================================================================
// Quiz Attempt Input DTOs
// ============================================================================

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
