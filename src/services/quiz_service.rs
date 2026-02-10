use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::domain::Quiz,
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

    pub async fn create_quiz() -> AppResult<Quiz> {
        request.validate()?;
    }
}
