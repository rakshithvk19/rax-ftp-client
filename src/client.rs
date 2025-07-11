use log::{debug, info};
use std::path::Path;

use crate::commands::{FtpCommand, get_help_text};
use crate::config::ClientConfig;
use crate::connection::{CommandConnection, DataConnection, DataConnectionInfo, DataMode};
use crate::error::{RaxFtpClientError, Result};
use crate::responses::{FtpResponse, is_authentication_success, parse_response};
use crate::transfer::{read_directory_listing, upload_file_with_progress, validate_upload_file};

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
    data_connection_info: Option<DataConnectionInfo>,
}

impl RaxFtpClient {
    /// Create a new FTP client with the given configuration
    pub fn new(config: ClientConfig) -> Self {
        info!("Creating RAX FTP Client with config: {}", config);

        Self {
            connection: CommandConnection::new(&config),
            state: ClientState::Disconnected,
            config,
            data_connection_info: None,
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
        self.data_connection_info = None; // Clear connection info
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

        // Send PORT command to server
        let command_str = format!("PORT {}", addr);
        self.send_command(&command_str)?;
        let response = self.read_response()?;

        // If successful, store data connection info
        if response.starts_with("200") {
            self.data_connection_info = Some(DataConnectionInfo {
                mode: DataMode::Active,
                host: parsed_addr.ip().to_string(),
                port: parsed_addr.port(),
            });
        }

        Ok(response)
    }
    /// Handle PASV command - switch to passive mode
    fn handle_pasv_command(&mut self) -> Result<String> {
        self.send_command("PASV")?;
        let response = self.read_response()?;

        // If successful, parse and store passive mode info
        if response.starts_with("227") {
            // Parse server's response format: "227 Entering Passive Mode (127.0.0.1:2122)"
            if let Some(start) = response.find('(') {
                if let Some(end) = response.find(')') {
                    let addr_str = &response[start + 1..end];
                    if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() {
                        self.data_connection_info = Some(DataConnectionInfo {
                            mode: DataMode::Passive,
                            host: addr.ip().to_string(),
                            port: addr.port(),
                        });
                    }
                }
            }
        }

        Ok(response)
    }

    fn handle_list_command(&mut self) -> Result<String> {
        // Check if data connection mode is already set
        if self.data_connection_info.is_none() {
            info!("No data connection mode set, automatically entering passive mode...");

            // Automatically execute PASV command
            let pasv_response = self.handle_pasv_command()?;

            // Check if PASV was successful
            if !pasv_response.starts_with("227") {
                return Err(RaxFtpClientError::DataConnectionFailed(format!(
                    "Failed to enter passive mode: {}",
                    pasv_response
                )));
            }

            // Print passive mode info for user
            println!("Automatically entered passive mode: {}", pasv_response);
        }

        // Now proceed with the existing LIST logic
        // Establish data connection based on current mode
        let mut data_connection = match DataConnection::establish_from_info(
            self.data_connection_info.as_ref(),
            &self.config,
        ) {
            Ok(conn) => conn,
            Err(e) => {
                // This should rarely happen now since we auto-PASV above
                return Err(RaxFtpClientError::DataConnectionFailed(format!(
                    "425 Can't open data connection: {}",
                    e
                )));
            }
        };

        // Rest of the existing LIST command logic...
        // Send PORT command if in active mode
        if self
            .data_connection_info
            .as_ref()
            .map_or(false, |info| info.mode == DataMode::Active)
        {
            let port_command = data_connection.get_port_command()?;
            self.send_command(&port_command)?;
            let port_response = self.read_response()?;

            if !port_response.starts_with("200") {
                return Ok(format!("PORT command failed: {}", port_response));
            }
        }

        // Send LIST command
        self.send_command("LIST")?;

        // Accept/Connect to data connection
        if self
            .data_connection_info
            .as_ref()
            .map_or(false, |info| info.mode == DataMode::Active)
        {
            data_connection.accept_connection()?;
        } else {
            data_connection.connect_to_server()?;
        }

        // Read directory listing from data channel
        match read_directory_listing(data_connection) {
            Ok(listing) => {
                // Display formatted directory listing
                self.display_directory_listing(&listing);

                // Read final response from server
                let final_response = self.read_response()?;
                Ok(final_response)
            }
            Err(e) => {
                // Read final response even if listing failed
                let _ = self.read_response();
                Err(e)
            }
        }
    }

    /// Display directory listing in formatted columns
    fn display_directory_listing(&self, listing: &[String]) {
        if listing.is_empty() {
            println!("Directory is empty.");
            return;
        }

        // TODO: Implement timeout-based approach for large directory listings
        // This would be useful for directories with thousands of files

        println!(
            "{:<30} {:<8} {:<10} {:<20}",
            "Name", "Type", "Size", "Modified"
        );
        println!("{}", "-".repeat(68));

        for entry in listing {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }

            // Simple classification - in future this could be enhanced
            // to parse detailed file information from server
            let (name, file_type, size, modified) = if entry == "." || entry == ".." {
                (entry, "Dir", "-", "-")
            } else if entry.ends_with('/') {
                (entry, "Dir", "-", "-")
            } else {
                (entry, "File", "-", "-")
            };

            println!(
                "{:<30} {:<8} {:<10} {:<20}",
                name, file_type, size, modified
            );
        }
    }

    /// Handle STOR command with connection timing fix
    fn handle_stor_command(&mut self, filename: &str) -> Result<String> {
        // Build local file path using the config's local directory
        let local_path = Path::new(self.config.local_directory()).join(filename);

        // Basic validation (keep your existing validate_upload_file call)
        validate_upload_file(&local_path)?;

        // Establish data connection based on current mode
        let mut data_connection = match DataConnection::establish_from_info(
            self.data_connection_info.as_ref(),
            &self.config,
        ) {
            Ok(conn) => conn,
            Err(e) => {
                return Err(RaxFtpClientError::DataConnectionFailed(format!(
                    "425 Can't open data connection: {}. Try running 'pasv' or 'port <your_ip>:<port>' command first.",
                    e
                )));
            }
        };

        // Send PORT command if in active mode
        if self
            .data_connection_info
            .as_ref()
            .map_or(false, |info| info.mode == DataMode::Active)
        {
            let port_command = data_connection.get_port_command()?;
            self.send_command(&port_command)?;
            let port_response = self.read_response()?;

            if !port_response.starts_with("200") {
                return Ok(format!("PORT command failed: {}", port_response));
            }
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

        // Server calls setup_data_stream after processing STOR command
        println!("Waiting for server to prepare data connection...");
        std::thread::sleep(std::time::Duration::from_millis(300));

        // NOW connect to data channel
        if self
            .data_connection_info
            .as_ref()
            .map_or(false, |info| info.mode == DataMode::Active)
        {
            data_connection.accept_connection()?;
        } else {
            data_connection.connect_to_server()?;
        }

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
                debug!("Received 221 response - checking if this is QUIT");
                // For QUIT command, server closes connection, so we should be disconnected
                // For LOGOUT, user remains connected
                // We can differentiate by checking if connection is still alive
                if !self.connection.is_connected() {
                    debug!("Connection closed by server, updating state to Disconnected");
                    self.state = ClientState::Disconnected;
                    self.data_connection_info = None; // Clear connection info
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
