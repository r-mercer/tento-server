use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{bson::doc, options::IndexOptions, Collection, IndexModel};
use uuid::Uuid;

use crate::{db::Database, errors::AppResult, models::domain::quiz_attempt::QuizAttempt};

#[async_trait]
pub trait QuizAttemptRepository: Send + Sync {
    async fn create(&self, attempt: QuizAttempt) -> AppResult<QuizAttempt>;
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<QuizAttempt>>;
    async fn find_by_user_and_quiz(
        &self,
        user_id: Uuid,
        quiz_id: Uuid,
    ) -> AppResult<Vec<QuizAttempt>>;
    async fn has_user_attempted_quiz(
        &self,
        user_id: Uuid,
        quiz_id: Uuid,
    ) -> AppResult<bool>;
    async fn count_user_attempts(&self, user_id: Uuid, quiz_id: Uuid) -> AppResult<usize>;
    async fn get_user_attempts(
        &self,
        user_id: Uuid,
        quiz_id: Option<Uuid>,
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

    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<QuizAttempt>> {
        let attempt = self
            .collection
            .find_one(doc! { "id": id.to_string() })
            .await?;
        Ok(attempt)
    }

    async fn find_by_user_and_quiz(
        &self,
        user_id: Uuid,
        quiz_id: Uuid,
    ) -> AppResult<Vec<QuizAttempt>> {
        let attempts = self
            .collection
            .find(
                doc! {
                    "user_id": user_id.to_string(),
                    "quiz_id": quiz_id.to_string()
                },
            )
            .await?
            .try_collect()
            .await?;
        Ok(attempts)
    }

    async fn has_user_attempted_quiz(
        &self,
        user_id: Uuid,
        quiz_id: Uuid,
    ) -> AppResult<bool> {
        let attempt = self
            .collection
            .find_one(
                doc! {
                    "user_id": user_id.to_string(),
                    "quiz_id": quiz_id.to_string()
                },
            )
            .await?;
        Ok(attempt.is_some())
    }

    async fn count_user_attempts(&self, user_id: Uuid, quiz_id: Uuid) -> AppResult<usize> {
        let count = self
            .collection
            .count_documents(
                doc! {
                    "user_id": user_id.to_string(),
                    "quiz_id": quiz_id.to_string()
                },
            )
            .await?;
        Ok(count as usize)
    }

    async fn get_user_attempts(
        &self,
        user_id: Uuid,
        quiz_id: Option<Uuid>,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<QuizAttempt>, i64)> {
        let mut filter = doc! { "user_id": user_id.to_string() };

        if let Some(qid) = quiz_id {
            filter.insert("quiz_id", qid.to_string());
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
