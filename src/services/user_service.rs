use mongodb::bson::doc;
use std::sync::Arc;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    models::{
        domain::User,
        dto::{
            request::{CreateUserRequest, UpdateUserRequest},
            response::{CreateUserResponse, DeleteUserResponse, DeleteResponseData, UpdateUserResponse, UserDto, PaginatedResponseUserDto, PaginationMetadata},
        },
    },
    repositories::UserRepository,
};
pub struct UserService {
    repository: Arc<dyn UserRepository>,
}
impl UserService {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }
    pub async fn create_user(&self, request: CreateUserRequest) -> AppResult<CreateUserResponse> {
        request.validate()?;

        if self
            .repository
            .find_by_username(&request.username)
            .await?
            .is_some()
        {
            return Err(AppError::AlreadyExists(format!(
                "User with username '{}' already exists",
                request.username
            )));
        }

        let user = User::from_request(request);
        let created_user = self.repository.create(user).await?;

        Ok(CreateUserResponse {
            data: UserDto::from(created_user),
            message: "User created successfully".to_string(),
        })
    }

    pub async fn get_user(&self, username: &str) -> AppResult<UserDto> {
        let user = self
            .repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("User with username '{}' not found", username))
            })?;

        Ok(UserDto::from(user))
    }

    pub async fn get_all_users(&self) -> AppResult<Vec<UserDto>> {
        let users = self.repository.find_all().await?;
        Ok(users.into_iter().map(UserDto::from).collect())
    }

    pub async fn get_all_users_paginated(&self, offset: i64, limit: i64) -> AppResult<PaginatedResponseUserDto> {
        let (users, total) = self.repository.find_all_paginated(offset, limit).await?;
        
        Ok(PaginatedResponseUserDto {
            data: users.into_iter().map(UserDto::from).collect(),
            pagination: PaginationMetadata {
                offset,
                limit,
                total,
            },
        })
    }

    pub async fn update_user(
        &self,
        username: &str,
        request: UpdateUserRequest,
    ) -> AppResult<UpdateUserResponse> {
        request.validate()?;

        let mut update_doc = doc! {};
        let mut set_fields = doc! {};

        if let Some(first_name) = request.first_name {
            set_fields.insert("first_name", first_name);
        }
        if let Some(last_name) = request.last_name {
            set_fields.insert("last_name", last_name);
        }
        if let Some(email) = request.email {
            set_fields.insert("email", email);
        }

        if set_fields.is_empty() {
            return Err(AppError::ValidationError(
                "No fields provided to update".to_string(),
            ));
        }

        update_doc.insert("$set", set_fields);

        let updated_user = self.repository.update(username, update_doc).await?;

        Ok(UpdateUserResponse {
            data: UserDto::from(updated_user),
            message: "User updated successfully".to_string(),
        })
    }

    pub async fn delete_user(&self, username: &str) -> AppResult<DeleteUserResponse> {
        self.repository.delete(username).await?;

        Ok(DeleteUserResponse::new(
            DeleteResponseData {
                message: format!("User '{}' deleted successfully", username),
            },
            format!("User '{}' deleted successfully", username),
        ))
    }

    /// Upsert user from OAuth flow - creates or updates user based on GitHub ID
    pub async fn upsert_oauth_user(&self, user: User) -> AppResult<User> {
        self.repository.upsert_by_github_id(user).await
    }

    /// Find user by GitHub ID (for OAuth lookups)
    pub async fn find_by_github_id(&self, github_id: &str) -> AppResult<Option<User>> {
        self.repository.find_by_github_id(github_id).await
    }

    /// Get full User domain object by username (for token generation)
    pub async fn get_user_for_token(&self, username: &str) -> AppResult<User> {
        self.repository
            .find_by_username(username)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("User with username '{}' not found", username))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::UserRepository;
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        pub UserRepo {}

        #[async_trait]
        impl UserRepository for UserRepo {
            async fn create(&self, user: User) -> AppResult<User>;
            async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
            async fn find_by_github_id(&self, github_id: &str) -> AppResult<Option<User>>;
            async fn find_all(&self) -> AppResult<Vec<User>>;
            async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<User>, i64)>;
            async fn update(&self, username: &str, update_doc: mongodb::bson::Document) -> AppResult<User>;
            async fn upsert_by_github_id(&self, user: User) -> AppResult<User>;
            async fn delete(&self, username: &str) -> AppResult<()>;
            async fn ensure_indexes(&self) -> AppResult<()>;
        }
    }

    fn create_test_request() -> CreateUserRequest {
        CreateUserRequest {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
        }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let mut mock_repo = MockUserRepo::new();

        // Mock the repository calls
        mock_repo.expect_find_by_username().returning(|_| Ok(None));

        mock_repo.expect_create().returning(|user| Ok(user));

        let service = UserService::new(Arc::new(mock_repo));
        let request = create_test_request();

        let result = service.create_user(request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.username, "johndoe");
    }

    #[tokio::test]
    async fn test_create_user_duplicate() {
        let mut mock_repo = MockUserRepo::new();

        // Mock finding an existing user
        mock_repo
            .expect_find_by_username()
            .returning(|_| Ok(Some(User::test_user_simple("johndoe"))));

        let service = UserService::new(Arc::new(mock_repo));
        let request = create_test_request();

        let result = service.create_user(request).await;

        assert!(result.is_err());
        match result {
            Err(AppError::AlreadyExists(_)) => (),
            _ => panic!("Expected AlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mut mock_repo = MockUserRepo::new();

        mock_repo.expect_find_by_username().returning(|_| Ok(None));

        let service = UserService::new(Arc::new(mock_repo));

        let result = service.get_user("nonexistent").await;

        assert!(result.is_err());
        match result {
            Err(AppError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_all_users() {
        let mut mock_repo = MockUserRepo::new();

        mock_repo.expect_find_all().returning(|| {
            Ok(vec![
                User::test_user_simple("user1"),
                User::test_user_simple("user2"),
            ])
        });

        let service = UserService::new(Arc::new(mock_repo));

        let result = service.get_all_users().await;

        assert!(result.is_ok());
        let users = result.unwrap();
        assert_eq!(users.len(), 2);
    }
}
