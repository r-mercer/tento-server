use async_graphql::SimpleObject;
use uuid::Uuid;
// use bson::{uuid, Uuid};
use chrono::{DateTime, Utc};
// use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};

// use crate::models::domain::quiz_question::QuizQuestion;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, SimpleObject)]
pub struct SummaryDocument {
    pub id: Uuid,
    pub url: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

impl SummaryDocument {
    pub fn new_summary_document(url: &str, content: &str) -> Self {
        SummaryDocument {
            id: Uuid::new_v4(),
            content: content.to_string(),
            url: url.to_string(),
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        }
    }
}
