use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::domain::{quiz::QuizStatus, Quiz, QuizQuestion, User};

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

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizDto {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub question_count: i16,
    pub required_score: i16,
    pub attempt_limit: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    pub status: QuizStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questions: Option<Vec<QuizQuestion>>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
}

impl From<Quiz> for QuizDto {
    fn from(quiz: Quiz) -> Self {
        QuizDto {
            id: quiz.id,
            name: quiz.name,
            title: quiz.title,
            description: quiz.description,
            question_count: quiz.question_count,
            required_score: quiz.required_score,
            attempt_limit: quiz.attempt_limit,
            topic: quiz.topic,
            status: quiz.status,
            questions: quiz.questions,
            url: quiz.url,
            created_at: quiz.created_at,
            modified_at: quiz.modified_at,
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
pub type CreateQuizDraftResponse = ApiResponse<QuizDto>;

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
            github_id: None,
            role: crate::models::domain::user::UserRole::default(),
            created_at: Some(Utc::now()),
        };

        let dto: UserDto = user.into();
        assert_eq!(dto.full_name, "John Doe");
        assert_eq!(dto.username, "johndoe");
    }
}
