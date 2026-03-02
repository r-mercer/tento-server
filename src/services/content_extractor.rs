use readability::extractor;
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;

const CHUNK_SIZE: usize = 2500;

pub struct ContentExtractor {
    http_client: Client,
}

#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub text_content: String,
    pub tables: Vec<TableContent>,
    pub headings: Vec<HeadingContent>,
    pub metadata: ContentMetadata,
}

#[derive(Debug, Clone)]
pub struct ContentMetadata {
    pub source_url: String,
    pub extraction_timestamp: String,
    pub content_status: ContentStatus,
    pub original_length: usize,
    pub extracted_length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentStatus {
    Complete,
    Partial,
    Inaccessible,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct TableContent {
    pub caption: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct HeadingContent {
    pub level: u8,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ContentChunk {
    pub index: usize,
    pub total_chunks: usize,
    pub content: String,
    pub is_first: bool,
}

impl ContentExtractor {
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("tento-server/1.0 (content extraction)")
            .build()
            .expect("Failed to create HTTP client");

        Self { http_client }
    }

    pub async fn extract(&self, url: &str) -> Result<ExtractedContent, String> {
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch URL: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP error: {} - {}", status, url));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        let html = String::from_utf8_lossy(&bytes);
        let original_length = html.len();

        let parsed_url = reqwest::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        let mut cursor = std::io::Cursor::new(html.as_bytes());
        let article = extractor::extract(&mut cursor, &parsed_url)
            .map_err(|e| format!("Readability extraction failed: {}", e))?;

        let document = Html::parse_fragment(&article.content);
        
        let title = article.title.clone();
        let tables = self.extract_tables(&document);
        let headings = self.extract_headings(&document);
        let text_content = self.extract_structured_text(&document, &tables);

        let extracted_length = text_content.len();
        let content_status = if extracted_length < 1000 {
            ContentStatus::Partial
        } else {
            ContentStatus::Complete
        };

        Ok(ExtractedContent {
            title,
            text_content,
            tables,
            headings,
            metadata: ContentMetadata {
                source_url: url.to_string(),
                extraction_timestamp: chrono::Utc::now().to_rfc3339(),
                content_status,
                original_length,
                extracted_length,
            },
        })
    }

    fn extract_tables(&self, document: &Html) -> Vec<TableContent> {
        let table_selector = Selector::parse("table").unwrap();
        let mut tables = Vec::new();

        for table in document.select(&table_selector) {
            let caption = table
                .select(&Selector::parse("caption").unwrap())
                .next()
                .map(|c| c.text().collect::<String>().trim().to_string());

            let header_selector = Selector::parse("thead th, thead td").unwrap();
            let headers: Vec<String> = table
                .select(&header_selector)
                .map(|h| h.text().collect::<String>().trim().to_string())
                .collect();

            let row_selector = Selector::parse("tbody tr, tr").unwrap();
            let mut rows = Vec::new();
            for row in table.select(&row_selector) {
                let cell_selector = Selector::parse("td, th").unwrap();
                let cells: Vec<String> = row
                    .select(&cell_selector)
                    .map(|c| c.text().collect::<String>().trim().to_string())
                    .collect();
                if !cells.is_empty() {
                    rows.push(cells);
                }
            }

            if !rows.is_empty() || headers.iter().any(|h| !h.is_empty()) {
                tables.push(TableContent {
                    caption,
                    headers,
                    rows,
                });
            }
        }

        tables
    }

    fn extract_headings(&self, document: &Html) -> Vec<HeadingContent> {
        let mut headings = Vec::new();
        
        for level in 1..=6 {
            let selector = Selector::parse(&format!("h{}", level)).unwrap();
            for heading in document.select(&selector) {
                let text = heading.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    headings.push(HeadingContent {
                        level,
                        text,
                    });
                }
            }
        }

        headings
    }

    fn extract_structured_text(&self, document: &Html, tables: &[TableContent]) -> String {
        let mut output = String::new();

        let mut current_section = String::new();
        
        for level in 1..=6 {
            let selector = Selector::parse(&format!("h{}", level)).unwrap();
            for heading in document.select(&selector) {
                let heading_text = heading.text().collect::<String>().trim().to_string();
                
                if !current_section.is_empty() {
                    output.push_str(&current_section);
                    output.push_str("\n\n");
                }
                
                output.push_str(&format!("### {}\n", heading_text));
                current_section.clear();
                
                let mut sibling = heading.next_sibling();
                while let Some(node) = sibling {
                    if let Some(element) = node.value().as_element() {
                        let tag_name = element.name();
                        if tag_name.starts_with('h') && tag_name.len() == 2 {
                            break;
                        }
                    }
                    
                    if let Some(text) = node.value().as_text() {
                        let text = text.trim();
                        if !text.is_empty() {
                            current_section.push_str(text);
                            current_section.push(' ');
                        }
                    }
                    sibling = node.next_sibling();
                }
            }
        }

        if !current_section.is_empty() {
            output.push_str(&current_section);
        }

        for table in tables {
            if !table.rows.is_empty() || !table.headers.is_empty() {
                output.push_str("\n\n### Tables\n");
                if let Some(ref caption) = table.caption {
                    output.push_str(&format!("#### {}\n", caption));
                }
                
                if !table.headers.is_empty() {
                    output.push_str(&table.headers.join(" | "));
                    output.push_str("\n");
                    output.push_str(&table.headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
                    output.push_str("\n");
                }
                
                for row in &table.rows {
                    output.push_str(&row.join(" | "));
                    output.push_str("\n");
                }
            }
        }

        output.trim().to_string()
    }

    pub fn chunk_content(&self, content: &ExtractedContent) -> Vec<ContentChunk> {
        let text = &content.text_content;
        
        if text.len() <= CHUNK_SIZE {
            return vec![ContentChunk {
                index: 0,
                total_chunks: 1,
                content: text.clone(),
                is_first: true,
            }];
        }

        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_len = chars.len();
        let chunk_count = (total_len / CHUNK_SIZE) + if total_len % CHUNK_SIZE > 0 { 1 } else { 0 };

        let overlap = 200;

        let mut start = 0;
        let mut chunk_index = 0;

        while start < total_len {
            let mut end = start + CHUNK_SIZE;
            if end > total_len {
                end = total_len;
            } else {
                if let Some(space_pos) = chars[start..end].iter().rposition(|&c| c == ' ' || c == '\n') {
                    end = start + space_pos;
                }
            }

            let chunk_text: String = chars[start..end].iter().collect();
            
            let is_first = chunk_index == 0;
            
            let final_content = if is_first {
                format!(
                    "[CONTENT_START]\n{}\n[END_FIRST_CHUNK]\n{}",
                    content.title,
                    chunk_text
                )
            } else {
                format!(
                    "[CHUNK_{}]\n{}",
                    chunk_index,
                    chunk_text
                )
            };

            chunks.push(ContentChunk {
                index: chunk_index,
                total_chunks: chunk_count,
                content: final_content,
                is_first,
            });

            start = end - overlap;
            chunk_index += 1;
        }

        chunks
    }

    pub fn format_for_llm(&self, content: &ExtractedContent, chunks: &[ContentChunk]) -> String {
        let mut output = String::new();

        output.push_str(&format!("[SOURCE_URL] {}\n", content.metadata.source_url));
        output.push_str(&format!(
            "[RETRIEVAL_TIMESTAMP] {}\n",
            content.metadata.extraction_timestamp
        ));
        
        let status_str = match content.metadata.content_status {
            ContentStatus::Complete => "complete",
            ContentStatus::Partial => "partial",
            ContentStatus::Inaccessible => "inaccessible",
            ContentStatus::Error(ref msg) => {
                output.push_str(&format!("[LIMITATIONS] {}\n", msg));
                "partial"
            }
        };
        output.push_str(&format!("[CONTENT_STATUS] {}\n", status_str));
        output.push_str(&format!(
            "[CONTENT_LENGTH] {} chars (original: {})\n\n",
            content.metadata.extracted_length, content.metadata.original_length
        ));

        if !content.headings.is_empty() {
            output.push_str("### DOCUMENT STRUCTURE\n");
            for h in &content.headings {
                let indent = "  ".repeat(h.level.saturating_sub(1) as usize);
                output.push_str(&format!("{}- {}\n", indent, h.text));
            }
            output.push_str("\n");
        }

        output.push_str("### CONTENT CHUNKS\n");
        for chunk in chunks {
            output.push_str(&chunk.content);
            output.push_str("\n\n");
        }

        if !content.tables.is_empty() {
            output.push_str("### TABLES\n");
            for (i, table) in content.tables.iter().enumerate() {
                if let Some(ref caption) = table.caption {
                    output.push_str(&format!("Table {}: {}\n", i + 1, caption));
                } else {
                    output.push_str(&format!("Table {}\n", i + 1));
                }
                
                if !table.headers.is_empty() {
                    output.push_str(&table.headers.join(" | "));
                    output.push_str("\n");
                    output.push_str(&table.headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
                    output.push_str("\n");
                }
                
                for row in &table.rows {
                    output.push_str(&row.join(" | "));
                    output.push_str("\n");
                }
                output.push_str("\n");
            }
        }

        output
    }
}

impl Default for ContentExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_content_small() {
        let extractor = ContentExtractor::new();
        let content = ExtractedContent {
            title: "Test".to_string(),
            text_content: "Short content".to_string(),
            tables: vec![],
            headings: vec![],
            metadata: ContentMetadata {
                source_url: "http://test.com".to_string(),
                extraction_timestamp: "2024-01-01T00:00:00Z".to_string(),
                content_status: ContentStatus::Complete,
                original_length: 100,
                extracted_length: 14,
            },
        };

        let chunks = extractor.chunk_content(&content);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].is_first);
    }

    #[test]
    fn test_chunk_content_large() {
        let extractor = ContentExtractor::new();
        let long_text = "a".repeat(6000);
        let content = ExtractedContent {
            title: "Test".to_string(),
            text_content: long_text,
            tables: vec![],
            headings: vec![],
            metadata: ContentMetadata {
                source_url: "http://test.com".to_string(),
                extraction_timestamp: "2024-01-01T00:00:00Z".to_string(),
                content_status: ContentStatus::Complete,
                original_length: 6000,
                extracted_length: 6000,
            },
        };

        let chunks = extractor.chunk_content(&content);
        assert!(chunks.len() > 1);
    }
}
