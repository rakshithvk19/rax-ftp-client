# RAX FTP Client

A Rust-based File Transfer Protocol (FTP) client designed to work with the [rax-ftp-server](https://github.com/rakshithvk19/rax-ftp-server). This client provides a complete FTP implementation with interactive CLI, file transfers, and flexible connection modes.

## Features

- **Interactive CLI Interface** - Real-time command execution with user-friendly prompts
- **File Operations** - Upload (STOR), download (RETR), and delete (DEL) files
- **Directory Management** - List contents (LIST), navigate directories (CWD), print working directory (PWD)
- **Dual Connection Modes** - Both active (PORT) and passive (PASV) data connections
- **Progress Tracking** - Real-time progress bars for file transfers with speed monitoring
- **Authentication** - Secure USER/PASS login with session management
- **Connection Management** - Automatic retry logic and graceful error handling
- **Configuration System** - TOML-based config with environment variable overrides
- **Docker Support** - Ready-to-use containerization with Docker Compose
- **Comprehensive Logging** - Detailed command and transfer logging

## Installation

### Prerequisites
- Rust (stable, edition 2021 or later)
- Cargo

### Build from Source
```bash
git clone <repository-url>
cd rax-ftp-client
cargo build --release
```

### Docker
```bash
docker compose up
```

## Usage

### Interactive Mode
```bash
cargo run --release
```

Once started, you'll enter an interactive session:
```
RAX FTP Client - Interactive Session
Connected to: RAX FTP Server
Current state: connected

rax-ftp-client (connected)> USER myuser
230 User logged in, proceed

rax-ftp-client (authenticated)> STOR myfile.txt
150 Opening data connection
Uploading 'myfile.txt' (1.2 KB)...
myfile.txt: [##################################################] 100.0% (1.2 KB) 125.3 KB/s
226 Transfer complete

rax-ftp-client (authenticated)> LIST
150 Opening data connection
Name                           Type     Size       Modified            
--------------------------------------------------------------------
myfile.txt                     File     1.2 KB     2025-01-09 14:30   
sample_dir/                    Dir      -          2025-01-09 12:15   
226 Directory send OK

rax-ftp-client (authenticated)> QUIT
221 Goodbye
```

## Supported Commands

| Command | Description | Example |
|---------|-------------|---------|
| `USER <username>` | Authenticate with username | `USER john` |
| `PASS <password>` | Provide password | `PASS secret` |
| `STOR <filename>` | Upload file to server | `STOR document.pdf` |
| `RETR <filename>` | Download file from server | `RETR report.txt` |
| `LIST` | List directory contents | `LIST` |
| `DEL <filename>` | Delete file on server | `DEL oldfile.txt` |
| `PWD` | Print working directory | `PWD` |
| `CWD <directory>` | Change working directory | `CWD /home/user` |
| `PORT <ip:port>` | Set active mode data connection | `PORT 127.0.0.1:2122` |
| `PASV` | Enter passive mode | `PASV` |
| `LOGOUT` | Log out current user | `LOGOUT` |
| `RAX` | Custom server command | `RAX` |
| `QUIT` | Disconnect and exit | `QUIT` |
| `HELP` | Show available commands | `HELP` |

## Configuration

The client uses `config.toml` for settings with environment variable overrides:

```toml
# Server connection
host = "127.0.0.1"
port = 2121
timeout = 5
max_retries = 3

# Client settings
local_directory = "./client_root"
data_port_start = 2122
data_port_end = 2130

# Optional display name
host_name = "My FTP Server"
```

### Environment Variables
Override any config value with `RAX_FTP_` prefixed environment variables:
```bash
export RAX_FTP_HOST=192.168.1.100
export RAX_FTP_PORT=21
export RAX_FTP_LOCAL_DIRECTORY=/home/user/ftp_files
```

## Docker Setup

### Using Docker Compose
```yaml
services:
  rax-ftp-client:
    build: .
    container_name: rax-ftp-client
    environment:
      - RUST_LOG=info
      - RAX_FTP_HOST=rax-ftp-server
      - RAX_FTP_LOCAL_DIRECTORY=/app/rax-ftp-client/client_root
    volumes:
      - ./client_root:/app/rax-ftp-client/client_root
    stdin_open: true
    tty: true
```

### Build and Run
```bash
docker compose up
```

## Connection Modes

### Active Mode (PORT)
Client creates a data connection listener and tells the server where to connect:
```
rax-ftp-client (authenticated)> PORT 127.0.0.1:2122
200 PORT command successful
```

### Passive Mode (PASV)
Server creates a data connection listener and tells the client where to connect:
```
rax-ftp-client (authenticated)> PASV
227 Entering Passive Mode (127,0,0,1,8,79)
```

## File Structure
```
src/
├── main.rs                 # Entry point
├── client.rs              # Main FTP client
├── config.rs              # Configuration management
├── error.rs               # Error handling
├── commands/              # Command parsing and handling
├── connection/            # Control and data connections
├── responses/             # Response parsing
├── terminal/              # CLI interface and display
└── transfer/              # File transfer operations
```

## License

MIT License. See [LICENSE](LICENSE) for details.

## Related Projects

- [rax-ftp-server](https://github.com/rakshithvk19/rax-ftp-server) - The companion FTP server