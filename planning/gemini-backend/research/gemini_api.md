# Research: Gemini API

This document examines the Gemini API to understand how to implement a Gemini backend for Amazon Q CLI chat.

## API Endpoints

The Gemini API uses REST endpoints for communication:

```
https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent
```

## Authentication

Authentication is done via API key, which is passed as a query parameter:

```
?key=YOUR_API_KEY
```

## Function Calling

Gemini supports function calling, which allows the model to request the execution of specific functions. This is similar to Amazon Q's tool execution capability.

### Function Declaration Format

Functions are declared using a JSON schema format:

```json
{
  "name": "function_name",
  "description": "Function description",
  "parameters": {
    "type": "object",
    "properties": {
      "param1": {
        "type": "string",
        "description": "Parameter description"
      }
    },
    "required": ["param1"]
  }
}
```

### Function Calling Flow

1. The client sends a request with function declarations
2. The model may respond with a function call
3. The client executes the function
4. The client sends the function result back to the model
5. The model generates a final response

## Models

Gemini offers several models with different capabilities:

- Gemini 2.0 Flash: Supports function calling and parallel function calling
- Gemini 1.5 Pro: Supports function calling and parallel function calling
- Gemini 1.5 Flash: Supports function calling and parallel function calling

## Request Format

A basic request to the Gemini API looks like:

```json
{
  "contents": [
    {
      "parts": [
        {
          "text": "Your prompt here"
        }
      ]
    }
  ]
}
```

With function calling:

```json
{
  "contents": [
    {
      "parts": [
        {
          "text": "Your prompt here"
        }
      ]
    }
  ],
  "tools": [
    {
      "functionDeclarations": [
        {
          "name": "function_name",
          "description": "Function description",
          "parameters": {
            "type": "object",
            "properties": {
              "param1": {
                "type": "string",
                "description": "Parameter description"
              }
            },
            "required": ["param1"]
          }
        }
      ]
    }
  ]
}
```

## Response Format

The response from Gemini includes the generated content and any function calls:

```json
{
  "candidates": [
    {
      "content": {
        "parts": [
          {
            "functionCall": {
              "name": "function_name",
              "args": {
                "param1": "value1"
              }
            }
          }
        ]
      }
    }
  ]
}
```

## Function Call Response

After executing a function, the client sends the result back to the model:

```json
{
  "contents": [
    {
      "role": "user",
      "parts": [
        {
          "text": "Your prompt here"
        }
      ]
    },
    {
      "role": "model",
      "parts": [
        {
          "functionCall": {
            "name": "function_name",
            "args": {
              "param1": "value1"
            }
          }
        }
      ]
    },
    {
      "role": "user",
      "parts": [
        {
          "functionResponse": {
            "name": "function_name",
            "response": {
              "result": "function result"
            }
          }
        }
      ]
    }
  ]
}
```
