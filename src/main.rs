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
    let config = match ClientConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load client configuration: {e}");
            process::exit(1);
        }
    };
    // Create client (starts in disconnected state)
    let client = RaxFtpClient::new(config.clone());

    // Create terminal and run interactive session
    // Terminal will handle the connection attempt internally
    let mut terminal = Terminal::new(client, config);
    if let Err(e) = terminal.run_interactive() {
        eprintln!("Terminal error: {e}");
        process::exit(1);
    }
}
