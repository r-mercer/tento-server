use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, SimpleObject)]
pub struct SummaryDocument {
    pub id: String,
    pub quiz_id: String,
    pub url: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

impl SummaryDocument {
    pub fn new_summary_document(url: &str, quiz_id: &str, content: &str) -> Self {
        SummaryDocument {
            id: uuid::Uuid::new_v4().to_string(),
            quiz_id: quiz_id.to_string(),
            content: content.to_string(),
            url: url.to_string(),
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        }
    }
}
