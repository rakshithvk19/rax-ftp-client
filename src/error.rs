use std::fmt;

/// Main error type for the RAX FTP Client
#[derive(Debug)]
pub enum RaxFtpClientError {
    // Connection Errors
    ConnectionRefused(String),
    ConnectionTimeout(String),
    ConnectionLost(String),
    NotConnected(String),
    InvalidHost(String),

    // Authentication Errors
    NotAuthenticated(String),

    // Transfer Errors
    FileNotFound { code: u16, message: String },
    TransferFailed { code: u16, message: String },
    DataConnectionFailed(String),
    PermissionDenied { code: u16, message: String },

    // Configuration Errors
    InvalidPort(String),
    ConfigFileNotFound(String),
    ConfigFileParseError(String),
    InvalidConfigValue(String),

    // IO Errors
    Io(std::io::Error),
}

impl fmt::Display for RaxFtpClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Connection Errors
            Self::ConnectionRefused(msg) => write!(f, "Connection refused: {msg}"),
            Self::ConnectionTimeout(msg) => write!(f, "Connection timeout: {msg}"),
            Self::ConnectionLost(msg) => write!(f, "Connection lost: {msg}"),
            Self::NotConnected(msg) => write!(f, "Not connected: {msg}"),
            Self::InvalidHost(msg) => write!(f, "Invalid host: {msg}"),

            // Authentication Errors
            Self::NotAuthenticated(msg) => write!(f, "{msg}"),

            // Transfer Errors
            Self::FileNotFound { code, message } => {
                write!(f, "File not found ({code}): {message}")
            }
            Self::TransferFailed { code, message } => {
                write!(f, "Transfer failed ({code}): {message}")
            }
            Self::DataConnectionFailed(msg) => write!(f, "Data connection failed: {msg}"),
            Self::PermissionDenied { code, message } => {
                write!(f, "Permission denied ({code}): {message}")
            }

            // Configuration Errors
            Self::InvalidPort(msg) => write!(f, "Invalid port: {msg}"),
            Self::ConfigFileNotFound(msg) => write!(f, "Config file not found: {msg}"),
            Self::ConfigFileParseError(msg) => write!(f, "Config file parse error: {msg}"),
            Self::InvalidConfigValue(msg) => write!(f, "Invalid config value: {msg}"),

            // IO Errors
            Self::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl std::error::Error for RaxFtpClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RaxFtpClientError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<std::num::ParseIntError> for RaxFtpClientError {
    fn from(_: std::num::ParseIntError) -> Self {
        Self::InvalidPort("Failed to parse port number".to_string())
    }
}


/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, RaxFtpClientError>;
