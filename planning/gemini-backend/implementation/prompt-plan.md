# Implementation Prompt Plan

## Checklist
- [ ] Prompt 1: Create configuration management for Gemini API
- [ ] Prompt 2: Implement Gemini API client and request/response types
- [ ] Prompt 3: Implement data structure conversion between Amazon Q and Gemini
- [ ] Prompt 4: Integrate Gemini client with StreamingClient
- [ ] Prompt 5: Implement tool execution conversion
- [ ] Prompt 6: Add error handling and tests
- [ ] Prompt 7: Update documentation

## Prompts

### Prompt 1: Create configuration management for Gemini API
Implement the configuration management for the Gemini API integration with Amazon Q CLI. Create a module that handles loading, saving, and validating Gemini API configuration from a JSON file at `~/.aws/amazonq/gemini_config.json`.

1. Create a new module `gemini_config.rs` in the appropriate directory
2. Define a struct `GeminiConfig` with fields for `api_key`, `model`, and `temperature`
3. Implement functions to check if a Gemini configuration exists, load the configuration, and validate the configuration
4. Ensure proper error handling for missing or invalid configuration
5. Add logging that clearly indicates whether the Gemini configuration was found and loaded successfully or if there were errors

The configuration file should have the following structure:
```json
{
  "api_key": "YOUR_GEMINI_API_KEY",
  "model": "gemini-2.0-flash",
  "temperature": 0.7
}
```

Make sure to handle cases where the configuration file doesn't exist or is invalid, and provide appropriate error messages.

**User-Facing Behavior to Verify:**
1. Users should be able to create a `~/.aws/amazonq/gemini_config.json` file with their API key, model, and temperature settings
2. The implementation should output log messages that show:
   - Success: "Gemini configuration found and loaded successfully" (with basic details like model name)
   - Error: Clear error messages for cases like "Gemini configuration file not found" or "Invalid Gemini configuration format"
3. These log messages should be visible when running the CLI with appropriate verbosity settings

### Prompt 2: Implement Gemini API client and request/response types
Create the core Gemini API client and the necessary request/response types for interacting with the Gemini API. This should include all the data structures needed to represent Gemini's API format.

1. Create a new module `gemini_client.rs` in the appropriate directory
2. Define the request and response types for the Gemini API:
   - `GeminiRequest` with fields for contents, tools, and generation configuration
   - `GeminiResponse` with fields for candidates and other response data
   - Supporting types like `GeminiContent`, `GeminiPart`, `GeminiTool`, etc.
3. Implement a `GeminiStreamingClient` struct with methods for:
   - Creating a new client with an API key, model, and temperature
   - Sending a message to the Gemini API
   - Processing the response from the Gemini API
4. Use the `reqwest` crate for HTTP requests to the Gemini API
5. Implement proper error handling for API requests
6. Add a test call during StreamingClient initialization that logs the full request/response details when a Gemini configuration is found

Ensure that the client can handle the basic functionality of sending a message to the Gemini API and receiving a response, without worrying about conversion to/from Amazon Q's data structures yet.

**User-Facing Behavior to Verify:**
1. When the StreamingClient is initialized and a Gemini configuration is found, the code will:
   - Make a simple test call to the Gemini API with a prompt like "Hello world"
   - Log the request details (non-sensitive) and the complete response
   - This happens automatically during initialization, with no user action required
2. Users can verify functionality by checking logs that show:
   - "Testing Gemini API connection with configuration from ~/.aws/amazonq/gemini_config.json"
   - The complete response from the Gemini API
   - Or appropriate error messages if the test fails

### Prompt 3: Implement data structure conversion between Amazon Q and Gemini
Implement the conversion functions between Amazon Q's data structures and Gemini's API format. This should include conversions for conversation state, messages, and other relevant data structures.

1. Implement `From<ConversationState> for GeminiRequest` to convert Amazon Q's conversation state to a Gemini API request
2. Implement `From<GeminiResponse> for Vec<ChatResponseStream>` to convert a Gemini API response to Amazon Q's chat response stream format
3. Implement any other necessary conversion functions between Amazon Q and Gemini data structures
4. Ensure that the conversions handle all the necessary fields and maintain the conversation context
5. Add logging to show the conversion process and results

Focus on the basic conversation flow first, without worrying about tool execution yet. Make sure that the conversions correctly handle the conversation history, user messages, and assistant responses.

**User-Facing Behavior to Verify:**
1. Building on the test call from Prompt 2, the enhanced test will:
   - Convert a simple Amazon Q ConversationState to a Gemini request
   - Convert the Gemini response back to Amazon Q's ChatResponseStream format
   - Log both the original structures and the converted structures
2. Users can verify in the logs:
   - "Converting Amazon Q ConversationState to Gemini request format"
   - "Converting Gemini response to Amazon Q ChatResponseStream format"
   - Details of the conversion results
   - Any conversion errors that might occur

### Prompt 4: Integrate Gemini client with StreamingClient
Integrate the Gemini client with the existing `StreamingClient` struct in the Amazon Q CLI codebase. This should allow the Gemini client to be used as an alternative backend for the chat functionality.

1. Update the `Inner` enum in `StreamingClient` to include a new variant for the Gemini client
2. Implement a new client creation method `new_gemini_client()` for the `StreamingClient` struct
3. Update the `new()` method to check for Gemini configuration and use the Gemini client if available
4. Update the `send_message()` method to handle the Gemini backend
5. Add a new variant to the `SendMessageOutput` enum for Gemini responses
6. Implement the necessary methods for the `SendMessageOutput` enum to handle Gemini responses

Ensure that the integration maintains compatibility with the existing codebase and doesn't break any existing functionality.

**User-Facing Behavior to Verify:**
1. Users should be able to start a chat session with `q chat` and:
   - If a valid Gemini configuration exists, the chat will use the Gemini backend
   - If no Gemini configuration exists, the chat will use the default Amazon Q backend
2. The chat interface should work the same way regardless of which backend is being used
3. Log messages should indicate which backend is being used:
   - "Using Gemini backend with model: [model name]"
   - "Using default Amazon Q backend"

### Prompt 5: Implement tool execution conversion
Implement the conversion functions for tool execution between Amazon Q and Gemini. This should allow the Gemini backend to use Amazon Q's tool execution capabilities.

1. Implement `From<Tool> for GeminiFunctionDeclaration` to convert Amazon Q's tool specifications to Gemini's function declarations
2. Implement the necessary logic to convert Gemini's function calls to Amazon Q's tool use events
3. Implement the necessary logic to convert Amazon Q's tool results to Gemini's function responses
4. Update the conversation state conversion to include tool specifications and tool results
5. Ensure that the tool execution flow works correctly with the Gemini backend

This is a critical part of the implementation, as it allows the Gemini backend to leverage Amazon Q's existing tool execution capabilities.

**User-Facing Behavior to Verify:**
1. Users should be able to use tools (like file operations, bash commands, etc.) in their chat with the Gemini backend
2. The tool execution flow should work the same way as with the default Amazon Q backend:
   - The model requests to use a tool
   - The user is prompted for permission if needed
   - The tool is executed
   - The result is sent back to the model
3. Log messages should show the tool execution conversion process:
   - "Converting Amazon Q tool specifications to Gemini function declarations"
   - "Converting Gemini function call to Amazon Q tool use event"
   - "Converting Amazon Q tool result to Gemini function response"

### Prompt 6: Add error handling and tests
Implement comprehensive error handling for the Gemini backend and add tests to ensure that the implementation works correctly.

1. Update the `ApiClientError` enum to include Gemini-specific errors
2. Implement error handling for API key validation, network errors, rate limiting, invalid responses, and model-specific errors
3. Ensure that error messages don't include sensitive information like API keys
4. Add unit tests for configuration loading, data structure conversion, and other components
5. Add integration tests for the Gemini client and its integration with the `StreamingClient` struct
6. Add mock tests for handling various Gemini API responses
7. Add end-to-end tests for the complete flow from user input to model response

Ensure that the error handling is comprehensive and that the tests cover all the important functionality.

**User-Facing Behavior to Verify:**
1. All tests should pass, including:
   - Unit tests for configuration loading and data structure conversion
   - Integration tests for the Gemini client
   - End-to-end tests for the complete flow
2. Error messages should be clear and helpful when things go wrong:
   - "Invalid Gemini API key" instead of generic HTTP errors
   - "Rate limit exceeded for Gemini API" for rate limiting issues
   - "Network error connecting to Gemini API" for connectivity issues
3. Error messages should not contain sensitive information like API keys

### Prompt 7: Update documentation
Update the documentation to include information about the Gemini backend and how to use it.

1. Add documentation comments to all new code
2. Update the README or other documentation files to include information about the Gemini backend
3. Add instructions for configuring the Gemini backend, including how to obtain and configure a Gemini API key
4. Add information about supported Gemini models and their capabilities
5. Add troubleshooting information for common issues

Ensure that the documentation is clear, comprehensive, and easy to follow.
