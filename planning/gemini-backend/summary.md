# Project Summary: Gemini Backend for Amazon Q CLI

## Overview

This project aims to implement a Gemini model backend for the Amazon Q CLI chat functionality. The implementation will allow users to leverage Google's Gemini models as an alternative to the default Amazon Q backend, using their own Gemini API key.

## Directory Structure

- planning/gemini-backend/
  - rough-idea.md (initial concept)
  - idea-honing.md (requirements clarification)
  - research/
    - chat_cli_structure.md (analysis of existing codebase)
    - gemini_api.md (Gemini API documentation)
  - design/
    - detailed-design.md (comprehensive design document)
  - implementation/
    - prompt-plan.md (implementation steps)
  - summary.md (this document)

## Key Design Elements

### Configuration Management

- JSON configuration file at `~/.aws/amazonq/gemini_config.json`
- Stores API key, model selection, and temperature
- Secure file permissions to protect API key

### Gemini Client Implementation

- New `gemini_streaming_client` crate for interacting with Gemini API
- HTTP requests using the `reqwest` crate
- Conversion between Amazon Q and Gemini data formats

### StreamingClient Integration

- New variant in the `Inner` enum for Gemini
- New client creation method `new_gemini_client()`
- Updated `send_message()` method to handle Gemini backend
- New variant in the `SendMessageOutput` enum for Gemini responses

### Tool Execution Support

- Conversion between Amazon Q's tool specifications and Gemini's function declarations
- Handling of function calls and responses
- Maintaining compatibility with existing tool execution flow

### Error Handling

- Comprehensive error handling for API key validation, network errors, rate limiting, etc.
- Mapping to existing `ApiClientError` enum
- Protection of sensitive information in error messages

## Implementation Approach

The implementation is broken down into 7 main steps:

1. Create configuration management for Gemini API
2. Implement Gemini API client and request/response types
3. Implement data structure conversion between Amazon Q and Gemini
4. Integrate Gemini client with StreamingClient
5. Implement tool execution conversion
6. Add error handling and tests
7. Update documentation

## Next Steps

1. Begin implementation following the prompt plan
2. Focus on one component at a time, ensuring each is fully functional before moving to the next
3. Maintain compatibility with the existing codebase
4. Add comprehensive tests for each component
5. Update documentation to include information about the Gemini backend

## Future Extensibility

The design allows for future extensibility to support other model backends by:

1. Adding new variants to the `Inner` enum
2. Implementing new client creation methods
3. Adding new variants to the `SendMessageOutput` enum
4. Implementing conversion functions for new backends

This approach maintains the existing pattern in the codebase and allows for easy addition of new backends in the future.
