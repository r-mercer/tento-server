use crate::{
    app_state::AppState,
    repositories::summary_document_respository,
    services::agent_orchestrator_service::{AgentJob, JobStep},
};
use serde_json::json;

/// Maps step names to their corresponding step types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStepType {
    CreateQuizDraft,
    CreateSummaryDocument,
    CreateQuizQuestions,
    FinalizeQuiz,
}

impl JobStepType {
    /// Parse a step name into a JobStepType
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

/// Trait for executing job steps
pub trait StepExecutor: Send + Sync {
    fn execute_step(
        &self,
        step_type: JobStepType,
        step: &JobStep,
        job: &AgentJob,
    ) -> impl std::future::Future<Output = Result<serde_json::Value, String>> + Send;
}

/// Handler for dispatching and executing job steps
pub struct StepHandler;

impl StepHandler {
    /// Execute a step and return its result
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

        // The quiz was already created when create_quiz_draft was called in QuizService
        // This handler just validates and stores the quiz_id in results
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

        // Get URL from quiz (would need quiz fetch first - placeholder for now)
        // For now, just call the model service's website summariser
        // TODO: Extract URL from quiz metadata
        match app_state.model_service.website_summariser().await {
            Ok(summary) => {
                log::info!(
                    "Successfully created summary document for job {}",
                    job.job_id
                );
                app_state
                    .summary_document_repository
                    .create_summary_document(summary.clone())
                    .await
                    .map_err(|e| format!("Failed to save summary document: {}", e))?;
                Ok(json!({ "summary": summary }))
            }
            Err(e) => Err(format!("Failed to create summary: {}", e)),
        }
    }

    async fn handle_create_quiz_questions(
        _step: &JobStep,
        job: &AgentJob,
        _app_state: &AppState,
    ) -> Result<serde_json::Value, String> {
        log::info!(
            "Executing create_quiz_questions step for job {}",
            job.job_id
        );

        // Extract quiz and summary from job results
        let _quiz_id = job
            .results
            .get("quiz_id")
            .ok_or_else(|| "Quiz ID not found in job results".to_string())?;

        let _summary = job
            .results
            .get("summary")
            .ok_or_else(|| "Summary not found in job results".to_string())?;

        // TODO: Call model_service.quiz_generator with quiz and summary
        // For now, placeholder implementation
        Ok(json!({
            "status": "quiz_fields_generated",
            "questions_generated": 0
        }))
    }

    async fn handle_finalize_quiz(
        _step: &JobStep,
        job: &AgentJob,
        _app_state: &AppState,
    ) -> Result<serde_json::Value, String> {
        log::info!("Executing finalize_quiz step for job {}", job.job_id);

        // Extract quiz_id from results
        let _quiz_id = job
            .results
            .get("quiz_id")
            .ok_or_else(|| "Quiz ID not found in job results".to_string())?;

        // TODO: Fetch quiz from database, update status from Draft to Ready, save back to database
        // For now, placeholder implementation
        Ok(json!({
            "status": "quiz_finalized",
            "quiz_status": "ready"
        }))
    }
}
