use std::sync::Arc;

use crate::{
    auth::JwtService,
    config::Config,
    db::Database,
    errors::AppResult,
    repositories::{MongoQuizRepository, MongoUserRepository, UserRepository},
    services::{model_service::ModelService, quiz_service::QuizService, user_service::UserService},
};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub quiz_service: Arc<QuizService>,
    pub model_service: Arc<ModelService>,
    pub jwt_service: Arc<JwtService>,
    pub config: Arc<Config>,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        let db = Database::connect(&config).await?;
        
        let user_repository = Arc::new(MongoUserRepository::new(&db));
        user_repository.ensure_indexes().await?;
        let user_service = Arc::new(UserService::new(user_repository));
        
        let quiz_repository = Arc::new(MongoQuizRepository::new(&db));
        quiz_repository.ensure_indexes().await?;
        let quiz_service = Arc::new(QuizService::new(quiz_repository));
        
        let model_service = Arc::new(ModelService::new(&config));
        
        let jwt_service = Arc::new(JwtService::new(
            &config.jwt_secret,
            config.jwt_expiration_hours,
        ));
        
        Ok(Self {
            user_service,
            quiz_service,
            model_service,
            jwt_service,
            config: Arc::new(config),
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
