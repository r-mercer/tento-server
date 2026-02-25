use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::domain::quiz_question::QuizQuestion;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, SimpleObject, JsonSchema)]
// #[serde(deny_unknown_fields)]
pub struct Quiz {
    pub id: String,                  // Either set by db, client or API
    pub name: String,                // Friendly name, set on create
    pub created_by_user_id: String,  // User who created the quiz
    pub title: Option<String>,       // Set on create
    pub description: Option<String>, // Set on create
    pub question_count: i16,         // Set on draft, mutable
    pub required_score: i16,         // Set on draft
    pub attempt_limit: i16,          // Set on draft
    pub topic: Option<String>,       // Set on create - Possible tag system
    pub status: QuizStatus,
    pub questions: Option<Vec<QuizQuestion>>, // Set on create
    pub url: String,                          // currently set on draft - subject to change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Enum, Copy, JsonSchema)]
#[serde(deny_unknown_fields)]
pub enum QuizStatus {
    Draft,
    Pending,
    Ready,
    Complete,
}

impl Quiz {
    pub fn new_draft(
        name: &str,
        created_by_user_id: &str,
        question_count: i16,
        required_score: i16,
        attempt_limit: i16,
        url: &str,
    ) -> Self {
        Quiz {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            created_by_user_id: created_by_user_id.to_string(),
            title: None,
            description: None,
            question_count,
            required_score,
            attempt_limit,
            topic: None,
            status: QuizStatus::Draft,
            questions: None,
            url: url.to_string(),
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        }
    }
}

impl Quiz {
    pub fn test_quiz(name: &str, user_id: &str) -> Self {
        Quiz::new_draft(name, user_id, 5, 70, 3, "https://example.com")
    }

    pub fn test_quiz_with_title(name: &str, user_id: &str, title: &str, description: &str) -> Self {
        let mut quiz = Quiz::new_draft(name, user_id, 5, 70, 3, "https://example.com");
        quiz.title = Some(title.to_string());
        quiz.description = Some(description.to_string());
        quiz
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiz_status_serializes_and_deserializes() {
        let status = QuizStatus::Ready;
        let json = serde_json::to_string(&status).expect("status should serialize");
        let parsed: QuizStatus = serde_json::from_str(&json).expect("status should deserialize");

        assert_eq!(status, parsed);
    }

    #[test]
    fn new_draft_initializes_expected_defaults() {
        let quiz = Quiz::new_draft("Rust Basics", "user-1", 10, 80, 2, "https://example.com/rust");

        assert_eq!(quiz.name, "Rust Basics");
        assert_eq!(quiz.created_by_user_id, "user-1");
        assert_eq!(quiz.question_count, 10);
        assert_eq!(quiz.required_score, 80);
        assert_eq!(quiz.attempt_limit, 2);
        assert_eq!(quiz.status, QuizStatus::Draft);
        assert!(quiz.title.is_none());
        assert!(quiz.description.is_none());
        assert!(quiz.questions.is_none());
        assert!(quiz.created_at.is_some());
        assert!(quiz.modified_at.is_some());
    }

    #[test]
    fn test_quiz_with_title_sets_title_and_description() {
        let quiz = Quiz::test_quiz_with_title("Name", "user-1", "Title", "Description");

        assert_eq!(quiz.title.as_deref(), Some("Title"));
        assert_eq!(quiz.description.as_deref(), Some("Description"));
    }
}
