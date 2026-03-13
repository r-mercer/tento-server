use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 5,
            burst_size: 3,
        }
    }
}

#[derive(Clone)]
pub struct RateLimitState {
    config: RateLimitConfig,
    clients: Arc<RwLock<HashMap<String, ClientState>>>,
}

struct ClientState {
    tokens: f64,
    last_update: Instant,
}

impl RateLimitState {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(&self, client_id: &str) -> Result<(), actix_web::Error> {
        let mut clients = self.clients.write().await;

        let now = Instant::now();
        let state = clients
            .entry(client_id.to_string())
            .or_insert_with(|| ClientState {
                tokens: self.config.burst_size as f64,
                last_update: now,
            });

        let elapsed = now.duration_since(state.last_update).as_secs_f64();
        state.last_update = now;

        let refill_rate = self.config.requests_per_minute as f64 / 60.0;
        state.tokens = (state.tokens + elapsed * refill_rate).min(self.config.burst_size as f64);

        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            Ok(())
        } else {
            Err(actix_web::error::ErrorTooManyRequests(
                "Rate limit exceeded",
            ))
        }
    }
}

pub fn rate_limiter(config: RateLimitConfig) -> RateLimitState {
    RateLimitState::new(config)
}
