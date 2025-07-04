use log::{debug, info};

use crate::config::ClientConfig;
use crate::connection::CommandConnection;
use crate::error::Result;
use crate::responses::{FtpResponse, is_authentication_success, parse_response};

/// Client connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    Disconnected,
    Connected,
    Authenticated,
}

impl std::fmt::Display for ClientState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientState::Disconnected => write!(f, "disconnected"),
            ClientState::Connected => write!(f, "connected"),
            ClientState::Authenticated => write!(f, "authenticated"),
        }
    }
}

/// Main FTP Client
pub struct RaxFtpClient {
    connection: CommandConnection,
    state: ClientState,
}

impl RaxFtpClient {
    /// Create a new FTP client with the given configuration
    pub fn new(config: ClientConfig) -> Self {
        info!("Creating RAX FTP Client with config: {}", config);

        Self {
            connection: CommandConnection::new(&config),
            state: ClientState::Disconnected,
        }
    }

    /// Connect to the FTP server with retry logic
    pub fn connect_with_retries(&mut self) -> Result<()> {
        self.connection.connect_with_retries()?;
        self.state = ClientState::Connected;
        Ok(())
    }

    /// Get current client state for display
    pub fn get_state(&self) -> &ClientState {
        &self.state
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        self.connection.is_connected() && self.state != ClientState::Disconnected
    }

    /// Check if client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.state == ClientState::Authenticated
    }

    /// Disconnect from the server
    pub fn disconnect(&mut self) -> Result<()> {
        self.connection.disconnect()?;
        self.state = ClientState::Disconnected;
        Ok(())
    }

    /// Send a raw FTP command to the server
    pub fn send_command(&mut self, command: &str) -> Result<()> {
        self.connection.send_command(command)
    }

    /// Read a response from the server and update state if needed
    pub fn read_response(&mut self) -> Result<String> {
        let response_str = self.connection.read_response()?;

        // Parse the response to potentially update client state
        match parse_response(&response_str) {
            Ok(parsed_response) => {
                self.update_state_from_response(&parsed_response);
                Ok(response_str)
            }
            Err(parse_error) => {
                debug!(
                    "Failed to parse response '{}': {}",
                    response_str, parse_error
                );
                // Return the original response even if parsing failed
                Ok(response_str)
            }
        }
    }

    /// Update client state based on server response
    fn update_state_from_response(&mut self, response: &FtpResponse) {
        match response.code {
            // User successfully logged in
            230 if is_authentication_success(response.code) => {
                debug!("Authentication successful, updating state to Authenticated");
                self.state = ClientState::Authenticated;
            }

            // Authentication failed - user remains connected but not authenticated
            530 => {
                debug!("Authentication failed, keeping state as Connected");
                // State remains Connected (don't change to Disconnected)
            }

            // Logout successful - user remains connected but not authenticated
            221 => {
                debug!("Logout successful, updating state to Connected");
                self.state = ClientState::Connected;
            }

            // For all other responses, don't change state
            _ => {
                debug!("Response code {} - no state change needed", response.code);
            }
        }
    }
}
