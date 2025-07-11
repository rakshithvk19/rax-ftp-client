//! Terminal module for RAX FTP Client
//!
//! Handles user interaction and coordinates between parser and client.

use log::{debug, error, info};
use std::io::{self, Write};

use crate::client::RaxFtpClient;
use crate::commands::parse_command;
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

    /// Run the interactive FTP session with automatic connection attempt
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
            Ok(()) => {
                println!("Connected successfully! State: {}", self.client.get_state());
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

        // Cleanup - only disconnect if still connected
        // (QUIT command may have already handled the disconnection)
        if self.client.is_connected() {
            println!("Closing remaining connection...");
            self.client.disconnect()?;
        }
        Ok(())
    }

    /// Handle a user command using parser and client communication
    fn handle_command(&mut self, input: &str) -> Result<bool> {
        let parsed_command = parse_command(input);

        // Pass all commands directly to the client for execution
        match self.client.execute_command(&parsed_command) {
            Ok(response) => {
                // Display response to user
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
                Ok(true) // Continue with session despite error
            }
        }
    }
}
