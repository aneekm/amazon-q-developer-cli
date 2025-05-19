# StreamingClient Structure Research

This document examines the structure of the existing streaming clients and how they integrate with the Amazon Q CLI chat functionality.

## Overview

The Amazon Q CLI chat functionality uses a `StreamingClient` struct to interact with different model backends. Currently, it supports two backends:

1. `CodewhispererStreamingClient`
2. `QDeveloperStreamingClient`

The `StreamingClient` struct is designed to abstract away the differences between these backends and provide a unified interface for the chat functionality.

## StreamingClient Implementation

### Inner Enum

The `StreamingClient` struct uses an enum called `Inner` to represent the different backends:

```rust
enum Inner {
    Codewhisperer(CodewhispererStreamingClient),
    QDeveloper(QDeveloperStreamingClient),
    Mock(Arc<Mutex<std::vec::IntoIter<Vec<ChatResponseStream>>>>),
}
```

This enum allows the `StreamingClient` to contain different client implementations while providing a unified interface.

### Client Creation

The `StreamingClient` struct provides several methods for creating clients:

1. `new()`: Creates a client based on environment conditions
2. `new_codewhisperer_client()`: Creates a CodeWhisperer client
3. `new_qdeveloper_client()`: Creates a QDeveloper client
4. `mock()`: Creates a mock client for testing

The `new()` method already includes a check for Gemini configuration and attempts to load it, but currently doesn't create a Gemini client.

### Message Sending

The `send_message()` method is responsible for sending messages to the model and handling the response:

```rust
pub async fn send_message(
    &self,
    conversation_state: ConversationState,
) -> Result<SendMessageOutput, ApiClientError> {
    match &self.inner {
        Inner::Codewhisperer(client) => {
            // Convert conversation_state to Codewhisperer format
            // Send message to Codewhisperer client
            // Return SendMessageOutput::Codewhisperer
        },
        Inner::QDeveloper(client) => {
            // Convert conversation_state to QDeveloper format
            // Send message to QDeveloper client
            // Return SendMessageOutput::QDeveloper
        },
        Inner::Mock(events) => {
            // Return mock events
        },
    }
}
```

### Response Handling

The `SendMessageOutput` enum represents the response from the model:

```rust
pub enum SendMessageOutput {
    Codewhisperer(
        amzn_codewhisperer_streaming_client::operation::generate_assistant_response::GenerateAssistantResponseOutput,
    ),
    QDeveloper(amzn_qdeveloper_streaming_client::operation::send_message::SendMessageOutput),
    Mock(Vec<ChatResponseStream>),
}
```

The `SendMessageOutput` enum provides methods for extracting the response content:

1. `request_id()`: Returns the request ID from the response
2. `recv()`: Receives the next event from the response stream

## Streaming Client Crates

### Common Structure

Both the `amzn-codewhisperer-streaming-client` and `amzn-qdeveloper-streaming-client` crates have a similar structure:

1. `Client` struct: The main client for interacting with the API
2. `types` module: Defines the data structures used for communication
3. `operation` module: Contains the operations that can be performed
4. `config` module: Handles client configuration

### Client Implementation

The `Client` struct in both crates provides methods for creating a client and sending messages:

```rust
pub struct Client {
    // Client implementation details
}

impl Client {
    pub fn from_conf(conf: Config) -> Self {
        // Create client from configuration
    }

    pub fn generate_assistant_response(&self) -> operation::generate_assistant_response::Builder {
        // Create operation builder
    }

    pub fn send_message(&self) -> operation::send_message::Builder {
        // Create operation builder
    }
}
```

### Operation Implementation

The operations in both crates follow a similar pattern:

1. `Builder` struct: Builds the operation request
2. `Input` struct: Represents the operation input
3. `Output` struct: Represents the operation output

For example, the `send_message` operation in `amzn-qdeveloper-streaming-client`:

```rust
pub struct SendMessage;

impl SendMessage {
    pub fn new() -> Self {
        Self
    }

    pub(crate) async fn orchestrate(
        runtime_plugins: &RuntimePlugins,
        input: SendMessageInput,
    ) -> Result<SendMessageOutput, SdkError<SendMessageError, HttpResponse>> {
        // Orchestrate the operation
    }
}

pub struct SendMessageInput {
    // Input fields
}

pub struct SendMessageOutput {
    pub send_message_response: EventReceiver<ChatResponseStream, ChatResponseStreamError>,
    _request_id: Option<String>,
}
```

## Data Structure Conversion

The `chat-cli` crate provides conversion implementations between its own data structures and the data structures used by the streaming clients:

```rust
impl From<UserInputMessage> for amzn_codewhisperer_streaming_client::types::UserInputMessage {
    fn from(value: UserInputMessage) -> Self {
        // Convert UserInputMessage to Codewhisperer UserInputMessage
    }
}

impl From<UserInputMessage> for amzn_qdeveloper_streaming_client::types::UserInputMessage {
    fn from(value: UserInputMessage) -> Self {
        // Convert UserInputMessage to QDeveloper UserInputMessage
    }
}
```

These conversions are used in the `send_message()` method to convert the `ConversationState` to the format expected by the streaming client.

## Gemini Streaming Client

The `gemini_streaming_client` crate follows a similar structure to the existing streaming clients:

1. `GeminiStreamingClient` struct: The main client for interacting with the Gemini API
2. `types` module: Defines the data structures used for communication
3. `config` module: Handles client configuration
4. `error` module: Defines the errors that can occur

The `GeminiStreamingClient` struct provides methods for creating a client and sending messages:

```rust
pub struct GeminiStreamingClient {
    api_key: String,
    model: String,
    temperature: f32,
    client: reqwest::Client,
}

impl GeminiStreamingClient {
    pub fn new(config: GeminiConfig) -> Self {
        // Create client from configuration
    }

    pub async fn test_connection(&self) -> Result<GeminiResponse, GeminiError> {
        // Test connection to Gemini API
    }

    pub async fn generate_content(
        &self,
        request: GeminiRequest,
    ) -> Result<GeminiResponse, GeminiError> {
        // Send request to Gemini API
    }
}
```

## Integration Points

To integrate the Gemini streaming client with the `StreamingClient` struct, we need to:

1. Add a new variant to the `Inner` enum:
   ```rust
   enum Inner {
       Codewhisperer(CodewhispererStreamingClient),
       QDeveloper(QDeveloperStreamingClient),
       Gemini(GeminiStreamingClient),
       Mock(Arc<Mutex<std::vec::IntoIter<Vec<ChatResponseStream>>>>),
   }
   ```

2. Implement a new client creation method:
   ```rust
   pub async fn new_gemini_client() -> Result<Self, ApiClientError> {
       // Load Gemini configuration
       // Create Gemini client
       // Return StreamingClient with Gemini client
   }
   ```

3. Update the `send_message()` method to handle the Gemini backend:
   ```rust
   match &self.inner {
       Inner::Gemini(client) => {
           // Convert conversation_state to Gemini format
           // Send message to Gemini client
           // Return SendMessageOutput::Gemini
       },
       // Other cases
   }
   ```

4. Add a new variant to the `SendMessageOutput` enum:
   ```rust
   pub enum SendMessageOutput {
       Codewhisperer(...),
       QDeveloper(...),
       Gemini(GeminiResponse),
       Mock(...),
   }
   ```

5. Implement conversion functions between Amazon Q's data structures and Gemini's API format:
   ```rust
   impl From<ConversationState> for GeminiRequest {
       fn from(state: ConversationState) -> Self {
           // Convert ConversationState to GeminiRequest
       }
   }

   impl From<GeminiResponse> for Vec<ChatResponseStream> {
       fn from(response: GeminiResponse) -> Self {
           // Convert GeminiResponse to Vec<ChatResponseStream>
       }
   }
   ```

## Conclusion

The existing `StreamingClient` structure provides a good foundation for integrating the Gemini backend. By following the same pattern as the existing backends, we can add Gemini support with minimal changes to the existing codebase.

The key components we need to implement are:

1. Data structure conversion between Amazon Q and Gemini formats
2. Integration with the `StreamingClient` struct
3. Error handling and response processing

The existing code already has a pattern of converting between different backend formats, which we can follow for our implementation.
