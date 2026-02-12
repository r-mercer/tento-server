use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson::doc, options::IndexOptions, Collection, IndexModel};

use crate::{db::Database, errors::AppResult, models::domain::quiz_attempt::QuizAttempt};

#[async_trait]
pub trait QuizAttemptRepository: Send + Sync {
    async fn create(&self, attempt: QuizAttempt) -> AppResult<QuizAttempt>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<QuizAttempt>>;
    async fn find_by_user_and_quiz(
        &self,
        user_id: &str,
        quiz_id: &str,
    ) -> AppResult<Vec<QuizAttempt>>;
    async fn has_user_attempted_quiz(
        &self,
        user_id: &str,
        quiz_id: &str,
    ) -> AppResult<bool>;
    async fn count_user_attempts(&self, user_id: &str, quiz_id: &str) -> AppResult<usize>;
    async fn get_user_attempts(
        &self,
        user_id: &str,
        quiz_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<QuizAttempt>, i64)>;
}

pub struct MongoQuizAttemptRepository {
    collection: Collection<QuizAttempt>,
}

impl MongoQuizAttemptRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.get_collection("quiz_attempts");
        Self { collection }
    }

    pub async fn ensure_indexes(&self) -> AppResult<()> {
        log::info!("Creating indexes for quiz_attempts collection");

        let id_index = IndexModel::builder()
            .keys(doc! { "id": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name("id_unique".to_string())
                    .build(),
            )
            .build();

        let user_quiz_index = IndexModel::builder()
            .keys(doc! { "user_id": 1, "quiz_id": 1 })
            .options(
                IndexOptions::builder()
                    .name("user_quiz".to_string())
                    .build(),
            )
            .build();

        let user_id_index = IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .options(
                IndexOptions::builder()
                    .name("user_id".to_string())
                    .build(),
            )
            .build();

        self.collection.create_index(id_index).await?;
        self.collection.create_index(user_quiz_index).await?;
        self.collection.create_index(user_id_index).await?;

        log::info!("Successfully created indexes for quiz_attempts collection");
        Ok(())
    }
}

#[async_trait]
impl QuizAttemptRepository for MongoQuizAttemptRepository {
    async fn create(&self, attempt: QuizAttempt) -> AppResult<QuizAttempt> {
        self.collection.insert_one(&attempt).await?;
        Ok(attempt)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<QuizAttempt>> {
        let attempt = self
            .collection
            .find_one(doc! { "id": id })
            .await?;
        Ok(attempt)
    }

    async fn find_by_user_and_quiz(
        &self,
        user_id: &str,
        quiz_id: &str,
    ) -> AppResult<Vec<QuizAttempt>> {
        let attempts = self
            .collection
            .find(
                doc! {
                    "user_id": user_id,
                    "quiz_id": quiz_id
                },
            )
            .await?
            .try_collect()
            .await?;
        Ok(attempts)
    }

    async fn has_user_attempted_quiz(
        &self,
        user_id: &str,
        quiz_id: &str,
    ) -> AppResult<bool> {
        let attempt = self
            .collection
            .find_one(
                doc! {
                    "user_id": user_id,
                    "quiz_id": quiz_id
                },
            )
            .await?;
        Ok(attempt.is_some())
    }

    async fn count_user_attempts(&self, user_id: &str, quiz_id: &str) -> AppResult<usize> {
        let count = self
            .collection
            .count_documents(
                doc! {
                    "user_id": user_id,
                    "quiz_id": quiz_id
                },
            )
            .await?;
        Ok(count as usize)
    }

    async fn get_user_attempts(
        &self,
        user_id: &str,
        quiz_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<QuizAttempt>, i64)> {
        let mut filter = doc! { "user_id": user_id };

        if let Some(qid) = quiz_id {
            filter.insert("quiz_id", qid);
        }

        let total = self.collection.count_documents(filter.clone()).await?;

        let attempts = self
            .collection
            .find(filter)
            .skip(offset as u64)
            .limit(limit)
            .sort(doc! { "submitted_at": -1 })
            .await?
            .try_collect()
            .await?;

        Ok((attempts, total as i64))
    }
}
