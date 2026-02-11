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
            .create_index(job_id_index, None)
            .await
            .map_err(|e| format!("Failed to create job_id index: {}", e))?;

        let status_index = IndexModel::builder()
            .keys(doc! { "status": 1 })
            .build();

        self.collection
            .create_index(status_index, None)
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
            .insert_one(&job, None)
            .await
            .map_err(|e| format!("Failed to create job: {}", e))?;

        Ok(job_id)
    }

    async fn get_job(&self, job_id: &str) -> Result<Option<AgentJob>, String> {
        self.collection
            .find_one(doc! { "job_id": job_id }, None)
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
                None,
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

        if let (Some(step), Some(result_value)) = (updated_job.get_current_step(), result) {
            updated_job.results.insert(step.id.clone(), result_value);
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
            .update_one(doc! { "job_id": job_id }, update_doc, None)
            .await
            .map_err(|e| format!("Failed to update job: {}", e))?;

        Ok(())
    }

    async fn fail_step(&self, job_id: &str, error: String) -> Result<(), String> {
        let job = self
            .get_job(job_id)
            .await?
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        if job.status != JobStatus::Running {
            return Err("Job is not running".to_string());
        }

        let completed_at: String = Utc::now().to_string();

        self.collection
            .update_one(
                doc! { "job_id": job_id },
                doc! {
                    "$set": {
                        "status": "failed",
                        "error_message": error,
                        "completed_at": completed_at,
                    }
                },
                None,
            )
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
                None,
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
                None,
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
            .find(filter, None)
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
            .delete_one(doc! { "job_id": job_id }, None)
            .await
            .map_err(|e| format!("Failed to delete job: {}", e))?;

        if result.deleted_count == 0 {
            return Err(format!("Job {} not found", job_id));
        }

        Ok(())
    }
}
