# Tool Execution Conversion Format Analysis

This document analyzes the format used by Amazon Q CLI to send tools to a model and how it should be converted for the Gemini API.

## Amazon Q CLI Tool Format

### Tool Specification

In the Amazon Q CLI, tools are specified using the following structure:

```rust
pub enum Tool {
    ToolSpecification(ToolSpecification),
}

pub struct ToolSpecification {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
}

pub struct ToolInputSchema {
    pub schema: Document,
}

pub enum Document {
    Object(HashMap<String, Document>),
    Array(Vec<Document>),
    String(String),
    Number(Number),
    Boolean(bool),
    Null,
}
```

The `schema` field in `ToolInputSchema` contains a JSON Schema that defines the parameters for the tool. This is typically represented as a JSON object with the following structure:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "parameter1": {
      "type": "string",
      "description": "Description of parameter1"
    },
    "parameter2": {
      "type": "number",
      "description": "Description of parameter2"
    }
  },
  "required": ["parameter1"]
}
```

### Tool Use Event

When the model wants to use a tool, it sends a `ToolUseEvent` with the following structure:

```rust
pub struct ToolUse {
    pub tool_use_id: String,
    pub name: String,
    pub input: Document,
}
```

The `input` field contains the parameters for the tool as a `Document` object, which is a recursive enum that can represent any JSON value.

### Tool Result

After executing a tool, the result is sent back to the model with the following structure:

```rust
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: Vec<ToolResultContentBlock>,
    pub status: ToolResultStatus,
}

pub enum ToolResultContentBlock {
    Text(String),
}

pub enum ToolResultStatus {
    Success,
    Error,
}
```

## Gemini API Tool Format

### Function Declaration

In the Gemini API, tools are specified using function declarations following the OpenAPI schema format (a subset of JSON Schema):

```json
{
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

Key points about function declarations:
- Use descriptive function names without spaces (use underscores or camelCase)
- Provide detailed descriptions to help the model understand when to use the function
- The `parameters` field must follow the OpenAPI schema format
- Parameter descriptions should include examples and constraints when helpful

### Function Call

When the model wants to call a function, it responds with a function call in camelCase format:

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

Note that the field name is `functionCall` (camelCase), which is important for proper serialization/deserialization.

### Function Response

After executing a function, the result is sent back to the model as a user message with a `functionResponse` part:

```json
{
  "contents": [
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

Important notes:
- The `functionResponse` is added as a user message in the conversation history
- The field name is `functionResponse` (camelCase)
- The response should be structured as a JSON object

### Complete Function Call Flow

The complete flow for function calling in Gemini involves:
1. Sending a request with function declarations
2. Receiving a response with a function call
3. Executing the function locally
4. Sending the function result back to the model as a user message
5. Receiving a final response that incorporates the function result

## Conversion Issues

The main issues with the current implementation appear to be:

1. **Field Naming**: The Gemini API uses camelCase for field names like `functionCall` and `functionResponse`, which must be correctly handled in serialization/deserialization.

2. **Parameter Format**: The `parameters` field in Gemini's function declaration should follow the OpenAPI schema format, which is a subset of JSON Schema.

3. **Response Structure**: The function response must be added to the conversation history as a user message with a `functionResponse` part.

4. **Conversation History**: We need to maintain the correct conversation history when sending function results back to the model.

## Correct Conversion Implementation

### Tool Specification to Function Declaration

```rust
impl From<Tool> for GeminiFunctionDeclaration {
    fn from(tool: Tool) -> Self {
        match tool {
            Tool::ToolSpecification(spec) => {
                let parameters = match spec.input_schema.schema {
                    Document::Object(obj) => obj,
                    _ => HashMap::new(),
                };
                
                Self {
                    name: spec.name,
                    description: spec.description,
                    parameters: serde_json::to_value(parameters).unwrap_or(serde_json::Value::Null),
                }
            }
        }
    }
}
```

With proper serde attributes for serialization:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
```

### Function Call to Tool Use Event

```rust
fn function_call_to_tool_use(function_call: &GeminiFunctionCall) -> ToolUse {
    let input = serde_json::to_value(&function_call.args)
        .map(document_from_json_value)
        .unwrap_or(Document::Null);
    
    ToolUse {
        tool_use_id: Uuid::new_v4().to_string(),
        name: function_call.name.clone(),
        input,
    }
}
```

With proper serde attributes for deserialization:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiPart {
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<GeminiFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    // Other part types...
}
```

### Tool Result to Function Response

```rust
fn tool_result_to_function_response(tool_result: &ToolResult) -> GeminiFunctionResponse {
    let content = tool_result.content.iter()
        .map(|block| match block {
            ToolResultContentBlock::Text(text) => text.clone(),
        })
        .collect::<Vec<String>>()
        .join("\n");
    
    let response = match tool_result.status {
        ToolResultStatus::Success => serde_json::json!({ "result": content }),
        ToolResultStatus::Error => serde_json::json!({ "error": content }),
    };
    
    GeminiFunctionResponse {
        name: tool_result.tool_use_id.clone(),
        response,
    }
}
```

With proper serde attributes for serialization:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiFunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeminiPart {
    // ... other fields
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    pub function_response: Option<GeminiFunctionResponse>,
}
```

### Adding Function Response to Conversation

```rust
fn add_function_response_to_conversation(
    conversation: &mut Vec<GeminiContent>,
    function_call: &GeminiFunctionCall,
    function_response: &GeminiFunctionResponse
) {
    // Add the model's function call message
    conversation.push(GeminiContent {
        role: Some("model".to_string()),
        parts: vec![GeminiPart {
            function_call: Some(function_call.clone()),
            text: None,
            function_response: None,
        }],
    });
    
    // Add the user's function response message
    conversation.push(GeminiContent {
        role: Some("user".to_string()),
        parts: vec![GeminiPart {
            function_call: None,
            text: None,
            function_response: Some(function_response.clone()),
        }],
    });
}
```

## Testing Strategy

To verify the correct conversion, we should:

1. Create a sample tool specification with various parameter types
2. Convert it to a Gemini function declaration
3. Verify that the parameters are correctly formatted and field names use the correct casing
4. Create a sample function call with various argument types
5. Convert it to a tool use event
6. Verify that the input is correctly formatted
7. Create a sample tool result with various content types
8. Convert it to a function response
9. Verify that the response is correctly formatted and added to the conversation history
10. Test the complete function call flow with the Gemini API

This testing should cover all the edge cases and ensure that the conversion works correctly for all types of tools and parameters.
