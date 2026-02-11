use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use secrecy::ExposeSecret;
use serde_json::{json, Value};

use crate::{
    config::Config,
    constants::{prompts::QUIZ_GENERATOR_PROMPT, WEBSITE_SUMMARISER_PROMPT},
    errors::{AppError, AppResult},
    models::domain::{
        summary_document::SummaryDocument,
        Quiz,
    },
};

pub struct ModelService {
    client: Client<OpenAIConfig>,
}

impl ModelService {
    pub fn new(config: &Config) -> Self {
        let openai_config = OpenAIConfig::new()
            .with_api_key(config.openai_api_key.expose_secret())
            .with_api_base(&config.openai_base_url);

        let client = Client::with_config(openai_config);
        // let summariser_client = Client::with_config(openai_config);

        Self { client }
    }

    pub async fn chat_completion(&self, prompt: &str, model: &str) -> AppResult<String> {
        let message = ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()
            .map_err(|e| AppError::InternalError(format!("Failed to build chat message: {}", e)))?;

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(vec![ChatCompletionRequestMessage::User(message)])
            .build()
            .map_err(|e| AppError::InternalError(format!("Failed to build chat request: {}", e)))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| AppError::InternalError(format!("LLM request failed: {}", e)))?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or_else(|| AppError::InternalError("No response from LLM".to_string()))?;

        Ok(content)
    }

    pub async fn website_summariser(&self) -> AppResult<String> {
        let summ_client = Client::new();
        let url_string = "";
        let response: Value = summ_client
            .chat()
            .create_byot(json!({
                "messages": [
                    {
                        "role": "website_content_summariser",
                        "content": WEBSITE_SUMMARISER_PROMPT
                    },
                    {
                        "role": "content_url",
                        "content": url_string
                    }
                ],
                "model": "liquid/lfm2-1.2b",
                "store": false
            }))
            .await?;

        // if let Some(content) = response.output_text().to_string() {
        // Ok(response.output_text())
        println!("response: {}", response);
        Ok(response.to_string())
    }

    pub async fn quiz_generator(
        &self,
        quiz: Quiz,
        summary_document: SummaryDocument,
    ) -> AppResult<String> {
        let summ_client = Client::new();
        let response: Value = summ_client
            .chat()
            .create_byot(json!({
                "messages": [
                    {
                        "role": "quiz_generator",
                        "content": QUIZ_GENERATOR_PROMPT
                    },
                    {
                        "role": "quiz_draft",
                        "content": quiz
                    },
                    {
                        "role": "summary_document",
                        "content": summary_document
                    }
                ],
                "model": "liquid/lfm2-1.2b",
                "store": false
            }))
            .await?;

        // if let Some(content) = response.output_text().to_string() {
        // Ok(response.output_text())
        println!("response: {}", response);
        Ok(response.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_service_creation() {
        let config = Config::test_config();
        let service = ModelService::new(&config);

        assert!(std::mem::size_of_val(&service) > 0);
    }
}
