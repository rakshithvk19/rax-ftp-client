//! FTP response parsing functionality

use log::debug;

/// Parsed FTP response from server
#[derive(Debug, Clone, PartialEq)]
pub struct FtpResponse {
    /// Response code (e.g., 230, 530, 331)
    pub code: u16,

    /// Response message (e.g., "User logged in, proceed")
    pub message: String,
}

impl FtpResponse {
    /// Create a new FTP response
    pub fn new(code: u16, message: String) -> Self {
        Self { code, message }
    }
}

impl std::fmt::Display for FtpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.message)
    }
}

/// Parse FTP response string into structured response
pub fn parse_response(response: &str) -> Result<FtpResponse, String> {
    let response = response.trim();

    if response.is_empty() {
        return Err("Empty response".to_string());
    }

    // FTP responses start with 3-digit code followed by space or dash
    if response.len() < 4 {
        return Err("Response too short".to_string());
    }

    // Extract the response code (first 3 characters)
    let code_str = &response[0..3];
    let code = code_str
        .parse::<u16>()
        .map_err(|_| format!("Invalid response code: {}", code_str))?;

    // Check separator (should be space for single line, dash for multi-line start)
    let separator = response.chars().nth(3).unwrap_or(' ');
    if separator != ' ' && separator != '-' {
        return Err(format!(
            "Invalid response format: missing separator after code"
        ));
    }

    // Extract message (everything after "XXX " or "XXX-")
    let message = if response.len() > 4 {
        response[4..].to_string()
    } else {
        String::new()
    };

    debug!("Parsed FTP response: code={}, message='{}'", code, message);

    Ok(FtpResponse::new(code, message))
}
