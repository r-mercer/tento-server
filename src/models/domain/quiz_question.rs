use async_graphql::{Enum, SimpleObject};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, SimpleObject, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuizQuestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub question_type: QuizQuestionType,
    pub options: Vec<QuizQuestionOption>,
    pub option_count: i16, // default of four
    pub order: i16,
    pub attempt_limit: i16,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, SimpleObject, JsonSchema)]
pub struct QuizQuestionOption {
    pub id: String,
    pub text: String,
    pub correct: bool,
    pub explanation: String, // explanation for why this option is correct or incorrect
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Enum, Copy, JsonSchema)]
pub enum QuizQuestionType {
    Single, // Only one correct option
    Multi,  // Multiple correct options
    Bool,   // True/False question
}
