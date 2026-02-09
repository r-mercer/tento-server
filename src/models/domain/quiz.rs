use uuid::Uuid;
// use bson::{uuid, Uuid};
use chrono::{DateTime, Utc};
// use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};

use crate::models::{domain::quiz_question::QuizQuestion, dto::request};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Quiz {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub question_count: i16,
    pub required_score: i16,
    pub attempt_limit: i16,
    pub topic: String,
    pub status: QuizStatus,
    pub questions: Vec<QuizQuestion>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum QuizStatus {
    Draft,
    Pending,
    Ready,
    Complete,
}
