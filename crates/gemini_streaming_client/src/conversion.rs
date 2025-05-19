//! Conversion functions between Amazon Q and Gemini data structures.

use serde_json::Value;

use crate::types::{
    GeminiContent,
    GeminiFunctionCall,
    GeminiFunctionDeclaration,
    GeminiFunctionResponse,
    GeminiGenerationConfig,
    GeminiPart,
    GeminiRequest,
    GeminiTool,
};

/// Converts a conversation state to a Gemini request.
///
/// This function is meant to be used by the chat_cli crate, which will provide its own
/// ConversationState type. The implementation in this crate is for testing purposes only.
pub fn conversation_state_to_gemini_request(
    user_message: &MockChatMessage,
    history: &[MockChatMessage],
    tools: Option<&[MockTool]>,
    temperature: f32,
) -> GeminiRequest {
    // Create a vector to hold all the contents
    let mut contents = Vec::new();

    // Create a map to track tool call IDs to tool names
    let mut tool_id_to_name = std::collections::HashMap::new();

    // // First pass: build the tool ID to name mapping
    // for message in history.iter() {
    //     if let MockChatMessage::AssistantMessage { tool_uses, .. } = message {
    //         if let Some(tool_uses) = tool_uses {
    //             for tool_use in tool_uses {
    //                 // Store the mapping from the next message's potential tool_use_id to this tool's
    // name                 // This assumes that tool results follow tool uses in the conversation
    //                 tool_id_to_name.insert(tool_use.tool_use_id.clone(), tool_use.name.clone());
    //             }
    //         }
    //     }
    // }

    // Create a combined iterator of history + user_message
    let all_messages = history.iter().chain(std::iter::once(user_message));

    // Process all messages (history + current user message)
    for message in all_messages {
        match message {
            MockChatMessage::UserMessage { content, tool_results } => {
                // Add the user's text message
                if !content.is_empty() {
                    contents.push(GeminiContent {
                        role: Some("user".to_string()),
                        parts: vec![GeminiPart::Text { text: content.clone() }],
                    });
                }

                // Add any tool results as function responses
                if let Some(tool_results) = tool_results {
                    for result in tool_results {
                        // Look up the tool name from the ID
                        let tool_name = tool_id_to_name
                            .get(&result.tool_use_id)
                            .cloned()
                            .unwrap_or_else(|| result.tool_use_id.clone());

                        let function_response = tool_result_to_gemini_function_response(
                            &tool_name, // Use the tool name instead of the ID
                            &result.content,
                            &result.status,
                        );

                        contents.push(GeminiContent {
                            role: Some("user".to_string()),
                            parts: vec![GeminiPart::FunctionResponse { function_response }],
                        });
                    }
                }
            },
            MockChatMessage::AssistantMessage { content, tool_uses } => {
                // Add the assistant's text response only if it's not empty
                if !content.is_empty() {
                    contents.push(GeminiContent {
                        role: Some("model".to_string()),
                        parts: vec![GeminiPart::Text { text: content.clone() }],
                    });
                }

                // For each tool use, add a function call part
                if let Some(tool_uses) = tool_uses {
                    for tool_use in tool_uses {
                        // Store the mapping from tool_use_id to tool name
                        tool_id_to_name.insert(tool_use.tool_use_id.clone(), tool_use.name.clone());

                        contents.push(GeminiContent {
                            role: Some("model".to_string()),
                            parts: vec![GeminiPart::FunctionCall {
                                function_call: GeminiFunctionCall {
                                    name: tool_use.name.clone(),
                                    args: tool_use.args.clone(),
                                },
                            }],
                        });
                    }
                }
            },
        }
    }

    // Extract tools if they exist
    let tools = tools.map(tools_to_gemini_tools);

    // Create the Gemini request
    GeminiRequest {
        contents,
        tools,
        generation_config: Some(GeminiGenerationConfig {
            temperature: Some(temperature),
            max_output_tokens: Some(4096),
            top_k: None,
            top_p: None,
        }),
    }
}

/// Converts tools to Gemini tools.
fn tools_to_gemini_tools(tools: &[MockTool]) -> Vec<GeminiTool> {
    let mut function_declarations = Vec::new();

    for tool in tools {
        // Clean up the parameters to ensure they follow the OpenAPI schema format
        let parameters = clean_parameters_for_gemini(&tool.parameters);

        function_declarations.push(GeminiFunctionDeclaration {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters,
        });
    }

    vec![GeminiTool { function_declarations }]
}

/// Cleans up parameters to ensure they follow the OpenAPI schema format.
/// This function aggressively simplifies the schema to ensure compatibility with Gemini API.
pub fn clean_parameters_for_gemini(parameters: &Value) -> Value {
    // Start with a simplified schema structure
    let mut simplified = serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    });

    // Extract only the essential parts from the original schema
    if let Some(obj) = parameters.as_object() {
        // Copy required fields if they exist
        if let Some(required) = obj.get("required") {
            simplified["required"] = required.clone();
        }

        // Process properties if they exist
        if let Some(props) = obj.get("properties") {
            if let Some(props_obj) = props.as_object() {
                let mut simplified_props = serde_json::Map::new();

                // Process each property
                for (prop_name, prop_value) in props_obj {
                    if let Some(prop_obj) = prop_value.as_object() {
                        let mut simplified_prop = serde_json::Map::new();

                        // Copy only essential fields
                        if let Some(type_value) = prop_obj.get("type") {
                            // Handle array of types by taking the first one
                            if type_value.is_array() {
                                if let Some(first_type) = type_value.as_array().and_then(|arr| arr.first()) {
                                    simplified_prop.insert("type".to_string(), first_type.clone());
                                }
                            } else {
                                simplified_prop.insert("type".to_string(), type_value.clone());
                            }
                        } else {
                            // Default to string if no type is specified
                            simplified_prop.insert("type".to_string(), Value::String("string".to_string()));
                        }

                        // Copy description if it exists
                        if let Some(desc) = prop_obj.get("description") {
                            simplified_prop.insert("description".to_string(), desc.clone());
                        }

                        // Handle enum if it exists
                        if let Some(enum_values) = prop_obj.get("enum") {
                            simplified_prop.insert("enum".to_string(), enum_values.clone());
                        }

                        // Handle items for arrays
                        if let Some(type_value) = prop_obj.get("type") {
                            if type_value.is_string() && type_value.as_str() == Some("array") {
                                if let Some(items) = prop_obj.get("items") {
                                    // Recursively clean items
                                    let cleaned_items = clean_array_items(items);
                                    simplified_prop.insert("items".to_string(), cleaned_items);
                                } else {
                                    // Default items type if not specified
                                    simplified_prop.insert("items".to_string(), serde_json::json!({"type": "string"}));
                                }
                            }
                        }

                        // Handle nested objects
                        if let Some(type_value) = prop_obj.get("type") {
                            if type_value.is_string() && type_value.as_str() == Some("object") {
                                if let Some(_nested_props) = prop_obj.get("properties") {
                                    // Recursively clean nested properties
                                    let cleaned_props = clean_parameters_for_gemini(prop_value);
                                    if let Some(cleaned_props_obj) = cleaned_props.as_object() {
                                        if let Some(props) = cleaned_props_obj.get("properties") {
                                            simplified_prop.insert("properties".to_string(), props.clone());
                                        }
                                        if let Some(req) = cleaned_props_obj.get("required") {
                                            simplified_prop.insert("required".to_string(), req.clone());
                                        }
                                    }
                                }
                            }
                        }

                        simplified_props.insert(prop_name.clone(), Value::Object(simplified_prop));
                    }
                }

                simplified["properties"] = Value::Object(simplified_props);
            }
        }
    }

    simplified
}

/// Cleans array items to ensure they follow the OpenAPI schema format.
fn clean_array_items(items: &Value) -> Value {
    if let Some(obj) = items.as_object() {
        let mut simplified = serde_json::Map::new();

        // Copy only essential fields
        if let Some(type_value) = obj.get("type") {
            // Handle array of types by taking the first one
            if type_value.is_array() {
                if let Some(first_type) = type_value.as_array().and_then(|arr| arr.first()) {
                    simplified.insert("type".to_string(), first_type.clone());
                }
            } else {
                simplified.insert("type".to_string(), type_value.clone());
            }
        } else {
            // Default to string if no type is specified
            simplified.insert("type".to_string(), Value::String("string".to_string()));
        }

        // Copy description if it exists
        if let Some(desc) = obj.get("description") {
            simplified.insert("description".to_string(), desc.clone());
        }

        // Handle enum if it exists
        if let Some(enum_values) = obj.get("enum") {
            simplified.insert("enum".to_string(), enum_values.clone());
        }

        Value::Object(simplified)
    } else {
        // Default to a simple string type if items is not an object
        serde_json::json!({"type": "string"})
    }
}

/// Converts a tool result to a Gemini function response.
pub fn tool_result_to_gemini_function_response(
    tool_use_id: &str,
    content: &Value,
    status: &str,
) -> GeminiFunctionResponse {
    let response_value = match status {
        "success" => {
            // For successful results, use a simple "result" field
            serde_json::json!({ "result": content })
        },
        _ => {
            // For errors, use an "error" field
            serde_json::json!({ "error": content })
        },
    };

    GeminiFunctionResponse {
        name: tool_use_id.to_string(),
        response: response_value,
    }
}

/// Adds a function response to the conversation history.
pub fn add_function_response_to_conversation(
    conversation: &mut Vec<GeminiContent>,
    function_call: &GeminiFunctionCall,
    function_response: &GeminiFunctionResponse,
) {
    // Add the model's function call message
    conversation.push(GeminiContent {
        role: Some("model".to_string()),
        parts: vec![GeminiPart::FunctionCall {
            function_call: function_call.clone(),
        }],
    });

    // Add the user's function response message
    conversation.push(GeminiContent {
        role: Some("user".to_string()),
        parts: vec![GeminiPart::FunctionResponse {
            function_response: function_response.clone(),
        }],
    });
}

/// Splits text into chunks of approximately the specified size.
pub fn split_text_into_chunks(text: &str, chunk_size: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_size = 0;

    for char in text.chars() {
        current_chunk.push(char);
        current_size += 1;

        if current_size >= chunk_size {
            chunks.push(current_chunk);
            current_chunk = String::new();
            current_size = 0;
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Generates a unique tool use ID.
pub fn generate_tool_use_id() -> String {
    use std::time::{
        SystemTime,
        UNIX_EPOCH,
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    format!("tool-{}", timestamp)
}

// Mock types for testing purposes
#[derive(Debug, Clone)]
pub enum MockChatMessage {
    UserMessage {
        content: String,
        tool_results: Option<Vec<MockToolResult>>,
    },
    AssistantMessage {
        content: String,
        tool_uses: Option<Vec<MockToolUse>>,
    },
}

// Update the MockToolUse struct to include tool_use_id
#[derive(Debug, Clone)]
pub struct MockToolUse {
    pub name: String,
    pub args: Value,
    pub tool_use_id: String,
}

#[derive(Debug, Clone)]
pub struct MockToolResult {
    pub tool_use_id: String,
    pub content: Value,
    pub status: String, // "success" or "error"
}

#[derive(Debug, Clone)]
pub struct MockTool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_state_to_gemini_request() {
        // Create a simple conversation state
        let user_message = "Hello, how are you?";
        let history = vec![
            MockChatMessage::UserMessage {
                content: "Hi there".to_string(),
                tool_results: None,
            },
            MockChatMessage::AssistantMessage {
                content: "Hello! How can I help you?".to_string(),
                tool_uses: None,
            },
        ];
        let tools = vec![MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "A parameter"
                    }
                }
            }),
        }];

        // Convert to Gemini request
        let request = conversation_state_to_gemini_request(
            &MockChatMessage::UserMessage {
                content: user_message.to_string(),
                tool_results: None,
            },
            &history,
            Some(&tools),
            0.7,
        );

        // Verify the request
        assert_eq!(request.contents.len(), 3);
        assert_eq!(request.contents[0].role, Some("user".to_string()));
        assert_eq!(request.contents[1].role, Some("model".to_string()));
        assert_eq!(request.contents[2].role, Some("user".to_string()));

        // Verify tools
        assert!(request.tools.is_some());
        let tools = request.tools.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function_declarations.len(), 1);
        assert_eq!(tools[0].function_declarations[0].name, "test_tool");

        // Verify generation config
        assert!(request.generation_config.is_some());
        let config = request.generation_config.unwrap();
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_conversation_with_tool_uses() {
        // Create a conversation with tool uses
        let user_message = "Tell me more about my file.";
        let history = vec![
            MockChatMessage::UserMessage {
                content: "Can you read my file?".to_string(),
                tool_results: None,
            },
            MockChatMessage::AssistantMessage {
                content: "I'll help you read that file.".to_string(),
                tool_uses: Some(vec![MockToolUse {
                    name: "fs_read".to_string(),
                    args: serde_json::json!({
                        "path": "test.txt"
                    }),
                    tool_use_id: "tool-123".to_string(),
                }]),
            },
            MockChatMessage::UserMessage {
                content: "".to_string(),
                tool_results: Some(vec![MockToolResult {
                    tool_use_id: "tool-123".to_string(),
                    content: serde_json::json!("This is the file content."),
                    status: "success".to_string(),
                }]),
            },
            MockChatMessage::AssistantMessage {
                content: "Your file says: This is the file content.".to_string(),
                tool_uses: None,
            },
        ];
        let tools = vec![MockTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "A parameter"
                    }
                }
            }),
        }];

        // Convert to Gemini request
        let request = conversation_state_to_gemini_request(
            &MockChatMessage::UserMessage {
                content: user_message.to_string(),
                tool_results: None,
            },
            &history,
            Some(&tools),
            0.7,
        );

        // Verify the request
        // user, model (text), model (function call), user (function response), model (text), user
        assert_eq!(request.contents.len(), 6);

        // Check that the function call and response are correctly formatted
        match &request.contents[2].parts[0] {
            GeminiPart::FunctionCall { function_call } => {
                assert_eq!(function_call.name, "fs_read");
                assert_eq!(
                    function_call.args,
                    serde_json::json!({
                        "path": "test.txt"
                    })
                );
            },
            _ => panic!("Expected function call part"),
        }

        match &request.contents[3].parts[0] {
            GeminiPart::FunctionResponse { function_response } => {
                assert_eq!(function_response.name, "fs_read");
                assert_eq!(
                    function_response.response,
                    serde_json::json!({
                        "result": "This is the file content."
                    })
                );
            },
            _ => panic!("Expected function response part"),
        }
    }

    #[test]
    fn test_tool_id_to_name_mapping() {
        // Create a conversation with tool uses and results with different IDs
        let user_message = "What's next?";
        let history = vec![
            // First tool use
            MockChatMessage::AssistantMessage {
                content: "Let me check that file for you.".to_string(),
                tool_uses: Some(vec![MockToolUse {
                    name: "fs_read".to_string(),
                    args: serde_json::json!({
                        "path": "test.txt"
                    }),
                    tool_use_id: "tool-123".to_string(),
                }]),
            },
            // First tool result
            MockChatMessage::UserMessage {
                content: "".to_string(),
                tool_results: Some(vec![MockToolResult {
                    tool_use_id: "tool-123".to_string(),
                    content: serde_json::json!("File content"),
                    status: "success".to_string(),
                }]),
            },
            // Second tool use
            MockChatMessage::AssistantMessage {
                content: "Now let me check another file.".to_string(),
                tool_uses: Some(vec![MockToolUse {
                    name: "fs_read".to_string(),
                    args: serde_json::json!({
                        "path": "another.txt"
                    }),
                    tool_use_id: "tool-456".to_string(),
                }]),
            },
            // Second tool result
            MockChatMessage::UserMessage {
                content: "".to_string(),
                tool_results: Some(vec![MockToolResult {
                    tool_use_id: "tool-456".to_string(),
                    content: serde_json::json!("Another file content"),
                    status: "success".to_string(),
                }]),
            },
        ];

        // Convert to Gemini request
        let request = conversation_state_to_gemini_request(
            &MockChatMessage::UserMessage {
                content: user_message.to_string(),
                tool_results: None,
            },
            &history,
            None,
            0.7,
        );

        // Verify the request
        // model (text), model (function call), user (function response),
        // model (text), model (function call), user (function response), user (current message)
        assert_eq!(request.contents.len(), 7);

        // Check that the function responses have the correct names (not IDs)
        match &request.contents[2].parts[0] {
            GeminiPart::FunctionResponse { function_response } => {
                assert_eq!(function_response.name, "fs_read");
                assert_eq!(
                    function_response.response,
                    serde_json::json!({
                        "result": "File content"
                    })
                );
            },
            _ => panic!("Expected function response part"),
        }

        match &request.contents[5].parts[0] {
            GeminiPart::FunctionResponse { function_response } => {
                assert_eq!(function_response.name, "fs_read");
                assert_eq!(
                    function_response.response,
                    serde_json::json!({
                        "result": "Another file content"
                    })
                );
            },
            _ => panic!("Expected function response part"),
        }
    }

    #[test]
    fn test_clean_parameters_for_gemini() {
        // Test with a complex schema that needs cleaning
        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "path": {
                    "type": ["string", "null"],
                    "description": "Path to file",
                    "additionalProperties": false,
                    "exclusiveMinimum": true
                }
            }
        });

        let cleaned = clean_parameters_for_gemini(&schema);

        // Verify that unsupported fields are removed
        assert!(!cleaned.as_object().unwrap().contains_key("$schema"));
        assert!(!cleaned.as_object().unwrap().contains_key("additionalProperties"));

        // Verify that the properties are cleaned
        let path_prop = &cleaned["properties"]["path"];
        assert!(!path_prop.as_object().unwrap().contains_key("additionalProperties"));
        assert!(!path_prop.as_object().unwrap().contains_key("exclusiveMinimum"));

        // Verify that array of types is converted to single type
        assert_eq!(path_prop["type"], "string");
    }

    #[test]
    fn test_tool_result_to_gemini_function_response() {
        // Test successful result
        let success_result =
            tool_result_to_gemini_function_response("tool-123", &serde_json::json!("File content"), "success");

        assert_eq!(success_result.name, "tool-123");
        assert_eq!(
            success_result.response,
            serde_json::json!({
                "result": "File content"
            })
        );

        // Test error result
        let error_result =
            tool_result_to_gemini_function_response("tool-456", &serde_json::json!("File not found"), "error");

        assert_eq!(error_result.name, "tool-456");
        assert_eq!(
            error_result.response,
            serde_json::json!({
                "error": "File not found"
            })
        );
    }

    #[test]
    fn test_add_function_response_to_conversation() {
        let mut conversation = Vec::new();

        let function_call = GeminiFunctionCall {
            name: "test_function".to_string(),
            args: serde_json::json!({
                "param1": "value1"
            }),
        };

        let function_response = GeminiFunctionResponse {
            name: "test_function".to_string(),
            response: serde_json::json!({
                "result": "function result"
            }),
        };

        add_function_response_to_conversation(&mut conversation, &function_call, &function_response);

        assert_eq!(conversation.len(), 2);
        assert_eq!(conversation[0].role, Some("model".to_string()));
        assert_eq!(conversation[1].role, Some("user".to_string()));

        match &conversation[0].parts[0] {
            GeminiPart::FunctionCall { function_call: fc } => {
                assert_eq!(fc.name, "test_function");
            },
            _ => panic!("Expected function call part"),
        }

        match &conversation[1].parts[0] {
            GeminiPart::FunctionResponse { function_response: fr } => {
                assert_eq!(fr.name, "test_function");
            },
            _ => panic!("Expected function response part"),
        }
    }

    #[test]
    fn test_split_text_into_chunks() {
        let text = "This is a test of the text splitting function.";
        let chunks = split_text_into_chunks(text, 10);

        assert_eq!(chunks.len(), 5);
        assert_eq!(chunks[0], "This is a ");
        assert_eq!(chunks[1], "test of th");
        assert_eq!(chunks[2], "e text spl");
        assert_eq!(chunks[3], "itting fun");
        assert_eq!(chunks[4], "ction.");
    }
}
