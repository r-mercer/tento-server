use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::domain::summary_document::SummaryDocument,
    repositories::SummaryDocumentRepository,
};

pub struct SummaryDocumentService {
    repository: Arc<dyn SummaryDocumentRepository>,
}

impl SummaryDocumentService {
    pub fn new(repository: Arc<dyn SummaryDocumentRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_summary_document(&self, id: &Uuid) -> AppResult<SummaryDocument> {
        let document = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Summary document with id '{}' not found", id))
            })?;

        Ok(document)
    }

    pub async fn create_summary_document(
        &self,
        document: SummaryDocument,
    ) -> AppResult<SummaryDocument> {
        let created_document = self.repository.create(document).await?;
        Ok(created_document)
    }
}
