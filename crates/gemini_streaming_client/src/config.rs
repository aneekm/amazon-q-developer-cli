use std::fs;
use std::path::PathBuf;

use serde::{
    Deserialize,
    Serialize,
};
use tracing::{
    debug,
    error,
    info,
};

use crate::error::GeminiError;

/// Configuration for the Gemini API client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// The API key for authenticating with the Gemini API.
    pub api_key: String,

    /// The Gemini model to use (e.g., "gemini-2.0-flash").
    pub model: String,

    /// The temperature parameter for controlling randomness (0.0 to 1.0).
    pub temperature: f32,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "gemini-2.0-flash".to_string(),
            temperature: 0.7,
        }
    }
}

/// Returns the path to the Gemini configuration file.
pub fn get_config_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    home_dir.join(".aws").join("amazonq").join("gemini_config.json")
}

/// Checks if the Gemini configuration file exists.
pub fn config_exists() -> bool {
    get_config_path().exists()
}

/// Loads the Gemini configuration from the configuration file.
///
/// # Returns
///
/// Returns the loaded configuration if successful, or an error if the configuration
/// file is missing, cannot be read, or contains invalid data.
pub fn load_config() -> Result<GeminiConfig, GeminiError> {
    let config_path = get_config_path();

    if !config_path.exists() {
        error!("Gemini configuration file not found at {:?}", config_path);
        return Err(GeminiError::ConfigurationError(format!(
            "Gemini configuration file not found at {:?}",
            config_path
        )));
    }

    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read Gemini configuration file: {}", e);
            return Err(GeminiError::ConfigurationError(format!(
                "Failed to read Gemini configuration file: {}",
                e
            )));
        },
    };

    match serde_json::from_str::<GeminiConfig>(&config_content) {
        Ok(config) => {
            validate_config(&config)?;
            info!(
                "Gemini configuration found and loaded successfully. Using model: {}",
                config.model
            );
            debug!(
                "Gemini configuration: model={}, temperature={}",
                config.model, config.temperature
            );
            Ok(config)
        },
        Err(e) => {
            error!("Invalid Gemini configuration format: {}", e);
            Err(GeminiError::ConfigurationError(format!(
                "Invalid Gemini configuration format: {}",
                e
            )))
        },
    }
}

/// Validates the Gemini configuration.
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid, or an error if any required
/// fields are missing or have invalid values.
fn validate_config(config: &GeminiConfig) -> Result<(), GeminiError> {
    if config.api_key.is_empty() {
        error!("Gemini API key is missing in configuration");
        return Err(GeminiError::ConfigurationError(
            "Gemini API key is missing in configuration".to_string(),
        ));
    }

    if config.model.is_empty() {
        error!("Gemini model is missing in configuration");
        return Err(GeminiError::ConfigurationError(
            "Gemini model is missing in configuration".to_string(),
        ));
    }

    // Temperature should be between 0.0 and 1.0
    if config.temperature < 0.0 || config.temperature > 1.0 {
        error!(
            "Invalid temperature value in Gemini configuration: {}. Value should be between 0.0 and 1.0",
            config.temperature
        );
        return Err(GeminiError::ConfigurationError(format!(
            "Invalid temperature value: {}. Value should be between 0.0 and 1.0",
            config.temperature
        )));
    }

    Ok(())
}
