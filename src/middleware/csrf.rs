use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct CsrfConfig {
    pub token_validity_secs: u64,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            token_validity_secs: 3600,
        }
    }
}

#[derive(Clone)]
pub struct CsrfState {
    config: CsrfConfig,
    tokens: Arc<RwLock<HashMap<String, (String, Instant)>>>,
}

impl CsrfState {
    pub fn new(config: CsrfConfig) -> Self {
        Self {
            config,
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn generate_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    pub async fn create_token(&self) -> String {
        let token = self.generate_token();
        let token_key = token.clone();

        {
            let mut tokens = self.tokens.write().await;
            tokens.insert(token_key, (token.clone(), Instant::now()));
        }

        token
    }

    pub async fn validate_token(&self, token: &str) -> Result<(), actix_web::Error> {
        let tokens = self.tokens.read().await;

        if let Some((_, created_at)) = tokens.get(token) {
            let elapsed = created_at.elapsed();
            if elapsed < Duration::from_secs(self.config.token_validity_secs) {
                return Ok(());
            }
        }

        Err(actix_web::error::ErrorBadRequest(
            "Invalid or expired CSRF token",
        ))
    }

    pub async fn invalidate_token(&self, token: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.remove(token);
    }
}

pub fn csrf_protector(config: CsrfConfig) -> CsrfState {
    CsrfState::new(config)
}
