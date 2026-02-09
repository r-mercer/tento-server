use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::domain::User;

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct UserDto {
    pub username: String,
    pub email: String,
    pub full_name: String,
    #[graphql(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        UserDto {
            username: user.username,
            email: user.email,
            full_name: format!("{} {}", user.first_name, user.last_name),
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, SimpleObject)]
pub struct ApiResponse<T: async_graphql::OutputType> {
    pub data: T,
    pub message: String,
}

pub type CreateUserResponse = ApiResponse<UserDto>;
pub type UpdateUserResponse = ApiResponse<UserDto>;

#[derive(Debug, Serialize, SimpleObject)]
pub struct DeleteUserResponse {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_dto_full_name() {
        let user = User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            created_at: Some(Utc::now()),
        };

        let dto: UserDto = user.into();
        assert_eq!(dto.full_name, "John Doe");
        assert_eq!(dto.username, "johndoe");
    }
}
