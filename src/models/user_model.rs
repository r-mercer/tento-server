use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
        };

        assert_eq!(user.first_name, "John");
        assert_eq!(user.last_name, "Doe");
        assert_eq!(user.username, "johndoe");
        assert_eq!(user.email, "john@example.com");
    }

    #[test]
    fn test_user_clone() {
        let user1 = User {
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            username: "janesmith".to_string(),
            email: "jane@example.com".to_string(),
        };

        let user2 = user1.clone();
        assert_eq!(user1, user2);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            first_name: "Alice".to_string(),
            last_name: "Wonder".to_string(),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

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
        assert_eq!(user.first_name, "Bob");
        assert_eq!(user.last_name, "Builder");
        assert_eq!(user.username, "bobbuilder");
        assert_eq!(user.email, "bob@example.com");
    }

    #[test]
    fn test_user_equality() {
        let user1 = User {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let user2 = User {
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        let user3 = User {
            first_name: "Different".to_string(),
            last_name: "User".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        assert_eq!(user1, user2);
        assert_ne!(user1, user3);
    }
}
