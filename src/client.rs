use log::{debug, info};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::commands::FtpCommand;
use crate::config::ClientConfig;
use crate::connection::{CommandConnection, DataConnection};
use crate::error::{RaxFtpClientError, Result};
use crate::responses::{FtpResponse, is_authentication_success, parse_response};
use crate::transfer::{read_directory_listing, upload_file_with_progress, validate_upload_file};

/// Data connection modes
#[derive(Debug, Clone, PartialEq)]
pub enum DataMode {
    Active,
    Passive,
}

/// Data connection information
#[derive(Debug, Clone)]
pub struct DataConnectionInfo {
    pub mode: DataMode,
    pub host: String,
    pub port: u16,
}

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
    current_data_mode: DataMode,
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
            current_data_mode: DataMode::Passive, // Default to passive mode
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
        self.data_connection_info = None; // Clear data connection info
        self.current_data_mode = DataMode::Passive; // Reset to default
        Ok(())
    }

    /// Send a command and handle file transfers if needed
    pub fn execute_command(&mut self, command: &FtpCommand) -> Result<String> {
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

        // If successful, update data connection mode
        if response.starts_with("200") {
            self.current_data_mode = DataMode::Active;
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
        debug!("Sending PASV command");
        self.send_command("PASV")?;

        debug!("Reading PASV response");
        let response = self.read_response()?;
        debug!("Received PASV response: '{}'", response);

        // If successful, parse the response and update data connection mode
        if response.starts_with("227") {
            debug!("Response starts with 227, attempting to parse");
            if let Some(addr_info) = self.parse_pasv_response(&response) {
                self.current_data_mode = DataMode::Passive;
                self.data_connection_info = Some(addr_info);
                debug!("PASV command successful, data_connection_info set");
            } else {
                debug!("PASV response parsing failed");
                return Err(RaxFtpClientError::DataConnectionFailed(
                    "Failed to parse PASV response".to_string(),
                ));
            }
        } else {
            debug!("Response does not start with 227");
        }

        Ok(response)
    }

    /// Parse PASV response to extract IP and port
    fn parse_pasv_response(&self, response: &str) -> Option<DataConnectionInfo> {
        debug!("Parsing PASV response: {}", response);

        // Expected format: "227 Entering Passive Mode (127.0.0.1:2122)"
        if let Some(start) = response.find('(') {
            if let Some(end) = response.find(')') {
                let addr_str = &response[start + 1..end];
                debug!("Extracted address string: '{}'", addr_str);

                if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() {
                    debug!("Successfully parsed address: {}", addr);
                    return Some(DataConnectionInfo {
                        mode: DataMode::Passive,
                        host: addr.ip().to_string(),
                        port: addr.port(),
                    });
                } else {
                    debug!("Failed to parse '{}' as SocketAddr", addr_str);
                }
            } else {
                debug!("No closing parenthesis found in response");
            }
        } else {
            debug!("No opening parenthesis found in response");
        }
        None
    }
    /// Establish data connection (reusable for STOR, RETR, LIST)
    fn establish_data_connection(&mut self) -> Result<DataConnection> {
        match self.current_data_mode {
            DataMode::Active => self.establish_active_connection(),
            DataMode::Passive => self.establish_passive_connection(),
        }
    }

    /// Establish active data connection
    fn establish_active_connection(&mut self) -> Result<DataConnection> {
        DataConnection::new_port_mode(&self.config)
    }

    /// Establish passive data connection with retry logic
    fn establish_passive_connection(&mut self) -> Result<DataConnection> {
        // Use existing data_connection_info if available
        if let Some(ref conn_info) = self.data_connection_info {
            return DataConnection::new_passive_mode(&conn_info.host, conn_info.port);
        }

        // If no existing connection info, establish new PASV connection
        const MAX_RETRIES: u32 = 3;

        for attempt in 1..=MAX_RETRIES {
            // Send PASV command to server
            match self.handle_pasv_command() {
                Ok(response) if response.starts_with("227") => {
                    // PASV successful, now create data connection
                    if let Some(ref conn_info) = self.data_connection_info {
                        match DataConnection::new_passive_mode(&conn_info.host, conn_info.port) {
                            Ok(data_conn) => return Ok(data_conn),
                            Err(e) => {
                                if attempt < MAX_RETRIES {
                                    let wait_time = attempt as u64; // Exponential backoff
                                    thread::sleep(Duration::from_secs(wait_time));
                                    continue;
                                } else {
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
                Ok(_) => {
                    if attempt < MAX_RETRIES {
                        let wait_time = attempt as u64;
                        thread::sleep(Duration::from_secs(wait_time));
                        continue;
                    } else {
                        return Err(RaxFtpClientError::DataConnectionFailed(
                            "PASV command failed after retries. Please try PORT command manually."
                                .to_string(),
                        ));
                    }
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        let wait_time = attempt as u64;
                        thread::sleep(Duration::from_secs(wait_time));
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(RaxFtpClientError::DataConnectionFailed(
        "Failed to establish passive connection after retries. Please try PORT command manually.".to_string()
    ))
    }

    /// Auto-establish PASV mode with exponential backoff
    fn auto_establish_pasv_mode(&mut self) -> Result<()> {
        const MAX_RETRIES: u32 = 3;

        for attempt in 1..=MAX_RETRIES {
            match self.handle_pasv_command() {
                Ok(response) if response.starts_with("227") => {
                    // PASV successful, store permanently for session
                    self.current_data_mode = DataMode::Passive;
                    return Ok(());
                }
                Ok(response) => {
                    debug!("PASV command failed with response: {}", response);
                }
                Err(e) => {
                    debug!("PASV command error: {}", e);
                }
            }

            // Retry with exponential backoff
            if attempt < MAX_RETRIES {
                let wait_time = attempt as u64; // Exponential backoff: 1s, 2s, 3s
                thread::sleep(Duration::from_secs(wait_time));
            }
        }

        Err(RaxFtpClientError::DataConnectionFailed(
            "Failed to auto-establish PASV mode after 3 retries".to_string(),
        ))
    }

    /// Handle LIST command with data channel
    fn handle_list_command(&mut self) -> Result<String> {
        // Auto-establish PASV mode if no data connection info exists
        if self.data_connection_info.is_none() {
            match self.auto_establish_pasv_mode() {
                Ok(()) => {
                    info!("Auto-established PASV mode for LIST command");
                }
                Err(e) => {
                    return Err(RaxFtpClientError::DataConnectionFailed(format!(
                        "425 Can't open data connection. Auto-PASV failed: {}. Try running 'port <your_ip>:<port>' command manually.",
                        e
                    )));
                }
            }
        }

        // Establish data connection
        let mut data_connection = self.establish_data_connection()?;

        // Send PORT command if in active mode
        if self.current_data_mode == DataMode::Active {
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
        if self.current_data_mode == DataMode::Active {
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

    /// Handle STOR command with file transfer
    fn handle_stor_command(&mut self, filename: &str) -> Result<String> {
        // Build local file path using the config's local directory
        let local_path = Path::new(self.config.local_directory()).join(filename);

        // Validate file before starting transfer
        validate_upload_file(&local_path)?;

        // Establish data connection
        let mut data_connection = self.establish_data_connection()?;

        // Send PORT command if in active mode
        if self.current_data_mode == DataMode::Active {
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

        // Accept/Connect to data connection
        if self.current_data_mode == DataMode::Active {
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
                    self.data_connection_info = None; // Clear data connection info
                    self.current_data_mode = DataMode::Passive; // Reset to default
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
