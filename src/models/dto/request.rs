use async_graphql::InputObject;
use serde::{Deserialize, Serialize};
use validator::Validate;

// ============================================================================
// User Request DTOs
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,

    #[validate(length(min = 1, max = 100))]
    pub last_name: String,

    #[validate(length(min = 3, max = 50))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

impl CreateUserRequest {
    /// Validate and normalize the request
    pub fn validate_and_normalize(&mut self) -> Result<(), validator::ValidationErrors> {
        self.first_name = self.first_name.trim().to_string();
        self.last_name = self.last_name.trim().to_string();
        self.username = self.username.trim().to_lowercase();
        self.email = self.email.trim().to_lowercase();
        self.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

impl UpdateUserRequest {
    /// Validate and normalize the request
    pub fn validate_and_normalize(&mut self) -> Result<(), validator::ValidationErrors> {
        if let Some(ref mut first_name) = self.first_name {
            *first_name = first_name.trim().to_string();
        }
        if let Some(ref mut last_name) = self.last_name {
            *last_name = last_name.trim().to_string();
        }
        if let Some(ref mut email) = self.email {
            *email = email.trim().to_lowercase();
        }
        self.validate()?;
        Ok(())
    }
}

// ============================================================================
// Quiz Request DTOs
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct CreateQuizDraftRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(range(min = 1, max = 1000))]
    pub question_count: i16,

    #[validate(range(min = 0, max = 100))]
    pub required_score: i16,

    #[validate(range(min = 1, max = 1000))]
    pub attempt_limit: i16,

    #[validate(url)]
    pub url: String,
}

impl CreateQuizDraftRequest {
    /// Validate and normalize the request
    pub fn validate_and_normalize(&mut self) -> Result<(), validator::ValidationErrors> {
        self.name = self.name.trim().to_string();
        self.url = self.url.trim().to_string();
        self.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct UpdateQuizRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(length(min = 1))]
    pub title: Option<String>,

    #[validate(length(min = 1))]
    pub description: Option<String>,

    #[validate(range(min = 0, max = 100))]
    pub required_score: Option<i16>,

    pub topic: Option<String>,
}

impl UpdateQuizRequest {
    /// Validate and normalize the request
    pub fn validate_and_normalize(&mut self) -> Result<(), validator::ValidationErrors> {
        if let Some(ref mut name) = self.name {
            *name = name.trim().to_string();
        }
        if let Some(ref mut title) = self.title {
            *title = title.trim().to_string();
        }
        if let Some(ref mut description) = self.description {
            *description = description.trim().to_string();
        }
        if let Some(ref mut topic) = self.topic {
            *topic = topic.trim().to_string();
        }
        self.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct ChatCompletionRequest {
    #[validate(length(min = 1, max = 10000))]
    pub prompt: String,

    #[validate(length(min = 1, max = 100))]
    pub model: String,
}

impl ChatCompletionRequest {
    /// Validate and normalize the request
    pub fn validate_and_normalize(&mut self) -> Result<(), validator::ValidationErrors> {
        self.model = self.model.trim().to_string();
        self.validate()?;
        Ok(())
    }
}

// ============================================================================
// Pagination DTOs
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
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
    /// Get the validated offset value
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0)
    }

    /// Get the validated limit value (capped at 100)
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20).min(100)
    }

    /// Validate the pagination parameters
    pub fn validate_params(&self) -> Result<(), validator::ValidationErrors> {
        let new_self = self.clone();
        new_self.validate()
    }
}

// ============================================================================
// Quiz Attempt Input DTOs
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct QuestionAnswerInput {
    pub question_id: String,
    pub selected_option_ids: Vec<String>,
}

impl QuestionAnswerInput {
    /// Validate that at least one option is selected
    pub fn validate_selection(&self) -> Result<(), String> {
        if self.selected_option_ids.is_empty() {
            Err(format!(
                "At least one option must be selected for question {}",
                self.question_id
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, InputObject)]
pub struct SubmitQuizAttemptInput {
    pub quiz_id: String,
    pub answers: Vec<QuestionAnswerInput>,
}

impl SubmitQuizAttemptInput {
    /// Validate that all answers have selections
    pub fn validate_all_answers(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for answer in &self.answers {
            if let Err(e) = answer.validate_selection() {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

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

    #[test]
    fn test_pagination_defaults() {
        let params = PaginationParams::default();
        assert_eq!(params.offset(), 0);
        assert_eq!(params.limit(), 20);
    }

    #[test]
    fn test_pagination_limit_capped() {
        let params = PaginationParams {
            offset: Some(0),
            limit: Some(150),
        };
        assert_eq!(params.limit(), 100);
    }

    #[test]
    fn test_question_answer_validation() {
        let answer = QuestionAnswerInput {
            question_id: "q1".to_string(),
            selected_option_ids: vec![],
        };
        assert!(answer.validate_selection().is_err());

        let answer = QuestionAnswerInput {
            question_id: "q1".to_string(),
            selected_option_ids: vec!["opt1".to_string()],
        };
        assert!(answer.validate_selection().is_ok());
    }
}
