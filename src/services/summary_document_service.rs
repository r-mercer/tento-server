use std::sync::Arc;

use chrono::Utc;

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

    pub async fn get_summary_document(&self, id: &str) -> AppResult<SummaryDocument> {
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
        mut document: SummaryDocument,
    ) -> AppResult<SummaryDocument> {
        if document.content.trim().is_empty() {
            return Err(AppError::ValidationError(
                "Summary document content cannot be empty".to_string(),
            ));
        }

        let now = Utc::now();
        if document.created_at.is_none() {
            document.created_at = Some(now);
        }
        if document.modified_at.is_none() {
            document.modified_at = Some(now);
        }

        let created_document = self.repository.create(document).await?;
        Ok(created_document)
    }
}
