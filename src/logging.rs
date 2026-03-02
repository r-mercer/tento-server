//! Logging utilities with correlation ID support for distributed tracing.
//!
//! Provides structured logging types that automatically include correlation IDs
//! for tracking requests across service boundaries.

use uuid::Uuid;

/// Generate a new correlation ID for request tracing.
///
/// # Example
/// ```
/// use tento_server::logging::generate_correlation_id;
/// let correlation_id = generate_correlation_id();
/// log::info!(target: "agent_orchestrator", "[{}] Job started", correlation_id);
/// ```
pub fn generate_correlation_id() -> String {
    Uuid::new_v4().to_string()
}

/// Structured log fields for AI model interactions.
///
/// # Example
/// ```
/// use tento_server::logging::ModelInteractionLog;
///
/// ModelInteractionLog::new("chat_completion")
///     .with_correlation_id("abc-123")
///     .with_model("mistralai/ministral-3-3b")
///     .with_token_count(1024)
///     .log_request();
///
/// ModelInteractionLog::new("chat_completion")
///     .with_correlation_id("abc-123")
///     .with_success(true)
///     .with_duration_ms(150)
///     .log_response();
/// ```
#[allow(dead_code)]
pub struct ModelInteractionLog {
    interaction_type: String,
    correlation_id: Option<String>,
    model: Option<String>,
    token_count: Option<u32>,
    success: Option<bool>,
    duration_ms: Option<u64>,
    error: Option<String>,
}

impl ModelInteractionLog {
    pub fn new(interaction_type: impl Into<String>) -> Self {
        Self {
            interaction_type: interaction_type.into(),
            correlation_id: None,
            model: None,
            token_count: None,
            success: None,
            duration_ms: None,
            error: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }

    pub fn with_model(mut self, model: impl Into<String> + std::fmt::Display) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn with_token_count(mut self, count: u32) -> Self {
        self.token_count = Some(count);
        self
    }

    pub fn with_success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }

    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    fn build_message(&self, prefix: &str) -> String {
        let mut parts = vec![prefix.to_string()];

        if let Some(ref cid) = self.correlation_id {
            parts.push(format!("correlation_id={}", cid));
        }

        if let Some(ref model) = self.model {
            parts.push(format!("model={}", model));
        }

        if let Some(tokens) = self.token_count {
            parts.push(format!("tokens={}", tokens));
        }

        if let Some(success) = self.success {
            parts.push(format!("success={}", success));
        }

        if let Some(duration) = self.duration_ms {
            parts.push(format!("duration_ms={}", duration));
        }

        if let Some(ref error) = self.error {
            parts.push(format!("error={}", error));
        }

        parts.join(" | ")
    }

    pub fn log_request(&self) {
        log::info!(target: "model_service", "{}", self.build_message("MODEL_REQUEST"));
    }

    pub fn log_response(&self) {
        if self.success == Some(true) {
            log::info!(target: "model_service", "{}", self.build_message("MODEL_RESPONSE"));
        } else if self.error.is_some() {
            log::error!(target: "model_service", "{}", self.build_message("MODEL_ERROR"));
        } else {
            log::debug!(target: "model_service", "{}", self.build_message("MODEL_RESPONSE"));
        }
    }
}

/// Structured log fields for agent job workflow.
///
/// # Example
/// ```
/// use tento_server::logging::JobWorkflowLog;
///
/// let correlation_id = "abc-123";
/// let job_id = "job-789";
/// JobWorkflowLog::new("create_job")
///     .with_correlation_id(&correlation_id)
///     .with_job_id(&job_id)
///     .log_start();
///
/// JobWorkflowLog::new("create_job")
///     .with_correlation_id(&correlation_id)
///     .with_job_id(&job_id)
///     .with_step_name("extract_content")
///     .log_step_start();
///
/// JobWorkflowLog::new("create_job")
///     .with_correlation_id(&correlation_id)
///     .with_job_id(&job_id)
///     .with_success(true)
///     .log_complete();
/// ```
pub struct JobWorkflowLog {
    workflow_type: String,
    correlation_id: Option<String>,
    job_id: Option<String>,
    step_name: Option<String>,
    step_index: Option<usize>,
    attempt: Option<u32>,
    success: Option<bool>,
    error: Option<String>,
    duration_ms: Option<u64>,
}

impl JobWorkflowLog {
    pub fn new(workflow_type: impl Into<String>) -> Self {
        Self {
            workflow_type: workflow_type.into(),
            correlation_id: None,
            job_id: None,
            step_name: None,
            step_index: None,
            attempt: None,
            success: None,
            error: None,
            duration_ms: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }

    pub fn with_job_id(mut self, job_id: &str) -> Self {
        self.job_id = Some(job_id.to_string());
        self
    }

    pub fn with_step_name(mut self, step_name: impl Into<String>) -> Self {
        self.step_name = Some(step_name.into());
        self
    }

    pub fn with_step_index(mut self, index: usize) -> Self {
        self.step_index = Some(index);
        self
    }

    pub fn with_attempt(mut self, attempt: u32) -> Self {
        self.attempt = Some(attempt);
        self
    }

    pub fn with_success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    fn build_message(&self, prefix: &str) -> String {
        let mut parts = vec![format!("{}:{}", self.workflow_type, prefix)];

        if let Some(ref cid) = self.correlation_id {
            parts.push(format!("correlation_id={}", cid));
        }

        if let Some(ref job_id) = self.job_id {
            parts.push(format!("job_id={}", job_id));
        }

        if let Some(ref step) = self.step_name {
            parts.push(format!("step={}", step));
        }

        if let Some(index) = self.step_index {
            parts.push(format!("step_index={}", index));
        }

        if let Some(attempt) = self.attempt {
            parts.push(format!("attempt={}", attempt));
        }

        if let Some(success) = self.success {
            parts.push(format!("success={}", success));
        }

        if let Some(duration) = self.duration_ms {
            parts.push(format!("duration_ms={}", duration));
        }

        if let Some(ref error) = self.error {
            parts.push(format!("error={}", error));
        }

        parts.join(" | ")
    }

    pub fn log_start(&self) {
        log::info!(target: "agent_orchestrator", "{}", self.build_message("START"));
    }

    pub fn log_step_start(&self) {
        log::info!(target: "agent_orchestrator", "{}", self.build_message("STEP_START"));
    }

    pub fn log_step_complete(&self) {
        if self.success == Some(true) {
            log::info!(target: "agent_orchestrator", "{}", self.build_message("STEP_COMPLETE"));
        } else {
            log::error!(target: "agent_orchestrator", "{}", self.build_message("STEP_FAILED"));
        }
    }

    pub fn log_complete(&self) {
        if self.success == Some(true) {
            log::info!(target: "agent_orchestrator", "{}", self.build_message("COMPLETE"));
        } else {
            log::error!(target: "agent_orchestrator", "{}", self.build_message("FAILED"));
        }
    }

    pub fn log_error(&self) {
        log::error!(target: "agent_orchestrator", "{}", self.build_message("ERROR"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_generation() {
        let id1 = generate_correlation_id();
        let id2 = generate_correlation_id();

        assert_ne!(id1, id2);
        assert!(id1.len() == 36); // UUID v4 format
    }

    #[test]
    fn test_model_interaction_log() {
        let log = ModelInteractionLog::new("chat_completion")
            .with_correlation_id("test-123")
            .with_model("gpt-4")
            .with_token_count(100)
            .with_duration_ms(50);

        let message = log.build_message("TEST");
        assert!(message.contains("correlation_id=test-123"));
        assert!(message.contains("model=gpt-4"));
        assert!(message.contains("tokens=100"));
        assert!(message.contains("duration_ms=50"));
    }

    #[test]
    fn test_job_workflow_log() {
        let log = JobWorkflowLog::new("process_job")
            .with_correlation_id("test-456")
            .with_job_id("job-789")
            .with_step_name("extract")
            .with_step_index(1)
            .with_attempt(2);

        let message = log.build_message("START");
        assert!(message.contains("correlation_id=test-456"));
        assert!(message.contains("job_id=job-789"));
        assert!(message.contains("step=extract"));
        assert!(message.contains("step_index=1"));
        assert!(message.contains("attempt=2"));
    }
}
