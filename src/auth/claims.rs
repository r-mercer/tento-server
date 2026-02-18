use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::models::domain::user::{User, UserRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user id)
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub exp: usize, // Expiration time (as UTC timestamp)
    pub iat: usize, // Issued at (as UTC timestamp)
}

impl Claims {
    pub fn new(user: &User, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expiration_hours);

        // Use MongoDB ObjectId hex string as subject when available, fallback to username
        let subject = user
            .id
            .as_ref()
            .map(|oid| oid.to_hex())
            .unwrap_or_else(|| user.username.clone());

        Self {
            sub: subject,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,        // user id
    pub token_type: String, // "refresh"
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at time
}

impl RefreshClaims {
    pub fn new(username: &str, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expiration_hours);

        Self {
            sub: username.to_string(),
            token_type: "refresh".to_string(),
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let user = User::new("John", "Doe", "johndoe", "john@example.com");
        let claims = Claims::new(&user, 24);

        // Without an ObjectId the subject falls back to username
        assert_eq!(claims.sub, "johndoe");
        assert_eq!(claims.username, "johndoe");
        assert_eq!(claims.email, "john@example.com");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_refresh_claims_creation() {
        let refresh_claims = RefreshClaims::new("johndoe", 168);

        assert_eq!(refresh_claims.sub, "johndoe");
        assert_eq!(refresh_claims.token_type, "refresh");
        assert!(refresh_claims.exp > refresh_claims.iat);
    }
}
