//! Directory listing functionality for FTP transfers

use log::{debug, error, info};
use std::io::{BufRead, BufReader};

use crate::connection::DataConnection;
use crate::error::{RaxFtpClientError, Result};

/// Read directory listing from data connection
pub fn read_directory_listing(data_connection: &mut DataConnection) -> Result<Vec<String>> {
    info!("Reading directory listing from data connection");

    let mut listing = Vec::new();
    let mut buffer = [0u8; 8192]; // 8KB buffer
    let mut accumulated_data = String::new();

    // Read data until connection closes naturally
    loop {
        match data_connection.receive_data(&mut buffer) {
            Ok(0) => {
                // Connection closed by server, we're done
                debug!("Data connection closed by server");
                break;
            }
            Ok(bytes_read) => {
                let data = String::from_utf8_lossy(&buffer[..bytes_read]);
                accumulated_data.push_str(&data);
                debug!("Received {} bytes of directory listing data", bytes_read);
            }
            Err(e) => {
                error!("Failed to read directory listing: {}", e);
                return Err(RaxFtpClientError::TransferFailed {
                    code: 426,
                    message: format!("Failed to read directory listing: {}", e),
                });
            }
        }
    }

    // Parse the accumulated data into individual entries
    let reader = BufReader::new(accumulated_data.as_bytes());
    for line in reader.lines() {
        match line {
            Ok(entry) => {
                let trimmed = entry.trim();
                if !trimmed.is_empty() {
                    listing.push(trimmed.to_string());
                }
            }
            Err(e) => {
                error!("Failed to parse directory listing line: {}", e);
                // Continue processing other lines
            }
        }
    }

    info!("Successfully read {} directory entries", listing.len());
    Ok(listing)
}
