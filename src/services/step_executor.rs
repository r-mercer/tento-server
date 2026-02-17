use crate::{
    app_state::AppState,
    models::{
        domain::{quiz_question::QuizQuestionOption, summary_document::SummaryDocument, Quiz},
        dto::request::{
            GenerateQuizRequestDto, QuizQuestionRequestDto, QuizRequestDto,
            SummaryDocumentRequestDto,
        },
    },
    services::agent_orchestrator_service::{AgentJob, JobStep},
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

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

        let quiz_dto = app_state
            .quiz_service
            .get_quiz(&quiz_id)
            .await
            .map_err(|e| format!("Failed to fetch quiz: {}", e))?;

        let quiz: Quiz = quiz_dto
            .try_into()
            .map_err(|e| format!("Failed to parse quiz: {}", e))?;

        match app_state.model_service.website_summariser(&quiz.url).await {
            Ok(summary_dto) => {
                log::info!(
                    "Successfully created summary document for job {}",
                    job.job_id
                );

                let now = Utc::now().to_rfc3339();
                let summary_request = SummaryDocumentRequestDto {
                    id: Uuid::new_v4().to_string(),
                    quiz_id: quiz.id.clone(),
                    url: quiz.url.clone(),
                    content: summary_dto,
                    created_at: now.clone(),
                    modified_at: now,
                };
                let new_doc: SummaryDocument = summary_request
                    .try_into()
                    .map_err(|e| format!("Failed to parse summary document: {}", e))?;
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

        let quiz_dto = app_state
            .quiz_service
            .get_quiz(&quiz_id)
            .await
            .map_err(|e| format!("Failed to fetch quiz: {}", e))?;

        let quiz: Quiz = quiz_dto
            .try_into()
            .map_err(|e| format!("Failed to parse quiz: {}", e))?;

        let summary_document = app_state
            .summary_document_service
            .get_summary_document(&summary_id)
            .await
            .map_err(|e| format!("Failed to fetch summary document: {}", e))?;

        let quiz_dto = crate::models::dto::quiz_dto::QuizDto::from(quiz);
        let quiz_request_dto = QuizRequestDto::from(quiz_dto);
        let summary_dto = SummaryDocumentRequestDto::from(summary_document);

        match app_state
            .model_service
            .structured_quiz_generator(quiz_request_dto, summary_dto)
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

        // let new_quiz_json = job.results.get("resonse");
        let mut response = String::new();
        if let Some(resp) = job.results.get("response").to_owned() {
            // println!("response: {}", resp.to_string());
            response = resp.to_string();
        }

        // let newval = job.results.get("response").to_owned();

        let generate_quiz_request_dto: GenerateQuizRequestDto = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse quiz from job results: {}", e))?;

        let quiz_dto = app_state
            .quiz_service
            .get_quiz(&quiz_id)
            .await
            .map_err(|e| format!("Failed to fetch quiz: {}", e))?;

        let mut quiz: Quiz = quiz_dto
            .try_into()
            .map_err(|e| format!("Failed to parse quiz: {}", e))?;

        let quiz_dto = crate::models::dto::quiz_dto::QuizDto::from(quiz.clone());
        let mut new_quiz_request = QuizRequestDto::from(quiz_dto);

        new_quiz_request.id = quiz.id.clone();
        new_quiz_request.name = quiz.name.clone();
        new_quiz_request.created_by_user_id = quiz.created_by_user_id.clone();
        new_quiz_request.question_count = quiz.question_count.to_string();
        new_quiz_request.required_score = quiz.required_score.to_string();
        new_quiz_request.attempt_limit = quiz.attempt_limit.to_string();
        new_quiz_request.status = format!("{:?}", quiz.status).to_lowercase();
        new_quiz_request.url = quiz.url.clone();
        new_quiz_request.created_at = quiz
            .created_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        new_quiz_request.modified_at = quiz
            .modified_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();

        let now = Utc::now().to_rfc3339();
        let generated_questions: Vec<QuizQuestionRequestDto> = generate_quiz_request_dto
            .questions
            .into_iter()
            .map(|question| {
                let options: Vec<QuizQuestionOption> = question
                    .options
                    .into_iter()
                    .map(|option| QuizQuestionOption {
                        id: Uuid::new_v4().to_string(),
                        text: option.text,
                        correct: option.correct.trim().eq_ignore_ascii_case("true"),
                        explanation: option.explanation,
                    })
                    .collect();
                let option_count = options.len() as i16;
                let options_json =
                    serde_json::to_string(&options).unwrap_or_else(|_| "[]".to_string());

                QuizQuestionRequestDto {
                    id: Uuid::new_v4().to_string(),
                    title: question.title,
                    description: question.description,
                    question_type: question.question_type,
                    options: options_json,
                    option_count: option_count.to_string(),
                    order: question.order,
                    attempt_limit: quiz.attempt_limit.to_string(),
                    topic: quiz.topic.clone().unwrap_or_default(),
                    created_at: now.clone(),
                    modified_at: now.clone(),
                }
            })
            .collect();

        new_quiz_request.questions = generated_questions;

        // for question in &mut generate_quiz_request_dto.questions {
        //     if question.attempt_limit.trim().is_empty() {
        //         question.attempt_limit = quiz.attempt_limit.to_string();
        //     }
        // }

        let new_quiz_dto: crate::models::dto::quiz_dto::QuizDto = new_quiz_request
            .try_into()
            .map_err(|e| format!("Failed to validate quiz from job results: {}", e))?;
        let new_quiz: Quiz = new_quiz_dto
            .try_into()
            .map_err(|e| format!("Failed to parse quiz: {}", e))?;

        quiz.status = crate::models::domain::quiz::QuizStatus::Ready;
        quiz.modified_at = Some(chrono::Utc::now());
        quiz.questions = new_quiz.questions;
        quiz.title = new_quiz.title;
        quiz.description = new_quiz.description;

        let updated_quiz = crate::models::dto::quiz_dto::QuizDto::from(quiz);
        app_state
            .quiz_service
            .update_quiz(updated_quiz)
            .await
            .map_err(|e| format!("Failed to update quiz: {}", e))?;

        log::info!(
            "Successfully finalized quiz {} for job {}",
            quiz_id,
            job.job_id
        );

        Ok(json!({
            "status": "quiz_finalized",
            "quiz_status": "ready"
        }))
    }
}
