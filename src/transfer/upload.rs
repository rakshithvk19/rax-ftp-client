//! File upload functionality

use log::{debug, error, info};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::connection::data::DataConnection;
use crate::error::{RaxFtpClientError, Result};
use crate::terminal::progress::{display_progress, finish_progress, format_bytes};
use crate::transfer::progress::TransferProgress;

/// Upload a file through the data connection with progress tracking
pub fn upload_file_with_progress(
    data_connection: &mut DataConnection,
    local_path: &Path,
    filename: &str,
) -> Result<()> {
    // Open the file
    let file = File::open(local_path).map_err(|e| RaxFtpClientError::FileNotFound {
        code: 550,
        message: format!("Cannot open local file '{}': {}", local_path.display(), e),
    })?;

    // Get file size for progress tracking
    let file_size = file
        .metadata()
        .map_err(|e| RaxFtpClientError::TransferFailed {
            code: 550,
            message: format!("Cannot get file metadata: {e}"),
        })?
        .len();

    info!("Starting upload of '{filename}' ({file_size} bytes)");
    println!("Uploading '{}' ({})...", filename, format_bytes(file_size));

    // Create progress tracker
    let mut progress = TransferProgress::new(file_size);

    // Create buffered reader
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8192]; // 8KB buffer
    let mut total_sent = 0u64;

    loop {
        // Read chunk from file
        match reader.read(&mut buffer) {
            Ok(0) => {
                // End of file reached
                debug!("Reached end of file, {total_sent} bytes sent");
                break;
            }
            Ok(bytes_read) => {
                // Send chunk over data connection
                match data_connection.send_data(&buffer[..bytes_read]) {
                    Ok(bytes_sent) => {
                        total_sent += bytes_sent as u64;
                        progress.add_bytes(bytes_sent as u64);

                        // Update progress display every 64KB or at end
                        if total_sent % 65536 == 0 || progress.is_complete() {
                            display_progress(
                                filename,
                                progress.percentage(),
                                progress.transferred_bytes(),
                                progress.speed_bps(),
                            );
                        }

                        debug!("Sent {bytes_sent} bytes, total: {total_sent}");
                    }
                    Err(e) => {
                        error!("Failed to send data: {e}");
                        println!("\nUpload failed: {e}");
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to read from file: {e}");
                println!("\nUpload failed: Failed to read file");
                return Err(RaxFtpClientError::TransferFailed {
                    code: 550,
                    message: format!("Failed to read file: {e}"),
                });
            }
        }
    }

    // Ensure final progress display
    display_progress(
        filename,
        progress.percentage(),
        progress.transferred_bytes(),
        progress.speed_bps(),
    );
    finish_progress(); // Move to next line after progress bar

    info!(
        "Upload completed: {} bytes in {:?}",
        total_sent,
        progress.elapsed()
    );
    Ok(())
}

/// Validate that a file can be uploaded
pub fn validate_upload_file(local_path: &Path) -> Result<()> {
    if !local_path.exists() {
        return Err(RaxFtpClientError::FileNotFound {
            code: 550,
            message: format!("Local file '{}' does not exist", local_path.display()),
        });
    }

    if !local_path.is_file() {
        return Err(RaxFtpClientError::TransferFailed {
            code: 550,
            message: format!("'{}' is not a file", local_path.display()),
        });
    }

    // Check if file is readable
    match File::open(local_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(RaxFtpClientError::PermissionDenied {
            code: 550,
            message: format!("Cannot read file '{}': {}", local_path.display(), e),
        }),
    }
}
