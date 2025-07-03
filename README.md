# RAX FTP Client

A Rust-based File Transfer Protocol (FTP) client designed to work with the [rax-ftp-server](https://github.com/rakshithvk19/rax-ftp-server). This client implements core FTP protocol features including authentication, file uploads, and active mode data connections.

## Features

- **Authentication**: USER/PASS command support with credential management
- **File Upload**: STOR command implementation for transferring files to server
- **Active Mode**: PORT command for client-specified data connections
- **Connection Management**: Persistent control connection with proper cleanup
- **Error Handling**: Comprehensive error recovery and retry logic
- **Logging**: Detailed command and transfer logging
- **CLI Interface**: Interactive and batch mode operations
- **Progress Tracking**: Real-time upload progress monitoring

## Architecture

The client follows a modular architecture with clear separation of concerns:

- **Session Manager**: Handles connection state and lifecycle
- **Command System**: Parses, validates, and dispatches FTP commands
- **Connection Management**: Manages control (port 2121) and data connections
- **File Transfer System**: Handles uploads with progress tracking
- **Configuration System**: Flexible configuration management

## Prerequisites

- Rust (stable, edition 2021 or later)
- Cargo
- [rax-ftp-server](https://github.com/rakshithvk19/rax-ftp-server) running for testing

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd rax-ftp-client
```

2. Build the project:
```bash
cargo build --release
```

3. Run the client:
```bash
cargo run --release
```

## Usage

### Interactive Mode

```bash
# Start interactive session
./target/release/ftp-client --host 127.0.0.1 --port 2121

# Interactive commands
ftp> USER user
230 Login successful
ftp> PASS pass
230 Authentication successful
ftp> STOR test.txt
150 Opening data connection
226 Transfer complete
ftp> QUIT
221 Goodbye
```

### Command Line Mode

```bash
# Single file upload
./target/release/ftp-client --host 127.0.0.1 --port 2121 --upload test.txt

# With credentials
./target/release/ftp-client --host 127.0.0.1 --user user --pass pass --upload test.txt

# Batch script
./target/release/ftp-client --script upload_script.ftp
```

### Configuration File

Create `ftp_client.toml`:

```toml
[connection]
host = "127.0.0.1"
port = 2121
timeout = 30
data_port_range = [2122, 2130]
max_retries = 3

[auth]
username = "user"
password = "pass"

[logging]
level = "info"
command_log = true
transfer_log = true
```

## API Usage

```rust
use ftp_client::{FtpClient, ClientConfig};

// Initialize client
let config = ClientConfig::default();
let mut client = FtpClient::new(config)?;

// Connect and authenticate
client.connect("127.0.0.1", 2121)?;
client.login("user", "pass")?;

// Upload file
client.upload_file("local_file.txt", "remote_file.txt")?;

// Disconnect
client.quit()?;
```

## Supported Commands

| Command | Description | Status |
|---------|-------------|---------|
| USER | Authenticate user | ✅ |
| PASS | Provide password | ✅ |
| PORT | Set data connection port | ✅ |
| STOR | Upload file to server | ✅ |
| QUIT | Disconnect from server | ✅ |

## Testing

### Unit Tests
```bash
cargo test
```

### Integration Tests
```bash
# Start rax-ftp-server first
cargo test --test integration
```

### Manual Testing

1. Start the rax-ftp-server:
```bash
cd rax-ftp-server
cargo run --release
```

2. Test file upload:
```bash
echo "Hello, FTP!" > test.txt
./target/release/ftp-client --host 127.0.0.1 --upload test.txt
```

3. Verify uploaded file in server directory.

## Configuration

### Command Line Arguments

```bash
ftp-client [OPTIONS] [COMMAND]

OPTIONS:
    -h, --host <HOST>           Server hostname [default: 127.0.0.1]
    -p, --port <PORT>           Server port [default: 2121]
    -u, --user <USERNAME>       Username for authentication
    -P, --pass <PASSWORD>       Password for authentication
    -t, --timeout <SECONDS>     Connection timeout [default: 30]
    -c, --config <FILE>         Configuration file path
    -v, --verbose               Enable verbose logging
    -q, --quiet                 Suppress output

COMMANDS:
    --upload <FILE>             Upload file to server
    --script <FILE>             Execute batch script
    --interactive               Start interactive session
```

### Environment Variables

```bash
export FTP_HOST=127.0.0.1
export FTP_PORT=2121
export FTP_USER=user
export FTP_PASS=pass
export FTP_TIMEOUT=30
```

## Error Handling

The client handles various error scenarios:

- **Connection Errors**: Network timeouts, connection refused
- **Authentication Errors**: Invalid credentials, server rejection
- **Transfer Errors**: File not found, permission denied, network issues
- **Protocol Errors**: Invalid responses, unexpected server behavior

## Logging

Enable detailed logging:

```bash
RUST_LOG=debug ./target/release/ftp-client --verbose
```

Log categories:
- **Command Log**: All FTP commands sent/received
- **Transfer Log**: File transfer operations and progress
- **Error Log**: Error conditions and recovery attempts
- **Performance Log**: Transfer speeds and timing

## Project Structure

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

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Commit changes: `git commit -am 'Add new feature'`
4. Push to branch: `git push origin feature/new-feature`
5. Submit a pull request

## Development

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests (requires server)
cargo test --test integration

# All tests
cargo test --all
```

### Code Coverage
```bash
cargo tarpaulin --out html
```

### Linting
```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## Roadmap

### Phase 1 (Current)
- [x] Basic client architecture
- [x] Authentication (USER/PASS)
- [x] File upload (STOR)
- [x] Active mode (PORT)
- [x] CLI interface

### Phase 2 (Planned)
- [ ] Passive mode (PASV)
- [ ] File download (RETR)
- [ ] Directory listing (LIST)
- [ ] Directory navigation (CWD, PWD)

### Phase 3 (Future)
- [ ] TLS/SSL support
- [ ] GUI interface
- [ ] Transfer resume
- [ ] Async/await implementation

## Compatibility

This client is specifically designed for the [rax-ftp-server](https://github.com/rakshithvk19/rax-ftp-server) but follows RFC 959 standards for compatibility with other FTP servers.

**Tested with:**
- rax-ftp-server v1.0.0
- Standard FTP servers (limited command set)

## License

MIT License. See [LICENSE](LICENSE) for details.

## Related Projects

- [rax-ftp-server](https://github.com/rakshithvk19/rax-ftp-server) - The companion FTP server
- [FTP RFC 959](https://tools.ietf.org/html/rfc959) - FTP Protocol Specification

## Support

For issues, questions, or contributions:
- Open an issue on GitHub
- Check the [documentation](docs/)
- Review the [architecture guide](docs/architecture.md)

---

**Note**: This is a learning project demonstrating Rust networking, protocol implementation, and systems programming. Production use should include additional security considerations.