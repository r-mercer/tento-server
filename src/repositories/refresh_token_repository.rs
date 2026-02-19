use async_trait::async_trait;
use log::info;
use mongodb::{
    bson::{doc, DateTime as BsonDateTime},
    options::IndexOptions,
    Collection, IndexModel,
};

use crate::{
    db::Database,
    errors::{AppError, AppResult},
    models::domain::RefreshToken,
};

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn create(&self, token: RefreshToken) -> AppResult<RefreshToken>;
    async fn find_by_token_hash(&self, hash: &str) -> AppResult<Option<RefreshToken>>;
    async fn revoke_by_token_hash(&self, hash: &str) -> AppResult<()>;
    async fn revoke_all_for_user(&self, user_id: &str) -> AppResult<u64>;
    async fn delete_expired(&self) -> AppResult<u64>;
    async fn ensure_indexes(&self) -> AppResult<()>;
}

pub struct MongoRefreshTokenRepository {
    collection: Collection<RefreshToken>,
}

impl MongoRefreshTokenRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.get_collection("refresh_tokens");
        Self { collection }
    }
}

#[async_trait]
impl RefreshTokenRepository for MongoRefreshTokenRepository {
    async fn create(&self, token: RefreshToken) -> AppResult<RefreshToken> {
        self.collection.insert_one(&token).await?;
        Ok(token)
    }

    async fn find_by_token_hash(&self, hash: &str) -> AppResult<Option<RefreshToken>> {
        let token = self
            .collection
            .find_one(doc! { "token_hash": hash })
            .await?;
        Ok(token)
    }

    async fn revoke_by_token_hash(&self, hash: &str) -> AppResult<()> {
        let result = self
            .collection
            .update_one(
                doc! { "token_hash": hash },
                doc! { "$set": { "revoked": true } },
            )
            .await?;

        if result.matched_count == 0 {
            return Err(AppError::NotFound("Refresh token not found".to_string()));
        }

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: &str) -> AppResult<u64> {
        let result = self
            .collection
            .update_many(
                doc! { "user_id": user_id, "revoked": false },
                doc! { "$set": { "revoked": true } },
            )
            .await?;

        Ok(result.modified_count)
    }

    async fn delete_expired(&self) -> AppResult<u64> {
        let now = BsonDateTime::now();
        let result = self
            .collection
            .delete_many(doc! { "expires_at": { "$lt": now } })
            .await?;

        Ok(result.deleted_count)
    }

    async fn ensure_indexes(&self) -> AppResult<()> {
        let token_hash_options = IndexOptions::builder().unique(true).build();
        let token_hash_model = IndexModel::builder()
            .keys(doc! { "token_hash": 1 })
            .options(token_hash_options)
            .build();
        self.collection.create_index(token_hash_model).await?;
        info!("Created unique index on refresh_tokens.token_hash");

        let user_id_model = IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .build();
        self.collection.create_index(user_id_model).await?;
        info!("Created index on refresh_tokens.user_id");

        let expires_at_model = IndexModel::builder()
            .keys(doc! { "expires_at": 1 })
            .build();
        self.collection.create_index(expires_at_model).await?;
        info!("Created index on refresh_tokens.expires_at");

        Ok(())
    }
}
