use crate::error::{RaxFtpClientError, Result};
use serde::Deserialize;
use std::env;
use std::fs;

/// Configuration for the RAX FTP Client
#[derive(Debug, Clone, Deserialize)]
pub struct ClientConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// Client configuration
    pub client: ClientSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// FTP server hostname or IP address (for connection)
    pub host: String,

    /// Friendly name for this server (for identification)
    pub host_name: Option<String>,

    /// FTP server port number
    pub port: u16,

    /// Connection timeout in seconds
    pub timeout: u64,

    /// Maximum number of retry attempts for connection
    pub max_retries: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientSettings {
    /// Local directory path for file operations
    pub local_directory: String,

    /// Data port range for active mode connections
    pub data_port_start: u16,
    pub data_port_end: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Logging level
    pub level: String,

    /// Enable command logging
    pub command_log: bool,

    /// Enable transfer logging
    pub transfer_log: bool,
}

impl ClientConfig {
    /// Create configuration from TOML file with environment variable overrides
    pub fn from_config_file(config_path: &str) -> Result<Self> {
        // Read and parse TOML file
        let config_content = fs::read_to_string(config_path).map_err(|e| {
            RaxFtpClientError::ConfigFileNotFound(format!(
                "Cannot read config file '{}': {}",
                config_path, e
            ))
        })?;

        let mut config: ClientConfig = toml::from_str(&config_content).map_err(|e| {
            RaxFtpClientError::ConfigFileParseError(format!(
                "Invalid TOML in '{}': {}",
                config_path, e
            ))
        })?;

        // Apply environment variable overrides
        config.apply_env_overrides()?;

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Apply environment variable overrides to config
    fn apply_env_overrides(&mut self) -> Result<()> {
        // Server overrides
        if let Ok(host) = env::var("RAX_FTP_HOST") {
            self.server.host = host;
        }

        if let Ok(host_name) = env::var("RAX_FTP_HOST_NAME") {
            self.server.host_name = Some(host_name);
        }

        if let Ok(port_str) = env::var("RAX_FTP_PORT") {
            self.server.port = port_str.parse().map_err(|_| {
                RaxFtpClientError::InvalidConfigValue(
                    "RAX_FTP_PORT must be a valid port number".to_string(),
                )
            })?;
        }

        if let Ok(timeout_str) = env::var("RAX_FTP_TIMEOUT") {
            self.server.timeout = timeout_str.parse().map_err(|_| {
                RaxFtpClientError::InvalidConfigValue(
                    "RAX_FTP_TIMEOUT must be a valid number of seconds".to_string(),
                )
            })?;
        }

        if let Ok(retries_str) = env::var("RAX_FTP_MAX_RETRIES") {
            self.server.max_retries = retries_str.parse().map_err(|_| {
                RaxFtpClientError::InvalidConfigValue(
                    "RAX_FTP_MAX_RETRIES must be a valid number".to_string(),
                )
            })?;
        }

        // Client overrides
        if let Ok(local_dir) = env::var("RAX_FTP_LOCAL_DIR") {
            self.client.local_directory = local_dir;
        }

        if let Ok(data_port_start_str) = env::var("RAX_FTP_DATA_PORT_START") {
            self.client.data_port_start = data_port_start_str.parse().map_err(|_| {
                RaxFtpClientError::InvalidConfigValue(
                    "RAX_FTP_DATA_PORT_START must be a valid port number".to_string(),
                )
            })?;
        }

        if let Ok(data_port_end_str) = env::var("RAX_FTP_DATA_PORT_END") {
            self.client.data_port_end = data_port_end_str.parse().map_err(|_| {
                RaxFtpClientError::InvalidConfigValue(
                    "RAX_FTP_DATA_PORT_END must be a valid port number".to_string(),
                )
            })?;
        }

        Ok(())
    }

    // Convenience methods for backward compatibility
    pub fn host(&self) -> &str {
        &self.server.host
    }

    pub fn port(&self) -> u16 {
        self.server.port
    }

    pub fn timeout(&self) -> u64 {
        self.server.timeout
    }

    pub fn max_retries(&self) -> u32 {
        self.server.max_retries
    }

    pub fn local_directory(&self) -> &str {
        &self.client.local_directory
    }

    /// Get the data port range for validation elsewhere
    pub fn get_data_port_range(&self) -> (u16, u16) {
        (self.client.data_port_start, self.client.data_port_end)
    }

    /// Get display name for the server (friendly name or host:port)
    pub fn display_name(&self) -> String {
        match &self.server.host_name {
            Some(name) => name.clone(),
            None => format!("{}:{}", self.server.host, self.server.port),
        }
    }

    /// Validate the basic configuration
    pub fn validate(&self) -> Result<()> {
        // Check if host is not empty
        if self.server.host.is_empty() {
            return Err(RaxFtpClientError::InvalidConfigValue(
                "Host cannot be empty".to_string(),
            ));
        }

        // Check port range
        if self.server.port == 0 {
            return Err(RaxFtpClientError::InvalidConfigValue(
                "Port cannot be 0".to_string(),
            ));
        }

        // Check timeout
        if self.server.timeout == 0 {
            return Err(RaxFtpClientError::InvalidConfigValue(
                "Timeout cannot be 0".to_string(),
            ));
        }

        // Check data port range
        if self.client.data_port_start >= self.client.data_port_end {
            return Err(RaxFtpClientError::InvalidConfigValue(
                "Data port start must be less than data port end".to_string(),
            ));
        }

        // Check local directory exists
        if !std::path::Path::new(&self.client.local_directory).exists() {
            return Err(RaxFtpClientError::InvalidConfigValue(format!(
                "Local directory '{}' does not exist",
                self.client.local_directory
            )));
        }

        Ok(())
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                host_name: None,
                port: 2121,
                timeout: 5,
                max_retries: 3,
            },
            client: ClientSettings {
                local_directory: "./client_root".to_string(),
                data_port_start: 2122,
                data_port_end: 2130,
            },
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
            self.server.timeout,
            self.client.data_port_start,
            self.client.data_port_end,
            self.server.max_retries,
            self.client.local_directory
        )
    }
}

