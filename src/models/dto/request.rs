use async_graphql::InputObject;
use serde::Deserialize;
use validator::Validate;

// static USERNAME_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
//     regex::Regex::new(r"^[a-zA-Z0-9_]+$")
//         .expect("USERNAME_REGEX is a valid regex pattern")
// });

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,

    #[validate(length(min = 1, max = 100))]
    pub last_name: String,

    #[validate(length(min = 3, max = 50))]
    // #[validate(regex(
    //     path = "*USERNAME_REGEX",
    //     message = "Username must be alphanumeric with underscores"
    // ))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate, InputObject)]
pub struct CreateQuizDraftRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,

    // #[validate(required)]
    // pub question_count: i16,
    //
    // #[validate(required)]
    // pub required_score: i16,
    //
    // #[validate(required)]
    // pub attempt_limit: i16,
    #[validate(url)]
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_valid_create_user_request() {
        let request = CreateUserRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let request = CreateUserRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "invalid-email".to_string(),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_username_too_short() {
        let request = CreateUserRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "ab".to_string(),
            email: "john@example.com".to_string(),
        };
        assert!(request.validate().is_err());
    }
}
