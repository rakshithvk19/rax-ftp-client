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
    InvalidCredentials { code: u16, message: String },
    AuthenticationFailed { code: u16, message: String },
    UserNotFound { code: u16, message: String },
    PermissionDenied { code: u16, message: String },

    // Transfer Errors
    FileNotFound { code: u16, message: String },
    FileAlreadyExists { code: u16, message: String },
    InsufficientStorage { code: u16, message: String },
    TransferFailed { code: u16, message: String },
    DataConnectionFailed(String),

    // Protocol Errors
    InvalidResponse(String),
    UnexpectedResponse { expected: String, received: String },
    ProtocolViolation { code: u16, message: String },
    CommandNotSupported { code: u16, message: String },
    ResponseParseError(String),

    // Configuration Errors
    InvalidPort(String),
    InvalidTimeout(String),
    MissingConfiguration(String),

    // IO Errors
    Io(std::io::Error),

    // General errors
    Other(String),
}

impl fmt::Display for RaxFtpClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Connection Errors
            Self::ConnectionRefused(msg) => write!(f, "Connection refused: {}", msg),
            Self::ConnectionTimeout(msg) => write!(f, "Connection timeout: {}", msg),
            Self::ConnectionLost(msg) => write!(f, "Connection lost: {}", msg),
            Self::NotConnected(msg) => write!(f, "Not connected: {}", msg),
            Self::InvalidHost(msg) => write!(f, "Invalid host: {}", msg),

            // Authentication Errors
            Self::InvalidCredentials { code, message } => {
                write!(f, "Invalid credentials ({}): {}", code, message)
            }
            Self::AuthenticationFailed { code, message } => {
                write!(f, "Authentication failed ({}): {}", code, message)
            }
            Self::UserNotFound { code, message } => {
                write!(f, "User not found ({}): {}", code, message)
            }
            Self::PermissionDenied { code, message } => {
                write!(f, "Permission denied ({}): {}", code, message)
            }

            // Transfer Errors
            Self::FileNotFound { code, message } => {
                write!(f, "File not found ({}): {}", code, message)
            }
            Self::FileAlreadyExists { code, message } => {
                write!(f, "File already exists ({}): {}", code, message)
            }
            Self::InsufficientStorage { code, message } => {
                write!(f, "Insufficient storage ({}): {}", code, message)
            }
            Self::TransferFailed { code, message } => {
                write!(f, "Transfer failed ({}): {}", code, message)
            }
            Self::DataConnectionFailed(msg) => write!(f, "Data connection failed: {}", msg),

            // Protocol Errors
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            Self::UnexpectedResponse { expected, received } => write!(
                f,
                "Unexpected response: expected '{}', got '{}'",
                expected, received
            ),
            Self::ProtocolViolation { code, message } => {
                write!(f, "Protocol violation ({}): {}", code, message)
            }
            Self::CommandNotSupported { code, message } => {
                write!(f, "Command not supported ({}): {}", code, message)
            }
            Self::ResponseParseError(msg) => write!(f, "Response parse error: {}", msg),

            // Configuration Errors
            Self::InvalidPort(msg) => write!(f, "Invalid port: {}", msg),
            Self::InvalidTimeout(msg) => write!(f, "Invalid timeout: {}", msg),
            Self::MissingConfiguration(msg) => write!(f, "Missing configuration: {}", msg),

            // IO Errors
            Self::Io(err) => write!(f, "IO error: {}", err),

            // General
            Self::Other(msg) => write!(f, "Error: {}", msg),
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

/// Helper function to create FTP response errors
impl RaxFtpClientError {
    pub fn from_ftp_response(code: u16, message: String) -> Self {
        match code {
            // 4xx and 5xx codes for authentication
            530 => Self::InvalidCredentials { code, message },
            331 => Self::AuthenticationFailed { code, message },

            // File/transfer related errors
            550 => Self::FileNotFound { code, message },
            553 => Self::FileAlreadyExists { code, message },
            552 => Self::InsufficientStorage { code, message },
            426 | 451 | 551 => Self::TransferFailed { code, message },

            // Permission errors
            532 => Self::PermissionDenied { code, message },

            // Command not supported
            502 | 504 => Self::CommandNotSupported { code, message },

            // General protocol violations
            _ if code >= 400 => Self::ProtocolViolation { code, message },

            // Unexpected positive responses where we expected negative
            _ => Self::UnexpectedResponse {
                expected: "error response".to_string(),
                received: format!("{} {}", code, message),
            },
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, RaxFtpClientError>;
