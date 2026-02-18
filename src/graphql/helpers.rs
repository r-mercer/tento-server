use crate::errors::{AppError, AppResult};
use crate::models::domain::quiz::QuizStatus;

/// Helper to validate UUID format and return as String
pub fn parse_id(id: &str) -> AppResult<String> {
    // Accept either a UUID or a 24-character MongoDB ObjectId hex string
    if uuid::Uuid::parse_str(id).is_ok() {
        return Ok(id.to_string());
    }

    // Check for 24-hex-char ObjectId
    if id.len() == 24 && id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(id.to_string());
    }

    Err(AppError::ValidationError(
        "Invalid id format; expected UUID or 24-char ObjectId hex".to_string(),
    ))
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
