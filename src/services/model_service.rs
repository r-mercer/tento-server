use std::error::Error;

use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestToolMessage, ChatCompletionRequestUserMessage,
        ChatCompletionRequestUserMessageArgs, ChatCompletionTool, ChatCompletionToolChoiceOption,
        ChatCompletionTools, CreateChatCompletionRequestArgs, FunctionCall, FunctionObject,
        ResponseFormat, ResponseFormatJsonSchema, ToolChoiceOptions,
    },
    Client,
};
use schemars::{schema_for, JsonSchema};
use secrecy::ExposeSecret;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    config::Config,
    constants::{
        prompts::QUIZ_GENERATOR_PROMPT,
        quiz_prompt::{STRUCTURED_QUIZ_GENERATOR_PROMPT, URL_EXTRACTION_PROMPT},
        WEBSITE_SUMMARISER_PROMPT,
    },
    errors::{AppError, AppResult},
    models::dto::request::{GenerateQuizRequestDto, QuizRequestDto, SummaryDocumentRequestDto},
};

pub struct ModelService {
    client: Client<OpenAIConfig>,
}

const TOOL_MAX_ATTEMPTS: u32 = 6;
const TOOL_MAX_CONTENT_LENGTH: usize = 20000;
const STRUCTURED_OUTPUT_MAX_TOKENS: u32 = 4096;

#[derive(Debug, Deserialize)]
struct FetchWebpageArgs {
    url: String,
    query: String,
}

#[derive(Debug, Deserialize)]
struct OpenSimpleBrowserArgs {
    url: String,
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
        let response: Value = self
            .client
            .chat()
            .create_byot(json!({
                "messages": [
                    {
                        "role": "system",
                        "content": URL_EXTRACTION_PROMPT
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
        log::debug!("website_summariser content length: {}", content.len());

        Ok(content)
    }

    pub async fn quiz_generator(
        &self,
        quiz: QuizRequestDto,
        summary_document: SummaryDocumentRequestDto,
    ) -> AppResult<String> {
        let quiz_json = serde_json::to_string(&quiz)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize quiz: {}", e)))?;
        let summary_json = serde_json::to_string(&summary_document).map_err(|e| {
            AppError::InternalError(format!("Failed to serialize summary document: {}", e))
        })?;
        let response: Value = self
            .client
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
        log::debug!("quiz_generator content length: {}", content.len());

        Ok(content)
        // Ok(response["choices"][0]["message"]["content"].to_string())
    }

    pub async fn structured_quiz_generator(
        &self,
        _quiz: QuizRequestDto,
        summary_document: SummaryDocumentRequestDto,
    ) -> AppResult<GenerateQuizRequestDto> {
        match self
            .structured_output::<GenerateQuizRequestDto>(vec![
                ChatCompletionRequestSystemMessage::from(
                    "Tool calls are disabled for structured output. Do not call tools.",
                )
                .into(),
                ChatCompletionRequestSystemMessage::from(STRUCTURED_QUIZ_GENERATOR_PROMPT).into(),
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

    pub async fn structured_summary_document(
        &self,
        url_string: &str,
    ) -> AppResult<SummaryDocumentRequestDto> {
        let tools = vec![
            Self::build_fetch_webpage_tool()?,
            Self::build_open_simple_browser_tool()?,
        ];
        match self
            .structured_output_with_tools::<SummaryDocumentRequestDto>(
                vec![
                    ChatCompletionRequestSystemMessage::from(WEBSITE_SUMMARISER_PROMPT).into(),
                    ChatCompletionRequestUserMessage::from(url_string).into(),
                ],
                tools,
            )
            .await
        {
            Ok(Some(summary_document)) => Ok(summary_document),
            Ok(None) => Err(AppError::InternalError(
                "LLM did not return a valid summary document".to_string(),
            )),
            Err(e) => Err(AppError::InternalError(format!(
                "Failed to generate structured summary document: {}",
                e
            ))),
        }
    }

    pub async fn structured_output<T: serde::Serialize + DeserializeOwned + JsonSchema>(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> Result<Option<T>, Box<dyn Error>> {
        let schema = schema_for!(T);
        let mut schema_value = serde_json::to_value(&schema)?;

        // Inline all $defs to avoid $ref issues with LM Studio's outlines processor
        let defs = if let Some(obj) = schema_value.get("$defs") {
            obj.clone()
        } else {
            Value::Object(Default::default())
        };

        Self::inline_schema_refs(&mut schema_value, &defs);

        if let Some(obj) = schema_value.as_object_mut() {
            obj.remove("$defs");
        }

        let response_format = ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                description: None,
                name: "math_reasoning".into(),
                schema: Some(schema_value),
                strict: Some(true),
            },
        };

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(STRUCTURED_OUTPUT_MAX_TOKENS)
            .model("liquid/lfm2-1.2b")
            .messages(messages)
            .response_format(response_format)
            .build()?;

        let response = self.client.chat().create(request).await?;

        for choice in response.choices {
            if let Some(content) = choice.message.content {
                return Ok(Some(serde_json::from_str::<T>(&content)?));
            }
        }

        Ok(None)
    }

    pub async fn structured_output_with_tools<
        T: serde::Serialize + DeserializeOwned + JsonSchema,
    >(
        &self,
        mut messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTools>,
    ) -> Result<Option<T>, Box<dyn Error>> {
        let mut attempts = 0;

        loop {
            attempts += 1;
            if attempts > TOOL_MAX_ATTEMPTS {
                return Err("Tool loop exceeded maximum attempts".into());
            }

            let schema = schema_for!(T);
            let mut schema_value = serde_json::to_value(&schema)?;

            let defs = if let Some(obj) = schema_value.get("$defs") {
                obj.clone()
            } else {
                Value::Object(Default::default())
            };

            Self::inline_schema_refs(&mut schema_value, &defs);

            if let Some(obj) = schema_value.as_object_mut() {
                obj.remove("$defs");
            }

            let response_format = ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    description: None,
                    name: "math_reasoning".into(),
                    schema: Some(schema_value),
                    strict: Some(true),
                },
            };

            let request = CreateChatCompletionRequestArgs::default()
                .max_tokens(STRUCTURED_OUTPUT_MAX_TOKENS)
                .model("liquid/lfm2-1.2b")
                .messages(messages.clone())
                .response_format(response_format)
                .tools(tools.clone())
                .tool_choice(ChatCompletionToolChoiceOption::Mode(
                    ToolChoiceOptions::Auto,
                ))
                .build()?;

            let response = self.client.chat().create(request).await?;
            let choice = response
                .choices
                .into_iter()
                .next()
                .ok_or("No response from LLM")?;

            if let Some(tool_calls) = choice.message.tool_calls {
                let assistant = ChatCompletionRequestAssistantMessage {
                    content: None,
                    tool_calls: Some(tool_calls.clone()),
                    ..Default::default()
                };
                messages.push(assistant.into());

                for tool_call in tool_calls {
                    match tool_call {
                        ChatCompletionMessageToolCalls::Function(call) => {
                            let tool_output = self.execute_tool_call(&call.function).await?;
                            let tool_message = ChatCompletionRequestToolMessage {
                                content: tool_output.into(),
                                tool_call_id: call.id,
                            };
                            messages.push(tool_message.into());
                        }
                        ChatCompletionMessageToolCalls::Custom(call) => {
                            let tool_message = ChatCompletionRequestToolMessage {
                                content: format!(
                                    "Unsupported custom tool: {}",
                                    call.custom_tool.name
                                )
                                .into(),
                                tool_call_id: call.id,
                            };
                            messages.push(tool_message.into());
                        }
                    }
                }

                continue;
            }

            if let Some(content) = choice.message.content {
                return Ok(Some(serde_json::from_str::<T>(&content)?));
            }

            return Ok(None);
        }
    }

    fn inline_schema_refs(schema: &mut Value, defs: &Value) {
        match schema {
            Value::Object(obj) => {
                // Check if this object has a $ref
                if let Some(ref_value) = obj.get("$ref").cloned() {
                    if let Some(ref_str) = ref_value.as_str() {
                        // Extract the definition name from #/$defs/Name
                        if let Some(def_name) = ref_str.strip_prefix("#/$defs/") {
                            if let Some(def) = defs.get(def_name) {
                                // Replace the $ref with the actual definition
                                obj.clear();
                                if let Value::Object(def_obj) = def {
                                    for (k, v) in def_obj {
                                        obj.insert(k.clone(), v.clone());
                                    }
                                }
                            }
                        }
                    }
                }

                // Recursively process all values
                for (_, v) in obj.iter_mut() {
                    Self::inline_schema_refs(v, defs);
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    Self::inline_schema_refs(item, defs);
                }
            }
            _ => {}
        }
    }

    fn build_fetch_webpage_tool() -> AppResult<ChatCompletionTools> {
        let parameters = json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "query": {"type": "string"}
            },
            "required": ["url", "query"]
        });

        let function = FunctionObject {
            name: "fetch_webpage".to_string(),
            description: Some(
                "Fetch the text content of a URL and return the relevant content for the query"
                    .to_string(),
            ),
            parameters: Some(parameters),
            strict: Some(true),
        };

        Ok(ChatCompletionTools::Function(ChatCompletionTool {
            function,
        }))
    }

    fn build_open_simple_browser_tool() -> AppResult<ChatCompletionTools> {
        let parameters = json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"}
            },
            "required": ["url"]
        });

        let function = FunctionObject {
            name: "open_simple_browser".to_string(),
            description: Some(
                "Preview a URL and return a short, plain-text snapshot of the page".to_string(),
            ),
            parameters: Some(parameters),
            strict: Some(true),
        };

        Ok(ChatCompletionTools::Function(ChatCompletionTool {
            function,
        }))
    }

    async fn execute_tool_call(&self, function: &FunctionCall) -> Result<String, Box<dyn Error>> {
        match function.name.as_str() {
            "fetch_webpage" => {
                let args: FetchWebpageArgs = serde_json::from_str(&function.arguments)?;
                let content = self.fetch_webpage(&args.url, &args.query).await?;
                Ok(json!({
                    "url": args.url,
                    "query": args.query,
                    "content": content
                })
                .to_string())
            }
            "open_simple_browser" => {
                let args: OpenSimpleBrowserArgs = serde_json::from_str(&function.arguments)?;
                let content = self.fetch_webpage(&args.url, "preview").await?;
                Ok(json!({
                    "url": args.url,
                    "content": content
                })
                .to_string())
            }
            _ => Ok(json!({
                "error": format!("Unsupported tool: {}", function.name)
            })
            .to_string()),
        }
    }

    async fn fetch_webpage(&self, url: &str, query: &str) -> Result<String, Box<dyn Error>> {
        let response = reqwest::Client::new()
            .get(url)
            .header("User-Agent", "tento-server/1.0")
            .send()
            .await?;

        let status = response.status();
        let mut body = response.text().await.unwrap_or_default();
        body = strip_html_tags(&body);
        body = collapse_whitespace(&body);
        if body.len() > TOOL_MAX_CONTENT_LENGTH {
            body.truncate(TOOL_MAX_CONTENT_LENGTH);
            body.push_str("\n[TRUNCATED]");
        }

        if !status.is_success() {
            return Ok(format!(
                "Failed to fetch URL. Status: {}. Body (truncated): {}",
                status, body
            ));
        }

        Ok(format!("Query: {}\n{}", query, body))
    }
}

fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => {
                if !in_tag {
                    out.push(ch);
                }
            }
        }
    }
    out
}

fn collapse_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut was_space = false;
    for ch in input.chars() {
        if ch.is_whitespace() {
            if !was_space {
                out.push(' ');
                was_space = true;
            }
        } else {
            was_space = false;
            out.push(ch);
        }
    }
    out.trim().to_string()
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
