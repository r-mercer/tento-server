use async_trait::async_trait;
use mongodb::{bson::doc, Collection};
use uuid::Uuid;

use crate::{db::Database, errors::AppResult, models::domain::Quiz};

#[async_trait]
pub trait QuizRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Quiz>>;
    async fn create_quiz_draft(&self, quiz: Quiz) -> AppResult<Quiz>;
}

pub struct MongoQuizRepository {
    collection: Collection<Quiz>,
}

impl MongoQuizRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.get_collection("quizzes");
        Self { collection }
    }
}

#[async_trait]
impl QuizRepository for MongoQuizRepository {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<Quiz>> {
        let quiz = self
            .collection
            .find_one(doc! { "id": id.to_string() })
            .await?;
        Ok(quiz)
    }

    async fn create_quiz_draft(&self, quiz: Quiz) -> AppResult<Quiz> {
        self.collection.insert_one(&quiz).await?;
        Ok(quiz)
    }
}
