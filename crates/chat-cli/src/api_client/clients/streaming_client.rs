use std::sync::{
    Arc,
    Mutex,
};

use amzn_codewhisperer_streaming_client::Client as CodewhispererStreamingClient;
use amzn_qdeveloper_streaming_client::Client as QDeveloperStreamingClient;
use aws_types::request_id::RequestId;
use gemini_streaming_client::Client as GeminiStreamingClient;
use tracing::{
    debug,
    error,
};

use super::shared::{
    bearer_sdk_config,
    sigv4_sdk_config,
    stalled_stream_protection_config,
};
use crate::api_client::interceptor::opt_out::OptOutInterceptor;
use crate::api_client::model::{
    ChatMessage,
    ChatResponseStream,
    ConversationState,
    FigDocument,
    Tool,
    ToolResultContentBlock,
    ToolResultStatus,
};
use crate::api_client::{
    ApiClientError,
    Endpoint,
};
use crate::auth::builder_id::BearerResolver;
use crate::aws_common::{
    UserAgentOverrideInterceptor,
    app_name,
};
use crate::database::{
    AuthProfile,
    Database,
};

mod inner {
    use std::sync::{
        Arc,
        Mutex,
    };

    use amzn_codewhisperer_streaming_client::Client as CodewhispererStreamingClient;
    use amzn_qdeveloper_streaming_client::Client as QDeveloperStreamingClient;
    use gemini_streaming_client::Client as GeminiStreamingClient;

    use crate::api_client::model::ChatResponseStream;

    #[derive(Clone, Debug)]
    pub enum Inner {
        Codewhisperer(CodewhispererStreamingClient),
        QDeveloper(QDeveloperStreamingClient),
        Gemini(GeminiStreamingClient),
        Mock(Arc<Mutex<std::vec::IntoIter<Vec<ChatResponseStream>>>>),
    }
}

#[derive(Clone, Debug)]
pub struct StreamingClient {
    inner: inner::Inner,
    profile: Option<AuthProfile>,
}

impl StreamingClient {
    pub async fn new(database: &mut Database) -> Result<Self, ApiClientError> {
        let client = if gemini_streaming_client::config::config_exists()
        {
            println!(
                "Gemini configuration found at {:?}",
                gemini_streaming_client::config::get_config_path()
            );

            // debug!("Gemini connection test result: {}", GeminiStreamingClient::test_gemini().await);
            Self::new_gemini_client().await?
        } else if crate::util::system_info::in_cloudshell()
            || std::env::var("Q_USE_SENDMESSAGE").is_ok_and(|v| !v.is_empty())
        {
            Self::new_qdeveloper_client(database, &Endpoint::load_q(database)).await?
        } else {
            Self::new_codewhisperer_client(database, &Endpoint::load_codewhisperer(database)).await?
        };
        Ok(client)
    }

    pub fn mock(events: Vec<Vec<ChatResponseStream>>) -> Self {
        Self {
            inner: inner::Inner::Mock(Arc::new(Mutex::new(events.into_iter()))),
            profile: None,
        }
    }

    pub async fn new_codewhisperer_client(
        database: &mut Database,
        endpoint: &Endpoint,
    ) -> Result<Self, ApiClientError> {
        let conf_builder: amzn_codewhisperer_streaming_client::config::Builder =
            (&bearer_sdk_config(database, endpoint).await).into();
        let conf = conf_builder
            .http_client(crate::aws_common::http_client::client())
            .interceptor(OptOutInterceptor::new(database))
            .interceptor(UserAgentOverrideInterceptor::new())
            .bearer_token_resolver(BearerResolver)
            .app_name(app_name())
            .endpoint_url(endpoint.url())
            .stalled_stream_protection(stalled_stream_protection_config())
            .build();
        let inner = inner::Inner::Codewhisperer(CodewhispererStreamingClient::from_conf(conf));

        let profile = match database.get_auth_profile() {
            Ok(profile) => profile,
            Err(err) => {
                error!("Failed to get auth profile: {err}");
                None
            },
        };

        Ok(Self { inner, profile })
    }

    pub async fn new_qdeveloper_client(database: &Database, endpoint: &Endpoint) -> Result<Self, ApiClientError> {
        let conf_builder: amzn_qdeveloper_streaming_client::config::Builder =
            (&sigv4_sdk_config(database, endpoint).await?).into();
        let conf = conf_builder
            .http_client(crate::aws_common::http_client::client())
            .interceptor(OptOutInterceptor::new(database))
            .interceptor(UserAgentOverrideInterceptor::new())
            .app_name(app_name())
            .endpoint_url(endpoint.url())
            .stalled_stream_protection(stalled_stream_protection_config())
            .build();
        let client = QDeveloperStreamingClient::from_conf(conf);
        Ok(Self {
            inner: inner::Inner::QDeveloper(client),
            profile: None,
        })
    }

    pub async fn new_gemini_client() -> Result<Self, ApiClientError> {
        // Load Gemini configuration
        let config = match gemini_streaming_client::config::load_config() {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to load Gemini configuration: {}", e);
                return Err(ApiClientError::ModelConfigurationError(format!(
                    "Failed to load Gemini configuration: {}",
                    e
                )));
            },
        };

        // Create Gemini client
        let client = GeminiStreamingClient::new(config);

        Ok(Self {
            inner: inner::Inner::Gemini(client),
            profile: None,
        })
    }

    pub async fn send_message(
        &self,
        conversation_state: ConversationState,
    ) -> Result<SendMessageOutput, ApiClientError> {
        debug!("Sending conversation: {:#?}", conversation_state);
        let ConversationState {
            conversation_id,
            user_input_message,
            history,
        } = conversation_state;

        match &self.inner {
            inner::Inner::Codewhisperer(client) => {
                let conversation_state = amzn_codewhisperer_streaming_client::types::ConversationState::builder()
                    .set_conversation_id(conversation_id)
                    .current_message(
                        amzn_codewhisperer_streaming_client::types::ChatMessage::UserInputMessage(
                            user_input_message.into(),
                        ),
                    )
                    .chat_trigger_type(amzn_codewhisperer_streaming_client::types::ChatTriggerType::Manual)
                    .set_history(
                        history
                            .map(|v| v.into_iter().map(|i| i.try_into()).collect::<Result<Vec<_>, _>>())
                            .transpose()?,
                    )
                    .build()
                    .expect("building conversation_state should not fail");
                let response = client
                    .generate_assistant_response()
                    .conversation_state(conversation_state)
                    .set_profile_arn(self.profile.as_ref().map(|p| p.arn.clone()))
                    .send()
                    .await;

                match response {
                    Ok(resp) => Ok(SendMessageOutput::Codewhisperer(resp)),
                    Err(e) => {
                        let is_quota_breach = e.raw_response().is_some_and(|resp| resp.status().as_u16() == 429);
                        let is_context_window_overflow = e.as_service_error().is_some_and(|err| {
                            matches!(err, err if err.meta().code() == Some("ValidationException")
                                && err.meta().message() == Some("Input is too long."))
                        });

                        if is_quota_breach {
                            Err(ApiClientError::QuotaBreach("quota has reached its limit"))
                        } else if is_context_window_overflow {
                            Err(ApiClientError::ContextWindowOverflow)
                        } else {
                            Err(e.into())
                        }
                    },
                }
            },
            inner::Inner::QDeveloper(client) => {
                let conversation_state_builder = amzn_qdeveloper_streaming_client::types::ConversationState::builder()
                    .set_conversation_id(conversation_id)
                    .current_message(amzn_qdeveloper_streaming_client::types::ChatMessage::UserInputMessage(
                        user_input_message.into(),
                    ))
                    .chat_trigger_type(amzn_qdeveloper_streaming_client::types::ChatTriggerType::Manual)
                    .set_history(
                        history
                            .map(|v| v.into_iter().map(|i| i.try_into()).collect::<Result<Vec<_>, _>>())
                            .transpose()?,
                    );

                Ok(SendMessageOutput::QDeveloper(
                    client
                        .send_message()
                        .conversation_state(conversation_state_builder.build().expect("fix me"))
                        .send()
                        .await?,
                ))
            },
            inner::Inner::Gemini(client) => {
                // Convert history to Gemini format
                let gemini_history = history
                    .map(|h| {
                        h.iter()
                            .map(|msg| {
                                match msg {
                                    ChatMessage::UserInputMessage(user_msg) => {
                                        // Check if there are tool results in the user message context
                                        let tool_results = user_msg
                                            .user_input_message_context
                                            .as_ref()
                                            .and_then(|ctx| ctx.tool_results.as_ref())
                                            .map(|results| {
                                                results
                                                    .iter()
                                                    .map(|result| {
                                                        // Convert the tool result content to a JSON value
                                                        let content = result
                                                            .content
                                                            .iter()
                                                            .map(|block| {
                                                                match block {
                                                                    ToolResultContentBlock::Text(text) => {
                                                                        serde_json::Value::String(text.clone())
                                                                    },
                                                                    ToolResultContentBlock::Json(doc) => {
                                                                        // Convert Document to a string representation
                                                                        serde_json::Value::String(format!("{:?}", doc))
                                                                    },
                                                                }
                                                            })
                                                            .next()
                                                            .unwrap_or(serde_json::Value::Null);

                                                        gemini_streaming_client::conversion::MockToolResult {
                                                            tool_use_id: result.tool_use_id.clone(),
                                                            content,
                                                            status: match result.status {
                                                                ToolResultStatus::Success => "success".to_string(),
                                                                ToolResultStatus::Error => "error".to_string(),
                                                            },
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                            });

                                        gemini_streaming_client::conversion::MockChatMessage::UserMessage {
                                            content: user_msg.content.clone(),
                                            tool_results,
                                        }
                                    },
                                    ChatMessage::AssistantResponseMessage(assistant_msg) => {
                                        // Convert tool uses if they exist
                                        let tool_uses = assistant_msg.tool_uses.as_ref().map(|tool_uses| {
                                            tool_uses
                                                .iter()
                                                .map(|tool_use| gemini_streaming_client::conversion::MockToolUse {
                                                    name: tool_use.name.clone(),
                                                    args: serde_json::to_value(&tool_use.input).unwrap_or_default(),
                                                    tool_use_id: tool_use.tool_use_id.clone(),
                                                })
                                                .collect::<Vec<_>>()
                                        });

                                        gemini_streaming_client::conversion::MockChatMessage::AssistantMessage {
                                            content: assistant_msg.content.clone(),
                                            tool_uses,
                                        }
                                    },
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                // Convert tools to Gemini format
                let tools =
                    user_input_message.user_input_message_context.as_ref().and_then(|ctx| {
                        ctx.tools.as_ref().map(|tools| {
                            tools.iter().map(|tool| {
                            match tool {
                                Tool::ToolSpecification(spec) => {
                                    gemini_streaming_client::conversion::MockTool {
                                        name: spec.name.clone(),
                                        description: spec.description.clone(),
                                        parameters: match &spec.input_schema.json {
                                            Some(json_doc) => {
                                                // Convert the FigDocument to a serde_json::Value
                                                let value = serde_json::to_value(json_doc).unwrap_or_default();
                                                // Clean the parameters for Gemini API compatibility
                                                gemini_streaming_client::conversion::clean_parameters_for_gemini(&value)
                                            },
                                            None => serde_json::json!({}),
                                        },
                                    }
                                }
                            }
                        }).collect::<Vec<_>>()
                        })
                    });

                // Convert user input message to MockChatMessage
                let mock_user_message = gemini_streaming_client::conversion::MockChatMessage::UserMessage {
                    content: user_input_message.content.clone(),
                    tool_results: user_input_message
                        .user_input_message_context
                        .as_ref()
                        .and_then(|ctx| ctx.tool_results.as_ref())
                        .map(|results| {
                            results
                                .iter()
                                .map(|result| {
                                    // Convert ToolResultContentBlock to a simple string or JSON value
                                    let content_value = match &result.content[0] {
                                        ToolResultContentBlock::Text(text) => serde_json::Value::String(text.clone()),
                                        ToolResultContentBlock::Json(doc) => {
                                            // Convert AwsDocument to serde_json::Value using FigDocument
                                            let fig_doc = FigDocument::from(doc.clone());
                                            serde_json::to_value(&fig_doc).unwrap_or(serde_json::Value::Null)
                                        },
                                    };

                                    gemini_streaming_client::conversion::MockToolResult {
                                        tool_use_id: result.tool_use_id.clone(),
                                        content: content_value,
                                        status: match result.status {
                                            ToolResultStatus::Success => "success".to_string(),
                                            ToolResultStatus::Error => "error".to_string(),
                                        },
                                    }
                                })
                                .collect()
                        }),
                };

                // Send request to Gemini API
                let request = gemini_streaming_client::conversion::conversation_state_to_gemini_request(
                    &mock_user_message,
                    &gemini_history,
                    tools.as_deref(),
                    client.temperature(),
                );

                match client.generate_content(request).await {
                    Ok(response) => {
                        // Convert Gemini response to a vector of ChatResponseStream events
                        let mut streams = Vec::new();
                        if let Some(candidate) = response.candidates.first() {
                            for part in &candidate.content.parts {
                                match part {
                                    gemini_streaming_client::GeminiPart::Text { text } => {
                                        streams
                                            .push(ChatResponseStream::AssistantResponseEvent { content: text.clone() });
                                    },
                                    gemini_streaming_client::GeminiPart::FunctionCall { function_call } => {
                                        // Convert function call to tool use event
                                        let tool_use_id = gemini_streaming_client::conversion::generate_tool_use_id();

                                        // Convert the args to a properly formatted JSON string
                                        let input = match &function_call.args {
                                            serde_json::Value::Object(map) => {
                                                serde_json::to_string(map).unwrap_or_default()
                                            },
                                            _ => serde_json::to_string(&function_call.args).unwrap_or_default(),
                                        };

                                        streams.push(ChatResponseStream::ToolUseEvent {
                                            tool_use_id: tool_use_id.clone(),
                                            name: function_call.name.clone(),
                                            input: None,
                                            stop: None,
                                        });
                                        streams.push(ChatResponseStream::ToolUseEvent {
                                            tool_use_id,
                                            name: function_call.name.clone(),
                                            input: Some(input),
                                            stop: Some(true),
                                        });
                                    },
                                    gemini_streaming_client::GeminiPart::FunctionResponse { .. } => {},
                                }
                            }
                        }
                        // Reverse the vector so we can pop from the end
                        streams.reverse();
                        Ok(SendMessageOutput::Gemini(streams))
                    },
                    Err(e) => {
                        error!("Gemini API request failed: {}", e);
                        Err(ApiClientError::ModelRuntimeError(format!(
                            "Gemini API request failed: {}",
                            e
                        )))
                    },
                }
            },
            inner::Inner::Mock(events) => {
                let mut new_events = events.lock().unwrap().next().unwrap_or_default().clone();
                new_events.reverse();
                Ok(SendMessageOutput::Mock(new_events))
            },
        }
    }
}

#[derive(Debug)]
pub enum SendMessageOutput {
    Codewhisperer(
        amzn_codewhisperer_streaming_client::operation::generate_assistant_response::GenerateAssistantResponseOutput,
    ),
    QDeveloper(amzn_qdeveloper_streaming_client::operation::send_message::SendMessageOutput),
    Gemini(Vec<ChatResponseStream>),
    Mock(Vec<ChatResponseStream>),
}

impl SendMessageOutput {
    pub fn request_id(&self) -> Option<&str> {
        match self {
            SendMessageOutput::Codewhisperer(output) => output.request_id(),
            SendMessageOutput::QDeveloper(output) => output.request_id(),
            SendMessageOutput::Gemini(_) => None, // Gemini doesn't provide a request ID
            SendMessageOutput::Mock(_) => None,
        }
    }

    pub async fn recv(&mut self) -> Result<Option<ChatResponseStream>, ApiClientError> {
        match self {
            SendMessageOutput::Codewhisperer(output) => Ok(output
                .generate_assistant_response_response
                .recv()
                .await?
                .map(|s| s.into())),
            SendMessageOutput::QDeveloper(output) => Ok(output.send_message_response.recv().await?.map(|s| s.into())),
            SendMessageOutput::Gemini(vec) => Ok(vec.pop()),
            SendMessageOutput::Mock(vec) => Ok(vec.pop()),
        }
    }
}

impl RequestId for SendMessageOutput {
    fn request_id(&self) -> Option<&str> {
        match self {
            SendMessageOutput::Codewhisperer(output) => output.request_id(),
            SendMessageOutput::QDeveloper(output) => output.request_id(),
            SendMessageOutput::Gemini(_) => Some("<gemini-request-id>"),
            SendMessageOutput::Mock(_) => Some("<mock-request-id>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_client::model::{
        AssistantResponseMessage,
        ChatMessage,
        UserInputMessage,
    };

    #[tokio::test]
    async fn create_clients() {
        let mut database = Database::new().await.unwrap();
        let endpoint = Endpoint::load_codewhisperer(&database);

        let _ = StreamingClient::new(&mut database).await;
        let _ = StreamingClient::new_codewhisperer_client(&mut database, &endpoint).await;
        let _ = StreamingClient::new_qdeveloper_client(&database, &endpoint).await;
    }

    #[tokio::test]
    async fn test_mock() {
        let client = StreamingClient::mock(vec![vec![
            ChatResponseStream::AssistantResponseEvent {
                content: "Hello!".to_owned(),
            },
            ChatResponseStream::AssistantResponseEvent {
                content: " How can I".to_owned(),
            },
            ChatResponseStream::AssistantResponseEvent {
                content: " assist you today?".to_owned(),
            },
        ]]);
        let mut output = client
            .send_message(ConversationState {
                conversation_id: None,
                user_input_message: UserInputMessage {
                    images: None,
                    content: "Hello".into(),
                    user_input_message_context: None,
                    user_intent: None,
                },
                history: None,
            })
            .await
            .unwrap();

        let mut output_content = String::new();
        while let Some(ChatResponseStream::AssistantResponseEvent { content }) = output.recv().await.unwrap() {
            output_content.push_str(&content);
        }
        assert_eq!(output_content, "Hello! How can I assist you today?");
    }

    #[ignore]
    #[tokio::test]
    async fn assistant_response() {
        let mut database = Database::new().await.unwrap();
        let client = StreamingClient::new(&mut database).await.unwrap();
        let mut response = client
            .send_message(ConversationState {
                conversation_id: None,
                user_input_message: UserInputMessage {
                    images: None,
                    content: "How about rustc?".into(),
                    user_input_message_context: None,
                    user_intent: None,
                },
                history: Some(vec![
                    ChatMessage::UserInputMessage(UserInputMessage {
                        images: None,
                        content: "What language is the linux kernel written in, and who wrote it?".into(),
                        user_input_message_context: None,
                        user_intent: None,
                    }),
                    ChatMessage::AssistantResponseMessage(AssistantResponseMessage {
                        content: "It is written in C by Linus Torvalds.".into(),
                        message_id: None,
                        tool_uses: None,
                    }),
                ]),
            })
            .await
            .unwrap();

        while let Some(event) = response.recv().await.unwrap() {
            println!("{:?}", event);
        }
    }
}
