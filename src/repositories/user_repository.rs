use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{
    bson::doc,
    options::{IndexOptions, ReplaceOptions},
    Collection, IndexModel,
};

use crate::{
    db::Database,
    errors::{AppError, AppResult},
    models::domain::User,
};
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> AppResult<User>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn find_all(&self) -> AppResult<Vec<User>>;
    async fn update(&self, username: &str, user: User) -> AppResult<User>;
    async fn delete(&self, username: &str) -> AppResult<()>;
    async fn ensure_indexes(&self) -> AppResult<()>;
}

pub struct MongoUserRepository {
    collection: Collection<User>,
}

impl MongoUserRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.get_collection("users");
        Self { collection }
    }
}

#[async_trait]
impl UserRepository for MongoUserRepository {
    async fn create(&self, user: User) -> AppResult<User> {
        self.collection.insert_one(&user).await?;
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let user = self
            .collection
            .find_one(doc! { "username": username })
            .await?;
        Ok(user)
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        let cursor = self.collection.find(doc! {}).await?;
        let users: Vec<User> = cursor.try_collect().await?;
        Ok(users)
    }

    async fn update(&self, username: &str, user: User) -> AppResult<User> {
        let filter = doc! { "username": username };
        let options = ReplaceOptions::builder().upsert(false).build();

        let result = self
            .collection
            .replace_one(filter, &user)
            .with_options(options)
            .await?;

        if result.matched_count == 0 {
            return Err(AppError::NotFound(format!(
                "User with username '{}' not found",
                username
            )));
        }

        Ok(user)
    }

    async fn delete(&self, username: &str) -> AppResult<()> {
        let result = self
            .collection
            .delete_one(doc! { "username": username })
            .await?;

        if result.deleted_count == 0 {
            return Err(AppError::NotFound(format!(
                "User with username '{}' not found",
                username
            )));
        }

        Ok(())
    }

    async fn ensure_indexes(&self) -> AppResult<()> {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(options)
            .build();

        self.collection.create_index(model).await?;
        println!("✓ Created unique index on username field");

        Ok(())
    }
}
