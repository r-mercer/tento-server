use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RefreshToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
}

impl RefreshToken {
    pub fn new(user_id: String, token_hash: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: None,
            user_id,
            token_hash,
            expires_at,
            created_at: Utc::now(),
            revoked: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.revoked && self.expires_at > Utc::now()
    }
}

pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_refresh_token_creation() {
        let expires_at = Utc::now() + Duration::days(7);
        let token = RefreshToken::new("user123".to_string(), "hash123".to_string(), expires_at);

        assert_eq!(token.user_id, "user123");
        assert_eq!(token.token_hash, "hash123");
        assert!(!token.revoked);
        assert!(token.is_valid());
    }

    #[test]
    fn test_refresh_token_expired() {
        let expires_at = Utc::now() - Duration::hours(1);
        let token = RefreshToken::new("user123".to_string(), "hash123".to_string(), expires_at);

        assert!(!token.is_valid());
    }

    #[test]
    fn test_refresh_token_revoked() {
        let expires_at = Utc::now() + Duration::days(7);
        let mut token = RefreshToken::new("user123".to_string(), "hash123".to_string(), expires_at);
        token.revoked = true;

        assert!(!token.is_valid());
    }

    #[test]
    fn test_hash_token_consistency() {
        let token = "my-secret-token";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let hash1 = hash_token("token1");
        let hash2 = hash_token("token2");

        assert_ne!(hash1, hash2);
    }
}
