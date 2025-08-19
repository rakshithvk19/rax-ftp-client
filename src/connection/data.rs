//! Data connection management for FTP transfers

use log::{debug, error, info};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use crate::config::ClientConfig;
use crate::error::{RaxFtpClientError, Result};

// Constants
const DEFAULT_BIND_IP: &str = "0.0.0.0";
const LOCAL_TEST_IP: &str = "127.0.0.1";
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_ACTIVE_TIMEOUT_SECS: u64 = 5;

/// Data connection modes
#[derive(Debug, Clone, PartialEq)]
pub enum DataMode {
    Active,
    Passive,
}

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
        local_addr: Option<SocketAddr>,
    },
    Passive {
        stream: Option<TcpStream>,
        server_host: String,
        server_port: u16,
    },
}

impl DataConnection {
    /// Create error for data connection failures
    fn data_error(msg: &str) -> RaxFtpClientError {
        RaxFtpClientError::DataConnectionFailed(msg.to_string())
    }

    /// Create mode-specific error messages
    fn mode_error(
        operation: &str,
        current_mode: &str,
        suggested_method: &str,
    ) -> RaxFtpClientError {
        Self::data_error(&format!(
            "{} not applicable in {} mode. Use {} instead.",
            operation, current_mode, suggested_method
        ))
    }

    /// Get mutable reference to the stream regardless of mode
    fn get_stream_mut(&mut self) -> Option<&mut TcpStream> {
        match &mut self.mode {
            DataConnectionMode::Active { stream, .. } => stream.as_mut(),
            DataConnectionMode::Passive { stream, .. } => stream.as_mut(),
        }
    }

    /// Create a new data connection for PORT mode (Active) on a specific port
    pub fn new_port_mode_specific_port(port: u16) -> Result<Self> {
        let addr = format!("{}:{}", DEFAULT_BIND_IP, port);

        let listener = TcpListener::bind(&addr).map_err(|e| {
            error!("Failed to bind to specific port {}: {}", port, e);
            Self::data_error(&format!("Failed to bind to port {}: {}", port, e))
        })?;

        let local_addr = listener.local_addr()?;
        info!("Created data connection listener on specific port {}", port);

        // Set listener to blocking mode (corrected from original)
        listener.set_nonblocking(false)?;

        Ok(Self {
            mode: DataConnectionMode::Active {
                listener: Some(listener),
                stream: None,
                local_addr: Some(local_addr),
            },
            timeout: Duration::from_secs(DEFAULT_ACTIVE_TIMEOUT_SECS),
        })
    }

    /// Create a new data connection for PORT mode (Active)
    pub fn new_port_mode(config: &ClientConfig) -> Result<Self> {
        let (start_port, end_port) = config.get_data_port_range();

        for port in start_port..=end_port {
            let addr = format!("{}:{}", DEFAULT_BIND_IP, port);

            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    let local_addr = listener.local_addr()?;
                    info!("Created data connection listener on {}", local_addr);

                    listener.set_nonblocking(false)?;

                    return Ok(Self {
                        mode: DataConnectionMode::Active {
                            listener: Some(listener),
                            stream: None,
                            local_addr: Some(local_addr),
                        },
                        timeout: Duration::from_secs(config.timeout()),
                    });
                }
                Err(e) => {
                    debug!("Failed to bind to port {}: {}", port, e);
                    continue;
                }
            }
        }

        Err(Self::data_error(&format!(
            "No available ports in range {}-{}",
            start_port, end_port
        )))
    }

    /// Create a new data connection for PASV mode (Passive)
    pub fn new_passive_mode(server_host: &str, server_port: u16) -> Result<Self> {
        info!(
            "Creating passive data connection to {}:{}",
            server_host, server_port
        );

        Ok(Self {
            mode: DataConnectionMode::Passive {
                stream: None,
                server_host: server_host.to_string(),
                server_port,
            },
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        })
    }

    /// Get the PORT command string for the server (Active mode only)
    pub fn get_port_command(&self) -> Result<String> {
        match &self.mode {
            DataConnectionMode::Active { local_addr, .. } => {
                let addr = local_addr
                    .as_ref()
                    .ok_or_else(|| Self::data_error("No local address available"))?;

                // For PORT command, we need the external IP, not 0.0.0.0
                // For local testing, use 127.0.0.1
                Ok(format!("PORT {}:{}", LOCAL_TEST_IP, addr.port()))
            }
            DataConnectionMode::Passive { .. } => {
                Err(Self::mode_error("PORT command", "passive", "PASV command"))
            }
        }
    }

    /// Wait for server to connect (Active mode only)
    pub fn accept_connection(&mut self) -> Result<()> {
        match &mut self.mode {
            DataConnectionMode::Active {
                listener, stream, ..
            } => {
                let listener = listener
                    .as_ref()
                    .ok_or_else(|| Self::data_error("No listener available"))?;

                info!("Waiting for server to connect to data port...");
                listener.set_nonblocking(false)?;

                match listener.accept() {
                    Ok((tcp_stream, peer_addr)) => {
                        info!("Server connected from {} for data transfer", peer_addr);
                        *stream = Some(tcp_stream);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to accept data connection: {}", e);
                        Err(Self::data_error(&format!(
                            "Failed to accept connection: {}",
                            e
                        )))
                    }
                }
            }
            DataConnectionMode::Passive { .. } => Err(Self::mode_error(
                "accept_connection",
                "passive",
                "connect_to_server",
            )),
        }
    }

    /// Connect to server (Passive mode only)
    pub fn connect_to_server(&mut self) -> Result<()> {
        match &mut self.mode {
            DataConnectionMode::Passive {
                stream,
                server_host,
                server_port,
            } => {
                info!("Connecting to server at {}:{}", server_host, server_port);

                let server_addr = format!("{}:{}", server_host, server_port);
                let parsed_addr = server_addr
                    .parse()
                    .map_err(|_| Self::data_error("Invalid server address"))?;

                match TcpStream::connect_timeout(&parsed_addr, self.timeout) {
                    Ok(tcp_stream) => {
                        info!("Connected to server for data transfer");
                        *stream = Some(tcp_stream);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to connect to server: {}", e);
                        Err(Self::data_error(&format!(
                            "Failed to connect to server: {}",
                            e
                        )))
                    }
                }
            }
            DataConnectionMode::Active { .. } => Err(Self::mode_error(
                "connect_to_server",
                "active",
                "accept_connection",
            )),
        }
    }

    /// Send data over the connection
    pub fn send_data(&mut self, data: &[u8]) -> Result<usize> {
        let stream = self
            .get_stream_mut()
            .ok_or_else(|| Self::data_error("No data connection established"))?;

        stream
            .write(data)
            .map_err(|e| Self::data_error(&format!("Failed to send data: {}", e)))
    }

    /// Receive data from the connection
    pub fn receive_data(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let stream = self
            .get_stream_mut()
            .ok_or_else(|| Self::data_error("No data connection established"))?;

        stream
            .read(buffer)
            .map_err(|e| Self::data_error(&format!("Failed to receive data: {}", e)))
    }

    /// Close the data connection
    pub fn close(&mut self) -> Result<()> {
        match &mut self.mode {
            DataConnectionMode::Active {
                stream, listener, ..
            } => {
                if let Some(stream) = stream.take() {
                    stream.shutdown(std::net::Shutdown::Both)?;
                    info!("Active mode data connection closed");
                }

                if let Some(listener) = listener.take() {
                    drop(listener);
                    debug!("Active mode data listener closed");
                }
            }
            DataConnectionMode::Passive { stream, .. } => {
                if let Some(stream) = stream.take() {
                    stream.shutdown(std::net::Shutdown::Both)?;
                    info!("Passive mode data connection closed");
                }
            }
        }

        Ok(())
    }

    /// Check if connection is established
    pub fn is_connected(&self) -> bool {
        match &self.mode {
            DataConnectionMode::Active { stream, .. } => stream.is_some(),
            DataConnectionMode::Passive { stream, .. } => stream.is_some(),
        }
    }
}

impl Drop for DataConnection {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
