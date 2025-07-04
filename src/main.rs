use std::process;

mod client;
mod commands;
mod config;
mod connection;
mod error;
mod logging;
mod responses;
mod terminal;
mod transfer;
mod utils;

use client::RaxFtpClient;
use config::ClientConfig;
use terminal::Terminal;

fn main() {
    // Initialize logging
    env_logger::init();

    // Parse configuration from environment variables
    let config = match ClientConfig::from_env_and_args() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            print_usage();
            process::exit(1);
        }
    };

    // Create client
    let mut client = RaxFtpClient::new(config.clone());

    // Connect to server
    if let Err(e) = client.connect_with_retries() {
        eprintln!("Failed to connect: {}", e);
        process::exit(1);
    }

    // Create terminal and run interactive session
    let mut terminal = Terminal::new(client, config);
    if let Err(e) = terminal.run_interactive() {
        eprintln!("Terminal error: {}", e);
        process::exit(1);
    }
}

fn print_usage() {
    println!("RAX FTP Client");
    println!("Environment Variables:");
    println!("  RAX_FTP_HOST=127.0.0.1");
    println!("  RAX_FTP_HOST_NAME=\"Comp Lab 2\"");
    println!("  RAX_FTP_PORT=2121");
    println!("  RAX_FTP_LOCAL_DIR=\"./downloads\"");
    println!("  RUST_LOG=info");
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_module_imports() {
        // Verify all modules are properly imported
        assert!(true);
    }

    #[test]
    fn test_config_creation() {
        // Test that configuration can be created
        let config = ClientConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 2121);
    }
}
