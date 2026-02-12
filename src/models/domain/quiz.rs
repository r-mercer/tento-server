use async_graphql::{Enum, SimpleObject};
use uuid::Uuid;
// use bson::{uuid, Uuid};
use chrono::{DateTime, Utc};
// use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};

use crate::models::domain::quiz_question::QuizQuestion;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, SimpleObject)]
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Enum, Copy)]
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
            id: Uuid::new_v4().to_string(),
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
