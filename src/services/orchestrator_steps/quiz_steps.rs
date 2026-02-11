use crate::services::agent_orchestrator_service::JobStep;

const DRAFT_CREATION_TIMEOUT: u64 = 10;
const SUMMARY_FETCH_TIMEOUT: u64 = 60;
const QUIZ_GENERATION_TIMEOUT: u64 = 120;
const FINALIZATION_TIMEOUT: u64 = 15;

const DEFAULT_RETRIES: u32 = 3;
const FINALIZATION_RETRIES: u32 = 2;

pub fn create_quiz_generation_steps() -> Vec<JobStep> {
    vec![
        create_draft_step(),
        fetch_summary_step(),
        generate_quiz_fields_step(),
        finalize_quiz_step(),
    ]
}

fn create_draft_step() -> JobStep {
    JobStep::new("create_quiz_draft")
        .with_description("Create new Quiz with draft status and add to database")
        .with_max_retries(DEFAULT_RETRIES)
        .with_timeout(DRAFT_CREATION_TIMEOUT)
}

fn fetch_summary_step() -> JobStep {
    JobStep::new("fetch_summary_document")
        .with_description("Fetch and create summary document from provided URL via model service")
        .with_max_retries(DEFAULT_RETRIES)
        .with_timeout(SUMMARY_FETCH_TIMEOUT)
}

fn generate_quiz_fields_step() -> JobStep {
    JobStep::new("generate_quiz_fields")
        .with_description("Generate quiz questions and fields via model service call")
        .with_max_retries(DEFAULT_RETRIES)
        .with_timeout(QUIZ_GENERATION_TIMEOUT)
}

fn finalize_quiz_step() -> JobStep {
    JobStep::new("finalize_quiz")
        .with_description("Deserialize quiz JSON, update database with complete quiz model, and change status to active")
        .with_max_retries(FINALIZATION_RETRIES)
        .with_timeout(FINALIZATION_TIMEOUT)
}
