use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::models::domain::user::{User, UserRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (username)
    pub email: String,
    pub role: UserRole,
    pub exp: usize, // Expiration time (as UTC timestamp)
    pub iat: usize, // Issued at (as UTC timestamp)
}

impl Claims {
    pub fn new(user: &User, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expiration_hours);

        Self {
            sub: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
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

        assert_eq!(claims.sub, "johndoe");
        assert_eq!(claims.email, "john@example.com");
        assert!(claims.exp > claims.iat);
    }
}
