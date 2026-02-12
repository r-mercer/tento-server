use async_trait::async_trait;
use chrono::Utc;
use mongodb::{bson::doc, options::IndexOptions, Collection, IndexModel};

use crate::db::Database;
use crate::services::agent_orchestrator_service::{AgentJob, JobStatus, JobStep};

#[async_trait]
pub trait AgentJobRepository: Send + Sync {
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

pub struct MongoAgentJobRepository {
    collection: Collection<AgentJob>,
}

impl MongoAgentJobRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.get_collection("jobs");
        Self { collection }
    }

    pub async fn ensure_indexes(&self) -> Result<(), String> {
        log::info!("Creating indexes for jobs collection");

        let job_id_index = IndexModel::builder()
            .keys(doc! { "job_id": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name("job_id_unique".to_string())
                    .build(),
            )
            .build();

        self.collection
            .create_index(job_id_index)
            .await
            .map_err(|e| format!("Failed to create job_id index: {}", e))?;

        let status_index = IndexModel::builder()
            .keys(doc! { "status": 1 })
            .build();

        self.collection
            .create_index(status_index)
            .await
            .map_err(|e| format!("Failed to create status index: {}", e))?;

        log::info!("Successfully created indexes for jobs collection");
        Ok(())
    }
}

#[async_trait]
impl AgentJobRepository for MongoAgentJobRepository {
    async fn create_job(&self, steps: Vec<JobStep>) -> Result<String, String> {
        let job = AgentJob::new(steps);
        let job_id = job.job_id.clone();

        self.collection
            .insert_one(&job)
            .await
            .map_err(|e| format!("Failed to create job: {}", e))?;

        Ok(job_id)
    }

    async fn get_job(&self, job_id: &str) -> Result<Option<AgentJob>, String> {
        self.collection
            .find_one(doc! { "job_id": job_id })
            .await
            .map_err(|e| format!("Failed to fetch job: {}", e))
    }

    async fn get_job_status(&self, job_id: &str) -> Result<Option<JobStatus>, String> {
        let job = self.get_job(job_id).await?;
        Ok(job.map(|j| j.status))
    }

    async fn start_job(&self, job_id: &str) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Pending {
            return Err(format!("Job is already {}", job.status));
        }

        let started_at: String = Utc::now().to_string();

        self.collection
            .update_one(
                doc! { "job_id": job_id },
                doc! {
                    "$set": {
                        "status": "running",
                        "started_at": started_at,
                    }
                },
            )
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    async fn complete_step(
        &self,
        job_id: &str,
        result: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Running {
            return Err("Job is not running".to_string());
        }

        let mut updated_job = job.clone();

        if let (Some(_step), Some(result_value)) = (updated_job.get_current_step(), result) {
            // If result is a JSON object, merge its contents into results
            // Otherwise, store it under the step ID
            if let Some(obj) = result_value.as_object() {
                for (key, value) in obj {
                    updated_job.results.insert(key.clone(), value.clone());
                }
            } else {
                // Fallback for non-object results
                if let Some(step) = updated_job.get_current_step() {
                    updated_job.results.insert(step.id.clone(), result_value);
                }
            }
        }

        updated_job.current_step_index += 1;

        let new_status = if updated_job.is_complete() {
            "completed"
        } else {
            "running"
        };

        let completed_at: String = Utc::now().to_string();

        let update_doc = if updated_job.is_complete() {
            doc! {
                "$set": {
                    "current_step_index": updated_job.current_step_index as i32,
                    "results": mongodb::bson::to_bson(&updated_job.results)
                        .unwrap_or(mongodb::bson::Bson::Document(doc! {})),
                    "status": new_status,
                    "completed_at": completed_at,
                }
            }
        } else {
            doc! {
                "$set": {
                    "current_step_index": updated_job.current_step_index as i32,
                    "results": mongodb::bson::to_bson(&updated_job.results)
                        .unwrap_or(mongodb::bson::Bson::Document(doc! {})),
                    "status": new_status,
                }
            }
        };

        self.collection
            .update_one(doc! { "job_id": job_id }, update_doc)
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    async fn fail_step(&self, job_id: &str, error: String) -> Result<(), String> {
        let mut job = self
            .get_job(job_id)
            .await?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Running {
            return Err("Job is not running".to_string());
        }

        // Increment retry count for current step
        if let Some(_current_step) = job.get_current_step() {
            job.steps[job.current_step_index].retry_count += 1;
        }

        let completed_at: String = Utc::now().to_string();

        // Check if we should keep retrying or fail the entire job
        let update_doc = if let Some(current_step) = job.get_current_step() {
            if current_step.retry_count >= current_step.max_retries {
                // Max retries exceeded - fail the job
                doc! {
                    "$set": {
                        "status": "failed",
                        "error_message": &error,
                        "completed_at": &completed_at,
                        "steps": mongodb::bson::to_bson(&job.steps)
                            .unwrap_or(mongodb::bson::Bson::Array(vec![])),
                    }
                }
            } else {
                // Keep retrying - stay in running state but increment retry count
                doc! {
                    "$set": {
                        "steps": mongodb::bson::to_bson(&job.steps)
                            .unwrap_or(mongodb::bson::Bson::Array(vec![])),
                        "error_message": &error,
                    }
                }
            }
        } else {
            doc! {
                "$set": {
                    "status": "failed",
                    "error_message": &error,
                    "completed_at": &completed_at,
                }
            }
        };

        self.collection
            .update_one(doc! { "job_id": job_id }, update_doc)
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    async fn pause_job(&self, job_id: &str) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Running {
            return Err("Job is not running".to_string());
        }

        self.collection
            .update_one(
                doc! { "job_id": job_id },
                doc! { "$set": { "status": "paused" } },
            )
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    async fn resume_job(&self, job_id: &str) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Paused {
            return Err("Job is not paused".to_string());
        }

        self.collection
            .update_one(
                doc! { "job_id": job_id },
                doc! { "$set": { "status": "running" } },
            )
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    async fn list_jobs(
        &self,
        status_filter: Option<JobStatus>,
    ) -> Result<Vec<AgentJob>, String> {
        let filter = if let Some(status) = status_filter {
            doc! { "status": status.to_string() }
        } else {
            doc! {}
        };

        let mut cursor = self
            .collection
            .find(filter)
            .await
            .map_err(|e| format!("Failed to list jobs: {}", e))?;

        let mut jobs = Vec::new();

        while cursor
            .advance()
            .await
            .map_err(|e| format!("Failed to iterate jobs: {}", e))?
        {
            jobs.push(
                cursor
                    .deserialize_current()
                    .map_err(|e| format!("Failed to deserialize job: {}", e))?,
            );
        }

        Ok(jobs)
    }

    async fn delete_job(&self, job_id: &str) -> Result<(), String> {
        let result = self
            .collection
            .delete_one(doc! { "job_id": job_id })
            .await
            .map_err(|e| format!("Failed to delete job: {}", e))?;

        if result.deleted_count == 0 {
            return Err(format!("Job {} not found", job_id));
        }

        Ok(())
    }

    async fn save(&self, job: &AgentJob) -> Result<(), String> {
        self.collection
            .replace_one(
                doc! { "job_id": &job.job_id },
                job,
            )
            .await
            .map_err(|e| format!("Failed to save job: {}", e))?;

        Ok(())
    }
}
