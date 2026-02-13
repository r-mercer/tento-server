use std::error::Error;

use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ResponseFormat, ResponseFormatJsonSchema,
    },
    Client,
};
use schemars::{schema_for, JsonSchema};
use secrecy::ExposeSecret;
use serde::de::DeserializeOwned;
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

    pub async fn structured_quiz_generator(
        &self,
        quiz: Quiz,
        summary_document: SummaryDocument,
    ) -> AppResult<Quiz> {
        // let quiz_json = serde_json::to_string(&quiz)
        //     .map_err(|e| AppError::InternalError(format!("Failed to serialize quiz: {}", e)))?;
        // let summary_json = serde_json::to_string(&summary_document).map_err(|e| {
        //     AppError::InternalError(format!("Failed to serialize summary document: {}", e))
        // })?;

        match Self::structured_output::<Quiz>(vec![
            ChatCompletionRequestSystemMessage::from(QUIZ_GENERATOR_PROMPT).into(),
            ChatCompletionRequestUserMessage::from(summary_document.content).into(),
        ])
        .await
        {
            Ok(Some(generated_quiz)) => Ok(generated_quiz),
            Ok(None) => Err(AppError::InternalError(
                "LLM did not return a valid quiz".to_string(),
            )),
            Err(e) => Err(AppError::InternalError(format!(
                "Failed to generate structured quiz: {}",
                e
            ))),
        }

        // Ok(response)
    }

    pub async fn structured_output<T: serde::Serialize + DeserializeOwned + JsonSchema>(
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<Option<T>, Box<dyn Error>> {
        let schema = schema_for!(T);
        let schema_value = serde_json::to_value(&schema)?;
        let response_format = ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                description: None,
                name: "math_reasoning".into(),
                schema: Some(schema_value),
                strict: Some(true),
            },
        };

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u32)
            .model("liquid/lfm2-1.2b")
            .messages(messages)
            .response_format(response_format)
            .build()?;

        let client = Client::new();
        let response = client.chat().create(request).await?;

        for choice in response.choices {
            if let Some(content) = choice.message.content {
                return Ok(Some(serde_json::from_str::<T>(&content)?));
            }
        }

        Ok(None)
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
