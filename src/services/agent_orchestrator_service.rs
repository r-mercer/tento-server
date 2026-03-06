use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::logging::JobWorkflowLog;
use crate::repositories::AgentJobRepository;
use crate::services::step_executor::{JobStepType, StepHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Running => write!(f, "running"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
            JobStatus::Paused => write!(f, "paused"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStep {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_seconds: Option<u64>,
}

impl JobStep {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            retry_count: 0,
            max_retries: 3,
            timeout_seconds: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentJob {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub job_id: String,
    pub correlation_id: String,
    pub status: JobStatus,
    pub steps: Vec<JobStep>,
    pub current_step_index: usize,
    pub results: std::collections::HashMap<String, serde_json::Value>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retries_remaining: u32,
}

impl AgentJob {
    pub fn new(steps: Vec<JobStep>) -> Self {
        let job_id = Uuid::new_v4().to_string();
        let correlation_id = Uuid::new_v4().to_string();
        Self {
            id: Some(job_id.clone()),
            job_id,
            correlation_id,
            status: JobStatus::Pending,
            steps,
            current_step_index: 0,
            results: std::collections::HashMap::new(),
            error_message: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retries_remaining: 3,
        }
    }

    pub fn new_with_ids(steps: Vec<JobStep>, job_id: String, correlation_id: String) -> Self {
        Self {
            id: Some(job_id.clone()),
            job_id,
            correlation_id,
            status: JobStatus::Pending,
            steps,
            current_step_index: 0,
            results: std::collections::HashMap::new(),
            error_message: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retries_remaining: 3,
        }
    }

    pub fn get_current_step(&self) -> Option<&JobStep> {
        self.steps.get(self.current_step_index)
    }

    pub fn is_complete(&self) -> bool {
        self.current_step_index >= self.steps.len()
    }
}

/// Orchestrator service for managing agent jobs with background worker
pub struct AgentOrchestrator {
    repository: Arc<dyn AgentJobRepository>,
    worker_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    app_state: Arc<RwLock<Option<Arc<AppState>>>>,
}

impl AgentOrchestrator {
    /// Create a new orchestrator with a job repository
    pub fn new(repository: Arc<dyn AgentJobRepository>) -> Self {
        Self {
            repository,
            worker_handle: Arc::new(RwLock::new(None)),
            app_state: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the app state for the orchestrator (called during app initialization)
    pub async fn set_app_state(&self, app_state: Arc<AppState>) {
        let mut state = self.app_state.write().await;
        *state = Some(app_state);
    }

    pub async fn create_job(&self, steps: Vec<JobStep>) -> Result<(String, String), String> {
        let job_id = Uuid::new_v4().to_string();
        let correlation_id = Uuid::new_v4().to_string();
        let step_count = steps.len();

        JobWorkflowLog::new("create_job")
            .with_correlation_id(&correlation_id)
            .with_job_id(&job_id)
            .with_success(true)
            .log_start();

        log::info!(
            target: "agent_orchestrator",
            "[{}] Job created: {} with {} steps",
            correlation_id,
            job_id,
            step_count
        );

        let steps_with_job_ids = steps
            .into_iter()
            .map(|mut step| {
                if step.id.is_empty() {
                    step.id = Uuid::new_v4().to_string();
                }
                step
            })
            .collect();

        self.repository
            .create_job(steps_with_job_ids, &job_id, &correlation_id)
            .await?;
        Ok((job_id, correlation_id))
    }

    pub async fn set_job_metadata(
        &self,
        job_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), String> {
        let mut job = self
            .repository
            .get_job(job_id)
            .await?
            .ok_or_else(|| format!("Job {} not found", job_id))?;
        job.results.insert(key.to_string(), value);
        self.repository.save(&job).await
    }

    pub async fn get_job(&self, job_id: &str) -> Result<Option<AgentJob>, String> {
        self.repository.get_job(job_id).await
    }

    pub async fn get_job_status(&self, job_id: &str) -> Result<Option<JobStatus>, String> {
        self.repository.get_job_status(job_id).await
    }

    pub async fn start_job(
        &self,
        job_id: &str,
        correlation_id: Option<&str>,
    ) -> Result<(), String> {
        if let Some(cid) = correlation_id {
            log::info!(
                target: "agent_orchestrator",
                "[{}] Starting job: {}",
                cid,
                job_id
            );
        }
        self.repository.start_job(job_id).await
    }

    pub async fn complete_step(
        &self,
        job_id: &str,
        result: Option<serde_json::Value>,
        correlation_id: Option<&str>,
        step_name: Option<&str>,
    ) -> Result<(), String> {
        if let Some(cid) = correlation_id {
            let step = step_name.unwrap_or("unknown");
            log::info!(
                target: "agent_orchestrator",
                "[{}] Step completed for job: {}, step: {}",
                cid,
                job_id,
                step
            );
        }
        self.repository.complete_step(job_id, result).await
    }

    pub async fn fail_step(
        &self,
        job_id: &str,
        error: String,
        correlation_id: Option<&str>,
        step_name: Option<&str>,
        retry_count: Option<u32>,
        max_retries: Option<u32>,
    ) -> Result<(), String> {
        if let Some(cid) = correlation_id {
            let step = step_name.unwrap_or("unknown");
            let retries = retry_count.unwrap_or(0);
            let max = max_retries.unwrap_or(0);

            if retries >= max {
                log::error!(
                    target: "agent_orchestrator",
                    "[{}] Step failed permanently for job: {}, step: {}, error: {}",
                    cid,
                    job_id,
                    step,
                    error
                );
            } else {
                log::warn!(
                    target: "agent_orchestrator",
                    "[{}] Step failed for job: {}, step: {}, attempt: {}/{}, error: {}",
                    cid,
                    job_id,
                    step,
                    retries + 1,
                    max + 1,
                    error
                );
            }
        }
        self.repository.fail_step(job_id, error).await
    }

    pub async fn pause_job(
        &self,
        job_id: &str,
        correlation_id: Option<&str>,
    ) -> Result<(), String> {
        if let Some(cid) = correlation_id {
            log::info!(
                target: "agent_orchestrator",
                "[{}] Pausing job: {}",
                cid,
                job_id
            );
        }
        self.repository.pause_job(job_id).await
    }

    pub async fn resume_job(
        &self,
        job_id: &str,
        correlation_id: Option<&str>,
    ) -> Result<(), String> {
        if let Some(cid) = correlation_id {
            log::info!(
                target: "agent_orchestrator",
                "[{}] Resuming job: {}",
                cid,
                job_id
            );
        }
        self.repository.resume_job(job_id).await
    }

    pub async fn list_jobs(
        &self,
        status_filter: Option<JobStatus>,
    ) -> Result<Vec<AgentJob>, String> {
        if let Some(status) = status_filter {
            log::debug!(
                target: "agent_orchestrator",
                "Listing jobs with status: {:?}",
                status
            );
        }
        self.repository.list_jobs(status_filter).await
    }

    pub async fn delete_job(
        &self,
        job_id: &str,
        correlation_id: Option<&str>,
    ) -> Result<(), String> {
        if let Some(cid) = correlation_id {
            log::info!(
                target: "agent_orchestrator",
                "[{}] Deleting job: {}",
                cid,
                job_id
            );
        }
        self.repository.delete_job(job_id).await
    }

    pub async fn start_worker(&self) -> Result<(), String> {
        log::info!(target: "agent_orchestrator", "Starting background worker");

        let repository = self.repository.clone();
        let app_state = self.app_state.clone();

        let worker_handle = tokio::spawn(async move {
            loop {
                if let Ok(jobs) = repository.list_jobs(Some(JobStatus::Running)).await {
                    for mut job in jobs {
                        let app_state_read = app_state.read().await;
                        let Some(app_state_ref) = app_state_read.as_ref() else {
                            log::warn!(target: "agent_orchestrator", "App state not set for orchestrator");
                            drop(app_state_read);
                            continue;
                        };

                        let app_state_clone: Arc<AppState> = app_state_ref.clone();
                        drop(app_state_read);

                        let correlation_id = job.correlation_id.clone();
                        let job_id = job.job_id.clone();

                        if let Some(current_step) = job.get_current_step() {
                            let step_name = current_step.name.clone();
                            let step_id = current_step.id.clone();
                            let attempt = current_step.retry_count + 1;
                            let max_attempts = current_step.max_retries + 1;
                            let step_index = job.current_step_index;

                            if let Some(step_type) = JobStepType::from_step_name(&step_name) {
                                // Log step start with structured format
                                JobWorkflowLog::new("execute_step")
                                    .with_correlation_id(&correlation_id)
                                    .with_job_id(&job_id)
                                    .with_step_name(&step_name)
                                    .with_step_index(step_index)
                                    .with_attempt(attempt)
                                    .log_step_start();

                                log::info!(
                                    target: "agent_orchestrator",
                                    "[{}] Processing job {} - step {} ({}, attempt {}/{})",
                                    correlation_id,
                                    job_id,
                                    step_id,
                                    step_name,
                                    attempt,
                                    max_attempts
                                );

                                // Execute the step
                                let start_time = std::time::Instant::now();
                                match StepHandler::execute(
                                    step_type,
                                    current_step,
                                    &job,
                                    &app_state_clone,
                                )
                                .await
                                {
                                    Ok(result) => {
                                        let duration_ms = start_time.elapsed().as_millis() as u64;

                                        if let Err(e) = repository
                                            .complete_step(&job.job_id, Some(result))
                                            .await
                                        {
                                            log::error!(
                                                target: "agent_orchestrator",
                                                "[{}] Failed to complete step for job {}: {}",
                                                correlation_id,
                                                job_id,
                                                e
                                            );
                                        } else {
                                            JobWorkflowLog::new("execute_step")
                                                .with_correlation_id(&correlation_id)
                                                .with_job_id(&job_id)
                                                .with_step_name(&step_name)
                                                .with_step_index(step_index)
                                                .with_attempt(attempt)
                                                .with_success(true)
                                                .with_duration_ms(duration_ms)
                                                .log_step_complete();

                                            log::info!(
                                                target: "agent_orchestrator",
                                                "[{}] Step {} completed for job {} ({}ms)",
                                                correlation_id,
                                                step_name,
                                                job_id,
                                                duration_ms
                                            );
                                        }
                                    }
                                    Err(error) => {
                                        let duration_ms = start_time.elapsed().as_millis() as u64;
                                        let retry_count = current_step.retry_count;
                                        let max_retries = current_step.max_retries;

                                        log::error!(
                                            target: "agent_orchestrator",
                                            "[{}] Step {} failed for job {}: {}",
                                            correlation_id,
                                            step_name,
                                            job_id,
                                            error
                                        );

                                        if let Err(e) =
                                            repository.fail_step(&job.job_id, error.clone()).await
                                        {
                                            log::error!(
                                                target: "agent_orchestrator",
                                                "[{}] Failed to mark step as failed for job {}: {}",
                                                correlation_id,
                                                job_id,
                                                e
                                            );
                                        } else {
                                            JobWorkflowLog::new("execute_step")
                                                .with_correlation_id(&correlation_id)
                                                .with_job_id(&job_id)
                                                .with_step_name(&step_name)
                                                .with_step_index(step_index)
                                                .with_attempt(attempt)
                                                .with_success(false)
                                                .with_error(&error)
                                                .with_duration_ms(duration_ms)
                                                .log_step_complete();

                                            log::warn!(
                                                target: "agent_orchestrator",
                                                "[{}] Step {} failed for job {}, attempt {}/{}, error: {}",
                                                correlation_id,
                                                step_name,
                                                job_id,
                                                retry_count + 1,
                                                max_retries + 1,
                                                error
                                            );
                                        }
                                    }
                                }
                            } else {
                                log::error!(
                                    target: "agent_orchestrator",
                                    "[{}] Unknown step type: {} for job {}",
                                    correlation_id,
                                    step_name,
                                    job_id
                                );
                                let error = format!("Unknown step type: {}", step_name);
                                let _ = repository.fail_step(&job.job_id, error).await;
                            }
                        } else {
                            log::info!(
                                target: "agent_orchestrator",
                                "[{}] Job {} has no more steps - marking as completed",
                                correlation_id,
                                job_id
                            );

                            job.status = JobStatus::Completed;
                            job.completed_at = Some(Utc::now());
                            if let Err(e) = repository.save(&job).await {
                                log::error!(
                                    target: "agent_orchestrator",
                                    "[{}] Failed to save completed job {}: {}",
                                    correlation_id,
                                    job_id,
                                    e
                                );
                            } else {
                                JobWorkflowLog::new("job_complete")
                                    .with_correlation_id(&correlation_id)
                                    .with_job_id(&job_id)
                                    .with_success(true)
                                    .log_complete();
                            }
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        let mut handle = self.worker_handle.write().await;
        *handle = Some(worker_handle);

        Ok(())
    }

    pub async fn stop_worker(&self) -> Result<(), String> {
        let mut handle = self.worker_handle.write().await;
        if let Some(join_handle) = handle.take() {
            join_handle.abort();
        }
        Ok(())
    }
}
