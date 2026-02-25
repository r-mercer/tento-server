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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attempt(passed: bool, points_earned: i16, required_score: i16) -> QuizAttempt {
        QuizAttempt {
            id: "attempt-1".to_string(),
            user_id: "user-1".to_string(),
            quiz_id: "quiz-1".to_string(),
            points_earned,
            required_score,
            total_possible: 5,
            passed,
            attempt_number: 1,
            question_answers: vec![QuizAttemptQuestion {
                id: "qa-1".to_string(),
                quiz_question_id: "q-1".to_string(),
                selected_option_ids: vec!["opt-1".to_string()],
                is_correct: points_earned > 0,
                points_earned,
            }],
            submitted_at: Utc::now(),
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        }
    }

    #[test]
    fn quiz_attempt_round_trip_serialization_preserves_grading_fields() {
        let attempt = make_attempt(true, 4, 3);

        let json = serde_json::to_string(&attempt).expect("attempt should serialize");
        let parsed: QuizAttempt = serde_json::from_str(&json).expect("attempt should deserialize");

        assert_eq!(parsed.points_earned, 4);
        assert_eq!(parsed.required_score, 3);
        assert!(parsed.passed);
        assert_eq!(parsed.question_answers.len(), 1);
        assert!(parsed.question_answers[0].is_correct);
    }

    #[test]
    fn quiz_attempt_can_represent_failed_attempt() {
        let attempt = make_attempt(false, 1, 4);

        assert!(!attempt.passed);
        assert!(attempt.points_earned < attempt.required_score);
        assert_eq!(attempt.question_answers[0].points_earned, 1);
    }
}
