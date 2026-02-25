use std::sync::Arc;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    models::{
        domain::{Quiz, QuizQuestion},
        dto::{
            quiz_dto::QuizDto,
            request::{QuizDraftDto, UpdateQuizInput},
            response::{CreateQuizDraftResponse, CreateQuizDraftResponseData, QuizResponseDto},
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

    pub async fn list_quizzes_by_user(
        &self,
        user_id: &str,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<QuizDto>, i64)> {
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
                quiz: QuizResponseDto::from(created_quiz),
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

    pub async fn update_quiz_partial(&self, input: UpdateQuizInput) -> AppResult<QuizDto> {
        let existing_quiz = self
            .repository
            .find_by_id(&input.id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Quiz with id '{}' not found", input.id)))?;

        let mut quiz = existing_quiz;
        let now = chrono::Utc::now();

        if let Some(title) = input.title {
            quiz.title = Some(title);
        }
        if let Some(description) = input.description {
            quiz.description = Some(description);
        }

        if let Some(questions_input) = input.questions {
            let merged_questions = merge_questions(&quiz, questions_input)?;
            quiz.questions = Some(merged_questions);
        }

        quiz.modified_at = Some(now);

        let updated_quiz = self.repository.update(quiz).await?;
        Ok(QuizDto::from(updated_quiz))
    }
}

fn merge_questions(
    existing: &Quiz,
    updates: Vec<crate::models::dto::request::UpdateQuizQuestionInput>,
) -> AppResult<Vec<QuizQuestion>> {
    let existing_questions = existing.questions.as_ref().ok_or_else(|| {
        AppError::ValidationError(
            "Cannot update questions on quiz without existing questions".to_string(),
        )
    })?;

    let mut result = Vec::new();
    for update in updates {
        if let Some(existing_question) = existing_questions.iter().find(|q| q.id == update.id) {
            let mut merged = existing_question.clone();
            if let Some(title) = update.title {
                merged.title = title;
            }
            if let Some(description) = update.description {
                merged.description = description;
            }
            if let Some(options_input) = update.options {
                let merged_options = merge_options(existing_question, options_input)?;
                merged.options = merged_options;
            }
            merged.modified_at = Some(chrono::Utc::now());
            result.push(merged);
        } else {
            return Err(AppError::NotFound(format!(
                "Question with id '{}' not found",
                update.id
            )));
        }
    }

    Ok(result)
}

fn merge_options(
    existing_question: &QuizQuestion,
    updates: Vec<crate::models::dto::request::UpdateQuizQuestionOptionInput>,
) -> AppResult<Vec<crate::models::domain::quiz_question::QuizQuestionOption>> {
    let mut result = Vec::new();
    for update in updates {
        if let Some(existing_option) = existing_question.options.iter().find(|o| o.id == update.id)
        {
            let mut merged = existing_option.clone();
            if let Some(text) = update.text {
                merged.text = text;
            }
            if let Some(correct) = update.correct {
                merged.correct = correct;
            }
            if let Some(explanation) = update.explanation {
                merged.explanation = explanation;
            }
            result.push(merged);
        } else {
            return Err(AppError::NotFound(format!(
                "Option with id '{}' not found",
                update.id
            )));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;
    use std::collections::HashMap;

    use crate::{
        models::dto::request::QuizDraftDto,
        repositories::AgentJobRepository,
        services::agent_orchestrator_service::{AgentJob, JobStatus, JobStep},
    };

    use super::*;

    mock! {
        pub QuizRepo {}

        #[async_trait]
        impl QuizRepository for QuizRepo {
            async fn find_by_id(&self, id: &str) -> AppResult<Option<Quiz>>;
            async fn list_quizzes(&self, offset: i64, limit: i64) -> AppResult<(Vec<Quiz>, i64)>;
            async fn list_quizzes_by_user(&self, user_id: &str, offset: i64, limit: i64) -> AppResult<(Vec<Quiz>, i64)>;
            async fn get_by_status_by_id(&self, id: &str, status: &str) -> AppResult<Option<Quiz>>;
            async fn create_quiz_draft(&self, quiz: Quiz) -> AppResult<Quiz>;
            async fn update(&self, quiz: Quiz) -> AppResult<Quiz>;
        }
    }

    mock! {
        pub AgentJobRepo {}

        #[async_trait]
        impl AgentJobRepository for AgentJobRepo {
            async fn create_job(&self, steps: Vec<JobStep>) -> Result<String, String>;
            async fn get_job(&self, job_id: &str) -> Result<Option<AgentJob>, String>;
            async fn get_job_status(&self, job_id: &str) -> Result<Option<JobStatus>, String>;
            async fn start_job(&self, job_id: &str) -> Result<(), String>;
            async fn complete_step(&self, job_id: &str, result: Option<serde_json::Value>) -> Result<(), String>;
            async fn fail_step(&self, job_id: &str, error: String) -> Result<(), String>;
            async fn pause_job(&self, job_id: &str) -> Result<(), String>;
            async fn resume_job(&self, job_id: &str) -> Result<(), String>;
            async fn list_jobs(&self, status_filter: Option<JobStatus>) -> Result<Vec<AgentJob>, String>;
            async fn delete_job(&self, job_id: &str) -> Result<(), String>;
            async fn save(&self, job: &AgentJob) -> Result<(), String>;
        }
    }

    fn create_service(mock_repo: MockQuizRepo, mock_job_repo: MockAgentJobRepo) -> QuizService {
        let orchestrator = AgentOrchestrator::new(Arc::new(mock_job_repo));
        QuizService::new(Arc::new(mock_repo), Arc::new(orchestrator))
    }

    fn make_test_quiz(name: &str, created_by_user_id: &str) -> Quiz {
        Quiz::new_draft(name, created_by_user_id, 5, 70, 3, "https://example.com")
    }

    #[tokio::test]
    async fn get_quiz_returns_not_found_for_invalid_id() {
        let mut mock_repo = MockQuizRepo::new();
        let mock_job_repo = MockAgentJobRepo::new();

        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let service = create_service(mock_repo, mock_job_repo);
        let result = service.get_quiz("missing-id").await;

        assert!(result.is_err());
        match result.expect_err("expected not found") {
            AppError::NotFound(msg) => assert!(msg.contains("Quiz with id 'missing-id' not found")),
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn list_quizzes_returns_paginated_results() {
        let mut mock_repo = MockQuizRepo::new();
        let mock_job_repo = MockAgentJobRepo::new();

        mock_repo.expect_list_quizzes().returning(|offset, limit| {
            assert_eq!(offset, 10);
            assert_eq!(limit, 5);
            Ok((
                vec![
                    make_test_quiz("quiz-1", "user-a"),
                    make_test_quiz("quiz-2", "user-b"),
                ],
                12,
            ))
        });

        let service = create_service(mock_repo, mock_job_repo);
        let result = service.list_quizzes(10, 5).await.expect("expected success");

        let (quizzes, total) = result;
        assert_eq!(total, 12);
        assert_eq!(quizzes.len(), 2);
        assert_eq!(quizzes[0].name, "quiz-1");
        assert_eq!(quizzes[1].name, "quiz-2");
    }

    #[tokio::test]
    async fn list_quizzes_by_user_returns_user_filtered_results() {
        let mut mock_repo = MockQuizRepo::new();
        let mock_job_repo = MockAgentJobRepo::new();

        mock_repo
            .expect_list_quizzes_by_user()
            .returning(|user_id, offset, limit| {
                assert_eq!(user_id, "user-123");
                assert_eq!(offset, 0);
                assert_eq!(limit, 20);
                Ok((vec![make_test_quiz("user-quiz", "user-123")], 1))
            });

        let service = create_service(mock_repo, mock_job_repo);
        let result = service
            .list_quizzes_by_user("user-123", 0, 20)
            .await
            .expect("expected success");

        let (quizzes, total) = result;
        assert_eq!(total, 1);
        assert_eq!(quizzes.len(), 1);
        assert_eq!(quizzes[0].created_by_user_id, "user-123");
    }

    #[tokio::test]
    async fn create_quiz_draft_persists_quiz_and_starts_job_flow() {
        let mut mock_repo = MockQuizRepo::new();
        let mut mock_job_repo = MockAgentJobRepo::new();

        mock_repo.expect_create_quiz_draft().returning(|quiz| Ok(quiz));

        mock_job_repo.expect_create_job().returning(|steps| {
            assert!(!steps.is_empty());
            Ok("job-123".to_string())
        });

        mock_job_repo.expect_get_job().returning(|job_id| {
            assert_eq!(job_id, "job-123");
            Ok(Some(AgentJob {
                id: Some("job-123".to_string()),
                job_id: "job-123".to_string(),
                status: JobStatus::Pending,
                steps: vec![JobStep::new("extract_content")],
                current_step_index: 0,
                results: HashMap::new(),
                error_message: None,
                created_at: Utc::now(),
                started_at: None,
                completed_at: None,
                retries_remaining: 3,
            }))
        });

        mock_job_repo.expect_save().returning(|job| {
            assert_eq!(job.job_id, "job-123");
            assert!(job.results.contains_key("quiz_id"));
            Ok(())
        });

        mock_job_repo.expect_start_job().returning(|job_id| {
            assert_eq!(job_id, "job-123");
            Ok(())
        });

        let service = create_service(mock_repo, mock_job_repo);

        let request = QuizDraftDto {
            name: "Draft Quiz".to_string(),
            question_count: 8,
            required_score: 75,
            attempt_limit: 3,
            url: "https://example.com/learning".to_string(),
        };

        let result = service
            .create_quiz_draft(request, "user-abc")
            .await
            .expect("expected draft creation to succeed");

        assert_eq!(result.data.job_id, "job-123");
        assert_eq!(result.data.quiz.name, "Draft Quiz");
        assert_eq!(result.data.quiz.created_by_user_id, "user-abc");
        assert_eq!(result.message, "Draft created successfully and processing started");
    }
}
