use std::sync::Arc;

use crate::{
    config::Config,
    db::Database,
    errors::AppResult,
    repositories::{MongoUserRepository, UserRepository},
    services::user_service::UserService,
};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub config: Arc<Config>,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        let db = Database::connect(&config).await?;
        let user_repository = Arc::new(MongoUserRepository::new(&db));
        user_repository.ensure_indexes().await?;
        let user_service = Arc::new(UserService::new(user_repository));
        Ok(Self {
            user_service,
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
