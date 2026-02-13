use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::app_state::AppState;
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
        Self {
            id: Some(job_id.clone()),
            job_id,
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

    pub async fn create_job(&self, steps: Vec<JobStep>) -> Result<String, String> {
        self.repository.create_job(steps).await
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

    pub async fn start_job(&self, job_id: &str) -> Result<(), String> {
        self.repository.start_job(job_id).await
    }

    pub async fn complete_step(
        &self,
        job_id: &str,
        result: Option<serde_json::Value>,
    ) -> Result<(), String> {
        self.repository.complete_step(job_id, result).await
    }

    pub async fn fail_step(&self, job_id: &str, error: String) -> Result<(), String> {
        self.repository.fail_step(job_id, error).await
    }

    pub async fn pause_job(&self, job_id: &str) -> Result<(), String> {
        self.repository.pause_job(job_id).await
    }

    pub async fn resume_job(&self, job_id: &str) -> Result<(), String> {
        self.repository.resume_job(job_id).await
    }

    pub async fn list_jobs(
        &self,
        status_filter: Option<JobStatus>,
    ) -> Result<Vec<AgentJob>, String> {
        self.repository.list_jobs(status_filter).await
    }

    pub async fn delete_job(&self, job_id: &str) -> Result<(), String> {
        self.repository.delete_job(job_id).await
    }

    pub async fn start_worker(&self) -> Result<(), String> {
        log::info!("Starting background worker");

        let repository = self.repository.clone();
        let app_state = self.app_state.clone();

        let worker_handle = tokio::spawn(async move {
            loop {
                if let Ok(jobs) = repository.list_jobs(Some(JobStatus::Running)).await {
                    for mut job in jobs {
                        let app_state_read = app_state.read().await;
                        let Some(app_state_ref) = app_state_read.as_ref() else {
                            log::warn!("App state not set for orchestrator");
                            drop(app_state_read);
                            continue;
                        };

                        let app_state_clone: Arc<AppState> = app_state_ref.clone();
                        drop(app_state_read);

                        if let Some(current_step) = job.get_current_step() {
                            let step_name = current_step.name.clone();
                            let step_id = current_step.id.clone();

                            if let Some(step_type) = JobStepType::from_step_name(&step_name) {
                                log::info!(
                                    "Processing job {} - step {} ({}, attempt {}/{})",
                                    job.job_id,
                                    step_id,
                                    step_name,
                                    current_step.retry_count + 1,
                                    current_step.max_retries + 1
                                );

                                // Execute the step
                                match StepHandler::execute(
                                    step_type,
                                    current_step,
                                    &job,
                                    &app_state_clone,
                                )
                                .await
                                {
                                    Ok(result) => {
                                        // Step succeeded - complete it
                                        if let Err(e) = repository
                                            .complete_step(&job.job_id, Some(result))
                                            .await
                                        {
                                            log::error!("Failed to complete step: {}", e);
                                        } else {
                                            log::info!(
                                                "Step {} completed for job {}",
                                                step_name,
                                                job.job_id
                                            );
                                        }
                                    }
                                    Err(error) => {
                                        log::error!(
                                            "Step {} failed for job {}: {}",
                                            step_name,
                                            job.job_id,
                                            error
                                        );

                                        if let Err(e) =
                                            repository.fail_step(&job.job_id, error).await
                                        {
                                            log::error!("Failed to mark step as failed: {}", e);
                                        }
                                    }
                                }
                            } else {
                                log::error!("Unknown step type: {}", step_name);
                                let error = format!("Unknown step type: {}", step_name);
                                let _ = repository.fail_step(&job.job_id, error).await;
                            }
                        } else {
                            log::info!(
                                "Job {} has no more steps - marking as completed",
                                job.job_id
                            );

                            job.status = JobStatus::Completed;
                            job.completed_at = Some(Utc::now());
                            if let Err(e) = repository.save(&job).await {
                                log::error!("Failed to save completed job: {}", e);
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
