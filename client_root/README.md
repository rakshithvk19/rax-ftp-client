# Client Root Directory

This directory serves as the working directory for the RAX FTP Client.

## Purpose

- **File Operations**: All file uploads (`STOR`) and downloads (`RETR`) operate within this directory
- **Visual Verification**: Users evaluating this project can easily see transferred files
- **Isolation**: Keeps FTP operations separate from project source code

## Directory Structure

```
client_root/
├── README.md          # This file
├── uploads/           # Files ready to be uploaded to server
├── downloads/         # Files downloaded from server  
├── samples/           # Sample files for testing
└── (working files)    # Files transferred during FTP operations
```

## Usage Examples

### Uploading Files
1. Place files in this directory (or subdirectories)
2. Use the FTP client: `STOR filename.txt`
3. File will be transferred to the server

### Downloading Files  
1. Use the FTP client: `RETR filename.txt`
2. File will be downloaded to this directory

### Testing the Client
- Use files from the `samples/` directory to test uploads
- Verify downloaded files appear in `downloads/`

## Configuration

This directory location is configured in `config/client_config.toml`:

```toml
[client]
local_directory = "./client_root"
```

It can be overridden using the environment variable:
```bash
export RAX_FTP_LOCAL_DIR="/path/to/your/directory"
```
