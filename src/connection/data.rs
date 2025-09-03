//! Data connection management for FTP transfers

use log::{debug, error, info, warn};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use crate::error::{RaxFtpClientError, Result};

// Constants
const DEFAULT_BIND_IP: &str = "0.0.0.0";
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_ACTIVE_TIMEOUT_SECS: u64 = 5;

/// Manages FTP data connections for file transfers
pub struct DataConnection {
    mode: DataConnectionMode,
    timeout: Duration,
}

/// Data connection mode
#[derive(Debug)]
enum DataConnectionMode {
    Active {
        listener: Option<TcpListener>,
        stream: Option<TcpStream>,
    },
    Passive {
        stream: Option<TcpStream>,
        server_host: String,
        server_port: u16,
    },
}

impl DataConnection {
    /// Create a new data connection for PORT mode (Active) on a specific port
    pub fn active_mode(port: u16) -> Result<Self> {
        let addr = format!("{DEFAULT_BIND_IP}:{port}");

        let listener = TcpListener::bind(&addr).map_err(|e| {
            warn!("Failed to bind to specific port {port}: {e}");
            RaxFtpClientError::DataConnectionFailed(format!("Failed to bind to port {port}: {e}"))
        })?;

        info!("Created data connection listener on specific port {port}");

        // Set listener to blocking mode
        listener.set_nonblocking(false)?;

        Ok(Self {
            mode: DataConnectionMode::Active {
                listener: Some(listener),
                stream: None,
            },
            timeout: Duration::from_secs(DEFAULT_ACTIVE_TIMEOUT_SECS),
        })
    }

    /// Create a new data connection for PASV mode (Passive)
    pub fn passive_mode(server_host: &str, server_port: u16) -> Result<Self> {
        info!("Creating passive data connection to {server_host}:{server_port}");

        let connection = Self {
            mode: DataConnectionMode::Passive {
                stream: None,
                server_host: server_host.to_string(),
                server_port,
            },
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        };

        info!("Passive data connection configured for {server_host}:{server_port}");
        Ok(connection)
    }

    /// Connect to server (handles both Active and Passive modes)
    pub fn connect_to_server(&mut self) -> Result<()> {
        match &mut self.mode {
            DataConnectionMode::Passive {
                stream,
                server_host,
                server_port,
            } => {
                info!("Passive mode: Connecting to server at {server_host}:{server_port}");

                let server_addr = format!("{server_host}:{server_port}");
                debug!(
                    "Attempting connection to {} with {}s timeout",
                    server_addr,
                    self.timeout.as_secs()
                );
                let parsed_addr = server_addr.parse().map_err(|_| {
                    RaxFtpClientError::DataConnectionFailed("Invalid server address".to_string())
                })?;

                match TcpStream::connect_timeout(&parsed_addr, self.timeout) {
                    Ok(tcp_stream) => {
                        info!("Successfully connected to server at {parsed_addr}");
                        *stream = Some(tcp_stream);
                        Ok(())
                    }
                    Err(e) => {
                        warn!("Connection attempt failed to {parsed_addr}: {e}");
                        error!("Failed to establish passive data connection: {e}");
                        Err(RaxFtpClientError::DataConnectionFailed(format!(
                            "Failed to connect to server: {e}"
                        )))
                    }
                }
            }
            DataConnectionMode::Active { listener, stream } => {
                info!("Active mode: Waiting for server connection");
                if let Some(listener) = listener.as_ref() {
                    debug!(
                        "Waiting for server connection on listener (timeout: {}s)",
                        self.timeout.as_secs()
                    );
                    // Accept incoming connection from server
                    match listener.accept() {
                        Ok((tcp_stream, server_addr)) => {
                            info!("Server successfully connected from: {server_addr}");
                            *stream = Some(tcp_stream);
                            Ok(())
                        }
                        Err(e) => {
                            warn!("Server connection attempt failed: {e}");
                            error!("Failed to accept connection in active mode: {e}");
                            Err(RaxFtpClientError::DataConnectionFailed(format!(
                                "Failed to accept connection: {e}"
                            )))
                        }
                    }
                } else {
                    error!("No listener available for active mode");
                    Err(RaxFtpClientError::DataConnectionFailed(
                        "No listener available for active mode".to_string(),
                    ))
                }
            }
        }
    }

    /// Send data over the connection
    pub fn send_data(&mut self, data: &[u8]) -> Result<usize> {
        let stream = match &mut self.mode {
            DataConnectionMode::Active { stream, .. } => stream.as_mut().ok_or_else(|| {
                RaxFtpClientError::DataConnectionFailed(
                    "No active data connection established".to_string(),
                )
            })?,
            DataConnectionMode::Passive { stream, .. } => stream.as_mut().ok_or_else(|| {
                RaxFtpClientError::DataConnectionFailed(
                    "No passive data connection established".to_string(),
                )
            })?,
        };

        match stream.write(data) {
            Ok(bytes_sent) => {
                debug!("Sent {bytes_sent} bytes over data connection");
                Ok(bytes_sent)
            }
            Err(e) => {
                error!("Failed to send data: {e}");
                Err(RaxFtpClientError::DataConnectionFailed(format!(
                    "Send failed: {e}"
                )))
            }
        }
    }

    /// Receive data from the connection
    pub fn receive_data(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let stream = match &mut self.mode {
            DataConnectionMode::Active { stream, .. } => stream.as_mut().ok_or_else(|| {
                RaxFtpClientError::DataConnectionFailed(
                    "No active data connection established".to_string(),
                )
            })?,
            DataConnectionMode::Passive { stream, .. } => stream.as_mut().ok_or_else(|| {
                RaxFtpClientError::DataConnectionFailed(
                    "No passive data connection established".to_string(),
                )
            })?,
        };

        match stream.read(buffer) {
            Ok(bytes_received) => {
                debug!("Received {bytes_received} bytes over data connection");
                Ok(bytes_received)
            }
            Err(e) => {
                error!("Failed to receive data: {e}");
                Err(RaxFtpClientError::DataConnectionFailed(format!(
                    "Receive failed: {e}"
                )))
            }
        }
    }

    /// Clean up the current connection and prepare for next transfer
    pub fn reset_connection(&mut self) -> Result<()> {
        match &mut self.mode {
            DataConnectionMode::Active { stream, listener } => {
                // Close the data stream
                if let Some(stream) = stream.take() {
                    stream.shutdown(std::net::Shutdown::Both)?;
                    info!("Active mode data stream closed");
                }

                // Set listener back to non-blocking for next accept
                if let Some(listener) = listener.as_ref() {
                    listener.set_nonblocking(false)?;
                    info!("Active mode listener set to blocking for next transfer");
                }
            }
            DataConnectionMode::Passive { stream, .. } => {
                // Just close the data stream
                if let Some(stream) = stream.take() {
                    stream.shutdown(std::net::Shutdown::Both)?;
                    info!("Passive mode data stream closed");
                }
            }
        }
        info!("Data connection reset complete - ready for next transfer");
        Ok(())
    }
}
