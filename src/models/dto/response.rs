use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::domain::quiz_attempt::QuizAttempt;
use crate::models::domain::quiz_question::QuizQuestionType;
use crate::models::domain::{quiz::QuizStatus, Quiz, QuizQuestion, User};

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct UserDto {
    pub id: String,
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
            id: user
                .id
                .map(|o| o.to_hex())
                .unwrap_or_else(|| user.username.clone()),
            username: user.username,
            email: user.email,
            full_name: format!("{} {}", user.first_name, user.last_name),
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizResponseDto {
    pub id: String,
    pub name: String,
    pub created_by_user_id: String,
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

impl From<Quiz> for QuizResponseDto {
    fn from(quiz: Quiz) -> Self {
        QuizResponseDto {
            id: quiz.id,
            name: quiz.name,
            created_by_user_id: quiz.created_by_user_id,
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

#[derive(Debug, Serialize, SimpleObject)]
pub struct CreateQuizDraftResponseData {
    pub quiz: QuizResponseDto,
    pub job_id: String,
}

pub type CreateUserResponse = ApiResponse<UserDto>;
pub type UpdateUserResponse = ApiResponse<UserDto>;

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct DeleteResponse {
    pub message: String,
}

pub type DeleteUserResponse = DeleteResponse;
pub type CreateQuizDraftResponse = ApiResponse<CreateQuizDraftResponseData>;

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct ChatCompletionResponse {
    pub content: String,
    pub model: String,
}

pub type ChatCompletionApiResponse = ApiResponse<ChatCompletionResponse>;

#[derive(Debug, Serialize, SimpleObject)]
pub struct QuizForTaking {
    pub id: String,
    pub name: String,
    pub created_by_user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub question_count: i16,
    pub required_score: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    pub status: QuizStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questions: Option<Vec<QuizQuestionForTaking>>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct PaginationMetadata {
    pub offset: i64,
    pub limit: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, SimpleObject)]
pub struct PaginatedResponse<T: async_graphql::OutputType> {
    pub data: Vec<T>,
    pub pagination: PaginationMetadata,
}

#[derive(Debug, Serialize, SimpleObject)]
pub struct PaginatedUserResponse {
    pub data: Vec<UserDto>,
    pub pagination: PaginationMetadata,
}

pub type PaginatedResponseUserDto = PaginatedUserResponse;

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizQuestionOptionForTaking {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizQuestionForTaking {
    pub id: String,
    pub title: String,
    pub description: String,
    pub question_type: QuizQuestionType,
    pub options: Vec<QuizQuestionOptionForTaking>,
    pub option_count: i16,
    pub order: i16,
    pub topic: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl QuizForTaking {
    pub fn from_quiz(quiz: Quiz) -> Self {
        let questions = quiz.questions.map(|qs| {
            qs.into_iter()
                .map(|q| QuizQuestionForTaking {
                    id: q.id,
                    title: q.title,
                    description: q.description,
                    question_type: q.question_type,
                    options: q
                        .options
                        .into_iter()
                        .map(|opt| QuizQuestionOptionForTaking {
                            id: opt.id,
                            text: opt.text,
                        })
                        .collect(),
                    option_count: q.option_count,
                    order: q.order,
                    topic: q.topic,
                    created_at: q.created_at,
                })
                .collect()
        });

        QuizForTaking {
            id: quiz.id,
            name: quiz.name,
            created_by_user_id: quiz.created_by_user_id,
            title: quiz.title,
            description: quiz.description,
            question_count: quiz.question_count,
            required_score: quiz.required_score,
            topic: quiz.topic,
            status: quiz.status,
            questions,
            url: quiz.url,
            created_at: quiz.created_at,
        }
    }
}

// ============================================================================
// Quiz Attempt DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizAttemptResponse {
    pub id: String,
    pub quiz_id: String,
    pub points_earned: i16,
    pub total_possible: i16,
    pub required_score: i16,
    pub passed: bool,
    pub attempt_number: i16,
    pub submitted_at: DateTime<Utc>,
}

impl From<QuizAttempt> for QuizAttemptResponse {
    fn from(attempt: QuizAttempt) -> Self {
        QuizAttemptResponse {
            id: attempt.id,
            quiz_id: attempt.quiz_id,
            points_earned: attempt.points_earned,
            required_score: attempt.required_score,
            total_possible: attempt.total_possible,
            passed: attempt.passed,
            attempt_number: attempt.attempt_number,
            submitted_at: attempt.submitted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuestionAttemptDetail {
    pub question_id: String,
    pub user_selected_option_ids: Vec<String>,
    pub correct_option_ids: Vec<String>,
    pub is_correct: bool,
    pub points_earned: i16,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
pub struct QuizAttemptReview {
    pub attempt: QuizAttemptResponse,
    pub quiz: QuizResponseDto,
    pub question_results: Vec<QuestionAttemptDetail>,
}

#[derive(Debug, Serialize, SimpleObject)]
pub struct PaginatedQuizAttemptResponse {
    pub data: Vec<QuizAttemptResponse>,
    pub pagination: PaginationMetadata,
}

pub type PaginatedResponseQuizAttempt = PaginatedQuizAttemptResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_dto_full_name() {
        let user = User {
            id: None,
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
