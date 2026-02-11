use crate::errors::{AppError, AppResult};
use crate::models::domain::quiz::QuizStatus;
use uuid::Uuid;

/// Helper to parse UUID from GraphQL ID string
pub fn parse_id(id: &str) -> AppResult<Uuid> {
    Uuid::parse_str(id).map_err(|_| AppError::ValidationError("Invalid UUID format".to_string()))
}

/// Check if quiz is available for taking
pub fn is_quiz_available_for_taking(status: &QuizStatus) -> bool {
    matches!(status, QuizStatus::Ready | QuizStatus::Complete)
}

/// Validate quiz is available for taking, return error if not
pub fn validate_quiz_available_for_taking(status: &QuizStatus) -> AppResult<()> {
    if !is_quiz_available_for_taking(status) {
        return Err(AppError::BadRequest(
            "Quiz is not available for taking".to_string(),
        ));
    }
    Ok(())
}
