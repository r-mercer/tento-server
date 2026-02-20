use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct QuizAttempt {
    pub id: String,
    pub user_id: String,
    pub quiz_id: String,
    pub points_earned: i16,
    pub required_score: i16,
    pub total_possible: i16,
    pub passed: bool,
    pub attempt_number: i16,
    pub question_answers: Vec<QuizAttemptQuestion>,
    pub submitted_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct QuizAttemptQuestion {
    pub id: String,
    pub quiz_question_id: String,
    pub selected_option_ids: Vec<String>,
    pub is_correct: bool,
    pub points_earned: i16,
}
