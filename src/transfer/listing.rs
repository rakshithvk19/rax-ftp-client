//! Directory listing functionality for FTP transfers

use log::{debug, error, info};
use std::io::{BufRead, BufReader};

use crate::connection::DataConnection;
use crate::error::{RaxFtpClientError, Result};

/// Read directory listing from data connection
pub fn read_directory_listing(mut data_connection: DataConnection) -> Result<Vec<String>> {
    info!("Reading directory listing from data connection");

    let mut listing = Vec::new();
    let mut buffer = [0u8; 8192]; // 8KB buffer
    let mut accumulated_data = String::new();

    // TODO: Implement timeout-based approach for large directory listings
    // This would handle directories with thousands of files more efficiently
    // by setting read timeouts and processing data in chunks with progress updates

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

    // Close the data connection
    data_connection.close()?;

    info!("Successfully read {} directory entries", listing.len());
    Ok(listing)
}

/// Validate directory listing response format
pub fn validate_listing_format(listing: &[String]) -> Result<()> {
    // Basic validation - ensure we have reasonable directory entries
    if listing.is_empty() {
        return Ok(()); // Empty directory is valid
    }

    // Check for common invalid patterns
    for entry in listing {
        if entry.len() > 255 {
            return Err(RaxFtpClientError::InvalidResponse(
                "Directory entry name too long".to_string(),
            ));
        }

        // Check for null bytes or other invalid characters
        if entry.contains('\0') {
            return Err(RaxFtpClientError::InvalidResponse(
                "Directory entry contains invalid characters".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_listing_format() {
        let valid_listing = vec![
            "file1.txt".to_string(),
            "file2.txt".to_string(),
            "folder1".to_string(),
        ];
        assert!(validate_listing_format(&valid_listing).is_ok());

        let empty_listing = vec![];
        assert!(validate_listing_format(&empty_listing).is_ok());

        let invalid_listing = vec!["file1.txt".to_string(), "file\0with\0nulls.txt".to_string()];
        assert!(validate_listing_format(&invalid_listing).is_err());
    }

    #[test]
    fn test_long_filename_validation() {
        let long_name = "a".repeat(300);
        let invalid_listing = vec![long_name];
        assert!(validate_listing_format(&invalid_listing).is_err());
    }
}
