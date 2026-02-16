use std::error::Error;

use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestToolMessage, ChatCompletionRequestToolMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs,
        ChatCompletionTool, ChatCompletionToolChoiceOption, ChatCompletionTools,
        CreateChatCompletionRequestArgs, FunctionCall, FunctionObject, ResponseFormat,
        ResponseFormatJsonSchema, ToolChoiceOptions,
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
    constants::{prompts::QUIZ_GENERATOR_PROMPT, WEBSITE_SUMMARISER_PROMPT},
    errors::{AppError, AppResult},
    models::dto::request::{QuizRequestDto, SummaryDocumentRequestDto},
};

pub struct ModelService {
    client: Client<OpenAIConfig>,
}

const TOOL_MAX_ATTEMPTS: u32 = 10;
const TOOL_MAX_CONTENT_LENGTH: usize = 20000;
const STRUCTURED_OUTPUT_MAX_TOKENS: u32 = 12288;

#[derive(Debug, Deserialize)]
struct FetchWebpageArgs {
    url: String,
    #[serde(default = "default_query")]
    query: String,
}

fn default_query() -> String {
    "content".to_string()
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
        quiz: QuizRequestDto,
        summary_document: SummaryDocumentRequestDto,
    ) -> AppResult<QuizRequestDto> {
        let quiz_json = serde_json::to_string(&quiz)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize quiz: {}", e)))?;

        log::info!(
            "Starting structured quiz generation with summary content length: {} bytes",
            summary_document.content.len()
        );

        match self
            .structured_output::<QuizRequestDto>(vec![
                ChatCompletionRequestSystemMessage::from(
                    "Tool calls are disabled for structured output. Do not call tools.",
                )
                .into(),
                ChatCompletionRequestSystemMessage::from(QUIZ_GENERATOR_PROMPT).into(),
                ChatCompletionRequestUserMessage::from(quiz_json).into(),
                ChatCompletionRequestUserMessage::from(summary_document.content).into(),
            ])
            .await
        {
            Ok(Some(generated_quiz)) => {
                log::info!(
                    "Successfully generated quiz with {} questions",
                    generated_quiz.questions.len()
                );
                Ok(generated_quiz)
            }
            Ok(None) => Err(AppError::InternalError(
                "LLM did not return a valid quiz".to_string(),
            )),
            Err(e) => {
                log::error!("Quiz generation failed: {}", e);
                Err(AppError::InternalError(format!(
                    "Failed to generate structured quiz: {}",
                    e
                )))
            }
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

        let content = self
            .chat_completion_with_tools(
                vec![
                    ChatCompletionRequestSystemMessage::from(WEBSITE_SUMMARISER_PROMPT).into(),
                    ChatCompletionRequestUserMessage::from(url_string).into(),
                ],
                tools,
            )
            .await
            .map_err(|e| AppError::InternalError(format!(
                "Failed to generate summary with tools: {}",
                e
            )))?;

        // Return unstructured text wrapped in DTO
        Ok(SummaryDocumentRequestDto {
            id: String::new(), // Will be set by caller
            quiz_id: String::new(), // Will be set by caller
            url: url_string.to_string(),
            content,
            created_at: String::new(), // Will be set by caller
            modified_at: String::new(), // Will be set by caller
        })
    }

    pub async fn chat_completion_with_tools(
        &self,
        mut messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTools>,
    ) -> Result<String, Box<dyn Error>> {
        let mut attempts = 0;
        let mut tool_calls_made = 0;

        loop {
            attempts += 1;
            if attempts > TOOL_MAX_ATTEMPTS {
                log::error!(
                    "Tool loop exceeded maximum attempts ({}) after {} tool calls",
                    TOOL_MAX_ATTEMPTS,
                    tool_calls_made
                );
                return Err(format!(
                    "Tool loop exceeded maximum attempts ({})",
                    TOOL_MAX_ATTEMPTS
                )
                .into());
            }

            // After 3 attempts, inject a strong instruction and disable tools
            if attempts == 4 {
                log::warn!("Attempt {}: Injecting finalization instruction and disabling tools", attempts);
                messages.push(
                    ChatCompletionRequestSystemMessage::from(
                        "IMPORTANT: You have gathered sufficient information. Provide your final summary response now as plain text based on the content retrieved."
                    )
                    .into(),
                );
            }

            // Disable tools after attempt 3 to force completion
            let use_tools = attempts <= 3;
            log::debug!(
                "Attempt {}: tools_enabled={}, tool_calls_made={}",
                attempts,
                use_tools,
                tool_calls_made
            );

            let request = if use_tools {
                CreateChatCompletionRequestArgs::default()
                    .max_tokens(STRUCTURED_OUTPUT_MAX_TOKENS)
                    .model("liquid/lfm2-1.2b")
                    .messages(messages.clone())
                    .tools(tools.clone())
                    .tool_choice(ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Auto))
                    .build()?
            } else {
                CreateChatCompletionRequestArgs::default()
                    .max_tokens(STRUCTURED_OUTPUT_MAX_TOKENS)
                    .model("liquid/lfm2-1.2b")
                    .messages(messages.clone())
                    .build()?
            };
            let response = self.client.chat().create(request).await?;
            let choice = response
                .choices
                .into_iter()
                .next()
                .ok_or("No response from LLM")?;

            // If there are tool calls, execute them and continue (only if tools are enabled)
            if let Some(tool_calls) = choice.message.tool_calls {
                if !use_tools {
                    log::warn!(
                        "Attempt {}: Model attempted {} tool call(s) after tools were disabled - extracting any content instead",
                        attempts,
                        tool_calls.len()
                    );
                    
                    // Try to extract content from this response
                    if let Some(content) = choice.message.content {
                        if !content.is_empty() {
                            log::info!(
                                "Successfully extracted content despite tool call attempts after {} attempts",
                                attempts
                            );
                            return Ok(content);
                        }
                    }
                    
                    // No usable content - synthesize from conversation history
                    log::warn!("No content found, synthesizing from tool results");
                    return self.synthesize_summary_from_history(&messages).await;
                }
                
                tool_calls_made += tool_calls.len();
                log::debug!(
                    "Attempt {}: Model requested {} tool call(s)",
                    attempts,
                    tool_calls.len()
                );
                
                let assistant = ChatCompletionRequestAssistantMessage {
                    content: choice.message.content.clone().map(|c| c.into()),
                    tool_calls: Some(tool_calls.clone()),
                    ..Default::default()
                };
                messages.push(assistant.into());

                for tool_call in tool_calls {
                    match tool_call {
                        ChatCompletionMessageToolCalls::Function(call) => {
                            log::debug!(
                                "Executing tool: {} with args: {}",
                                call.function.name,
                                &call.function.arguments[..call.function.arguments.len().min(100)]
                            );
                            let tool_output = self.execute_tool_call(&call.function).await?;
                            log::debug!(
                                "Tool {} returned {} bytes",
                                call.function.name,
                                tool_output.len()
                            );
                            let tool_message = ChatCompletionRequestToolMessage {
                                content: tool_output.into(),
                                tool_call_id: call.id,
                            };
                            messages.push(tool_message.into());
                        }
                        ChatCompletionMessageToolCalls::Custom(call) => {
                            log::warn!("Unsupported custom tool: {}", call.custom_tool.name);
                            let tool_message = ChatCompletionRequestToolMessage {
                                content: format!(
                                    "Unsupported custom tool: {}",
                                    call.custom_tool.name
                                )
                                .into(),
                                tool_call_id: call.id,
                            };
                            messages.push(tool_message.into())
                        }
                    }
                }

                continue;
            }

            // No tool calls - return the content
            if let Some(content) = choice.message.content {
                log::info!(
                    "Model returned final response after {} attempts and {} tool calls",
                    attempts,
                    tool_calls_made
                );
                return Ok(content);
            }

            // Neither tool calls nor content - try to synthesize from history
            log::warn!(
                "Attempt {}: Model returned neither tool calls nor content - synthesizing from history",
                attempts
            );
            return self.synthesize_summary_from_history(&messages).await;
        }
    }

    async fn synthesize_summary_from_history(
        &self,
        messages: &[ChatCompletionRequestMessage],
    ) -> Result<String, Box<dyn Error>> {
        log::info!("Synthesizing summary from {} messages", messages.len());
        
        // Extract all tool result content from the conversation
        let mut tool_results = Vec::new();
        for msg in messages {
            if let ChatCompletionRequestMessage::Tool(tool_msg) = msg {
                if let ChatCompletionRequestToolMessageContent::Text(text) = &tool_msg.content {
                    tool_results.push(text.clone());
                }
            }
        }
        
        if tool_results.is_empty() {
            return Err("No tool results found in conversation history".into());
        }
        
        log::info!("Found {} tool results, combining into summary", tool_results.len());
        
        // Combine all tool results into a coherent summary
        let combined = tool_results.join("\n\n---\n\n");
        
        // Truncate if too long
        if combined.len() > TOOL_MAX_CONTENT_LENGTH * 2 {
            let truncated = &combined[..TOOL_MAX_CONTENT_LENGTH * 2];
            Ok(format!("{}\n\n[Content truncated from tool results]", truncated))
        } else {
            Ok(format!("Summary synthesized from fetched content:\n\n{}", combined))
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
                log::debug!("Structured output received {} bytes", content.len());
                log::debug!("Structured output preview: {}...", &content[..content.len().min(500)]);
                
                match serde_json::from_str::<T>(&content) {
                    Ok(parsed) => return Ok(Some(parsed)),
                    Err(e) => {
                        log::error!("Failed to parse structured output: {}", e);
                        log::error!("Full content length: {} bytes", content.len());
                        log::error!("Content tail (last 300 chars): ...{}", 
                            &content[content.len().saturating_sub(300)..]);
                        return Err(e.into());
                    }
                }
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

        // Phase 1: Tool calling loop (without structured output to avoid conflicts)
        loop {
            attempts += 1;
            if attempts > TOOL_MAX_ATTEMPTS {
                log::warn!(
                    "Tool loop reached maximum attempts ({}), forcing final structured output",
                    TOOL_MAX_ATTEMPTS
                );
                break;
            }

            let request = CreateChatCompletionRequestArgs::default()
                .max_tokens(STRUCTURED_OUTPUT_MAX_TOKENS)
                .model("liquid/lfm2-1.2b")
                .messages(messages.clone())
                .tools(tools.clone())
                .tool_choice(ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Auto))
                .build()?;

            let response = self.client.chat().create(request).await?;
            let choice = response
                .choices
                .into_iter()
                .next()
                .ok_or("No response from LLM")?;

            // If there are tool calls, execute them and continue
            if let Some(tool_calls) = choice.message.tool_calls {
                log::debug!("Model requested {} tool call(s)", tool_calls.len());
                let assistant = ChatCompletionRequestAssistantMessage {
                    content: choice.message.content.clone().map(|c| c.into()),
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

            // No tool calls, but we got content - break to structured output phase
            if choice.message.content.is_some() {
                log::debug!("Model returned content without tool calls, proceeding to structured output");
                if let Some(content) = choice.message.content {
                    // Add assistant message to maintain context
                    let assistant = ChatCompletionRequestAssistantMessage {
                        content: Some(content.clone().into()),
                        ..Default::default()
                    };
                    messages.push(assistant.into());
                }
                break;
            }

            // Neither tool calls nor content - something went wrong
            log::warn!("Model returned neither tool calls nor content");
            break;
        }

        // Phase 2: Final structured output request (without tools)
        log::debug!("Making final structured output request");
        messages.push(
            ChatCompletionRequestSystemMessage::from(
                "Now provide your complete response as a valid JSON object matching the schema.",
            )
            .into(),
        );

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
            .messages(messages)
            .response_format(response_format)
            .build()?;

        let response = self.client.chat().create(request).await?;

        for choice in response.choices {
            if let Some(content) = choice.message.content {
                log::debug!("Received structured output, parsing JSON");
                return Ok(Some(serde_json::from_str::<T>(&content)?));
            }
        }

        Ok(None)
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
                "url": {
                    "type": "string",
                    "description": "The URL to fetch content from"
                },
                "query": {
                    "type": "string",
                    "description": "Optional query describing what information to extract (defaults to 'content')"
                }
            },
            "required": ["url"]
        });

        let function = FunctionObject {
            name: "fetch_webpage".to_string(),
            description: Some(
                "Fetch the text content of a URL. Provide a URL to retrieve content."
                    .to_string(),
            ),
            parameters: Some(parameters),
            strict: Some(false),
        };

        Ok(ChatCompletionTools::Function(ChatCompletionTool { function }))
    }

    fn build_open_simple_browser_tool() -> AppResult<ChatCompletionTools> {
        let parameters = json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to preview"
                }
            },
            "required": ["url"]
        });

        let function = FunctionObject {
            name: "open_simple_browser".to_string(),
            description: Some(
                "Preview a URL and return a short, plain-text snapshot of the page"
                    .to_string(),
            ),
            parameters: Some(parameters),
            strict: Some(false),
        };

        Ok(ChatCompletionTools::Function(ChatCompletionTool { function }))
    }

    async fn execute_tool_call(
        &self,
        function: &FunctionCall,
    ) -> Result<String, Box<dyn Error>> {
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
                status,
                body
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
