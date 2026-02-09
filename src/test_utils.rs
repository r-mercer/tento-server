// Most, if not all of these are written by copilot

use crate::models::domain::User;

#[cfg(test)]
pub mod fixtures {
    use super::*;

    pub fn test_user() -> User {
        User::new("Test", "User", "testuser", "test@example.com")
    }
    pub fn test_user_with_username(username: &str) -> User {
        User::new(
            "Test",
            "User",
            username,
            &format!("{}@example.com", username),
        )
    }
    pub fn test_users() -> Vec<User> {
        vec![
            User::new("John", "Doe", "johndoe", "john@example.com"),
            User::new("Jane", "Smith", "janesmith", "jane@example.com"),
            User::new("Alice", "Wonder", "alice", "alice@example.com"),
        ]
    }
}

#[cfg(test)]
pub mod test_helpers {
    use actix_web::http::StatusCode;
    pub fn assert_error_status(status: StatusCode) {
        assert!(
            status.is_client_error() || status.is_server_error(),
            "Expected error status, got: {}",
            status
        );
    }
    pub fn assert_success_status(status: StatusCode) {
        assert!(
            status.is_success(),
            "Expected success status, got: {}",
            status
        );
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;

    #[test]
    fn test_fixtures_test_user() {
        let user = test_user();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_fixtures_test_user_with_username() {
        let user = test_user_with_username("custom");
        assert_eq!(user.username, "custom");
        assert_eq!(user.email, "custom@example.com");
    }

    #[test]
    fn test_fixtures_test_users() {
        let users = test_users();
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].username, "johndoe");
        assert_eq!(users[1].username, "janesmith");
        assert_eq!(users[2].username, "alice");
    }
}
