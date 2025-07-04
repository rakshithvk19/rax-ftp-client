use crate::error::{RaxFtpClientError, Result};
use std::env;

/// Configuration for the RAX FTP Client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// FTP server hostname or IP address (for connection)
    pub host: String,

    /// Friendly name for this server (for identification)
    pub host_name: Option<String>,

    /// FTP server port number
    pub port: u16,

    /// Connection timeout in seconds
    pub timeout: u64,

    /// Data port range for active mode connections
    pub data_port_range: (u16, u16),

    /// Maximum number of retry attempts for connection
    pub max_retries: u32,

    /// Local directory path for file operations
    pub local_directory: String,
}

impl ClientConfig {
    /// Create configuration from environment variables
    pub fn from_env_and_args() -> Result<Self> {
        let host = env::var("RAX_FTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let host_name = env::var("RAX_FTP_HOST_NAME").ok();

        let port = env::var("RAX_FTP_PORT")
            .unwrap_or_else(|_| "2121".to_string())
            .parse::<u16>()
            .map_err(|_| {
                RaxFtpClientError::InvalidPort(
                    "RAX_FTP_PORT must be a valid port number".to_string(),
                )
            })?;

        let timeout = env::var("RAX_FTP_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .map_err(|_| {
                RaxFtpClientError::InvalidTimeout(
                    "RAX_FTP_TIMEOUT must be a valid number of seconds".to_string(),
                )
            })?;

        let data_port_start = env::var("RAX_FTP_DATA_PORT_START")
            .unwrap_or_else(|_| "2122".to_string())
            .parse::<u16>()
            .map_err(|_| {
                RaxFtpClientError::InvalidPort(
                    "RAX_FTP_DATA_PORT_START must be a valid port number".to_string(),
                )
            })?;

        let data_port_end = env::var("RAX_FTP_DATA_PORT_END")
            .unwrap_or_else(|_| "2130".to_string())
            .parse::<u16>()
            .map_err(|_| {
                RaxFtpClientError::InvalidPort(
                    "RAX_FTP_DATA_PORT_END must be a valid port number".to_string(),
                )
            })?;

        let max_retries = env::var("RAX_FTP_MAX_RETRIES")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u32>()
            .map_err(|_| {
                RaxFtpClientError::InvalidTimeout(
                    "RAX_FTP_MAX_RETRIES must be a valid number".to_string(),
                )
            })?;

        let local_directory = env::var("RAX_FTP_LOCAL_DIR").unwrap_or_else(|_| "./".to_string());

        // Validate port range
        if data_port_start >= data_port_end {
            return Err(RaxFtpClientError::InvalidPort(
                "RAX_FTP_DATA_PORT_START must be less than RAX_FTP_DATA_PORT_END".to_string(),
            ));
        }

        Ok(ClientConfig {
            host,
            host_name,
            port,
            timeout,
            data_port_range: (data_port_start, data_port_end),
            max_retries,
            local_directory,
        })
    }

    /// Get the data port range for validation elsewhere
    pub fn get_data_port_range(&self) -> (u16, u16) {
        self.data_port_range
    }

    /// Get display name for the server (friendly name or host:port)
    pub fn display_name(&self) -> String {
        match &self.host_name {
            Some(name) => name.clone(),
            None => format!("{}:{}", self.host, self.port),
        }
    }

    /// Validate the basic configuration
    pub fn validate(&self) -> Result<()> {
        // Check if host is not empty
        if self.host.is_empty() {
            return Err(RaxFtpClientError::InvalidHost(
                "Host cannot be empty".to_string(),
            ));
        }

        // Check port range
        if self.port == 0 {
            return Err(RaxFtpClientError::InvalidPort(
                "Port cannot be 0".to_string(),
            ));
        }

        // Check timeout
        if self.timeout == 0 {
            return Err(RaxFtpClientError::InvalidTimeout(
                "Timeout cannot be 0".to_string(),
            ));
        }

        // Check local directory exists
        if !std::path::Path::new(&self.local_directory).exists() {
            return Err(RaxFtpClientError::InvalidHost(format!(
                "Local directory '{}' does not exist",
                self.local_directory
            )));
        }

        Ok(())
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            host_name: None,
            port: 2121,
            timeout: 5,
            data_port_range: (2122, 2130),
            max_retries: 3,
            local_directory: "./".to_string(),
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
            self.data_port_range.0,
            self.data_port_range.1,
            self.max_retries,
            self.local_directory
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_with_host_name() {
        let mut config = ClientConfig::default();
        config.host_name = Some("Comp Lab 2".to_string());
        assert_eq!(config.display_name(), "Comp Lab 2");
    }

    #[test]
    fn test_display_name_without_host_name() {
        let config = ClientConfig::default();
        assert_eq!(config.display_name(), "127.0.0.1:2121");
    }
}
