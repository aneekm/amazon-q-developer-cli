# Idea Honing - Gemini Backend for Amazon Q CLI

This document captures the requirements clarification process for implementing a Gemini model backend for Amazon Q CLI chat.

## Question 1
What specific features or capabilities of the Gemini model are you looking to leverage in the Amazon Q CLI chat?

## Answer 1
Nothing specific - I just want the ability to use my Gemini API key to drive the tool using a Gemini model instead of being tied to whatever model backs Amazon Q.

## Question 2
How would you like users to configure their Gemini API key and other settings? (e.g., through command-line arguments, environment variables, configuration files)

## Answer 2
I think a config file seems most appropriate, similar to the common mcp.json for configuring MCP servers.

## Question 3
What specific Gemini model(s) would you like to support initially? (e.g., Gemini 1.0 Pro, Gemini 1.5 Pro, Gemini 1.5 Flash)

## Answer 3
The implementation should be flexible, allowing users to specify which Gemini model they want to use as part of the configuration along with the API key. Since the Gemini API allows passing a model name parameter, this approach would enable users to select any available Gemini model without requiring code changes.

## Question 4
How should the Amazon Q CLI handle tool execution when using the Gemini backend? (Gemini might not support the same tool execution capabilities as Amazon Q's native backend)

## Answer 4
The implementation should be minimally invasive to the existing code and use the lowest possible abstraction layer to leverage Amazon Q's native feature set as much as possible. At the lowest level, the code should translate Q tool specifications to Gemini's format and translate response requests for tool execution back to Q's expected format.

## Question 5
How should the implementation handle authentication and API key management to ensure security?

## Answer 5
The implementation should store the API key in a user-managed configuration file with appropriate file permissions (readable only by the user). This approach is similar to how other CLI tools handle API keys and secrets.

## Question 6
Which part of the codebase should be modified to implement the Gemini backend?

## Answer 6
The implementation should target the standalone chat_cli crate, which appears to be the new version of the chat functionality that's currently being refactored. The right level of abstraction would be to create new implementations of the StreamingClient struct for Gemini, as this wraps the interactions with the model.
