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

    /// Run the interactive FTP session
    pub fn run_interactive(&mut self) -> Result<()> {
        println!("RAX FTP Client - Interactive Session");
        println!("Connected to: {}", self.config.display_name());
        println!("Type 'HELP' for available commands or 'QUIT' to exit");
        println!();

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
            FtpCommand::Quit => {
                println!("Disconnecting from server...");
                Ok(false)
            }
            FtpCommand::Unknown(msg) => {
                println!("Error: {}", msg);
                Ok(true)
            }
            // For all other commands, send to server and display response
            cmd => self.execute_ftp_command(cmd),
        }
    }

    /// Execute FTP command by sending to server and displaying response
    fn execute_ftp_command(&mut self, command: FtpCommand) -> Result<bool> {
        if command.is_client_only() {
            // This shouldn't happen since we handle HELP above, but just in case
            return Ok(true);
        }

        // Convert to FTP protocol string and send to server
        let ftp_command_str = command.to_ftp_string();
        debug!("Sending FTP command: {}", ftp_command_str);

        // Send command to server
        if let Err(e) = self.client.send_command(&ftp_command_str) {
            println!("Failed to send command: {}", e);
            return Ok(true);
        }

        // Read response from server
        match self.client.read_response() {
            Ok(response) => {
                // Display server response to user
                print!("{}", response);
                if !response.ends_with('\n') {
                    println!(); // Ensure newline
                }
                Ok(true)
            }
            Err(e) => {
                println!("Failed to read server response: {}", e);
                Ok(true)
            }
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
}
