use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct QuizAttempt {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quiz_id: Uuid,
    pub explanation: String,
    pub question_attempts: Vec<QuizAttemptQuestions>,
    pub total_score: i16,
    pub attempt_number: i16,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct QuizAttemptQuestions {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub quiz_question_id: Uuid,
    pub correct: bool,
}
