use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use tento_server::errors::AppError;
use tento_server::models::domain::Quiz;
use tento_server::models::dto::request::QuizDraftDto;
use tento_server::repositories::QuizRepository;
use tento_server::services::agent_orchestrator_service::{AgentOrchestrator, AgentJob, JobStep, JobStatus};
use tento_server::services::quiz_service::QuizService;
use tento_server::repositories::agent_job_repository::AgentJobRepository;

struct InMemoryQuizRepository {
    quizzes: Arc<RwLock<HashMap<String, Quiz>>>,
}

impl InMemoryQuizRepository {
    fn new() -> Self {
        Self {
            quizzes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl QuizRepository for InMemoryQuizRepository {
    async fn find_by_id(&self, id: &str) -> tento_server::errors::AppResult<Option<Quiz>> {
        let q = self.quizzes.read().await.get(id).cloned();
        Ok(q)
    }

    async fn list_quizzes(&self, offset: i64, limit: i64) -> tento_server::errors::AppResult<(Vec<Quiz>, i64)> {
        let quizzes = self.quizzes.read().await;
        let mut items: Vec<_> = quizzes.values().cloned().collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));
        let total = items.len() as i64;
        let start = offset.max(0) as usize;
        let end = (start + limit.max(0) as usize).min(items.len());
        let page = if start >= items.len() { vec![] } else { items[start..end].to_vec() };
        Ok((page, total))
    }

    async fn list_quizzes_by_user(&self, user_id: &str, offset: i64, limit: i64) -> tento_server::errors::AppResult<(Vec<Quiz>, i64)> {
        let quizzes = self.quizzes.read().await;
        let mut items: Vec<_> = quizzes.values().cloned().filter(|q| q.created_by_user_id == user_id).collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));
        let total = items.len() as i64;
        let start = offset.max(0) as usize;
        let end = (start + limit.max(0) as usize).min(items.len());
        let page = if start >= items.len() { vec![] } else { items[start..end].to_vec() };
        Ok((page, total))
    }

    async fn get_by_status_by_id(&self, id: &str, _status: &str) -> tento_server::errors::AppResult<Option<Quiz>> {
        let q = self.quizzes.read().await.get(id).cloned();
        Ok(q)
    }

    async fn create_quiz_draft(&self, quiz: Quiz) -> tento_server::errors::AppResult<Quiz> {
        let mut w = self.quizzes.write().await;
        if w.contains_key(&quiz.id) {
            return Err(AppError::AlreadyExists(format!("Quiz '{}' exists", quiz.id)));
        }
        w.insert(quiz.id.clone(), quiz.clone());
        Ok(quiz)
    }

    async fn update(&self, quiz: Quiz) -> tento_server::errors::AppResult<Quiz> {
        let mut w = self.quizzes.write().await;
        if !w.contains_key(&quiz.id) {
            return Err(AppError::NotFound(format!("Quiz '{}' not found", quiz.id)));
        }
        w.insert(quiz.id.clone(), quiz.clone());
        Ok(quiz)
    }
}

// In-memory AgentJobRepository stub used by the real AgentOrchestrator
struct StubAgentJobRepository {
    jobs: Arc<RwLock<HashMap<String, AgentJob>>> ,
    pub created: Arc<RwLock<bool>>,
    pub metadata_set: Arc<RwLock<bool>>,
    pub started: Arc<RwLock<bool>>,
}

impl StubAgentJobRepository {
    fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            created: Arc::new(RwLock::new(false)),
            metadata_set: Arc::new(RwLock::new(false)),
            started: Arc::new(RwLock::new(false)),
        }
    }
}

#[async_trait]
impl AgentJobRepository for StubAgentJobRepository {
    async fn create_job(&self, steps: Vec<JobStep>, job_id: &str, correlation_id: &str) -> Result<(), String> {
        let job = AgentJob::new_with_ids(steps, job_id.to_string(), correlation_id.to_string());
        self.jobs.write().await.insert(job_id.to_string(), job);
        *self.created.write().await = true;
        Ok(())
    }

    async fn get_job(&self, job_id: &str) -> Result<Option<AgentJob>, String> {
        Ok(self.jobs.read().await.get(job_id).cloned())
    }

    async fn get_job_status(&self, job_id: &str) -> Result<Option<JobStatus>, String> {
        Ok(self.jobs.read().await.get(job_id).map(|j| j.status))
    }

    async fn start_job(&self, job_id: &str) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Running;
            *self.started.write().await = true;
            return Ok(());
        }
        Err(format!("Job {} not found", job_id))
    }

    async fn complete_step(&self, _job_id: &str, _result: Option<serde_json::Value>) -> Result<(), String> { Ok(()) }
    async fn fail_step(&self, _job_id: &str, _error: String) -> Result<(), String> { Ok(()) }
    async fn pause_job(&self, _job_id: &str) -> Result<(), String> { Ok(()) }
    async fn resume_job(&self, _job_id: &str) -> Result<(), String> { Ok(()) }
    async fn list_jobs(&self, _status_filter: Option<JobStatus>) -> Result<Vec<AgentJob>, String> { Ok(vec![]) }
    async fn delete_job(&self, job_id: &str) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;
        if jobs.remove(job_id).is_some() { Ok(()) } else { Err(format!("Job {} not found", job_id)) }
    }
    async fn save(&self, job: &AgentJob) -> Result<(), String> {
        self.jobs.write().await.insert(job.job_id.clone(), job.clone());
        // mark metadata_set if quiz_id metadata stored
        if job.results.contains_key("quiz_id") {
            *self.metadata_set.write().await = true;
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_get_quiz_not_found() {
    let repo = Arc::new(InMemoryQuizRepository::new());
    // create an in-memory agent job repository and use the real orchestrator
    let stub_repo = Arc::new(StubAgentJobRepository::new());
    let orch = Arc::new(AgentOrchestrator::new(stub_repo.clone()));
    let service = QuizService::new(repo, orch);

    let result = service.get_quiz("missing").await;
    assert!(result.is_err());
    match result.expect_err("expected not found") {
        AppError::NotFound(msg) => assert!(msg.contains("missing")),
        other => panic!("expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_list_quizzes_and_by_user() {
    let repo = Arc::new(InMemoryQuizRepository::new());
    let stub_repo = Arc::new(StubAgentJobRepository::new());
    let orch = Arc::new(AgentOrchestrator::new(stub_repo.clone()));
    // populate
    {
        let mut w = repo.quizzes.write().await;
        for i in 0..5 {
            let q = Quiz::new_draft(&format!("quiz-{}", i), "user-a", 5, 3, 2, "https://example.com");
            let mut q = q;
            q.id = format!("quiz-{}", i);
            if i % 2 == 0 {
                q.created_by_user_id = "user-b".to_string();
            }
            w.insert(q.id.clone(), q);
        }
    }

    let service = QuizService::new(repo.clone(), orch);

    let (list, total) = service.list_quizzes(0, 10).await.expect("list should work");
    assert_eq!(total, 5);
    assert_eq!(list.len(), 5);

    let (user_list, user_total) = service.list_quizzes_by_user("user-b", 0, 10).await.expect("user list");
    assert!(user_total >= 0);
    assert!(user_list.len() > 0);
}

#[tokio::test]
async fn test_create_quiz_draft_starts_job() {
    let repo = Arc::new(InMemoryQuizRepository::new());
    let stub_repo = Arc::new(StubAgentJobRepository::new());
    let orch = Arc::new(AgentOrchestrator::new(stub_repo.clone()));

    let service = QuizService::new(repo.clone(), orch.clone());

    let request = QuizDraftDto {
        name: "New Draft".to_string(),
        question_count: 5,
        required_score: 70,
        attempt_limit: 3,
        url: "https://example.com/article".to_string(),
    };

    let resp = service.create_quiz_draft(request, "user-1").await.expect("create should succeed");

    assert!(resp.data.quiz.id.len() > 0);
    // job id should be returned and non-empty
    assert!(!resp.data.job_id.is_empty());

    // ensure repo has quiz
    let stored = repo.find_by_id(&resp.data.quiz.id).await.unwrap();
    assert!(stored.is_some());

    // ensure repository/orchestrator flags set on the stub repository
    assert!(*stub_repo.created.read().await);
    assert!(*stub_repo.metadata_set.read().await);
    assert!(*stub_repo.started.read().await);
}
