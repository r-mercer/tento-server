use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    models::{
        domain::Quiz,
        dto::{request::CreateQuizDraftRequest, response::CreateQuizDraftResponse},
    },
    repositories::QuizRepository,
};

pub struct QuizService {
    repository: Arc<dyn QuizRepository>,
}

impl QuizService {
    pub fn new(repository: Arc<dyn QuizRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_quiz(&self, id: &Uuid) -> AppResult<Quiz> {
        let quiz = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quiz with id '{}' not found", id)))?;

        Ok(quiz)
    }

    pub async fn create_quiz_draft(&self, request: CreateQuizDraftRequest) -> AppResult<Quiz> {
        request.validate()?;

        let quiz = Quiz::new_draft(
            &request.name,
            request.question_count,
            request.required_score,
            request.attempt_limit,
            &request.url,
        );
        Ok(quiz)
    }
}
