use log::{debug, info};
use std::path::Path;

use crate::commands::FtpCommand;
use crate::config::ClientConfig;
use crate::connection::{CommandConnection, DataConnection};
use crate::error::Result;
use crate::responses::{FtpResponse, is_authentication_success, parse_response};
use crate::transfer::{upload_file_with_progress, validate_upload_file};

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
    config: ClientConfig,
}

impl RaxFtpClient {
    /// Create a new FTP client with the given configuration
    pub fn new(config: ClientConfig) -> Self {
        info!("Creating RAX FTP Client with config: {}", config);

        Self {
            connection: CommandConnection::new(&config),
            state: ClientState::Disconnected,
            config,
        }
    }

    /// Connect to the FTP server with retry logic
    pub fn connect_with_retries(&mut self) -> Result<String> {
        let greeting = self.connection.connect_with_retries()?;
        self.state = ClientState::Connected;
        Ok(greeting)
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

    /// Send a command and handle file transfers if needed
    pub fn execute_command(&mut self, command: &FtpCommand) -> Result<String> {
        match command {
            FtpCommand::Stor(filename) => self.handle_stor_command(filename),
            // For all other commands, send normally
            _ => {
                let command_str = command.to_ftp_string();
                self.send_command(&command_str)?;
                self.read_response()
            }
        }
    }

    /// Handle STOR command with file transfer
    fn handle_stor_command(&mut self, filename: &str) -> Result<String> {
        // Build local file path
        let local_path = Path::new(&self.config.local_directory).join(filename);

        // Validate file before starting transfer
        validate_upload_file(&local_path)?;

        // Create data connection for PORT mode
        let mut data_connection = DataConnection::new_port_mode(&self.config)?;

        // Send PORT command
        let port_command = data_connection.get_port_command()?;
        self.send_command(&port_command)?;
        let port_response = self.read_response()?;

        // Check if PORT command was successful
        if !port_response.starts_with("200") {
            return Ok(format!("PORT command failed: {}", port_response));
        }

        // Send STOR command
        let stor_command = format!("STOR {}", filename);
        self.send_command(&stor_command)?;
        let stor_response = self.read_response()?;

        // Check if STOR command was accepted
        if !stor_response.starts_with("150") {
            return Ok(format!("STOR command failed: {}", stor_response));
        }

        // Print initial response
        print!("{}", stor_response);

        // Accept data connection from server
        data_connection.accept_connection()?;

        // Upload the file with progress
        match upload_file_with_progress(data_connection, &local_path, filename) {
            Ok(()) => {
                // Read final response from server
                let final_response = self.read_response()?;
                Ok(final_response)
            }
            Err(e) => {
                // Read final response even if upload failed
                let _ = self.read_response();
                Err(e)
            }
        }
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
