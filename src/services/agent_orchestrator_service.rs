use mongodb::{bson::doc, Client, Collection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Job status stored in MongoDB
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

/// Represents a job step to be executed sequentially
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

/// Job document stored in MongoDB
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

/// MongoDB-backed job orchestrator with background worker
pub struct AgentOrchestrator {
    db: Database,
    collection: Collection<AgentJob>,
    worker_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl AgentOrchestrator {
    /// Create a new orchestrator with MongoDB connection
    pub async fn new(db: Database) -> Result<Self, mongodb::error::Error> {
        let collection: Collection<AgentJob> = db.collection("jobs");

        // Create index on job_id for faster lookups
        collection
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(doc! { "job_id": 1 })
                    .build(),
                None,
            )
            .await?;

        // Create index on status for worker polling
        collection
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(doc! { "status": 1 })
                    .build(),
                None,
            )
            .await?;

        Ok(Self {
            db,
            collection,
            worker_handle: Arc::new(RwLock::new(None)),
        })
    }

    /// Create a new job from steps
    pub async fn create_job(&self, steps: Vec<JobStep>) -> Result<String, mongodb::error::Error> {
        let job = AgentJob::new(steps);
        let job_id = job.job_id.clone();

        self.collection.insert_one(&job, None).await?;

        Ok(job_id)
    }

    /// Get job by ID
    pub async fn get_job(&self, job_id: &str) -> Result<Option<AgentJob>, mongodb::error::Error> {
        self.collection
            .find_one(doc! { "job_id": job_id }, None)
            .await
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: &str) -> Result<Option<JobStatus>, mongodb::error::Error> {
        let job = self.get_job(job_id).await?;
        Ok(job.map(|j| j.status))
    }

    /// Start executing a job
    pub async fn start_job(&self, job_id: &str) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await
            .map_err(|e| format!("Failed to fetch job: {}", e))?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Pending {
            return Err(format!("Job is already {}", job.status));
        }

        self.collection
            .update_one(
                doc! { "job_id": job_id },
                doc! {
                    "$set": {
                        "status": "running",
                        "started_at": Utc::now()
                    }
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    /// Complete current step and advance to next
    pub async fn complete_step(
        &self,
        job_id: &str,
        result: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await
            .map_err(|e| format!("Failed to fetch job: {}", e))?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Running {
            return Err("Job is not running".to_string());
        }

        let mut updated_job = job.clone();
        
        // Store result if provided
        if let (Some(step), Some(result_value)) = (updated_job.get_current_step(), result) {
            updated_job.results.insert(step.id.clone(), result_value);
        }

        // Move to next step
        updated_job.current_step_index += 1;

        // Check if completed
        let new_status = if updated_job.is_complete() {
            "completed"
        } else {
            "running"
        };

        let update_doc = if updated_job.is_complete() {
            doc! {
                "$set": {
                    "current_step_index": updated_job.current_step_index,
                    "results": serde_json::to_value(&updated_job.results)
                        .unwrap_or(serde_json::json!({})),
                    "status": new_status,
                    "completed_at": Utc::now(),
                }
            }
        } else {
            doc! {
                "$set": {
                    "current_step_index": updated_job.current_step_index,
                    "results": serde_json::to_value(&updated_job.results)
                        .unwrap_or(serde_json::json!({})),
                    "status": new_status,
                }
            }
        };

        self.collection
            .update_one(doc! { "job_id": job_id }, update_doc, None)
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    /// Fail current step with error message
    pub async fn fail_step(&self, job_id: &str, error: String) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await
            .map_err(|e| format!("Failed to fetch job: {}", e))?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Running {
            return Err("Job is not running".to_string());
        }

        self.collection
            .update_one(
                doc! { "job_id": job_id },
                doc! {
                    "$set": {
                        "status": "failed",
                        "error_message": error,
                        "completed_at": Utc::now(),
                    }
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    /// Pause a running job
    pub async fn pause_job(&self, job_id: &str) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await
            .map_err(|e| format!("Failed to fetch job: {}", e))?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Running {
            return Err("Job is not running".to_string());
        }

        self.collection
            .update_one(
                doc! { "job_id": job_id },
                doc! { "$set": { "status": "paused" } },
                None,
            )
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    /// Resume a paused job
    pub async fn resume_job(&self, job_id: &str) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await
            .map_err(|e| format!("Failed to fetch job: {}", e))?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Paused {
            return Err("Job is not paused".to_string());
        }

        self.collection
            .update_one(
                doc! { "job_id": job_id },
                doc! { "$set": { "status": "running" } },
                None,
            )
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    /// List jobs with optional status filter
    pub async fn list_jobs(
        &self,
        status_filter: Option<JobStatus>,
    ) -> Result<Vec<AgentJob>, mongodb::error::Error> {
        let filter = if let Some(status) = status_filter {
            doc! { "status": status.to_string() }
        } else {
            doc! {}
        };

        let mut cursor = self.collection.find(filter, None).await?;
        let mut jobs = Vec::new();

        while cursor.advance().await? {
            jobs.push(cursor.deserialize_current()?);
        }

        Ok(jobs)
    }

    /// Delete a job
    pub async fn delete_job(&self, job_id: &str) -> Result<(), String> {
        let result = self
            .collection
            .delete_one(doc! { "job_id": job_id }, None)
            .await
            .map_err(|e| format!("Failed to delete job: {}", e))?;

        if result.deleted_count == 0 {
            return Err(format!("Job {} not found", job_id));
        }

        Ok(())
    }

    /// Start background worker to process pending jobs
    /// This should be called once at application startup
    pub async fn start_worker(&self) -> Result<(), String> {
        let collection = self.collection.clone();
        let worker_collection = collection.clone();

        let worker_handle = tokio::spawn(async move {
            loop {
                // Poll for pending jobs every 5 seconds
                // TODO: Implement actual job processing logic
                if let Ok(Some(_job)) = worker_collection
                    .find_one(doc! { "status": "running" }, None)
                    .await
                {
                    // Process job here
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        let mut handle = self.worker_handle.write().await;
        *handle = Some(worker_handle);

        Ok(())
    }

    /// Stop the background worker
    pub async fn stop_worker(&self) -> Result<(), String> {
        let mut handle = self.worker_handle.write().await;
        if let Some(join_handle) = handle.take() {
            join_handle.abort();
        }
        Ok(())
    }
}

// ============================================================================
// MOCK/EXAMPLE USAGE (uncomment and modify as needed)
// ============================================================================

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_agent_job_workflow() {
//         // TODO: Setup MongoDB test instance
//         // let client = Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
//         // let db = client.database("test_db");
//         // let orchestrator = AgentOrchestrator::new(db).await.unwrap();
//         
//         // Create a job with steps
//         // let steps = vec![
//         //     JobStep::new("fetch_data")
//         //         .with_description("Fetch data from external API")
//         //         .with_timeout(30),
//         //     JobStep::new("validate_data")
//         //         .with_description("Validate fetched data"),
//         //     JobStep::new("store_result")
//         //         .with_description("Store result in database"),
//         // ];
//         
//         // let job_id = orchestrator.create_job(steps).await.unwrap();
//         
//         // Start job
//         // let _ = orchestrator.start_job(&job_id).await;
//         
//         // Complete steps
//         // let _ = orchestrator.complete_step(&job_id, Some(serde_json::json!({"data": "example"}))).await;
//         // let _ = orchestrator.complete_step(&job_id, Some(serde_json::json!({"valid": true}))).await;
//         // let _ = orchestrator.complete_step(&job_id, Some(serde_json::json!({"stored": true}))).await;
//         
//         // Verify completion
//         // let status = orchestrator.get_job_status(&job_id).await.unwrap();
//         // assert_eq!(status, Some(JobStatus::Completed));
//     }
// }
