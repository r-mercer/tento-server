use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    models::{
        domain::Quiz,
        dto::{
            request::CreateQuizDraftRequest,
            response::{CreateQuizDraftResponse, QuizDto},
        },
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

    // let user = User::from_request(request);
    // let created_user = self.repository.create(user).await?;
    //
    // Ok(CreateUserResponse {
    //     data: UserDto::from(created_user),
    //     message: "User created successfully".to_string(),
    // })
    pub async fn create_quiz_draft(
        &self,
        request: CreateQuizDraftRequest,
    ) -> AppResult<CreateQuizDraftResponse> {
        request.validate()?;

        // possibly include if URL has summary or similar if we get there

        let quiz = Quiz::new_draft(
            &request.name,
            request.question_count,
            request.required_score,
            request.attempt_limit,
            &request.url,
        );

        let created_quiz = self.repository.create_quiz_draft(quiz).await?;
        Ok(CreateQuizDraftResponse {
            data: QuizDto::from(created_quiz),
            message: "Draft created successfully".to_string(),
        })
    }
}
