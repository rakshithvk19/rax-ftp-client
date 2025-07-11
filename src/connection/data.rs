//! Data connection management for FTP transfers

use log::{debug, error, info};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use crate::config::ClientConfig;
use crate::error::{RaxFtpClientError, Result};

/// Data connection modes
#[derive(Debug, Clone, PartialEq)]
pub enum DataMode {
    Active,
    Passive,
}

/// Data connection information
#[derive(Debug, Clone)]
pub struct DataConnectionInfo {
    pub mode: DataMode,
    pub host: String,
    pub port: u16,
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
    /// Establish data connection based on current connection info
    pub fn establish_from_info(
        connection_info: Option<&DataConnectionInfo>,
        config: &ClientConfig,
    ) -> Result<Self> {
        match connection_info {
            Some(info) => match info.mode {
                DataMode::Active => Self::establish_active(config),
                DataMode::Passive => Self::establish_passive(&info.host, info.port),
            },
            None => {
                // Default to passive mode if no connection info
                Err(RaxFtpClientError::DataConnectionFailed(
                    "No data connection mode set. Use PASV or PORT command first".to_string(),
                ))
            }
        }
    }

    /// Establish active data connection
    pub fn establish_active(config: &ClientConfig) -> Result<Self> {
        Self::new_port_mode(config)
    }

    /// Establish passive data connection WITHOUT connecting yet
    pub fn establish_passive(server_host: &str, server_port: u16) -> Result<Self> {
        Self::new_passive_mode(server_host, server_port)
    }

    /// Create a new data connection for PORT mode (Active)
    pub fn new_port_mode(config: &ClientConfig) -> Result<Self> {
        // Try to bind to an available port in the configured range
        let (start_port, end_port) = config.get_data_port_range();

        for port in start_port..=end_port {
            let addr = format!("0.0.0.0:{}", port);

            match TcpListener::bind(&addr) {
                Ok(listener) => {
                    let local_addr = listener.local_addr()?;
                    info!("Created data connection listener on {}", local_addr);

                    // Set listener to non-blocking
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

        Err(RaxFtpClientError::DataConnectionFailed(format!(
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
            timeout: Duration::from_secs(5), // Default timeout for passive mode
        })
    }

    /// Get the PORT command string for the server (Active mode only)
    pub fn get_port_command(&self) -> Result<String> {
        match &self.mode {
            DataConnectionMode::Active { local_addr, .. } => {
                match local_addr {
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
            DataConnectionMode::Passive { .. } => Err(RaxFtpClientError::DataConnectionFailed(
                "PORT command not applicable in passive mode".to_string(),
            )),
        }
    }

    /// Wait for server to connect (Active mode only)
    pub fn accept_connection(&mut self) -> Result<()> {
        match &mut self.mode {
            DataConnectionMode::Active {
                listener, stream, ..
            } => {
                match listener {
                    Some(listener) => {
                        info!("Waiting for server to connect to data port...");

                        // Set timeout for accept
                        listener.set_nonblocking(false)?;

                        match listener.accept() {
                            Ok((tcp_stream, peer_addr)) => {
                                info!("Server connected from {} for data transfer", peer_addr);

                                // Set timeouts on the data stream
                                tcp_stream.set_read_timeout(Some(self.timeout))?;
                                tcp_stream.set_write_timeout(Some(self.timeout))?;

                                *stream = Some(tcp_stream);
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
            DataConnectionMode::Passive { .. } => Err(RaxFtpClientError::DataConnectionFailed(
                "accept_connection not applicable in passive mode. Use connect_to_server instead."
                    .to_string(),
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
                match TcpStream::connect_timeout(
                    &server_addr.parse().map_err(|_| {
                        RaxFtpClientError::DataConnectionFailed(
                            "Invalid server address".to_string(),
                        )
                    })?,
                    self.timeout,
                ) {
                    Ok(tcp_stream) => {
                        info!("Connected to server for data transfer");

                        // Set timeouts on the data stream
                        tcp_stream.set_read_timeout(Some(self.timeout))?;
                        tcp_stream.set_write_timeout(Some(self.timeout))?;

                        *stream = Some(tcp_stream);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to connect to server: {}", e);
                        Err(RaxFtpClientError::DataConnectionFailed(format!(
                            "Failed to connect to server: {}",
                            e
                        )))
                    }
                }
            }
            DataConnectionMode::Active { .. } => Err(RaxFtpClientError::DataConnectionFailed(
                "connect_to_server not applicable in active mode. Use accept_connection instead."
                    .to_string(),
            )),
        }
    }

    /// Send data over the connection
    pub fn send_data(&mut self, data: &[u8]) -> Result<usize> {
        let stream = match &mut self.mode {
            DataConnectionMode::Active { stream, .. } => stream,
            DataConnectionMode::Passive { stream, .. } => stream,
        };

        match stream {
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
        let stream = match &mut self.mode {
            DataConnectionMode::Active { stream, .. } => stream,
            DataConnectionMode::Passive { stream, .. } => stream,
        };

        match stream {
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
        match &mut self.mode {
            DataConnectionMode::Active {
                stream, listener, ..
            } => {
                if let Some(stream) = stream.take() {
                    stream.shutdown(std::net::Shutdown::Both)?;
                    info!("Data connection closed");
                }

                if let Some(listener) = listener.take() {
                    drop(listener);
                    debug!("Data listener closed");
                }
            }
            DataConnectionMode::Passive { stream, .. } => {
                if let Some(stream) = stream.take() {
                    stream.shutdown(std::net::Shutdown::Both)?;
                    info!("Data connection closed");
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
