use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct QuizQuestion {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub question_type: QuizQuestionType,
    pub options: Vec<QuizQuestionOption>,
    pub correct_question_option: String,
    pub order: i16,
    pub attempt_limit: i16,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct QuizQuestionOption {
    pub id: Uuid,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum QuizQuestionType {
    Single,
    Multi,
    Bool,
}
