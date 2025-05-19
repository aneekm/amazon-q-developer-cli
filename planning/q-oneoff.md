# Gemini Backend Integration Fix

## Overview

This document outlines the steps needed to fix the current build issues with the Gemini backend integration for Amazon Q CLI. The main issue is that the `gemini_streaming_client` crate is trying to directly reference types from the `chat_cli` crate, creating a circular dependency.

## Current Issues

1. **Circular Dependency**: The `gemini_streaming_client` crate is directly referencing `chat_cli` types, which creates a circular dependency.
2. **Streaming Implementation Issues**: There are issues with the streaming implementation in the client.
3. **Type Conversion Problems**: The conversion between Amazon Q types and Gemini types is implemented in the wrong place.

## Proper Architecture

The correct architecture should follow the pattern used by other streaming clients:

1. Each streaming client defines its own type system
2. The `chat_cli` crate is responsible for converting between its types and the streaming client types
3. The streaming client only works with its own types

## Implementation Plan

### 1. Fix `gemini_streaming_client` Crate

#### a. Remove Direct References to `chat_cli` Types

- Update `conversion.rs` to remove direct references to `chat_cli` types
- Create a mock module in `conversion.rs` for testing purposes only
- Ensure the client only works with its own types

#### b. Fix Streaming Implementation

- Update the streaming implementation in `client.rs` to properly handle byte streams
- Fix the `bytes_stream()` method usage and related errors
- Ensure proper error handling for streaming responses

#### c. Fix Number Type Methods

- Update the `document_to_json_value` function to properly handle `Number` types
- Use the correct methods for accessing number values

### 2. Update `chat_cli` Integration

#### a. Update `Inner` Enum

- Add a new variant to the `Inner` enum in `StreamingClient` for Gemini

```rust
enum Inner {
    Codewhisperer(CodewhispererStreamingClient),
    QDeveloper(QDeveloperStreamingClient),
    Gemini(GeminiStreamingClient),
    Mock(Arc<Mutex<std::vec::IntoIter<Vec<ChatResponseStream>>>>),
}
```

#### b. Implement `new_gemini_client()` Method

- Create a new method in `StreamingClient` to create a Gemini client
- Load the Gemini configuration
- Create and return a new Gemini client

#### c. Update `send_message()` Method

- Update the `send_message()` method to handle the Gemini backend
- Implement conversion from `chat_cli` types to Gemini types
- Handle the response from the Gemini client

#### d. Add Gemini Variant to `SendMessageOutput`

- Add a new variant to the `SendMessageOutput` enum for Gemini responses
- Implement necessary methods for handling Gemini responses

#### e. Implement Type Conversions

- Implement conversion functions between `chat_cli` types and `gemini_streaming_client` types
- Follow the pattern used for other streaming clients

## Testing Plan

1. Verify that the `gemini_streaming_client` crate builds successfully
2. Test the Gemini client with a simple request
3. Test the full integration with the `chat_cli` crate
4. Verify that tool execution works correctly

## Future Work

1. Implement comprehensive error handling
2. Add unit tests for the Gemini client
3. Add integration tests for the full flow
4. Update documentation to include information about the Gemini backend

## References

- Existing streaming client implementations (`amzn_codewhisperer_streaming_client`, `amzn_qdeveloper_streaming_client`)
- Gemini API documentation
- Current implementation in `planning/gemini-backend/`
