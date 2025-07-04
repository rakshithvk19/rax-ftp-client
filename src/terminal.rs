//! Terminal module for RAX FTP Client
//!
//! Handles user interaction and command dispatching for interactive sessions.

use log::{debug, error, info};
use std::io::{self, Write};

use crate::client::RaxFtpClient;
use crate::config::ClientConfig;
use crate::error::{RaxFtpClientError, Result};

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

                    // Handle the command
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

    /// Handle a user command and return whether to continue the session
    /// NOTE: This is temporary - will be replaced by command parser later
    fn handle_command(&mut self, command: &str) -> Result<bool> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(true);
        }

        let cmd = parts[0].to_uppercase();

        match cmd.as_str() {
            "HELP" => {
                self.show_help();
                Ok(true)
            }
            "QUIT" => {
                println!("Disconnecting from server...");
                Ok(false)
            }
            "USER" => {
                if parts.len() < 2 {
                    println!("Usage: USER <username>");
                    return Ok(true);
                }
                self.handle_user_command(parts[1])
            }
            "PASS" => {
                if parts.len() < 2 {
                    println!("Usage: PASS <password>");
                    return Ok(true);
                }
                self.handle_pass_command(parts[1])
            }
            "STOR" => {
                if parts.len() < 2 {
                    println!("Usage: STOR <filename>");
                    return Ok(true);
                }
                self.handle_stor_command(parts[1])
            }
            "RETR" => {
                if parts.len() < 2 {
                    println!("Usage: RETR <filename>");
                    return Ok(true);
                }
                self.handle_retr_command(parts[1])
            }
            "PORT" => {
                if parts.len() < 2 {
                    println!("Usage: PORT <ip:port>");
                    return Ok(true);
                }
                self.handle_port_command(parts[1])
            }
            "PASV" => self.handle_pasv_command(),
            "LIST" => self.handle_list_command(),
            "PWD" => self.handle_pwd_command(),
            "CWD" => {
                if parts.len() < 2 {
                    println!("Usage: CWD <directory>");
                    return Ok(true);
                }
                self.handle_cwd_command(parts[1])
            }
            "DEL" => {
                if parts.len() < 2 {
                    println!("Usage: DEL <filename>");
                    return Ok(true);
                }
                self.handle_del_command(parts[1])
            }
            "LOGOUT" => self.handle_logout_command(),
            "RAX" => self.handle_rax_command(),
            _ => {
                println!(
                    "Unknown command: {}. Type 'HELP' for available commands.",
                    cmd
                );
                Ok(true)
            }
        }
    }

    /// Show help information
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

    // TODO: These command handlers are temporary stubs
    // They will be replaced by proper command parsing and client methods

    fn handle_user_command(&mut self, username: &str) -> Result<bool> {
        println!("331 User name okay, need password.");
        Ok(true)
    }

    fn handle_pass_command(&mut self, password: &str) -> Result<bool> {
        println!("230 User logged in, proceed.");
        Ok(true)
    }

    fn handle_stor_command(&mut self, filename: &str) -> Result<bool> {
        println!("150 Opening data connection for file transfer.");
        println!("226 Transfer complete.");
        Ok(true)
    }

    fn handle_retr_command(&mut self, filename: &str) -> Result<bool> {
        println!("150 Opening data connection for file transfer.");
        println!("226 Transfer complete.");
        Ok(true)
    }

    fn handle_port_command(&mut self, port_spec: &str) -> Result<bool> {
        println!("200 Port command successful.");
        Ok(true)
    }

    fn handle_pasv_command(&mut self) -> Result<bool> {
        println!("227 Entering passive mode (127,0,0,1,8,235).");
        Ok(true)
    }

    fn handle_list_command(&mut self) -> Result<bool> {
        println!("150 Opening data connection for directory listing.");
        println!("drwxr-xr-x   2 user     user         4096 Jul 03 16:30 documents");
        println!("-rw-r--r--   1 user     user         1024 Jul 03 16:25 test.txt");
        println!("226 Directory send OK.");
        Ok(true)
    }

    fn handle_pwd_command(&mut self) -> Result<bool> {
        println!("257 \"/home/user\" is current directory.");
        Ok(true)
    }

    fn handle_cwd_command(&mut self, directory: &str) -> Result<bool> {
        println!("250 Directory changed successfully.");
        Ok(true)
    }

    fn handle_del_command(&mut self, filename: &str) -> Result<bool> {
        println!("250 File deleted successfully.");
        Ok(true)
    }

    fn handle_logout_command(&mut self) -> Result<bool> {
        println!("221 Logout successful.");
        Ok(true)
    }

    fn handle_rax_command(&mut self) -> Result<bool> {
        println!("200 Rax is the best.");
        Ok(true)
    }
}
