# Research: chat_cli Crate Structure

This document examines the structure of the `chat_cli` crate to understand how to implement a Gemini backend.

## Key Components to Investigate

1. `StreamingClient` implementation
2. Model interaction abstractions
3. Tool execution handling
4. Configuration management

## Research Findings

### Current Structure

The `chat_cli` crate contains several key components that are relevant to our implementation:

#### StreamingClient

The `StreamingClient` struct in `./crates/chat-cli/src/api_client/clients/streaming_client.rs` is the main interface for interacting with the model. It has the following key features:

1. **Multiple Backend Support**: It already supports multiple backends through an enum called `Inner`:
   ```rust
   enum Inner {
       Codewhisperer(CodewhispererStreamingClient),
       QDeveloper(QDeveloperStreamingClient),
       Mock(Arc<Mutex<std::vec::IntoIter<Vec<ChatResponseStream>>>>),
   }
   ```

2. **Client Creation Methods**:
   - `new()`: Creates a client based on environment conditions
   - `new_codewhisperer_client()`: Creates a CodeWhisperer client
   - `new_qdeveloper_client()`: Creates a QDeveloper client
   - `mock()`: Creates a mock client for testing

3. **Message Sending**:
   - `send_message()`: Sends a message to the model and returns a `SendMessageOutput`

4. **Response Handling**:
   - `SendMessageOutput` enum with variants for each backend
   - Methods to extract response content from the output

#### Model Types

The `model.rs` file defines the data structures used for communication with the model:

1. **Conversation State**:
   ```rust
   pub struct ConversationState {
       pub conversation_id: Option<String>,
       pub user_input_message: UserInputMessage,
       pub history: Option<Vec<ChatMessage>>,
   }
   ```

2. **Messages**:
   ```rust
   pub enum ChatMessage {
       AssistantResponseMessage(AssistantResponseMessage),
       UserInputMessage(UserInputMessage),
   }
   ```

3. **Tool Definitions**:
   ```rust
   pub enum Tool {
       ToolSpecification(ToolSpecification),
   }
   
   pub struct ToolSpecification {
       pub name: String,
       pub description: String,
       pub input_schema: ToolInputSchema,
   }
   ```

4. **Tool Execution**:
   ```rust
   pub struct ToolUse {
       pub tool_use_id: String,
       pub name: String,
       pub input: Document,
   }
   
   pub struct ToolResult {
       pub tool_use_id: String,
       pub content: Vec<ToolResultContentBlock>,
       pub status: ToolResultStatus,
   }
   ```

5. **Response Streaming**:
   ```rust
   pub enum ChatResponseStream {
       AssistantResponseEvent { content: String },
       CodeEvent { content: String },
       // ... other event types
       ToolUseEvent { tool_use_id: String, name: String, input: Option<String>, stop: Option<bool> },
   }
   ```

### Integration Points

To implement a Gemini backend, we would need to:

1. Add a new variant to the `Inner` enum in `StreamingClient`
2. Implement a new client creation method similar to `new_codewhisperer_client()`
3. Update the `send_message()` method to handle the Gemini backend
4. Implement conversion functions between Amazon Q's data structures and Gemini's API format
5. Add configuration loading for Gemini API keys and model selection

The existing code already has a pattern of converting between different backend formats, which we can follow for our implementation.
