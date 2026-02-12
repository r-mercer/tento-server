use crate::{
    app_state::AppState,
    models::domain::{summary_document::SummaryDocument, Quiz},
    services::agent_orchestrator_service::{AgentJob, JobStep},
};
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStepType {
    CreateQuizDraft,
    CreateSummaryDocument,
    CreateQuizQuestions,
    FinalizeQuiz,
}

impl JobStepType {

    pub fn from_step_name(name: &str) -> Option<Self> {
        match name {
            "create_quiz_draft" => Some(JobStepType::CreateQuizDraft),
            "create_summary_document" => Some(JobStepType::CreateSummaryDocument),
            "create_quiz_questions" => Some(JobStepType::CreateQuizQuestions),
            "finalize_quiz" => Some(JobStepType::FinalizeQuiz),
            _ => None,
        }
    }
}

pub trait StepExecutor: Send + Sync {
    fn execute_step(
        &self,
        step_type: JobStepType,
        step: &JobStep,
        job: &AgentJob,
    ) -> impl std::future::Future<Output = Result<serde_json::Value, String>> + Send;
}

pub struct StepHandler;

impl StepHandler {
    pub async fn execute(
        step_type: JobStepType,
        step: &JobStep,
        job: &AgentJob,
        app_state: &AppState,
    ) -> Result<serde_json::Value, String> {
        match step_type {
            JobStepType::CreateQuizDraft => {
                Self::handle_create_quiz_draft(step, job, app_state).await
            }
            JobStepType::CreateSummaryDocument => {
                Self::handle_create_summary_document(step, job, app_state).await
            }
            JobStepType::CreateQuizQuestions => {
                Self::handle_create_quiz_questions(step, job, app_state).await
            }
            JobStepType::FinalizeQuiz => Self::handle_finalize_quiz(step, job, app_state).await,
        }
    }

    async fn handle_create_quiz_draft(
        _step: &JobStep,
        job: &AgentJob,
        _app_state: &AppState,
    ) -> Result<serde_json::Value, String> {
        log::info!("Executing create_quiz_draft step for job {}", job.job_id);

        let quiz_id = job
            .results
            .get("quiz_id")
            .ok_or_else(|| "Quiz ID not found in job results".to_string())?;

        Ok(json!({
            "status": "quiz_draft_created",
            "quiz_id": quiz_id
        }))
    }

    async fn handle_create_summary_document(
        _step: &JobStep,
        job: &AgentJob,
        app_state: &AppState,
    ) -> Result<serde_json::Value, String> {
        log::info!(
            "Executing create_summary_document step for job {}",
            job.job_id
        );

        let quiz_id = job
            .results
            .get("quiz_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Invalid or missing quiz_id in job results".to_string())?
            .to_string();

        let quiz: Quiz = app_state
            .quiz_service
            .get_quiz(&quiz_id)
            .await
            .map_err(|e| format!("Failed to fetch quiz: {}", e))?;

        match app_state.model_service.website_summariser().await {
            Ok(summary) => {
                log::info!(
                    "Successfully created summary document for job {}",
                    job.job_id
                );

                let new_doc = SummaryDocument::new_summary_document(&quiz.url, &quiz.id, &summary);
                app_state
                    .summary_document_service
                    .create_summary_document(new_doc.clone())
                    .await
                    .map_err(|e| format!("Failed to save summary document: {}", e))?;
                Ok(json!({ "summary_id": new_doc.id }))
            }
            Err(e) => Err(format!("Failed to create summary: {}", e)),
        }
    }

    async fn handle_create_quiz_questions(
        _step: &JobStep,
        job: &AgentJob,
        app_state: &AppState,
    ) -> Result<serde_json::Value, String> {
        log::info!(
            "Executing create_quiz_questions step for job {}",
            job.job_id
        );

        let quiz_id = job
            .results
            .get("quiz_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Invalid or missing quiz_id in job results".to_string())?
            .to_string();

        let summary_id = job
            .results
            .get("summary_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Invalid or missing summary_id in job results".to_string())?
            .to_string();

        let quiz = app_state
            .quiz_service
            .get_quiz(&quiz_id)
            .await
            .map_err(|e| format!("Failed to fetch quiz: {}", e))?;

        let summary_document = app_state
            .summary_document_service
            .get_summary_document(&summary_id)
            .await
            .map_err(|e| format!("Failed to fetch summary document: {}", e))?;

        match app_state
            .model_service
            .quiz_generator(quiz, summary_document)
            .await
        {
            Ok(response) => {
                log::info!(
                    "Successfully generated quiz questions for job {}",
                    job.job_id
                );
                Ok(json!({
                    "status": "quiz_fields_generated",
                    "response": response
                }))
            }
            Err(e) => Err(format!("Failed to generate quiz questions: {}", e)),
        }
    }

    async fn handle_finalize_quiz(
        _step: &JobStep,
        job: &AgentJob,
        app_state: &AppState,
    ) -> Result<serde_json::Value, String> {
        log::info!("Executing finalize_quiz step for job {}", job.job_id);

        let quiz_id = job
            .results
            .get("quiz_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Invalid or missing quiz_id in job results".to_string())?
            .to_string();

        let mut quiz = app_state
            .quiz_service
            .get_quiz(&quiz_id)
            .await
            .map_err(|e| format!("Failed to fetch quiz: {}", e))?;

        quiz.status = crate::models::domain::quiz::QuizStatus::Ready;
        quiz.modified_at = Some(chrono::Utc::now());

        app_state
            .quiz_service
            .update_quiz(quiz)
            .await
            .map_err(|e| format!("Failed to update quiz: {}", e))?;

        log::info!("Successfully finalized quiz {} for job {}", quiz_id, job.job_id);

        Ok(json!({
            "status": "quiz_finalized",
            "quiz_status": "ready"
        }))
    }
}
