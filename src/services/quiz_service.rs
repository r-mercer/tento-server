use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    models::{
        domain::Quiz,
        dto::{
            request::CreateQuizDraftRequest,
            response::{CreateQuizDraftResponse, CreateQuizDraftResponseData, QuizDto},
        },
    },
    repositories::{QuizRepository, UserRepository},
    services::{agent_orchestrator_service::AgentOrchestrator, orchestrator_steps::create_quiz_generation_steps},
};

pub struct QuizService {
    repository: Arc<dyn QuizRepository>,
    user_repository: Arc<dyn UserRepository>,
    orchestrator: Arc<AgentOrchestrator>,
}

impl QuizService {
    pub fn new(repository: Arc<dyn QuizRepository>, user_repository: Arc<dyn UserRepository>, orchestrator: Arc<AgentOrchestrator>) -> Self {
        Self {
            repository,
            user_repository,
            orchestrator,
        }
    }

    pub async fn get_quiz(&self, id: &Uuid) -> AppResult<Quiz> {
        let quiz = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quiz with id '{}' not found", id)))?;

        Ok(quiz)
    }

    pub async fn create_quiz_draft(
        &self,
        request: CreateQuizDraftRequest,
        username: &str,
    ) -> AppResult<CreateQuizDraftResponse> {
        request.validate()?;

        // Look up the user to get their ID
        let user = self
            .user_repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User '{}' not found", username)))?;

        let user_id = user
            .id
            .ok_or_else(|| AppError::InternalError("User does not have an ID".to_string()))?;

        let quiz = Quiz::new_draft(
            &request.name,
            Uuid::nil(), // TODO: Get from authenticated user
            request.question_count,
            request.required_score,
            request.attempt_limit,
            &request.url,
        );

        let created_quiz = self.repository.create_quiz_draft(quiz).await?;

        let steps = create_quiz_generation_steps();
        let job_id = self
            .orchestrator
            .create_job(steps)
            .await
            .map_err(|e| AppError::InternalError(format!("Job creation failed: {}", e)))?;

        self.orchestrator
            .start_job(&job_id)
            .await
            .map_err(|e| AppError::InternalError(format!("Job startup failed: {}", e)))?;

        Ok(CreateQuizDraftResponse {
            data: CreateQuizDraftResponseData {
                quiz: QuizDto::from(created_quiz),
                job_id,
            },
            message: "Draft created successfully and processing started".to_string(),
        })
    }
}
