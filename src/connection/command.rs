//! Command connection management for RAX FTP Client
//!
//! Handles TCP connection for FTP command channel (port 2121).

use log::{debug, error, info, warn};
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::config::ClientConfig;
use crate::error::{RaxFtpClientError, Result};

/// Manages the FTP command connection (main control channel)
pub struct CommandConnection {
    stream: Option<TcpStream>,
    host: String,
    port: u16,
    timeout: u64,
    max_retries: u32,
}

impl CommandConnection {
    /// Create a new command connection with the given configuration
    pub fn new(config: &ClientConfig) -> Self {
        info!(
            "Creating command connection for {}:{}",
            config.host, config.port
        );

        Self {
            stream: None,
            host: config.host.clone(),
            port: config.port,
            timeout: config.timeout,
            max_retries: config.max_retries,
        }
    }

    /// Connect to the FTP server with retry logic
    pub fn connect_with_retries(&mut self) -> Result<String> {
        info!("Attempting to connect to {}:{}", self.host, self.port);

        for attempt in 1..=self.max_retries {
            match self.connect() {
                Ok(greeting) => {
                    info!(
                        "Successfully connected to FTP server on attempt {}",
                        attempt
                    );
                    return Ok(greeting);
                }
                Err(e) => {
                    error!("Connection attempt {} failed: {}", attempt, e);

                    if attempt < self.max_retries {
                        let wait_time = attempt as u64; // Simple backoff: 1s, 2s, 3s
                        warn!("Retrying in {} seconds...", wait_time);
                        std::thread::sleep(Duration::from_secs(wait_time));
                    }
                }
            }
        }

        Err(RaxFtpClientError::ConnectionTimeout(format!(
            "Failed to connect after {} attempts",
            self.max_retries
        )))
    }
    /// Connect to the FTP server (single attempt)
    fn connect(&mut self) -> Result<String> {
        // Create TCP connection with timeout
        let stream = TcpStream::connect_timeout(
            &format!("{}:{}", self.host, self.port)
                .parse()
                .map_err(|_| RaxFtpClientError::InvalidHost("Invalid host format".to_string()))?,
            Duration::from_secs(self.timeout),
        )
        .map_err(|e| match e.kind() {
            io::ErrorKind::TimedOut => {
                RaxFtpClientError::ConnectionTimeout("Connection timed out".to_string())
            }
            io::ErrorKind::ConnectionRefused => RaxFtpClientError::ConnectionRefused(format!(
                "Connection refused to {}:{}",
                self.host, self.port
            )),
            _ => RaxFtpClientError::Io(e),
        })?;

        // Set timeouts for read/write operations
        stream
            .set_read_timeout(Some(Duration::from_secs(self.timeout)))
            .map_err(RaxFtpClientError::Io)?;
        stream
            .set_write_timeout(Some(Duration::from_secs(self.timeout)))
            .map_err(RaxFtpClientError::Io)?;

        self.stream = Some(stream);
        info!("Connected to FTP server at {}:{}", self.host, self.port);

        // Read the server greeting immediately
        let greeting = self.read_response()?;
        info!("Server greeting: {}", greeting.trim());

        Ok(greeting)
    }
    /// Check if the connection is active
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Send raw bytes over the command connection
    pub fn send_bytes(&mut self, data: &[u8]) -> Result<()> {
        if self.stream.is_none() {
            return Err(RaxFtpClientError::ConnectionLost(
                "Not connected".to_string(),
            ));
        }

        // Perform I/O operations first
        let result = {
            let stream = self.stream.as_mut().unwrap();
            stream.write_all(data).and_then(|_| stream.flush())
        };

        // Handle the result after borrowing is done
        match result {
            Ok(()) => {
                debug!("Sent {} bytes", data.len());
                Ok(())
            }
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionAborted => {
                        self.stream = None; // Now we can safely modify self.stream
                        Err(RaxFtpClientError::ConnectionLost(
                            "Connection lost while sending".to_string(),
                        ))
                    }
                    _ => Err(RaxFtpClientError::Io(e)),
                }
            }
        }
    }

    /// Read a line from the command connection (for FTP responses)
    pub fn read_line(&mut self) -> Result<String> {
        if self.stream.is_none() {
            return Err(RaxFtpClientError::NotConnected("Not connected".to_string()));
        }

        let result = {
            let stream = self.stream.as_mut().unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).map(|_| line)
        };

        match result {
            Ok(line) => {
                debug!("Read line: {}", line.trim());
                Ok(line)
            }
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof | io::ErrorKind::ConnectionAborted => {
                        self.stream = None; // Now we can safely modify self.stream
                        Err(RaxFtpClientError::ConnectionLost(
                            "Connection lost while reading".to_string(),
                        ))
                    }
                    _ => Err(RaxFtpClientError::Io(e)),
                }
            }
        }
    }

    /// Send an FTP command (adds CRLF automatically)
    pub fn send_command(&mut self, command: &str) -> Result<()> {
        let formatted_command = if command.ends_with("\r\n") {
            command.to_string()
        } else {
            format!("{}\r\n", command)
        };

        debug!("Sending command: {}", command);
        self.send_bytes(formatted_command.as_bytes())
    }

    /// Read an FTP response (handles multi-line responses)
    pub fn read_response(&mut self) -> Result<String> {
        let mut response = String::new();
        let mut first_line = true;
        let mut expected_code = None;

        loop {
            let line = self.read_line()?;
            response.push_str(&line);

            if first_line {
                // Parse the response code from first line
                if line.len() >= 4 {
                    let code_part = &line[0..3];
                    let separator = line.chars().nth(3).unwrap_or(' ');

                    if separator == ' ' {
                        // Single line response
                        break;
                    } else if separator == '-' {
                        // Multi-line response
                        expected_code = Some(code_part.to_string());
                    }
                }
                first_line = false;
            } else if let Some(ref code) = expected_code {
                // Check if this is the end of multi-line response
                if line.len() >= 4 && line.starts_with(code) && line.chars().nth(3) == Some(' ') {
                    break;
                }
            }
        }

        debug!("Received response: {}", response.trim());
        Ok(response)
    }

    /// Disconnect from the server
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            info!("Disconnecting from FTP server");
            stream
                .shutdown(std::net::Shutdown::Both)
                .map_err(RaxFtpClientError::Io)?;
        }
        info!("Disconnected from server");
        Ok(())
    }

    /// Get connection info for display
    pub fn connection_info(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Drop for CommandConnection {
    fn drop(&mut self) {
        if self.stream.is_some() {
            let _ = self.disconnect();
        }
    }
}
