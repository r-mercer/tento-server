use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use secrecy::ExposeSecret;

use crate::{
    config::Config,
    errors::{AppError, AppResult},
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
