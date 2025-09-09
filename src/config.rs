//! Configuration management for RAX FTP Client
//!
//! Loads configuration from config.toml with environment variable overrides.

use config::{Config, Environment, File};
use serde::Deserialize;

/// Complete client configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ClientConfig {
    // ═══ SERVER CONNECTION SETTINGS ═══
    /// FTP server hostname or IP address
    pub host: String,

    /// FTP server port number  
    pub port: u16,

    /// Connection timeout in seconds
    pub timeout: u64,

    /// Maximum number of retry attempts
    pub max_retries: u32,

    // ═══ CLIENT SETTINGS ═══
    /// Local directory for file operations
    pub local_directory: String,

    /// Data port range for active mode connections
    pub data_port_start: u16,
    pub data_port_end: u16,

    // ═══ OPTIONAL SETTINGS ═══
    /// Friendly name for server display (optional)
    pub host_name: Option<String>,
}

impl ClientConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_paths = vec![
            "rax-ftp-client/config", // Docker
            "config",                // Local
        ];

        let mut last_error = None;

        for config_path in &config_paths {
            match Config::builder()
                .add_source(File::with_name(config_path))
                .add_source(Environment::with_prefix("RAX_FTP").separator("_"))
                .build()
            {
                Ok(settings) => {
                    let config: ClientConfig = settings.try_deserialize()?;
                    config.validate(config_path)?;
                    return Ok(config);
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        panic!(
            "Failed to load config.toml from any location. Tried: {config_paths:?}. Last error: {last_error:?}"
        );
    }

    /// Validation for configuration values
    fn validate(&self, config_path: &str) -> Result<(), config::ConfigError> {
        // Determine if we're in Docker based on config path
        let is_docker = config_path == "rax-ftp-client/config";

        // Basic validations
        if self.host.is_empty() {
            return Err(config::ConfigError::Message("Host cannot be empty".into()));
        }

        if self.port == 0 {
            return Err(config::ConfigError::Message("Port cannot be 0".into()));
        }

        if self.timeout == 0 {
            return Err(config::ConfigError::Message("Timeout cannot be 0".into()));
        }

        if self.data_port_start >= self.data_port_end {
            return Err(config::ConfigError::Message(
                "data_port_start must be less than data_port_end".into(),
            ));
        }

        if self.data_port_end - self.data_port_start < 5 {
            return Err(config::ConfigError::Message(
                "Data port range too small (need at least 5 ports)".into(),
            ));
        }

        // Local directory validation - create if doesn't exist
        let local_dir_path = std::path::Path::new(&self.local_directory);

        if !local_dir_path.exists() && !is_docker {
            if let Err(e) = std::fs::create_dir_all(local_dir_path) {
                return Err(config::ConfigError::Message(format!(
                    "Failed to create local directory '{}': {}",
                    self.local_directory, e
                )));
            }
        }

        if local_dir_path.exists() && !local_dir_path.is_dir() && !is_docker {
            return Err(config::ConfigError::Message(format!(
                "'{}' exists but is not a directory",
                self.local_directory
            )));
        }

        Ok(())
    }
}

// Convenience methods (backward compatibility)
impl ClientConfig {
    pub fn get_data_port_range(&self) -> (u16, u16) {
        (self.data_port_start, self.data_port_end)
    }

    pub fn display_name(&self) -> String {
        match &self.host_name {
            Some(name) => name.clone(),
            None => format!("{}:{}", self.host, self.port),
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 2121,
            timeout: 5,
            max_retries: 3,
            local_directory: "./client_root".to_string(),
            data_port_start: 2122,
            data_port_end: 2130,
            host_name: None,
        }
    }
}

impl std::fmt::Display for ClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_name = self.display_name();
        write!(
            f,
            "RAX FTP Config - Server: {}, Timeout: {}s, Data Ports: {}-{}, Max Retries: {}, Local Dir: {}",
            display_name,
            self.timeout,
            self.data_port_start,
            self.data_port_end,
            self.max_retries,
            self.local_directory
        )
    }
}
