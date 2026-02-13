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
    models::domain::{summary_document::SummaryDocument, Quiz},
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

    pub async fn website_summariser(&self, url_string: &str) -> AppResult<String> {
        let summ_client = Client::new();
        let response: Value = summ_client
            .chat()
            .create_byot(json!({
                "messages": [
                    {
                        "role": "system",
                        "content": WEBSITE_SUMMARISER_PROMPT
                    },
                    {
                        "role": "user",
                        "content": url_string
                    }
                ],
                "model": "liquid/lfm2-1.2b",
                "store": false
            }))
            .await?;

        let content = response["choices"][0]["message"]["content"].to_string();
        // if let Some(content) = response.output_text().to_string() {
        // Ok(response.output_text())
        println!("content: {}", content);
        // Ok(response.to_string())

        Ok(content)
    }

    pub async fn quiz_generator(
        &self,
        quiz: Quiz,
        summary_document: SummaryDocument,
    ) -> AppResult<String> {
        let quiz_json = serde_json::to_string(&quiz)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize quiz: {}", e)))?;
        let summary_json = serde_json::to_string(&summary_document).map_err(|e| {
            AppError::InternalError(format!("Failed to serialize summary document: {}", e))
        })?;
        let summ_client = Client::new();
        let response: Value = summ_client
            .chat()
            .create_byot(json!({
                "messages": [
                    {
                        "role": "system",
                        "content": QUIZ_GENERATOR_PROMPT
                    },
                    {
                        "role": "user",
                        "content": quiz_json
                    },
                    {
                        "role": "user",
                        "content": summary_json
                    }
                ],
                "model": "liquid/lfm2-1.2b",
                "store": false
            }))
            .await?;

        // if let Some(content) = response.output_text().to_string() {
        // Ok(response.output_text())
        // response["choices"][0]["message"]["content"].as_str()
        // let content = response["choices"][0]["message"]["content"]

        // println!("response-content: {}", content);

        let content = response["choices"][0]["message"]["content"].to_string();
        // if let Some(content) = response.output_text().to_string() {
        // Ok(response.output_text())
        println!("content: {}", content);
        // Ok(response.to_string())

        Ok(content)
        // Ok(response["choices"][0]["message"]["content"].to_string())
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
