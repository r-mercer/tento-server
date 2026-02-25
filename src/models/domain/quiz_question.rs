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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiz_question_type_round_trip_serialization() {
        let variants = [
            QuizQuestionType::Single,
            QuizQuestionType::Multi,
            QuizQuestionType::Bool,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("variant should serialize");
            let parsed: QuizQuestionType =
                serde_json::from_str(&json).expect("variant should deserialize");
            assert_eq!(variant, parsed);
        }
    }

    #[test]
    fn quiz_question_type_rejects_unknown_variant() {
        let invalid = "\"Essay\"";
        let parsed = serde_json::from_str::<QuizQuestionType>(invalid);

        assert!(parsed.is_err());
    }

    #[test]
    fn quiz_question_with_options_preserves_type_and_options() {
        let options = vec![
            QuizQuestionOption {
                id: "opt-1".to_string(),
                text: "True".to_string(),
                correct: true,
                explanation: "Correct statement".to_string(),
            },
            QuizQuestionOption {
                id: "opt-2".to_string(),
                text: "False".to_string(),
                correct: false,
                explanation: "Incorrect statement".to_string(),
            },
        ];

        let question = QuizQuestion {
            id: "q-1".to_string(),
            title: "Sample Bool Question".to_string(),
            description: "Pick the correct option".to_string(),
            question_type: QuizQuestionType::Bool,
            option_count: options.len() as i16,
            options,
            order: 1,
            attempt_limit: 1,
            topic: "basics".to_string(),
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        };

        assert_eq!(question.question_type, QuizQuestionType::Bool);
        assert_eq!(question.option_count as usize, question.options.len());
        assert!(question.options.iter().any(|o| o.correct));
    }
}
