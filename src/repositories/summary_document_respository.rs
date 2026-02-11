use async_trait::async_trait;
use mongodb::{bson::doc, options::IndexOptions, Collection, IndexModel};
use uuid::Uuid;

use crate::{db::Database, errors::AppResult, models::domain::summary_document::SummaryDocument};

#[async_trait]
pub trait SummaryDocumentRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<SummaryDocument>>;
    async fn create(&self, document: SummaryDocument) -> AppResult<SummaryDocument>;
}

pub struct MongoSummaryDocumentRepository {
    collection: Collection<SummaryDocument>,
}

impl MongoSummaryDocumentRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.get_collection("summary_documents");
        Self { collection }
    }

    pub async fn ensure_indexes(&self) -> AppResult<()> {
        log::info!("Creating indexes for summary_documents collection");

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

        log::info!("Successfully created indexes for summary_documents collection");
        Ok(())
    }
}

#[async_trait]
impl SummaryDocumentRepository for MongoSummaryDocumentRepository {
    async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<SummaryDocument>> {
        let document = self
            .collection
            .find_one(doc! { "id": id.to_string() })
            .await?;
        Ok(document)
    }

    async fn create(&self, document: SummaryDocument) -> AppResult<SummaryDocument> {
        self.collection.insert_one(&document).await?;
        Ok(document)
    }
}
