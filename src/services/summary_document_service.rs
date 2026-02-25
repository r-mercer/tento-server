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
        let document = self.repository.find_by_id(id).await?.ok_or_else(|| {
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

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use chrono::Utc;
    use mockall::mock;

    use crate::repositories::SummaryDocumentRepository;

    use super::*;

    mock! {
        pub SummaryRepo {}

        #[async_trait]
        impl SummaryDocumentRepository for SummaryRepo {
            async fn find_by_id(&self, id: &str) -> AppResult<Option<SummaryDocument>>;
            async fn create(&self, document: SummaryDocument) -> AppResult<SummaryDocument>;
        }
    }

    fn make_document(content: &str) -> SummaryDocument {
        SummaryDocument {
            id: "doc-1".to_string(),
            quiz_id: "quiz-1".to_string(),
            url: "https://example.com".to_string(),
            content: content.to_string(),
            created_at: None,
            modified_at: None,
        }
    }

    #[tokio::test]
    async fn get_summary_document_returns_not_found_for_missing_id() {
        let mut mock_repo = MockSummaryRepo::new();
        mock_repo.expect_find_by_id().returning(|_| Ok(None));

        let service = SummaryDocumentService::new(Arc::new(mock_repo));
        let result = service.get_summary_document("missing").await;

        assert!(result.is_err());
        match result.expect_err("expected not found") {
            AppError::NotFound(msg) => {
                assert!(msg.contains("Summary document with id 'missing' not found"))
            }
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn create_summary_document_rejects_empty_content() {
        let mock_repo = MockSummaryRepo::new();
        let service = SummaryDocumentService::new(Arc::new(mock_repo));

        let document = make_document("   ");
        let result = service.create_summary_document(document).await;

        assert!(result.is_err());
        match result.expect_err("expected validation error") {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "Summary document content cannot be empty")
            }
            other => panic!("expected ValidationError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn create_summary_document_sets_missing_timestamps() {
        let mut mock_repo = MockSummaryRepo::new();
        mock_repo.expect_create().returning(|document| {
            assert!(document.created_at.is_some());
            assert!(document.modified_at.is_some());
            Ok(document)
        });

        let service = SummaryDocumentService::new(Arc::new(mock_repo));
        let result = service
            .create_summary_document(make_document("Valid summary content"))
            .await
            .expect("expected create success");

        assert!(result.created_at.is_some());
        assert!(result.modified_at.is_some());
    }

    #[tokio::test]
    async fn create_summary_document_preserves_existing_timestamps() {
        let created = Utc::now();
        let modified = created + chrono::Duration::minutes(5);

        let mut mock_repo = MockSummaryRepo::new();
        mock_repo.expect_create().returning(move |document| {
            assert_eq!(document.created_at, Some(created));
            assert_eq!(document.modified_at, Some(modified));
            Ok(document)
        });

        let service = SummaryDocumentService::new(Arc::new(mock_repo));

        let mut document = make_document("Valid summary content");
        document.created_at = Some(created);
        document.modified_at = Some(modified);

        let result = service
            .create_summary_document(document)
            .await
            .expect("expected create success");

        assert_eq!(result.created_at, Some(created));
        assert_eq!(result.modified_at, Some(modified));
    }
}
