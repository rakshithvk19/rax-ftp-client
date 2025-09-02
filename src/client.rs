use log::{debug, info};
use std::path::Path;

use crate::commands::{FtpCommand, get_help_text};
use crate::config::ClientConfig;
use crate::connection::{CommandConnection, DataConnection};
use crate::error::{RaxFtpClientError, Result};
use crate::responses::{FtpResponse, is_authentication_success, parse_response};
use crate::terminal::listing::format_directory_listing;
use crate::transfer::{
    download_file_with_progress, read_directory_listing, upload_file_with_progress,
    validate_download_path, validate_upload_file,
};

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
    data_connection: Option<DataConnection>,
}

impl RaxFtpClient {
    /// Create a new FTP client with the given configuration
    pub fn new(config: ClientConfig) -> Self {
        info!("Creating RAX FTP Client with config: {}", config);

        Self {
            connection: CommandConnection::new(&config),
            state: ClientState::Disconnected,
            config,
            data_connection: None,
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
        self.data_connection = None; // Clear data connection
        Ok(())
    }

    /// Send a command and handle file transfers if needed
    pub fn execute_command(&mut self, command: &FtpCommand) -> Result<String> {
        // Handle client-side only commands
        if command.is_client_only() {
            // For HELP command, get formatted help text
            if let FtpCommand::Help = command {
                let mut help_text = get_help_text();
                help_text = help_text.replace("[SERVER_PLACEHOLDER]", &self.config.host());
                help_text = help_text.replace("[STATE_PLACEHOLDER]", &self.state.to_string());
                help_text =
                    help_text.replace("[LOCAL_DIR_PLACEHOLDER]", self.config.local_directory());
                let (start, end) = self.config.get_data_port_range();
                help_text =
                    help_text.replace("[PORT_RANGE_PLACEHOLDER]", &format!("{}-{}", start, end));

                return Ok(help_text);
            }

            return Ok("Command executed locally".to_string());
        }

        // Check authentication for commands that require it
        // Commands like USER, PASS, QUIT, and UNKNOWN don't require authentication
        let requires_auth = !matches!(
            command,
            FtpCommand::User(_) | FtpCommand::Pass(_) | FtpCommand::Quit | FtpCommand::Unknown(_)
        );

        if requires_auth && !self.is_authenticated() {
            return Err(RaxFtpClientError::NotAuthenticated(format!(
                "{} Not authenticated. Please log in first.",
                crate::responses::status_codes::CLIENT_ERROR_NOT_AUTHENTICATED
            )));
        }

        // Check if trying to authenticate when already authenticated
        if matches!(command, FtpCommand::User(_)) && self.is_authenticated() {
            return Err(RaxFtpClientError::AlreadyAuthenticated(format!(
                "{} Already authenticated. Use LOGOUT first to change user.",
                crate::responses::status_codes::CLIENT_ERROR_ALREADY_AUTHENTICATED
            )));
        }

        // Execute the command
        match command {
            FtpCommand::Stor(filename) => self.handle_stor_command(filename),
            FtpCommand::Retr(filename) => self.handle_retr_command(filename),
            FtpCommand::List => self.handle_list_command(),
            FtpCommand::Port(addr) => self.handle_port_command(addr),
            FtpCommand::Pasv => self.handle_pasv_command(),
            // For all other commands, send normally
            _ => {
                let command_str = command.to_ftp_string();
                self.send_command(&command_str)?;
                self.read_response()
            }
        }
    }

    /// Handle PORT command - switch to active mode
    fn handle_port_command(&mut self, addr: &str) -> Result<String> {
        // Parse the address to validate format
        let parsed_addr: std::net::SocketAddr = addr.parse().map_err(|_| {
            RaxFtpClientError::InvalidConfigValue("Invalid address format. Use IP:PORT".to_string())
        })?;

        // Create the data connection with listener on the SPECIFIC port from PORT command
        let data_connection = DataConnection::active_mode(parsed_addr.port())?;

        // Send PORT command to server
        let command_str = format!("PORT {}", addr);
        self.send_command(&command_str)?;
        let response = self.read_response()?;

        // If successful, store both connection info and the actual connection
        if response.starts_with("200") {
            info!(
                "Switching to active mode, storing data connection with listener on port {}",
                parsed_addr.port()
            );

            self.data_connection = Some(data_connection);
        } else {
            // PORT command failed, don't store the connection
            info!("PORT command failed, not storing data connection");
        }

        Ok(response)
    }
    fn handle_pasv_command(&mut self) -> Result<String> {
        self.send_command("PASV")?;
        let response = self.read_response()?;

        if response.starts_with("227") {
            // Parse server's response to get host:port
            if let Some(start) = response.find('(') {
                if let Some(end) = response.find(')') {
                    let addr_str = &response[start + 1..end];
                    if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() {
                        // Create DataConnection immediately (like PORT does)
                        let data_connection =
                            DataConnection::passive_mode(&addr.ip().to_string(), addr.port())?;

                        // Store the actual connection
                        self.data_connection = Some(data_connection);
                    }
                }
            }
        }

        Ok(response)
    }

    fn handle_list_command(&mut self) -> Result<String> {
        let mut responses = Vec::new();

        // Auto-PASV if no data connection mode is set
        if self.data_connection.is_none() {
            let pasv_response = self.handle_pasv_command()?;
            responses.push(pasv_response);
        }

        info!("Data connection present");

        // 1. Send LIST command FIRST
        self.send_command("LIST")?;

        // 2. Read "150 Opening data connection" response
        let list_response = self.read_response()?;

        // Check if LIST command was accepted (should be "150", not "226")
        if !list_response.starts_with("150") {
            responses.push(format!("LIST command failed: {}", list_response));
            return Ok(responses.join("\n"));
        }

        // Add the "150" response
        responses.push(list_response);

        let mut data_connection = self.data_connection.take().unwrap();

        // 3. NOW establish data connection
        data_connection.connect_to_server()?;

        // 4. Read directory listing from data channel
        match read_directory_listing(&mut data_connection) {
            Ok(listing) => {
                // Format directory listing and add to responses
                let listing_display = format_directory_listing(&listing);
                responses.push(listing_display);

                // Reset connection for next use
                data_connection.reset_connection()?;

                // Put the data connection back
                self.data_connection = Some(data_connection);

                // 5. Read final "226 Directory send OK" response
                let final_response = self.read_response()?;
                responses.push(final_response);

                Ok(responses.join("\n"))
            }
            Err(e) => {
                // Reset connection even on error
                data_connection.reset_connection()?;

                // Put the data connection back
                self.data_connection = Some(data_connection);

                // Read final response even if listing failed
                let _ = self.read_response();
                Err(e)
            }
        }
    }

    fn handle_stor_command(&mut self, filename: &str) -> Result<String> {
        let mut responses = Vec::new();

        // Build local file path using the config's local directory
        let local_path = Path::new(self.config.local_directory()).join(filename);

        // Basic validation
        validate_upload_file(&local_path)?;

        // Auto-PASV if no data connection mode is set
        if self.data_connection.is_none() {
            let pasv_response = self.handle_pasv_command()?;
            responses.push(pasv_response);
        }

        info!("Data connection present");

        // 1. Send STOR command FIRST
        let stor_command = format!("STOR {}", filename);
        self.send_command(&stor_command)?;

        // 2. Read "150 Opening data connection" response
        let stor_response = self.read_response()?;

        // Check if STOR command was accepted (should be "150", not "226")
        if !stor_response.starts_with("150") {
            responses.push(format!("STOR command failed: {}", stor_response));
            return Ok(responses.join("\n"));
        }

        // Add the "150" response
        responses.push(stor_response);

        let mut data_connection = self.data_connection.take().unwrap();

        // 3. NOW establish data connection
        data_connection.connect_to_server()?;

        // 4. Upload the file with progress
        match upload_file_with_progress(&mut data_connection, &local_path, filename) {
            Ok(()) => {
                // Reset connection for next use
                data_connection.reset_connection()?;

                // Put the data connection back
                self.data_connection = Some(data_connection);

                // 5. Read final response from server
                let final_response = self.read_response()?;
                responses.push(final_response);
                Ok(responses.join("\n"))
            }
            Err(e) => {
                // Reset connection even on error
                data_connection.reset_connection()?;

                // Put the data connection back
                self.data_connection = Some(data_connection);

                // Read final response even if upload failed
                let _ = self.read_response();
                Err(e)
            }
        }
    }

    /// Handle RETR command with connection timing fix
    fn handle_retr_command(&mut self, filename: &str) -> Result<String> {
        let mut responses = Vec::new();

        // Build local file path using the config's local directory
        let local_path = Path::new(self.config.local_directory()).join(filename);

        // Basic validation (check if file already exists, directory is writable, etc.)
        validate_download_path(&local_path)?;

        // Auto-PASV if no data connection mode is set
        if self.data_connection.is_none() {
            let pasv_response = self.handle_pasv_command()?;
            responses.push(pasv_response);
        }

        info!("Data connection present");

        // 1. Send RETR command FIRST
        let retr_command = format!("RETR {}", filename);
        self.send_command(&retr_command)?;

        // 2. Read "150 Opening data connection" response
        let retr_response = self.read_response()?;

        // Check if RETR command was accepted (should be "150", not "226")
        if !retr_response.starts_with("150") {
            responses.push(format!("RETR command failed: {}", retr_response));
            return Ok(responses.join("\n"));
        }

        // Add the "150" response
        responses.push(retr_response);

        let mut data_connection = self.data_connection.take().unwrap();

        // 3. NOW establish data connection
        data_connection.connect_to_server()?;

        // 4. Download the file with progress
        match download_file_with_progress(&mut data_connection, &local_path, filename) {
            Ok(()) => {
                // Reset connection for next use
                data_connection.reset_connection()?;

                // Put the data connection back
                self.data_connection = Some(data_connection);

                // 5. Read final response from server
                let final_response = self.read_response()?;
                responses.push(final_response);
                Ok(responses.join("\n"))
            }
            Err(e) => {
                // Reset connection even on error
                data_connection.reset_connection()?;

                // Put the data connection back
                self.data_connection = Some(data_connection);

                // Read final response even if download failed
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
                debug!("Received 221 response - checking if this is QUIT");
                // For QUIT command, server closes connection, so we should be disconnected
                // For LOGOUT, user remains connected
                // We can differentiate by checking if connection is still alive
                if !self.connection.is_connected() {
                    debug!("Connection closed by server, updating state to Disconnected");
                    self.state = ClientState::Disconnected;
                    self.data_connection = None; // Clear data connection
                } else {
                    debug!("Logout successful, updating state to Connected");
                    self.state = ClientState::Connected;
                }
            }

            // For all other responses, don't change state
            _ => {
                debug!("Response code {} - no state change needed", response.code);
            }
        }
    }
}
