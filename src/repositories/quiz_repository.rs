use async_trait::async_trait;
use mongodb::{bson::doc, options::IndexOptions, Collection, IndexModel};

use crate::{db::Database, errors::AppResult, models::domain::Quiz};

#[async_trait]
pub trait QuizRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Quiz>>;
    async fn get_by_status_by_id(&self, id: &str, status: &str) -> AppResult<Option<Quiz>>;
    async fn create_quiz_draft(&self, quiz: Quiz) -> AppResult<Quiz>;
    async fn update(&self, quiz: Quiz) -> AppResult<Quiz>;
}

pub struct MongoQuizRepository {
    collection: Collection<Quiz>,
}

impl MongoQuizRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.get_collection("quizzes");
        Self { collection }
    }

    pub async fn ensure_indexes(&self) -> AppResult<()> {
        log::info!("Creating indexes for quizzes collection");

        let id_index = IndexModel::builder()
            .keys(doc! { "id": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name("id_unique".to_string())
                    .build(),
            )
            .build();

        self.collection.create_index(id_index).await?;

        log::info!("Successfully created indexes for quizzes collection");
        Ok(())
    }
}

#[async_trait]
impl QuizRepository for MongoQuizRepository {
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Quiz>> {
        let quiz = self.collection.find_one(doc! { "id": id }).await?;
        Ok(quiz)
    }

    async fn create_quiz_draft(&self, quiz: Quiz) -> AppResult<Quiz> {
        self.collection.insert_one(&quiz).await?;
        Ok(quiz)
    }
    async fn get_by_status_by_id(&self, id: &str, status: &str) -> AppResult<Option<Quiz>> {
        let quiz = self
            .collection
            .find_one(doc! { "id": id, "status": status  })
            .await?;
        Ok(quiz)
    }

    async fn update(&self, quiz: Quiz) -> AppResult<Quiz> {
        self.collection
            .replace_one(doc! { "id": &quiz.id }, &quiz)
            .await?;
        Ok(quiz)
    }
}
