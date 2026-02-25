use crate::errors::{AppError, AppResult};
use crate::models::domain::quiz_attempt::{QuizAttempt, QuizAttemptQuestion};
use crate::models::domain::quiz_question::QuizQuestionType;
use crate::models::domain::{Quiz, QuizQuestion};
use crate::models::dto::request::QuestionAnswerInput;
use chrono::Utc;

pub struct QuizAttemptService;

impl QuizAttemptService {
    pub fn grade_attempt(
        quiz: &Quiz,
        submitted_answers: &[QuestionAnswerInput],
    ) -> AppResult<(i16, Vec<QuizAttemptQuestion>)> {
        let question_map: std::collections::HashMap<&str, &QuizQuestion> = quiz
            .questions
            .as_ref()
            .ok_or(AppError::BadRequest("Quiz has no questions".to_string()))?
            .iter()
            .map(|q| (q.id.as_str(), q))
            .collect();

        for submitted_answer in submitted_answers {
            if !question_map.contains_key(submitted_answer.question_id.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Question '{}' not found in quiz",
                    submitted_answer.question_id
                )));
            }

            let question = question_map[submitted_answer.question_id.as_str()];
            for option_id in &submitted_answer.selected_option_ids {
                let valid_option = question.options.iter().any(|opt| opt.id == *option_id);
                if !valid_option {
                    return Err(AppError::BadRequest(format!(
                        "Option '{}' not found in question '{}'",
                        option_id, submitted_answer.question_id
                    )));
                }
            }
        }

        let mut total_points: i16 = 0;
        let mut question_results = Vec::new();

        for submitted_answer in submitted_answers {
            let question_id = &submitted_answer.question_id;

            let question = question_map
                .get(question_id.as_str())
                .ok_or(AppError::NotFound("Question not found".to_string()))?;

            let (is_correct, points) =
                Self::grade_question(question, submitted_answer.selected_option_ids.clone())?;

            total_points += points;

            question_results.push(QuizAttemptQuestion {
                id: uuid::Uuid::new_v4().to_string(),
                quiz_question_id: question_id.to_string(),
                selected_option_ids: submitted_answer.selected_option_ids.clone(),
                is_correct,
                points_earned: points,
            });
        }

        Ok((total_points, question_results))
    }

    fn grade_question(
        question: &QuizQuestion,
        selected_option_ids: Vec<String>,
    ) -> AppResult<(bool, i16)> {
        let correct_option_ids: Vec<&str> = question
            .options
            .iter()
            .filter(|opt| opt.correct)
            .map(|opt| opt.id.as_str())
            .collect();

        let (is_correct, points) = match question.question_type {
            QuizQuestionType::Single => {
                // Correct if exactly one option selected AND it's correct
                let is_correct = selected_option_ids.len() == 1
                    && !correct_option_ids.is_empty()
                    && selected_option_ids[0] == correct_option_ids[0];
                (is_correct, if is_correct { 1 } else { 0 })
            }
            QuizQuestionType::Multi => {
                // Correct if ALL correct options selected AND zero incorrect options
                if correct_option_ids.is_empty() {
                    // No correct options defined - invalid question
                    return Err(AppError::BadRequest(
                        "Multi-choice question has no correct options".to_string(),
                    ));
                }

                let has_all_correct = correct_option_ids
                    .iter()
                    .all(|id| selected_option_ids.contains(&id.to_string()));
                let has_no_incorrect = selected_option_ids
                    .iter()
                    .all(|id| correct_option_ids.contains(&id.as_str()));
                let is_correct = has_all_correct && has_no_incorrect;
                (is_correct, if is_correct { 1 } else { 0 })
            }
            QuizQuestionType::Bool => {
                // Correct if the single option matches the correct option
                let is_correct = selected_option_ids.len() == 1
                    && !correct_option_ids.is_empty()
                    && selected_option_ids[0] == correct_option_ids[0];
                (is_correct, if is_correct { 1 } else { 0 })
            }
        };

        Ok((is_correct, points))
    }

    /// Create a new quiz attempt from grading results
    pub fn create_attempt(
        user_id: &str,
        quiz_id: &str,
        points_earned: i16,
        total_possible: i16,
        attempt_number: i16,
        required_score: i16,
        question_answers: Vec<QuizAttemptQuestion>,
    ) -> QuizAttempt {
        let passed = points_earned >= required_score;

        QuizAttempt {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            quiz_id: quiz_id.to_string(),
            points_earned,
            required_score,
            total_possible,
            passed,
            attempt_number,
            question_answers,
            submitted_at: Utc::now(),
            created_at: Some(Utc::now()),
            modified_at: Some(Utc::now()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{
        domain::{
            quiz::QuizStatus,
            quiz_question::{QuizQuestionOption, QuizQuestionType},
        },
        dto::request::QuestionAnswerInput,
    };

    use super::*;

    fn make_option(id: &str, correct: bool) -> QuizQuestionOption {
        QuizQuestionOption {
            id: id.to_string(),
            text: format!("Option {}", id),
            correct,
            explanation: "test explanation".to_string(),
        }
    }

    fn make_question(
        id: &str,
        question_type: QuizQuestionType,
        options: Vec<QuizQuestionOption>,
    ) -> QuizQuestion {
        QuizQuestion {
            id: id.to_string(),
            title: format!("Question {}", id),
            description: "test question".to_string(),
            question_type,
            option_count: options.len() as i16,
            options,
            order: 1,
            attempt_limit: 1,
            topic: "test-topic".to_string(),
            created_at: None,
            modified_at: None,
        }
    }

    fn make_quiz_with_questions(questions: Vec<QuizQuestion>) -> Quiz {
        Quiz {
            id: "quiz-1".to_string(),
            name: "Test Quiz".to_string(),
            created_by_user_id: "user-1".to_string(),
            title: None,
            description: None,
            question_count: questions.len() as i16,
            required_score: 1,
            attempt_limit: 3,
            topic: None,
            status: QuizStatus::Ready,
            questions: Some(questions),
            url: "https://example.com".to_string(),
            created_at: None,
            modified_at: None,
        }
    }

    #[test]
    fn grade_attempt_returns_point_for_correct_single_answer() {
        let question = make_question(
            "q1",
            QuizQuestionType::Single,
            vec![make_option("o1", true), make_option("o2", false)],
        );
        let quiz = make_quiz_with_questions(vec![question]);
        let submitted_answers = vec![QuestionAnswerInput {
            question_id: "q1".to_string(),
            selected_option_ids: vec!["o1".to_string()],
        }];

        let result = QuizAttemptService::grade_attempt(&quiz, &submitted_answers);

        assert!(result.is_ok());
        let (total_points, question_results) = result.expect("grading should succeed");
        assert_eq!(total_points, 1);
        assert_eq!(question_results.len(), 1);
        assert!(question_results[0].is_correct);
        assert_eq!(question_results[0].points_earned, 1);
    }

    #[test]
    fn grade_attempt_returns_zero_for_incorrect_single_answer() {
        let question = make_question(
            "q1",
            QuizQuestionType::Single,
            vec![make_option("o1", true), make_option("o2", false)],
        );
        let quiz = make_quiz_with_questions(vec![question]);
        let submitted_answers = vec![QuestionAnswerInput {
            question_id: "q1".to_string(),
            selected_option_ids: vec!["o2".to_string()],
        }];

        let result = QuizAttemptService::grade_attempt(&quiz, &submitted_answers);

        assert!(result.is_ok());
        let (total_points, question_results) = result.expect("grading should succeed");
        assert_eq!(total_points, 0);
        assert_eq!(question_results.len(), 1);
        assert!(!question_results[0].is_correct);
        assert_eq!(question_results[0].points_earned, 0);
    }

    #[test]
    fn grade_attempt_returns_zero_for_multi_partial_credit_scenario() {
        let question = make_question(
            "q1",
            QuizQuestionType::Multi,
            vec![
                make_option("o1", true),
                make_option("o2", true),
                make_option("o3", false),
            ],
        );
        let quiz = make_quiz_with_questions(vec![question]);
        let submitted_answers = vec![QuestionAnswerInput {
            question_id: "q1".to_string(),
            selected_option_ids: vec!["o1".to_string()],
        }];

        let result = QuizAttemptService::grade_attempt(&quiz, &submitted_answers);

        assert!(result.is_ok());
        let (total_points, question_results) = result.expect("grading should succeed");
        assert_eq!(total_points, 0);
        assert_eq!(question_results.len(), 1);
        assert!(!question_results[0].is_correct);
        assert_eq!(question_results[0].points_earned, 0);
    }

    #[test]
    fn grade_attempt_returns_error_for_unknown_question_id() {
        let question = make_question(
            "q1",
            QuizQuestionType::Single,
            vec![make_option("o1", true), make_option("o2", false)],
        );
        let quiz = make_quiz_with_questions(vec![question]);
        let submitted_answers = vec![QuestionAnswerInput {
            question_id: "missing-question".to_string(),
            selected_option_ids: vec!["o1".to_string()],
        }];

        let result = QuizAttemptService::grade_attempt(&quiz, &submitted_answers);

        assert!(result.is_err());
        match result.expect_err("expected bad request error") {
            AppError::BadRequest(msg) => assert!(msg.contains("Question 'missing-question' not found")),
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn grade_attempt_returns_error_for_unknown_option_id() {
        let question = make_question(
            "q1",
            QuizQuestionType::Single,
            vec![make_option("o1", true), make_option("o2", false)],
        );
        let quiz = make_quiz_with_questions(vec![question]);
        let submitted_answers = vec![QuestionAnswerInput {
            question_id: "q1".to_string(),
            selected_option_ids: vec!["missing-option".to_string()],
        }];

        let result = QuizAttemptService::grade_attempt(&quiz, &submitted_answers);

        assert!(result.is_err());
        match result.expect_err("expected bad request error") {
            AppError::BadRequest(msg) => assert!(msg.contains("Option 'missing-option' not found")),
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn grade_attempt_returns_error_when_quiz_has_no_questions() {
        let quiz = Quiz {
            id: "quiz-1".to_string(),
            name: "No Question Quiz".to_string(),
            created_by_user_id: "user-1".to_string(),
            title: None,
            description: None,
            question_count: 0,
            required_score: 1,
            attempt_limit: 1,
            topic: None,
            status: QuizStatus::Ready,
            questions: None,
            url: "https://example.com".to_string(),
            created_at: None,
            modified_at: None,
        };

        let submitted_answers = vec![QuestionAnswerInput {
            question_id: "q1".to_string(),
            selected_option_ids: vec!["o1".to_string()],
        }];

        let result = QuizAttemptService::grade_attempt(&quiz, &submitted_answers);

        assert!(result.is_err());
        match result.expect_err("expected bad request error") {
            AppError::BadRequest(msg) => assert_eq!(msg, "Quiz has no questions"),
            other => panic!("expected BadRequest, got {:?}", other),
        }
    }

    #[test]
    fn grade_attempt_handles_empty_submitted_answers() {
        let question = make_question(
            "q1",
            QuizQuestionType::Single,
            vec![make_option("o1", true), make_option("o2", false)],
        );
        let quiz = make_quiz_with_questions(vec![question]);

        let result = QuizAttemptService::grade_attempt(&quiz, &[]);

        assert!(result.is_ok());
        let (total_points, question_results) = result.expect("grading should succeed");
        assert_eq!(total_points, 0);
        assert!(question_results.is_empty());
    }
}
