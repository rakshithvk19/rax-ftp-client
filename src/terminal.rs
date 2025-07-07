//! Terminal module for RAX FTP Client
//!
//! Handles user interaction and coordinates between parser and client.

use log::{debug, error, info};
use std::io::{self, Write};

use crate::client::RaxFtpClient;
use crate::commands::{FtpCommand, parse_command};
use crate::config::ClientConfig;
use crate::error::Result;

/// Terminal handler for interactive FTP sessions
pub struct Terminal {
    client: RaxFtpClient,
    config: ClientConfig,
}

impl Terminal {
    /// Create a new terminal with the given client and config
    pub fn new(client: RaxFtpClient, config: ClientConfig) -> Self {
        info!(
            "Creating terminal session for server: {}",
            config.display_name()
        );

        Self { client, config }
    }

    // Run the interactive FTP session with automatic connection attempt
    pub fn run_interactive(&mut self) -> Result<()> {
        // Show initial state (disconnected)
        println!("RAX FTP Client - Interactive Session");
        println!("Connected to: {}", self.config.display_name());
        println!("Current state: {}", self.client.get_state());
        println!("Type 'HELP' for available commands or 'QUIT' to exit");
        println!();

        // Attempt automatic connection
        println!("Attempting to connect to server...");
        match self.client.connect_with_retries() {
            Ok(greeting) => {
                println!("Connected successfully!");
                println!("{}", greeting.trim()); // Display server greeting
            }
            Err(e) => {
                println!("Connection failed: {}", e);
                println!("Continuing in disconnected mode...");
            }
        }
        println!();

        // Continue with interactive session
        let stdin = io::stdin();
        loop {
            // Show prompt with current state
            print!("rax-ftp-client ({})> ", self.client.get_state());
            io::stdout().flush()?;

            // Read user input
            let mut input = String::new();
            match stdin.read_line(&mut input) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let command = input.trim();
                    if command.is_empty() {
                        continue;
                    }

                    debug!("User entered command: {}", command);

                    // Parse command and handle it
                    match self.handle_command(command) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Command failed: {}", e);
                            println!("Error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read input: {}", e);
                    return Err(e.into());
                }
            }
        }

        // Cleanup
        println!("Closing session...");
        self.client.disconnect()?;
        Ok(())
    }

    /// Handle a user command using parser and client communication
    fn handle_command(&mut self, input: &str) -> Result<bool> {
        let parsed_command = parse_command(input);

        match parsed_command {
            FtpCommand::Help => {
                self.show_help();
                Ok(true)
            }
            FtpCommand::Unknown(msg) => {
                println!("Error: {}", msg);
                Ok(true)
            }
            // For all other commands, send to server and display response
            cmd => self.execute_ftp_command(cmd),
        }
    }

    /// Show help information (client-side only)
    fn show_help(&self) {
        println!("Available commands:");
        println!("  USER <username>   - Authenticate with username");
        println!("  PASS <password>   - Provide password");
        println!("  STOR <filename>   - Upload file to server");
        println!("  RETR <filename>   - Download file from server");
        println!("  PORT <ip:port>    - Set data connection port (active mode)");
        println!("  PASV              - Enter passive mode");
        println!("  LIST              - List directory contents");
        println!("  PWD               - Print working directory");
        println!("  CWD <directory>   - Change working directory");
        println!("  DEL <filename>    - Delete file on server");
        println!("  LOGOUT            - Log out current user");
        println!("  RAX               - Custom server command");
        println!("  QUIT              - Disconnect and exit");
        println!("  HELP              - Show this help message");
        println!();
        println!("Current server: {}", self.config.display_name());
        println!("Current state: {}", self.client.get_state());
        println!("Local directory: {}", self.config.local_directory);
    }

    /// Execute FTP command by sending to server and displaying response
    fn execute_ftp_command(&mut self, command: FtpCommand) -> Result<bool> {
        if command.is_client_only() {
            // This shouldn't happen since we handle HELP above, but just in case
            return Ok(true);
        }

        // Use the enhanced client execute_command method
        match self.client.execute_command(&command) {
            Ok(response) => {
                // Display server response to user
                print!("{}", response);
                if !response.ends_with('\n') {
                    println!(); // Ensure newline
                }

                // Check if client state changed to disconnected (for QUIT command)
                if !self.client.is_connected() {
                    println!("Connection closed by server. Closing session...");
                    return Ok(false); // Exit the interactive loop
                }

                Ok(true) // Continue with session
            }
            Err(e) => {
                println!("Command failed: {}", e);
                Ok(true)
            }
        }
    }
}
