use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::dto::request::CreateUserRequest;
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(first_name: &str, last_name: &str, username: &str, email: &str) -> Self {
        User {
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            username: username.to_string(),
            email: email.to_string(),
            created_at: Some(Utc::now()),
        }
    }
    pub fn from_request(request: CreateUserRequest) -> Self {
        User {
            first_name: request.first_name,
            last_name: request.last_name,
            username: request.username,
            email: request.email,
            created_at: Some(Utc::now()),
        }
    }
}

// Test by Copilot
#[cfg(test)]
impl User {
    pub fn test_user(username: &str, email: &str) -> Self {
        User::new("Test", "User", username, email)
    }
    pub fn test_user_simple(username: &str) -> Self {
        User::new(
            "Test",
            "User",
            username,
            &format!("{}@example.com", username),
        )
    }
    pub fn assert_fields(&self, first_name: &str, last_name: &str, username: &str, email: &str) {
        assert_eq!(self.first_name, first_name);
        assert_eq!(self.last_name, last_name);
        assert_eq!(self.username, username);
        assert_eq!(self.email, email);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("John", "Doe", "johndoe", "john@example.com");
        user.assert_fields("John", "Doe", "johndoe", "john@example.com");
        assert!(user.created_at.is_some());
    }

    #[test]
    fn test_user_from_request() {
        let request = CreateUserRequest {
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            username: "janesmith".to_string(),
            email: "jane@example.com".to_string(),
        };

        let user = User::from_request(request);
        assert_eq!(user.first_name, "Jane");
        assert_eq!(user.username, "janesmith");
    }
}
