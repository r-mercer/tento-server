use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}

impl User {
    /// Creates a new User instance
    pub fn new(first_name: &str, last_name: &str, username: &str, email: &str) -> Self {
        User {
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            username: username.to_string(),
            email: email.to_string(),
        }
    }
}

#[cfg(test)]
impl User {
    /// Creates a test user with standard values for testing
    pub fn test_user(username: &str, email: &str) -> Self {
        User::new("Test", "User", username, email)
    }

    /// Creates a test user with just a username (email derived from username)
    pub fn test_user_simple(username: &str) -> Self {
        User::new(
            "Test",
            "User",
            username,
            &format!("{}@example.com", username),
        )
    }

    /// Asserts that this user has the expected field values
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
    }

    #[test]
    fn test_user_clone() {
        let user1 = User::new("Jane", "Smith", "janesmith", "jane@example.com");
        let user2 = user1.clone();
        assert_eq!(user1, user2);
    }

    #[test]
    fn test_user_serialization() {
        let user = User::new("Alice", "Wonder", "alice", "alice@example.com");

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("Alice"));
        assert!(json.contains("alice@example.com"));
    }

    #[test]
    fn test_user_deserialization() {
        let json = r#"{
            "first_name": "Bob",
            "last_name": "Builder",
            "username": "bobbuilder",
            "email": "bob@example.com"
        }"#;

        let user: User = serde_json::from_str(json).unwrap();
        user.assert_fields("Bob", "Builder", "bobbuilder", "bob@example.com");
    }

    #[test]
    fn test_user_equality() {
        let user1 = User::test_user_simple("testuser");
        let user2 = User::test_user_simple("testuser");
        let user3 = User::new("Different", "User", "testuser", "test@example.com");

        assert_eq!(user1, user2);
        assert_ne!(user1, user3);
    }
}
