use std::sync::Arc;

use crate::{
    auth::JwtService,
    config::Config,
    db::Database,
    errors::AppResult,
    repositories::{
        MongoAgentJobRepository, MongoQuizRepository,
        MongoQuizAttemptRepository, MongoSummaryDocumentRepository, MongoUserRepository,
        UserRepository, QuizAttemptRepository,
    },
    services::{agent_orchestrator_service::AgentOrchestrator, model_service::ModelService, quiz_service::QuizService, user_service::UserService},
};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub quiz_service: Arc<QuizService>,
    pub quiz_attempt_repository: Arc<dyn QuizAttemptRepository>,
    pub model_service: Arc<ModelService>,
    pub jwt_service: Arc<JwtService>,
    pub config: Arc<Config>,
    pub agent_orchestrator: Arc<AgentOrchestrator>,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        let db = Database::connect(&config).await?;

        let user_repository = Arc::new(MongoUserRepository::new(&db));
        user_repository.ensure_indexes().await?;
        let user_service = Arc::new(UserService::new(user_repository));

        // Initialize agent orchestrator first since quiz_service depends on it
        let agent_job_repository = Arc::new(MongoAgentJobRepository::new(&db));
        agent_job_repository.ensure_indexes().await?;
        let agent_orchestrator = Arc::new(AgentOrchestrator::new(agent_job_repository));

        let quiz_repository = Arc::new(MongoQuizRepository::new(&db));
        quiz_repository.ensure_indexes().await?;
        let quiz_service = Arc::new(QuizService::new(quiz_repository, agent_orchestrator.clone()));

        let quiz_attempt_repository_mongo = Arc::new(MongoQuizAttemptRepository::new(&db));
        quiz_attempt_repository_mongo.ensure_indexes().await?;
        let quiz_attempt_repository: Arc<dyn QuizAttemptRepository> = quiz_attempt_repository_mongo;

        let model_service = Arc::new(ModelService::new(&config));
        let _summary_document_respository = Arc::new(MongoSummaryDocumentRepository::new(&db));

        let jwt_service = Arc::new(JwtService::new(
            &config.gh_client_secret,
            config.jwt_expiration_hours,
        ));

        Ok(Self {
            user_service,
            quiz_service,
            quiz_attempt_repository,
            model_service,
            jwt_service,
            config: Arc::new(config),
            agent_orchestrator,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_is_cloneable() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<AppState>();
    }
}
