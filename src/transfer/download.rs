//! File download functionality

use log::{debug, error, info};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::connection::data::DataConnection;
use crate::error::{RaxFtpClientError, Result};
use crate::terminal::progress::{display_progress, finish_progress, format_bytes};
use crate::transfer::progress::TransferProgress;

/// Download a file through the data connection with progress tracking
pub fn download_file_with_progress(
    data_connection: &mut DataConnection,
    local_path: &Path,
    filename: &str,
) -> Result<()> {
    info!("Starting download of '{filename}'");
    println!("Downloading '{filename}'...");

    // Create the local file
    let file = File::create(local_path).map_err(|e| RaxFtpClientError::TransferFailed {
        code: 550,
        message: format!("Cannot create local file '{}': {}", local_path.display(), e),
    })?;

    // Create buffered writer
    let mut writer = BufWriter::new(file);
    let mut buffer = [0u8; 8192]; // 8KB buffer
    let mut total_received = 0u64;

    // We don't know the file size ahead of time, so start with 0 and update as we go
    let mut progress = TransferProgress::new(0);

    loop {
        // Receive chunk from data connection
        match data_connection.receive_data(&mut buffer) {
            Ok(0) => {
                // End of file reached
                debug!("Reached end of file, {total_received} bytes received");
                break;
            }
            Ok(bytes_received) => {
                // Write chunk to local file
                match writer.write_all(&buffer[..bytes_received]) {
                    Ok(()) => {
                        total_received += bytes_received as u64;

                        // Update progress tracker total size as we go (since we don't know it beforehand)
                        if progress.total_bytes() < total_received {
                            progress = TransferProgress::new(total_received);
                        }
                        progress.add_bytes(bytes_received as u64);

                        // Update progress display every 64KB or at intervals
                        if total_received % 65536 == 0 {
                            display_progress(
                                filename,
                                100.0, // Show as 100% since we don't know total size
                                total_received,
                                progress.speed_bps(),
                            );
                        }

                        debug!("Received {bytes_received} bytes, total: {total_received}");
                    }
                    Err(e) => {
                        error!("Failed to write to local file: {e}");
                        println!("\nDownload failed: Failed to write to file");
                        return Err(RaxFtpClientError::TransferFailed {
                            code: 550,
                            message: format!("Failed to write to file: {e}"),
                        });
                    }
                }
            }
            Err(e) => {
                error!("Failed to receive data: {e}");
                println!("\nDownload failed: {e}");
                return Err(e);
            }
        }
    }

    // Ensure all data is written to disk
    if let Err(e) = writer.flush() {
        error!("Failed to flush file: {e}");
        return Err(RaxFtpClientError::TransferFailed {
            code: 550,
            message: format!("Failed to flush file: {e}"),
        });
    }

    // Final progress display
    display_progress(filename, 100.0, total_received, progress.speed_bps());
    finish_progress(); // Move to next line after progress bar

    info!(
        "Download completed: {} bytes in {:?}",
        total_received,
        progress.elapsed()
    );
    println!(
        "Download completed: {} ({})",
        filename,
        format_bytes(total_received)
    );
    Ok(())
}

/// Validate that a directory can be written to for downloads
pub fn validate_download_path(local_path: &Path) -> Result<()> {
    // Check if parent directory exists and is writable
    if let Some(parent) = local_path.parent() {
        if !parent.exists() {
            return Err(RaxFtpClientError::TransferFailed {
                code: 550,
                message: format!("Directory '{}' does not exist", parent.display()),
            });
        }

        if !parent.is_dir() {
            return Err(RaxFtpClientError::TransferFailed {
                code: 550,
                message: format!("'{}' is not a directory", parent.display()),
            });
        }
    }

    // Check if file already exists
    if local_path.exists() {
        return Err(RaxFtpClientError::TransferFailed {
            code: 550,
            message: format!("File '{}' already exists", local_path.display()),
        });
    }

    Ok(())
}
