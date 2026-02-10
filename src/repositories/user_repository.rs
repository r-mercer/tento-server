use async_trait::async_trait;
use futures::TryStreamExt;
use log::info;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndUpdateOptions, IndexOptions, ReturnDocument},
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
    async fn find_by_github_id(&self, github_id: &str) -> AppResult<Option<User>>;
    async fn find_all(&self) -> AppResult<Vec<User>>;
    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<User>, i64)>;
    async fn update(&self, username: &str, update_doc: Document) -> AppResult<User>;
    async fn upsert_by_github_id(&self, user: User) -> AppResult<User>;
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

    async fn find_by_github_id(&self, github_id: &str) -> AppResult<Option<User>> {
        let user = self
            .collection
            .find_one(doc! { "github_id": github_id })
            .await?;
        Ok(user)
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        let cursor = self.collection.find(doc! {}).await?;
        let users: Vec<User> = cursor.try_collect().await?;
        Ok(users)
    }

    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<User>, i64)> {
        use mongodb::options::FindOptions;
        
        // Get total count
        let total = self.collection.count_documents(doc! {}).await? as i64;
        
        // Get paginated results
        let find_options = FindOptions::builder()
            .skip(Some(offset as u64))
            .limit(Some(limit))
            .build();
        
        let cursor = self.collection.find(doc! {}).with_options(find_options).await?;
        let users: Vec<User> = cursor.try_collect().await?;
        
        Ok((users, total))
    }

    async fn update(&self, username: &str, update_doc: Document) -> AppResult<User> {
        let filter = doc! { "username": username };
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        let updated_user = self
            .collection
            .find_one_and_update(filter, update_doc)
            .with_options(options)
            .await?;

        updated_user.ok_or_else(|| {
            AppError::NotFound(format!("User with username '{}' not found", username))
        })
    }

    async fn upsert_by_github_id(&self, user: User) -> AppResult<User> {
        let github_id = user.github_id.clone().ok_or_else(|| {
            AppError::ValidationError("User must have a github_id for upsert".to_string())
        })?;

        let filter = doc! { "github_id": &github_id };
        let update_doc = doc! {
            "$set": {
                "first_name": &user.first_name,
                "last_name": &user.last_name,
                "username": &user.username,
                "email": &user.email,
                "github_id": &github_id,
                "role": mongodb::bson::to_bson(&user.role)?,
            },
            "$setOnInsert": {
                "created_at": mongodb::bson::to_bson(&user.created_at)?,
            }
        };

        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::After)
            .build();

        let result = self
            .collection
            .find_one_and_update(filter, update_doc)
            .with_options(options)
            .await?;

        result.ok_or_else(|| {
            AppError::InternalError("Failed to upsert user".to_string())
        })
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
        // Unique index on username
        let username_options = IndexOptions::builder().unique(true).build();
        let username_model = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(username_options)
            .build();

        self.collection.create_index(username_model).await?;
        info!("Created unique index on username field");

        // Unique index on github_id (for OAuth lookups)
        let github_options = IndexOptions::builder()
            .unique(true)
            .sparse(true)  // Allow null values since not all users may have github_id
            .build();
        let github_model = IndexModel::builder()
            .keys(doc! { "github_id": 1 })
            .options(github_options)
            .build();

        self.collection.create_index(github_model).await?;
        info!("Created unique sparse index on github_id field");

        Ok(())
    }
}
