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
        create_summary_document_step(),
        create_quiz_questions_step(),
        finalize_quiz_step(),
    ]
}

fn create_draft_step() -> JobStep {
    JobStep::new("create_quiz_draft")
        .with_description("Create new Quiz with draft status and add to database")
        .with_max_retries(DEFAULT_RETRIES)
        .with_timeout(DRAFT_CREATION_TIMEOUT)
}

fn create_summary_document_step() -> JobStep {
    JobStep::new("create_summary_document")
        .with_description("Create summary document from provided URL via model service")
        .with_max_retries(DEFAULT_RETRIES)
        .with_timeout(SUMMARY_FETCH_TIMEOUT)
}

fn create_quiz_questions_step() -> JobStep {
    JobStep::new("create_quiz_questions")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_quiz_generation_steps_returns_expected_order() {
        let steps = create_quiz_generation_steps();

        let names: Vec<&str> = steps.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "create_quiz_draft",
                "create_summary_document",
                "create_quiz_questions",
                "finalize_quiz"
            ]
        );
    }

    #[test]
    fn create_quiz_generation_steps_has_expected_retries_and_timeouts() {
        let steps = create_quiz_generation_steps();

        assert_eq!(steps[0].max_retries, DEFAULT_RETRIES);
        assert_eq!(steps[0].timeout_seconds, Some(DRAFT_CREATION_TIMEOUT));

        assert_eq!(steps[1].max_retries, DEFAULT_RETRIES);
        assert_eq!(steps[1].timeout_seconds, Some(SUMMARY_FETCH_TIMEOUT));

        assert_eq!(steps[2].max_retries, DEFAULT_RETRIES);
        assert_eq!(steps[2].timeout_seconds, Some(QUIZ_GENERATION_TIMEOUT));

        assert_eq!(steps[3].max_retries, FINALIZATION_RETRIES);
        assert_eq!(steps[3].timeout_seconds, Some(FINALIZATION_TIMEOUT));
    }

    #[test]
    fn create_quiz_generation_steps_have_descriptions_and_unique_ids() {
        let steps = create_quiz_generation_steps();

        let mut ids: Vec<&str> = steps.iter().map(|s| s.id.as_str()).collect();
        let original_len = ids.len();
        ids.sort_unstable();
        ids.dedup();

        assert_eq!(ids.len(), original_len);
        assert!(steps
            .iter()
            .all(|step| step.description.as_ref().is_some_and(|d| !d.is_empty())));
    }
}
