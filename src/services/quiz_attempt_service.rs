use crate::errors::{AppError, AppResult};
use crate::models::domain::quiz_attempt::{QuizAttempt, QuizAttemptQuestion};
use crate::models::domain::quiz_question::QuizQuestionType;
use crate::models::domain::{Quiz, QuizQuestion};
use crate::models::dto::request::QuestionAnswerInput;
use chrono::Utc;
use uuid::Uuid;

pub struct QuizAttemptService;

impl QuizAttemptService {
    /// Grade a quiz attempt based on submitted answers
    pub fn grade_attempt(
        quiz: &Quiz,
        submitted_answers: &[QuestionAnswerInput],
    ) -> AppResult<(i16, Vec<QuizAttemptQuestion>)> {
        let mut total_points: i16 = 0;
        let mut question_results = Vec::new();

        // Get questions from quiz
        let questions = quiz
            .questions
            .as_ref()
            .ok_or(AppError::BadRequest("Quiz has no questions".to_string()))?;

        // Create a map of questions by ID for quick lookup
        let question_map: std::collections::HashMap<Uuid, &QuizQuestion> =
            questions.iter().map(|q| (q.id, q)).collect();

        for submitted_answer in submitted_answers {
            let question_id = Uuid::parse_str(&submitted_answer.question_id)
                .map_err(|_| AppError::BadRequest("Invalid question ID format".to_string()))?;

            let selected_ids: Result<Vec<Uuid>, _> = submitted_answer
                .selected_option_ids
                .iter()
                .map(|id| Uuid::parse_str(id))
                .collect();

            let selected_ids = selected_ids
                .map_err(|_| AppError::BadRequest("Invalid option ID format".to_string()))?;

            // Find question
            let question = question_map
                .get(&question_id)
                .ok_or(AppError::NotFound("Question not found".to_string()))?;

            // Grade this question
            let (is_correct, points) = Self::grade_question(question, selected_ids.clone())?;

            total_points += points;

            question_results.push(QuizAttemptQuestion {
                id: Uuid::new_v4(),
                quiz_question_id: question_id,
                selected_option_ids: selected_ids,
                is_correct,
                points_earned: points,
            });
        }

        Ok((total_points, question_results))
    }

    /// Grade an individual question based on type
    fn grade_question(
        question: &QuizQuestion,
        selected_option_ids: Vec<Uuid>,
    ) -> AppResult<(bool, i16)> {
        let correct_option_ids: Vec<Uuid> = question
            .options
            .iter()
            .filter(|opt| opt.correct)
            .map(|opt| opt.id)
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
                    .all(|id| selected_option_ids.contains(id));
                let has_no_incorrect = selected_option_ids
                    .iter()
                    .all(|id| correct_option_ids.contains(id));
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
        user_id: Uuid,
        quiz_id: Uuid,
        points_earned: i16,
        total_possible: i16,
        attempt_number: i16,
        required_score: i16,
        question_answers: Vec<QuizAttemptQuestion>,
    ) -> QuizAttempt {
        let passed = points_earned >= required_score;

        QuizAttempt {
            id: Uuid::new_v4(),
            user_id,
            quiz_id,
            points_earned,
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
