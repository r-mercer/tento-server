use async_trait::async_trait;
use mongodb::{bson::doc, options::IndexOptions, Collection, IndexModel};

use crate::{db::Database, errors::AppResult, models::domain::Quiz};

#[async_trait]
pub trait QuizRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Quiz>>;
    async fn list_quizzes(&self, offset: i64, limit: i64) -> AppResult<(Vec<Quiz>, i64)>;
    async fn list_quizzes_by_user(&self, user_id: &str, offset: i64, limit: i64) -> AppResult<(Vec<Quiz>, i64)>;
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

    async fn list_quizzes(&self, offset: i64, limit: i64) -> AppResult<(Vec<Quiz>, i64)> {
        use futures::TryStreamExt;
        use mongodb::options::FindOptions;

        // Get total count
        let total = self.collection.count_documents(doc! {}).await? as i64;

        // Build find options
        let find_options = FindOptions::builder()
            .skip(Some(offset as u64))
            .limit(Some(limit))
            .build();

        let cursor = self.collection.find(doc! {}).with_options(find_options).await?;
        let items: Vec<Quiz> = cursor.try_collect().await?;

        Ok((items, total))
    }

    async fn list_quizzes_by_user(&self, user_id: &str, offset: i64, limit: i64) -> AppResult<(Vec<Quiz>, i64)> {
        use futures::TryStreamExt;
        use mongodb::options::FindOptions;

        let filter = doc! { "created_by_user_id": user_id };

        // Get total count for this filter
        let total = self.collection.count_documents(filter.clone()).await? as i64;

        let find_options = FindOptions::builder()
            .skip(Some(offset as u64))
            .limit(Some(limit))
            .build();

        let cursor = self.collection.find(filter.clone()).with_options(find_options).await?;
        let items: Vec<Quiz> = cursor.try_collect().await?;

        Ok((items, total))
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
