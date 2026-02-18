use std::sync::Arc;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    models::{
        domain::Quiz,
        dto::{
            quiz_dto::QuizDto,
            request::QuizDraftDto,
            response::{
                CreateQuizDraftResponse, CreateQuizDraftResponseData, QuizDto as ResponseQuizDto,
            },
        },
    },
    repositories::QuizRepository,
    services::{
        agent_orchestrator_service::AgentOrchestrator,
        orchestrator_steps::create_quiz_generation_steps,
    },
};

pub struct QuizService {
    repository: Arc<dyn QuizRepository>,
    orchestrator: Arc<AgentOrchestrator>,
}

impl QuizService {
    pub fn new(repository: Arc<dyn QuizRepository>, orchestrator: Arc<AgentOrchestrator>) -> Self {
        Self {
            repository,
            orchestrator,
        }
    }

    pub async fn get_quiz(&self, id: &str) -> AppResult<QuizDto> {
        let quiz = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quiz with id '{}' not found", id)))?;

        Ok(QuizDto::from(quiz))
    }

    pub async fn list_quizzes(&self, offset: i64, limit: i64) -> AppResult<(Vec<QuizDto>, i64)> {
        let (quizzes, total) = self.repository.list_quizzes(offset, limit).await?;
        let dtos = quizzes.into_iter().map(QuizDto::from).collect();
        Ok((dtos, total))
    }

    pub async fn list_quizzes_by_user(&self, user_id: &str, offset: i64, limit: i64) -> AppResult<(Vec<QuizDto>, i64)> {
        let (quizzes, total) = self
            .repository
            .list_quizzes_by_user(user_id, offset, limit)
            .await?;

        let dtos = quizzes.into_iter().map(QuizDto::from).collect();
        Ok((dtos, total))
    }

    pub async fn get_quiz_draft(&self, id: &str) -> AppResult<QuizDraftDto> {
        let quiz: Quiz = self
            .repository
            .get_by_status_by_id(id, "draft")
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quiz draft with id '{}' not found", id)))?;

        Ok(QuizDraftDto::from_quiz(quiz))
    }

    pub async fn create_quiz_draft(
        &self,
        request: QuizDraftDto,
        user_id: &str,
    ) -> AppResult<CreateQuizDraftResponse> {
        request.validate()?;

        let quiz = Quiz::new_draft(
            &request.name,
            user_id,
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

        // Store quiz metadata in job
        self.orchestrator
            .set_job_metadata(
                &job_id,
                "quiz_id",
                serde_json::json!(created_quiz.id.to_string()),
            )
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to set job metadata: {}", e)))?;

        self.orchestrator
            .start_job(&job_id)
            .await
            .map_err(|e| AppError::InternalError(format!("Job startup failed: {}", e)))?;

        Ok(CreateQuizDraftResponse {
            data: CreateQuizDraftResponseData {
                quiz: ResponseQuizDto::from(created_quiz),
                job_id,
            },
            message: "Draft created successfully and processing started".to_string(),
        })
    }

    pub async fn update_quiz(&self, quiz: QuizDto) -> AppResult<QuizDto> {
        let mut quiz: Quiz = quiz.try_into()?;
        let now = chrono::Utc::now();
        if quiz.created_at.is_none() {
            quiz.created_at = Some(now);
        }
        if quiz.modified_at.is_none() {
            quiz.modified_at = Some(now);
        }

        let updated_quiz = self.repository.update(quiz).await?;
        Ok(QuizDto::from(updated_quiz))
    }
}
