use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
use tokio::sync::RwLock;

use tento_server::{
    errors::{AppError, AppResult},
    models::domain::{
        quiz::QuizStatus,
        quiz_attempt::{QuizAttempt, QuizAttemptQuestion},
        User,
    },
    repositories::{QuizAttemptRepository, QuizRepository, UserRepository},
};

struct InMemoryQuizRepository {
    quizzes: Arc<RwLock<HashMap<String, tento_server::models::domain::Quiz>>>,
}

impl InMemoryQuizRepository {
    fn new() -> Self {
        Self {
            quizzes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl QuizRepository for InMemoryQuizRepository {
    async fn find_by_id(&self, id: &str) -> AppResult<Option<tento_server::models::domain::Quiz>> {
        let quizzes = self.quizzes.read().await;
        Ok(quizzes.get(id).cloned())
    }

    async fn list_quizzes(
        &self,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<tento_server::models::domain::Quiz>, i64)> {
        let quizzes = self.quizzes.read().await;
        let mut items: Vec<_> = quizzes.values().cloned().collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));

        let total = items.len() as i64;
        let start = offset.max(0) as usize;
        let end = (start + limit.max(0) as usize).min(items.len());

        let page = if start >= items.len() {
            vec![]
        } else {
            items[start..end].to_vec()
        };

        Ok((page, total))
    }

    async fn list_quizzes_by_user(
        &self,
        user_id: &str,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<tento_server::models::domain::Quiz>, i64)> {
        let quizzes = self.quizzes.read().await;
        let mut items: Vec<_> = quizzes
            .values()
            .filter(|q| q.created_by_user_id == user_id)
            .cloned()
            .collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));

        let total = items.len() as i64;
        let start = offset.max(0) as usize;
        let end = (start + limit.max(0) as usize).min(items.len());

        let page = if start >= items.len() {
            vec![]
        } else {
            items[start..end].to_vec()
        };

        Ok((page, total))
    }

    async fn get_by_status_by_id(
        &self,
        id: &str,
        status: &str,
    ) -> AppResult<Option<tento_server::models::domain::Quiz>> {
        let quizzes = self.quizzes.read().await;
        let Some(quiz) = quizzes.get(id) else {
            return Ok(None);
        };

        let quiz_status = match quiz.status {
            QuizStatus::Draft => "draft",
            QuizStatus::Pending => "pending",
            QuizStatus::Ready => "ready",
            QuizStatus::Complete => "complete",
        };

        if quiz_status == status {
            Ok(Some(quiz.clone()))
        } else {
            Ok(None)
        }
    }

    async fn create_quiz_draft(
        &self,
        quiz: tento_server::models::domain::Quiz,
    ) -> AppResult<tento_server::models::domain::Quiz> {
        let mut quizzes = self.quizzes.write().await;
        if quizzes.contains_key(&quiz.id) {
            return Err(AppError::AlreadyExists(format!(
                "Quiz with id '{}' already exists",
                quiz.id
            )));
        }

        quizzes.insert(quiz.id.clone(), quiz.clone());
        Ok(quiz)
    }

    async fn update(
        &self,
        quiz: tento_server::models::domain::Quiz,
    ) -> AppResult<tento_server::models::domain::Quiz> {
        let mut quizzes = self.quizzes.write().await;
        if !quizzes.contains_key(&quiz.id) {
            return Err(AppError::NotFound(format!(
                "Quiz with id '{}' not found",
                quiz.id
            )));
        }

        quizzes.insert(quiz.id.clone(), quiz.clone());
        Ok(quiz)
    }
}

struct InMemoryQuizAttemptRepository {
    attempts: Arc<RwLock<HashMap<String, QuizAttempt>>>,
}

impl InMemoryQuizAttemptRepository {
    fn new() -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl QuizAttemptRepository for InMemoryQuizAttemptRepository {
    async fn create(&self, attempt: QuizAttempt) -> AppResult<QuizAttempt> {
        let mut attempts = self.attempts.write().await;
        if attempts.contains_key(&attempt.id) {
            return Err(AppError::AlreadyExists(format!(
                "Attempt with id '{}' already exists",
                attempt.id
            )));
        }
        attempts.insert(attempt.id.clone(), attempt.clone());
        Ok(attempt)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<QuizAttempt>> {
        let attempts = self.attempts.read().await;
        Ok(attempts.get(id).cloned())
    }

    async fn find_by_user_and_quiz(&self, user_id: &str, quiz_id: &str) -> AppResult<Vec<QuizAttempt>> {
        let attempts = self.attempts.read().await;
        let mut items: Vec<_> = attempts
            .values()
            .filter(|a| a.user_id == user_id && a.quiz_id == quiz_id)
            .cloned()
            .collect();
        items.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
        Ok(items)
    }

    async fn has_user_attempted_quiz(&self, user_id: &str, quiz_id: &str) -> AppResult<bool> {
        let attempts = self.attempts.read().await;
        Ok(attempts
            .values()
            .any(|a| a.user_id == user_id && a.quiz_id == quiz_id))
    }

    async fn count_user_attempts(&self, user_id: &str, quiz_id: &str) -> AppResult<usize> {
        let attempts = self.attempts.read().await;
        Ok(attempts
            .values()
            .filter(|a| a.user_id == user_id && a.quiz_id == quiz_id)
            .count())
    }

    async fn get_user_attempts(
        &self,
        user_id: &str,
        quiz_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<QuizAttempt>, i64)> {
        let attempts = self.attempts.read().await;
        let mut items: Vec<_> = attempts
            .values()
            .filter(|a| a.user_id == user_id && quiz_id.map(|qid| a.quiz_id == qid).unwrap_or(true))
            .cloned()
            .collect();

        items.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));

        let total = items.len() as i64;
        let start = offset.max(0) as usize;
        let end = (start + limit.max(0) as usize).min(items.len());

        let page = if start >= items.len() {
            vec![]
        } else {
            items[start..end].to_vec()
        };

        Ok((page, total))
    }
}

struct InMemoryUserRepository {
    users_by_username: Arc<RwLock<HashMap<String, User>>>,
}

impl InMemoryUserRepository {
    fn new() -> Self {
        Self {
            users_by_username: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, user: User) -> AppResult<User> {
        let mut users = self.users_by_username.write().await;

        if users.contains_key(&user.username) {
            return Err(AppError::AlreadyExists(format!(
                "User with username '{}' already exists",
                user.username
            )));
        }

        if let Some(github_id) = &user.github_id {
            let duplicate = users
                .values()
                .any(|u| u.github_id.as_ref().map(|gid| gid == github_id).unwrap_or(false));
            if duplicate {
                return Err(AppError::AlreadyExists(format!(
                    "User with github_id '{}' already exists",
                    github_id
                )));
            }
        }

        users.insert(user.username.clone(), user.clone());
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let users = self.users_by_username.read().await;
        Ok(users.get(username).cloned())
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<User>> {
        let users = self.users_by_username.read().await;
        Ok(users
            .values()
            .find(|u| u.id.as_ref().map(|oid| oid.to_hex() == id).unwrap_or(false))
            .cloned())
    }

    async fn find_by_github_id(&self, github_id: &str) -> AppResult<Option<User>> {
        let users = self.users_by_username.read().await;
        Ok(users
            .values()
            .find(|u| u.github_id.as_deref() == Some(github_id))
            .cloned())
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        let users = self.users_by_username.read().await;
        let mut items: Vec<_> = users.values().cloned().collect();
        items.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(items)
    }

    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<User>, i64)> {
        let mut items = self.find_all().await?;
        let total = items.len() as i64;

        let start = offset.max(0) as usize;
        let end = (start + limit.max(0) as usize).min(items.len());

        items = if start >= items.len() {
            vec![]
        } else {
            items[start..end].to_vec()
        };

        Ok((items, total))
    }

    async fn update(&self, username: &str, update_doc: Document) -> AppResult<User> {
        let mut users = self.users_by_username.write().await;
        let mut user = users
            .get(username)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("User with username '{}' not found", username)))?;

        if let Some(Bson::Document(set_doc)) = update_doc.get("$set") {
            if let Some(Bson::String(first_name)) = set_doc.get("first_name") {
                user.first_name = first_name.clone();
            }
            if let Some(Bson::String(last_name)) = set_doc.get("last_name") {
                user.last_name = last_name.clone();
            }
            if let Some(Bson::String(email)) = set_doc.get("email") {
                user.email = email.clone();
            }
        }

        users.insert(user.username.clone(), user.clone());
        Ok(user)
    }

    async fn upsert_by_github_id(&self, user: User) -> AppResult<User> {
        let github_id = user
            .github_id
            .clone()
            .ok_or_else(|| AppError::ValidationError("User must have a github_id for upsert".to_string()))?;

        let mut users = self.users_by_username.write().await;

        let existing_username = users
            .iter()
            .find(|(_, u)| u.github_id.as_deref() == Some(&github_id))
            .map(|(username, _)| username.clone());

        if let Some(username) = existing_username {
            users.insert(username, user.clone());
            return Ok(user);
        }

        users.insert(user.username.clone(), user.clone());
        Ok(user)
    }

    async fn delete(&self, username: &str) -> AppResult<()> {
        let mut users = self.users_by_username.write().await;
        if users.remove(username).is_none() {
            return Err(AppError::NotFound(format!(
                "User with username '{}' not found",
                username
            )));
        }
        Ok(())
    }

    async fn ensure_indexes(&self) -> AppResult<()> {
        Ok(())
    }
}

fn make_quiz(id: &str, name: &str, user_id: &str) -> tento_server::models::domain::Quiz {
    let mut quiz = tento_server::models::domain::Quiz::new_draft(name, user_id, 5, 70, 3, "https://example.com");
    quiz.id = id.to_string();
    quiz
}

fn make_attempt(id: &str, user_id: &str, quiz_id: &str, attempt_number: i16) -> QuizAttempt {
    QuizAttempt {
        id: id.to_string(),
        user_id: user_id.to_string(),
        quiz_id: quiz_id.to_string(),
        points_earned: 1,
        required_score: 1,
        total_possible: 1,
        passed: true,
        attempt_number,
        question_answers: vec![QuizAttemptQuestion {
            id: format!("qa-{}", id),
            quiz_question_id: "q1".to_string(),
            selected_option_ids: vec!["o1".to_string()],
            is_correct: true,
            points_earned: 1,
        }],
        submitted_at: Utc::now(),
        created_at: Some(Utc::now()),
        modified_at: Some(Utc::now()),
    }
}

fn make_user(username: &str, github_id: Option<&str>) -> User {
    User {
        id: Some(ObjectId::new()),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        username: username.to_string(),
        email: format!("{}@example.com", username),
        github_id: github_id.map(|v| v.to_string()),
        role: Default::default(),
        created_at: Some(Utc::now()),
    }
}

#[tokio::test]
async fn quiz_repository_crud_and_error_paths() {
    let repo = InMemoryQuizRepository::new();

    let quiz1 = make_quiz("quiz-1", "Quiz One", "user-a");
    let quiz2 = make_quiz("quiz-2", "Quiz Two", "user-a");

    let created1 = repo.create_quiz_draft(quiz1.clone()).await.expect("create quiz1");
    assert_eq!(created1.id, "quiz-1");

    repo.create_quiz_draft(quiz2.clone()).await.expect("create quiz2");

    let duplicate = repo.create_quiz_draft(quiz1.clone()).await;
    assert!(matches!(duplicate, Err(AppError::AlreadyExists(_))));

    let found = repo.find_by_id("quiz-1").await.expect("find should work");
    assert!(found.is_some());

    let (page, total) = repo.list_quizzes(0, 1).await.expect("pagination should work");
    assert_eq!(total, 2);
    assert_eq!(page.len(), 1);

    let (user_page, user_total) = repo
        .list_quizzes_by_user("user-a", 0, 10)
        .await
        .expect("user pagination should work");
    assert_eq!(user_total, 2);
    assert_eq!(user_page.len(), 2);

    let draft_status = repo
        .get_by_status_by_id("quiz-1", "draft")
        .await
        .expect("status query should work");
    assert!(draft_status.is_some());

    let missing_status = repo
        .get_by_status_by_id("quiz-1", "ready")
        .await
        .expect("status query should work");
    assert!(missing_status.is_none());

    let mut quiz1_updated = quiz1.clone();
    quiz1_updated.name = "Updated Quiz One".to_string();
    let updated = repo.update(quiz1_updated.clone()).await.expect("update should work");
    assert_eq!(updated.name, "Updated Quiz One");

    let missing_update = repo.update(make_quiz("quiz-missing", "Missing", "user-z")).await;
    assert!(matches!(missing_update, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn quiz_attempt_repository_crud_counts_and_error_paths() {
    let repo = InMemoryQuizAttemptRepository::new();

    let attempt1 = make_attempt("attempt-1", "user-a", "quiz-1", 1);
    let attempt2 = make_attempt("attempt-2", "user-a", "quiz-1", 2);
    let attempt3 = make_attempt("attempt-3", "user-a", "quiz-2", 1);

    repo.create(attempt1.clone()).await.expect("create attempt1");
    repo.create(attempt2.clone()).await.expect("create attempt2");
    repo.create(attempt3.clone()).await.expect("create attempt3");

    let duplicate = repo.create(attempt1.clone()).await;
    assert!(matches!(duplicate, Err(AppError::AlreadyExists(_))));

    let found = repo.find_by_id("attempt-1").await.expect("find should work");
    assert!(found.is_some());

    let by_user_quiz = repo
        .find_by_user_and_quiz("user-a", "quiz-1")
        .await
        .expect("query should work");
    assert_eq!(by_user_quiz.len(), 2);

    let attempted = repo
        .has_user_attempted_quiz("user-a", "quiz-1")
        .await
        .expect("has attempted should work");
    assert!(attempted);

    let count = repo
        .count_user_attempts("user-a", "quiz-1")
        .await
        .expect("count should work");
    assert_eq!(count, 2);

    let (all_for_user, total_all) = repo
        .get_user_attempts("user-a", None, 0, 10)
        .await
        .expect("paginated list should work");
    assert_eq!(total_all, 3);
    assert_eq!(all_for_user.len(), 3);

    let (filtered, total_filtered) = repo
        .get_user_attempts("user-a", Some("quiz-1"), 0, 10)
        .await
        .expect("filtered list should work");
    assert_eq!(total_filtered, 2);
    assert_eq!(filtered.len(), 2);
}

#[tokio::test]
async fn user_repository_crud_upsert_and_error_paths() {
    let repo = InMemoryUserRepository::new();

    let user1 = make_user("alice", Some("gh-1"));
    let user2 = make_user("bob", Some("gh-2"));

    repo.create(user1.clone()).await.expect("create user1");
    repo.create(user2.clone()).await.expect("create user2");

    let duplicate_username = repo.create(make_user("alice", Some("gh-3"))).await;
    assert!(matches!(duplicate_username, Err(AppError::AlreadyExists(_))));

    let duplicate_github = repo.create(make_user("charlie", Some("gh-1"))).await;
    assert!(matches!(duplicate_github, Err(AppError::AlreadyExists(_))));

    let found = repo
        .find_by_username("alice")
        .await
        .expect("find by username should work");
    assert!(found.is_some());

    let id = user1.id.expect("user should have id").to_hex();
    let found_by_id = repo.find_by_id(&id).await.expect("find by id should work");
    assert!(found_by_id.is_some());

    let updated = repo
        .update("alice", doc! { "$set": { "first_name": "AliceUpdated", "email": "alice.updated@example.com" } })
        .await
        .expect("update should work");
    assert_eq!(updated.first_name, "AliceUpdated");
    assert_eq!(updated.email, "alice.updated@example.com");

    let missing_update = repo
        .update("missing-user", doc! { "$set": { "first_name": "Nobody" } })
        .await;
    assert!(matches!(missing_update, Err(AppError::NotFound(_))));

    let missing_delete = repo.delete("missing-user").await;
    assert!(matches!(missing_delete, Err(AppError::NotFound(_))));

    let no_github = repo.upsert_by_github_id(make_user("no-gh", None)).await;
    assert!(matches!(no_github, Err(AppError::ValidationError(_))));

    let mut upsert_existing = make_user("alice-updated-username", Some("gh-1"));
    upsert_existing.first_name = "AliceUpserted".to_string();
    let upserted = repo
        .upsert_by_github_id(upsert_existing.clone())
        .await
        .expect("upsert should work");
    assert_eq!(upserted.first_name, "AliceUpserted");
    assert_eq!(upserted.github_id.as_deref(), Some("gh-1"));

    repo.delete("bob").await.expect("delete should work");
    let deleted_user = repo
        .find_by_username("bob")
        .await
        .expect("find after delete should work");
    assert!(deleted_user.is_none());
}
