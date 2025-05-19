//! A client for interacting with the Google Gemini API.
//!
//! This crate provides a client for the Gemini API that can be used with the Amazon Q CLI.

pub mod client;
pub mod config;
pub mod conversion;
pub mod error;
pub mod types;

// Re-export key types for convenience
pub use client::Client;
pub use config::GeminiConfig;
pub use error::GeminiError;
pub use types::{
    GeminiCandidate,
    GeminiContent,
    GeminiFunctionCall,
    GeminiFunctionDeclaration,
    GeminiFunctionResponse,
    GeminiGenerationConfig,
    GeminiPart,
    GeminiRequest,
    GeminiResponse,
    GeminiStreamingResponse,
    GeminiTool,
};
