# Detailed Design: Gemini Backend for Amazon Q CLI

## Overview

This document outlines the design for implementing a Gemini model backend for the Amazon Q CLI chat functionality. The implementation will allow users to leverage Google's Gemini models as an alternative to the default Amazon Q backend, using their own Gemini API key.

## Requirements

1. Allow users to use Gemini models with the Amazon Q CLI chat interface
2. Support configuration of Gemini API key and model selection
3. Maintain compatibility with existing Amazon Q CLI tool execution capabilities
4. Ensure secure handling of API keys
5. Minimize changes to the existing codebase
6. Support future extensibility for other model backends

## Architecture

The implementation will follow the existing pattern in the Amazon Q CLI codebase, which already supports multiple backends through the `StreamingClient` struct. We will create a new crate called `gemini_streaming_client` for the Gemini backend, similar to how the existing clients like `amzn_codewhisperer_streaming_client` and `amzn_qdeveloper_streaming_client` are implemented. This approach maintains consistency with the existing architecture and provides a clean separation of concerns.

### High-Level Components

1. **Configuration Management**: Handle loading and storing Gemini API keys and model selection
2. **Gemini Client Implementation**: Create a new client for interacting with the Gemini API
3. **Data Structure Conversion**: Convert between Amazon Q and Gemini data formats
4. **StreamingClient Integration**: Integrate the Gemini client into the existing `StreamingClient` struct

### Component Details

#### 1. Configuration Management

We will create a new configuration file format for storing Gemini API keys and model selection. The configuration will be stored in a JSON file at `~/.aws/amazonq/gemini_config.json` with the following structure:

```json
{
  "api_key": "YOUR_GEMINI_API_KEY",
  "model": "gemini-2.0-flash",
  "temperature": 0.7
}
```

The configuration will be loaded when the `StreamingClient` is created, and the API key will be used to authenticate with the Gemini API.

#### 2. Gemini Client Implementation

We will create a new client struct `GeminiStreamingClient` that implements the necessary methods for interacting with the Gemini API. The client will handle:

- Authentication with the Gemini API
- Sending messages to the model
- Receiving and processing responses
- Converting between Amazon Q and Gemini data formats

```rust
pub struct GeminiStreamingClient {
    api_key: String,
    model: String,
    temperature: f32,
    client: reqwest::Client,
}

impl GeminiStreamingClient {
    pub fn new(api_key: String, model: String, temperature: f32) -> Self {
        Self {
            api_key,
            model,
            temperature,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_message(&self, conversation_state: ConversationState) -> Result<GeminiResponse, ApiClientError> {
        // Convert Amazon Q conversation state to Gemini request format
        let request = self.convert_to_gemini_request(conversation_state);
        
        // Send request to Gemini API
        let response = self.client
            .post(&self.get_api_url())
            .json(&request)
            .send()
            .await?;
        
        // Process response
        let gemini_response = response.json::<GeminiResponse>().await?;
        
        Ok(gemini_response)
    }

    fn get_api_url(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        )
    }

    fn convert_to_gemini_request(&self, conversation_state: ConversationState) -> GeminiRequest {
        // Convert Amazon Q conversation state to Gemini request format
        // ...
    }
}
```

#### 3. Data Structure Conversion

We will implement conversion functions between Amazon Q's data structures and Gemini's API format. This includes:

- Converting `ConversationState` to Gemini's request format
- Converting Gemini's response to `ChatResponseStream`
- Converting Amazon Q's tool specifications to Gemini's function declarations
- Converting Gemini's function calls to Amazon Q's tool use events

```rust
// Gemini request and response types
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    tools: Option<Vec<GeminiTool>>,
    generation_config: Option<GeminiGenerationConfig>,
}

struct GeminiContent {
    role: Option<String>,
    parts: Vec<GeminiPart>,
}

enum GeminiPart {
    Text(String),
    FunctionCall { name: String, args: serde_json::Value },
    FunctionResponse { name: String, response: serde_json::Value },
}

struct GeminiTool {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

struct GeminiCandidate {
    content: GeminiContent,
}

// Conversion functions
impl From<ConversationState> for GeminiRequest {
    fn from(state: ConversationState) -> Self {
        // Convert ConversationState to GeminiRequest
        // ...
    }
}

impl From<GeminiResponse> for Vec<ChatResponseStream> {
    fn from(response: GeminiResponse) -> Self {
        // Convert GeminiResponse to Vec<ChatResponseStream>
        // ...
    }
}

impl From<Tool> for GeminiFunctionDeclaration {
    fn from(tool: Tool) -> Self {
        // Convert Tool to GeminiFunctionDeclaration
        // ...
    }
}
```

#### 4. StreamingClient Integration

We will update the `StreamingClient` struct to include a new variant for the Gemini backend:

```rust
enum Inner {
    Codewhisperer(CodewhispererStreamingClient),
    QDeveloper(QDeveloperStreamingClient),
    Gemini(GeminiStreamingClient),
    Mock(Arc<Mutex<std::vec::IntoIter<Vec<ChatResponseStream>>>>),
}
```

We will also add a new client creation method:

```rust
impl StreamingClient {
    pub async fn new_gemini_client() -> Result<Self, ApiClientError> {
        // Load Gemini configuration
        let config = load_gemini_config()?;
        
        // Create Gemini client
        let client = GeminiStreamingClient::new(
            config.api_key,
            config.model,
            config.temperature,
        );
        
        Ok(Self {
            inner: Inner::Gemini(client),
            profile_arn: None,
        })
    }
}
```

And update the `new()` method to check for Gemini configuration:

```rust
impl StreamingClient {
    pub async fn new() -> Result<Self, ApiClientError> {
        // Check if Gemini configuration exists
        if gemini_config_exists() {
            return Self::new_gemini_client().await;
        }
        
        // Existing logic for other backends
        // ...
    }
}
```

Finally, we will update the `send_message()` method to handle the Gemini backend:

```rust
impl StreamingClient {
    pub async fn send_message(
        &self,
        conversation_state: ConversationState,
    ) -> Result<SendMessageOutput, ApiClientError> {
        match &self.inner {
            // Existing cases for other backends
            // ...
            
            Inner::Gemini(client) => {
                let response = client.send_message(conversation_state).await?;
                Ok(SendMessageOutput::Gemini(response))
            }
        }
    }
}
```

And add a new variant to the `SendMessageOutput` enum:

```rust
pub enum SendMessageOutput {
    Codewhisperer(
        amzn_codewhisperer_streaming_client::operation::generate_assistant_response::GenerateAssistantResponseOutput,
    ),
    QDeveloper(amzn_qdeveloper_streaming_client::operation::send_message::SendMessageOutput),
    Gemini(GeminiResponse),
    Mock(Vec<ChatResponseStream>),
}
```

## Error Handling

We will implement comprehensive error handling for the Gemini backend, including:

- API key validation
- Network errors
- Rate limiting
- Invalid responses
- Model-specific errors

Errors will be mapped to the existing `ApiClientError` enum to maintain compatibility with the rest of the codebase.

## Security Considerations

1. **API Key Storage**: The API key will be stored in a configuration file with appropriate file permissions (readable only by the user).
2. **API Key Transmission**: The API key will be transmitted securely over HTTPS when making requests to the Gemini API.
3. **Error Messages**: Error messages will not include sensitive information like API keys.

## Testing Strategy

1. **Unit Tests**: Test individual components like configuration loading and data structure conversion.
2. **Integration Tests**: Test the integration between the Gemini client and the `StreamingClient` struct.
3. **Mock Tests**: Use mock responses to test the handling of various Gemini API responses.
4. **End-to-End Tests**: Test the complete flow from user input to model response.

## Implementation Plan

1. Create configuration management functions
2. Implement Gemini client and data structure conversion
3. Update `StreamingClient` to include Gemini backend
4. Add error handling and tests
5. Update documentation

## Future Extensibility

The design allows for future extensibility to support other model backends by:

1. Adding new variants to the `Inner` enum
2. Implementing new client creation methods
3. Adding new variants to the `SendMessageOutput` enum
4. Implementing conversion functions for new backends

This approach maintains the existing pattern in the codebase and allows for easy addition of new backends in the future.
