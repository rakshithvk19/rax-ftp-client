//! Data connection management for FTP transfers

use log::{debug, error, info};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use crate::config::ClientConfig;
use crate::error::{RaxFtpClientError, Result};

/// Manages FTP data connections for file transfers
pub struct DataConnection {
    listener: Option<TcpListener>,
    stream: Option<TcpStream>,
    local_addr: Option<SocketAddr>,
    timeout: Duration,
}

impl DataConnection {
    /// Create a new data connection for PORT mode
    pub fn new_port_mode(config: &ClientConfig) -> Result<Self> {
        // Try to bind to an available port in the configured range
        let (start_port, end_port) = config.data_port_range;

        for port in start_port..=end_port {
            let addr = format!("0.0.0.0:{}", port);

            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    let local_addr = listener.local_addr()?;
                    info!("Created data connection listener on {}", local_addr);

                    // Set listener to non-blocking
                    listener.set_nonblocking(false)?;

                    return Ok(Self {
                        listener: Some(listener),
                        stream: None,
                        local_addr: Some(local_addr),
                        timeout: Duration::from_secs(config.timeout),
                    });
                }
                Err(e) => {
                    debug!("Failed to bind to port {}: {}", port, e);
                    continue;
                }
            }
        }

        Err(RaxFtpClientError::DataConnectionFailed(format!(
            "No available ports in range {}-{}",
            start_port, end_port
        )))
    }

    /// Get the PORT command string for the server
    pub fn get_port_command(&self) -> Result<String> {
        match &self.local_addr {
            Some(addr) => {
                // For PORT command, we need the external IP, not 0.0.0.0
                // For simplicity, use 127.0.0.1 for local testing
                let ip = "127.0.0.1";
                let port = addr.port();

                Ok(format!("PORT {}:{}", ip, port))
            }
            None => Err(RaxFtpClientError::DataConnectionFailed(
                "No local address available".to_string(),
            )),
        }
    }

    /// Wait for server to connect (for PORT mode)
    pub fn accept_connection(&mut self) -> Result<()> {
        match &self.listener {
            Some(listener) => {
                info!("Waiting for server to connect to data port...");

                // Set timeout for accept
                listener.set_nonblocking(false)?;

                match listener.accept() {
                    Ok((stream, peer_addr)) => {
                        info!("Server connected from {} for data transfer", peer_addr);

                        // Set timeouts on the data stream
                        stream.set_read_timeout(Some(self.timeout))?;
                        stream.set_write_timeout(Some(self.timeout))?;

                        self.stream = Some(stream);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to accept data connection: {}", e);
                        Err(RaxFtpClientError::DataConnectionFailed(format!(
                            "Failed to accept connection: {}",
                            e
                        )))
                    }
                }
            }
            None => Err(RaxFtpClientError::DataConnectionFailed(
                "No listener available".to_string(),
            )),
        }
    }

    /// Send data over the connection
    pub fn send_data(&mut self, data: &[u8]) -> Result<usize> {
        match &mut self.stream {
            Some(stream) => stream.write(data).map_err(|e| {
                RaxFtpClientError::DataConnectionFailed(format!("Failed to send data: {}", e))
            }),
            None => Err(RaxFtpClientError::DataConnectionFailed(
                "No data connection established".to_string(),
            )),
        }
    }

    /// Receive data from the connection
    pub fn receive_data(&mut self, buffer: &mut [u8]) -> Result<usize> {
        match &mut self.stream {
            Some(stream) => stream.read(buffer).map_err(|e| {
                RaxFtpClientError::DataConnectionFailed(format!("Failed to receive data: {}", e))
            }),
            None => Err(RaxFtpClientError::DataConnectionFailed(
                "No data connection established".to_string(),
            )),
        }
    }

    /// Close the data connection
    pub fn close(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            stream.shutdown(std::net::Shutdown::Both)?;
            info!("Data connection closed");
        }

        if let Some(listener) = self.listener.take() {
            drop(listener);
            debug!("Data listener closed");
        }

        Ok(())
    }

    /// Check if connection is established
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}

impl Drop for DataConnection {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
