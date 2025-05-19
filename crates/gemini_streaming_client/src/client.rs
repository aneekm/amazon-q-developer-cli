use std::time::Duration;

use reqwest::header::{
    CONTENT_TYPE,
    HeaderMap,
    HeaderValue,
};
use tracing::{
    debug,
    error,
    warn,
};

use crate::config::GeminiConfig;
use crate::error::GeminiError;
use crate::types::{
    GeminiContent,
    GeminiGenerationConfig,
    GeminiPart,
    GeminiRequest,
    GeminiResponse,
    GeminiStreamingResponse,
};

/// The base URL for the Gemini API.
const GEMINI_API_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Client for interacting with the Gemini API.
#[derive(Debug, Clone)]
pub struct Client {
    /// The API key for authenticating with the Gemini API.
    api_key: String,

    /// The model to use for generating content.
    model: String,

    /// The temperature parameter for controlling randomness (0.0 to 1.0).
    temperature: f32,

    /// The HTTP client for making requests.
    client: reqwest::Client,
}

impl Client {
    /// Creates a new Gemini streaming client with the given configuration.
    pub fn new(config: GeminiConfig) -> Self {
        // Create a new HTTP client with a timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            api_key: config.api_key,
            model: config.model,
            temperature: config.temperature,
            client,
        }
    }

    /// Gets the API URL for the specified endpoint.
    fn get_api_url(&self, endpoint: &str) -> String {
        format!(
            "{}/models/{}:{}?key={}",
            GEMINI_API_BASE_URL, self.model, endpoint, self.api_key
        )
    }

    /// Gets the temperature parameter.
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Generates content using the Gemini API.
    pub async fn generate_content(&self, request: GeminiRequest) -> Result<GeminiResponse, GeminiError> {
        // Create the request headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Log the request (but not the API key for security reasons)
        debug!("Sending request to Gemini API: {:#?}", request);

        // Send the request to the Gemini API
        let response = self
            .client
            .post(self.get_api_url("generateContent"))
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| GeminiError::HttpError(format!("Failed to send request: {}", e)))?;

        // Check if the request was successful
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Gemini API request failed with status {}: {}", status, error_text);
            return Err(GeminiError::ApiError(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        // let response_txt = response.text().await.unwrap_or("couldn't get response".to_string());
        // println!("Response: {}", response_txt);
        // Parse the response
        let response_json = response
            .json::<GeminiResponse>()
            .await
            .map_err(|e| GeminiError::SerializationError(format!("Failed to parse response: {}", e)))?;

        // let response_json: GeminiResponse = serde_json::from_str(&response_txt)
        //     .map_err(|e| GeminiError::SerializationError(format!("Failed to parse response: {}", e)))?;
        // Log the response
        debug!("Received response from Gemini API: {:#?}", response_json);

        Ok(response_json)
    }

    /// Generates content using the Gemini API with streaming.
    pub async fn generate_content_streaming(
        &self,
        request: GeminiRequest,
    ) -> Result<GeminiStreamingResponse, GeminiError> {
        // Create the request headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Log the request (but not the API key for security reasons)
        debug!("Sending streaming request to Gemini API: {:?}", request);

        // Send the request to the Gemini API
        let response = self
            .client
            .post(self.get_api_url("streamGenerateContent"))
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| GeminiError::HttpError(format!("Failed to send streaming request: {}", e)))?;

        // Check if the request was successful
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                "Gemini API streaming request failed with status {}: {}",
                status, error_text
            );
            return Err(GeminiError::ApiError(format!(
                "API streaming request failed with status {}: {}",
                status, error_text
            )));
        }

        // For now, we'll simplify this implementation to focus on getting the test connection working
        // We'll implement proper streaming in a future step
        let stream = response
            .bytes()
            .await
            .map_err(|e| GeminiError::HttpError(format!("Failed to get response bytes: {}", e)))?;

        let text = String::from_utf8_lossy(&stream);
        match serde_json::from_str::<GeminiStreamingResponse>(&text) {
            Ok(response) => Ok(response),
            Err(e) => {
                warn!("Failed to parse streaming response: {}", e);
                warn!("Response text: {}", text);
                Err(GeminiError::SerializationError(format!(
                    "Failed to parse streaming response: {}",
                    e
                )))
            },
        }
    }

    pub async fn test_gemini() -> bool {
        // Try to load the configuration
        match crate::config::load_config() {
            Ok(config) => {
                println!("Gemini configuration loaded successfully!");
                println!("Model: {}", config.model);
                println!("Temperature: {}", config.temperature);
                // Don't print the API key for security reasons
                println!("API key: [REDACTED]");

                // Create a Gemini client and test the connection
                let gemini_client = Self::new(config);

                // Test basic connection
                match gemini_client.test_connection().await {
                    Ok(response) => {
                        println!("Gemini API connection test successful!");
                        println!("Gemini full response: {:?}", response);
                        if let Some(candidate) = response.candidates.first() {
                            if let Some(part) = candidate.content.parts.first() {
                                match part {
                                    GeminiPart::Text { text } => {
                                        println!("Gemini response: {}", text);
                                    },
                                    _ => {
                                        println!("Gemini response received (non-text format)");
                                    },
                                }
                            }
                        }

                        // Test conversion flow
                        println!("Testing full conversion flow with Gemini API...");
                        match gemini_client.test_conversion_flow().await {
                            Ok(streams) => {
                                println!("Full conversion flow test successful!");
                                println!("Generated {} response streams", streams.len());

                                // Print the first few streams
                                for (i, stream) in streams.iter().take(3).enumerate() {
                                    println!("Stream {}: {:?}", i, stream);
                                }
                            },
                            Err(e) => {
                                println!("Full conversion flow test failed: {}", e);
                                return false;
                            },
                        }
                    },
                    Err(e) => {
                        println!("Gemini API connection test failed: {}", e);
                        return false;
                    },
                }

                // successful test
                true
            },
            Err(e) => {
                println!("Error loading Gemini configuration: {}", e);
                // failed test
                false
            }
        }
    }

    /// Tests the connection to the Gemini API.
    pub async fn test_connection(&self) -> Result<GeminiResponse, GeminiError> {
        // Create a simple request to test the connection
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                role: Some("user".to_string()),
                parts: vec![GeminiPart::Text {
                    text: "Hello, can you respond with a simple 'Hello world!' message?".to_string(),
                }],
            }],
            tools: None,
            generation_config: Some(GeminiGenerationConfig {
                temperature: Some(self.temperature),
                max_output_tokens: Some(50),
                top_k: None,
                top_p: None,
            }),
        };

        // Send the request to the Gemini API
        self.generate_content(request).await
    }

    /// Tests the full conversion flow with the Gemini API.
    pub async fn test_conversion_flow(&self) -> Result<Vec<String>, GeminiError> {
        // Create a simple test request
        let user_message = "Can you summarize the contents of my ~/.zshrc file?";
        let history = vec![
            crate::conversion::MockChatMessage::UserMessage {
                content: "Hello".to_string(),
                tool_results: None,
            },
            crate::conversion::MockChatMessage::AssistantMessage {
                content: "Hi there! How can I help you today?".to_string(),
                tool_uses: None,
            },
        ];

        // Create a simple tool
        let tools = vec![crate::conversion::MockTool {
            name: "fs_read".to_string(),
            description: "Read a file from the filesystem".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file, could be relative to current directory or to home directory via ~"
                    }
                },
                "required": ["path"]
            }),
        }];

        // Convert to Gemini request
        let request = crate::conversion::conversation_state_to_gemini_request(
            &crate::conversion::MockChatMessage::UserMessage {
                content: user_message.to_string(),
                tool_results: None,
            },
            &history,
            Some(&tools),
            self.temperature,
        );

        // Send the request to the Gemini API
        let response = self.generate_content(request).await?;
        println!("Gemini response: {:?}", response);

        // Extract text from the response
        let mut texts = Vec::new();
        for candidate in &response.candidates {
            for part in &candidate.content.parts {
                if let crate::types::GeminiPart::Text { text } = part {
                    texts.push(text.clone());
                }
            }
        }

        Ok(texts)
    }
}
