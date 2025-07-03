# FTP Client Architecture Design

## Overview
This document outlines the architecture for an FTP client designed to work with the rax-ftp-server. The client implements core FTP protocol features including authentication, file uploads, and active mode data connections.

## System Architecture

### 1. Core Components

#### 1.1 User Interface Layer
- **CLI Interface**: Command-line interface for user interaction
- **Interactive Mode**: Real-time command execution with prompt
- **Batch Mode**: Script-based command execution
- **Configuration Management**: Load settings from config files

#### 1.2 Session Manager
- **Connection State**: Track authentication status and session state
- **Control Connection**: Manage persistent TCP connection to server
- **Data Connection Factory**: Create data connections for transfers
- **Session Cleanup**: Proper connection teardown and resource cleanup

#### 1.3 Command System
- **Command Parser**: Parse user input into FTP commands
- **Command Dispatcher**: Route commands to appropriate handlers
- **Response Parser**: Parse server responses and extract status codes
- **Command Validation**: Validate commands before sending

### 2. Protocol Implementation

#### 2.1 Supported Commands (Based on Server)
```
USER <username>     - Authenticate user
PASS <password>     - Provide password
PORT <host,port>    - Set data connection port (active mode)
STOR <filename>     - Upload file to server
QUIT                - Disconnect from server
```

#### 2.2 Connection Management
- **Control Connection**: TCP socket to 127.0.0.1:2121
- **Data Connection**: TCP socket for file transfers (active mode)
- **Port Calculation**: Convert IP:Port to PORT command format
- **Connection Pooling**: Reuse connections when possible

### 3. File Transfer System

#### 3.1 Upload Handler
- **File Validation**: Check file existence and permissions
- **Progress Tracking**: Monitor upload progress
- **Error Recovery**: Handle transfer interruptions
- **Checksum Verification**: Optional file integrity checks

#### 3.2 Data Connection (Active Mode)
- **Port Negotiation**: Communicate data port to server
- **Data Socket**: Create listening socket for server connection
- **Transfer Protocol**: Handle binary/ASCII transfer modes
- **Connection Cleanup**: Properly close data connections

### 4. Error Handling

#### 4.1 Connection Errors
- **Network Timeouts**: Handle connection timeouts
- **Connection Refused**: Server unavailable scenarios
- **Socket Errors**: TCP-level error handling
- **Retry Logic**: Automatic reconnection attempts

#### 4.2 Protocol Errors
- **Invalid Commands**: Handle unsupported commands
- **Authentication Failures**: Invalid credentials
- **Transfer Errors**: File transfer failures
- **Server Errors**: Handle server-side error responses

### 5. Configuration System

#### 5.1 Connection Settings
```rust
pub struct ClientConfig {
    pub host: String,           // Default: 127.0.0.1
    pub port: u16,              // Default: 2121
    pub timeout: u64,           // Connection timeout in seconds
    pub data_port_range: (u16, u16), // Data port range
    pub max_retries: u32,       // Retry attempts
    pub log_level: String,      // Logging level
}
```

#### 5.2 User Credentials
```rust
pub struct Credentials {
    pub username: String,
    pub password: String,
}
```

### 6. Logging and Monitoring

#### 6.1 Logging Categories
- **Command Log**: All FTP commands sent/received
- **Transfer Log**: File transfer operations
- **Error Log**: Error conditions and recovery
- **Performance Log**: Transfer speeds and timing

#### 6.2 Monitoring
- **Connection Status**: Track connection health
- **Transfer Progress**: Real-time upload progress
- **Performance Metrics**: Transfer speeds and statistics

## Implementation Structure

### 7. Module Organization

```
src/
├── main.rs                 // Entry point and CLI
├── client.rs              // Main client implementation
├── config.rs              // Configuration management
├── connection/
│   ├── mod.rs            // Connection management
│   ├── control.rs        // Control connection
│   └── data.rs           // Data connection
├── commands/
│   ├── mod.rs            // Command system
│   ├── parser.rs         // Command parsing
│   ├── dispatcher.rs     // Command routing
│   └── handlers.rs       // Command handlers
├── transfer/
│   ├── mod.rs            // File transfer
│   ├── upload.rs         // Upload implementation
│   └── progress.rs       // Progress tracking
├── protocol/
│   ├── mod.rs            // Protocol implementation
│   ├── responses.rs      // Response parsing
│   └── status_codes.rs   // FTP status codes
├── error.rs              // Error types and handling
├── logging.rs            // Logging configuration
└── utils.rs              // Utility functions
```

### 8. Key Data Structures

#### 8.1 Client State
```rust
pub struct FtpClient {
    config: ClientConfig,
    control_connection: Option<TcpStream>,
    session_state: SessionState,
    logger: Logger,
}

pub enum SessionState {
    Disconnected,
    Connected,
    Authenticated,
}
```

#### 8.2 Command Types
```rust
pub enum FtpCommand {
    User(String),
    Pass(String),
    Port(String, u16),
    Stor(String),
    Quit,
}
```

#### 8.3 Response Types
```rust
pub struct FtpResponse {
    pub code: u16,
    pub message: String,
    pub is_multiline: bool,
}
```

## Usage Examples

### 9. Basic Usage Flow

1. **Initialize Client**
   ```rust
   let client = FtpClient::new(config)?;
   ```

2. **Connect to Server**
   ```rust
   client.connect("127.0.0.1", 2121)?;
   ```

3. **Authenticate**
   ```rust
   client.login("user", "pass")?;
   ```

4. **Upload File**
   ```rust
   client.upload_file("local_file.txt", "remote_file.txt")?;
   ```

5. **Disconnect**
   ```rust
   client.quit()?;
   ```

### 10. CLI Interface Examples

```bash
# Interactive mode
$ ftp-client --host 127.0.0.1 --port 2121
ftp> USER user
ftp> PASS pass
ftp> STOR test.txt
ftp> QUIT

# Batch mode
$ ftp-client --script upload_script.ftp

# Single command
$ ftp-client --host 127.0.0.1 --upload test.txt
```

## Testing Strategy

### 11. Test Categories

#### 11.1 Unit Tests
- Command parsing and validation
- Response parsing
- Configuration loading
- Error handling

#### 11.2 Integration Tests
- Full connection workflow
- Authentication flow
- File upload process
- Error recovery scenarios

#### 11.3 End-to-End Tests
- Complete client-server interaction
- Multiple concurrent connections
- Large file transfers
- Connection recovery

### 12. Performance Considerations

#### 12.1 Optimization Areas
- Connection pooling for multiple operations
- Buffered file transfers
- Asynchronous operations (future enhancement)
- Memory-efficient large file handling

#### 12.2 Scalability
- Support for multiple concurrent transfers
- Efficient memory usage for large files
- Configurable timeouts and retry logic

## Future Enhancements

### 13. Planned Features
- **Passive Mode**: PASV command support
- **Download Support**: RETR command implementation
- **Directory Operations**: LIST, CWD, PWD commands
- **TLS/SSL Support**: Secure FTP connections
- **GUI Interface**: Graphical user interface
- **Transfer Resume**: Resume interrupted transfers

### 14. Architecture Extensions
- **Plugin System**: Extensible command handlers
- **Protocol Abstraction**: Support for SFTP/FTPS
- **Async/Await**: Tokio-based async implementation
- **Configuration API**: Runtime configuration changes

This architecture provides a robust foundation for an FTP client that integrates seamlessly with your rax-ftp-server while being extensible for future enhancements.