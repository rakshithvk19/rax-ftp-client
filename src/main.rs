use std::process;

mod client;
mod commands;
mod config;
mod connection;
mod error;
mod responses;
mod terminal;
mod transfer;

use client::RaxFtpClient;
use config::ClientConfig;
use terminal::session::Terminal;

fn main() {
    // Initialize logging
    env_logger::init();

    // Parse configuration from TOML file with environment variable overrides
    let config = match ClientConfig::from_config_file("config/client_config.toml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            print_usage();
            process::exit(1);
        }
    };

    // Create client (starts in disconnected state)
    let client = RaxFtpClient::new(config.clone());

    // Create terminal and run interactive session
    // Terminal will handle the connection attempt internally
    let mut terminal = Terminal::new(client, config);
    if let Err(e) = terminal.run_interactive() {
        eprintln!("Terminal error: {}", e);
        process::exit(1);
    }
}

fn print_usage() {
    println!("RAX FTP Client");
    println!();
    println!("Configuration:");
    println!("  Edit 'config/client_config.toml' to configure server settings");
    println!();
    println!("Environment Variable Overrides:");
    println!("  RAX_FTP_HOST=127.0.0.1            # Override server host");
    println!("  RAX_FTP_HOST_NAME=\"Comp Lab 2\"   # Override server display name");
    println!("  RAX_FTP_PORT=2121                 # Override server port");
    println!("  RAX_FTP_LOCAL_DIR=\"./client_root\" # Override local directory");
    println!("  RAX_FTP_TIMEOUT=5                 # Override connection timeout");
    println!("  RAX_FTP_MAX_RETRIES=3             # Override retry attempts");
    println!("  RAX_FTP_DATA_PORT_START=2122      # Override data port start");
    println!("  RAX_FTP_DATA_PORT_END=2130        # Override data port end");
    println!();
    println!("Logging:");
    println!("  RUST_LOG=info                     # Set logging level");
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
        assert_eq!(config.host(), "127.0.0.1");
        assert_eq!(config.port(), 2121);
    }
}
