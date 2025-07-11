//! Help text and documentation for FTP commands

/// Returns the help text for all FTP commands
pub fn get_help_text() -> String {
    String::from(
        "Available commands:
  USER <username>   - Authenticate with username
  PASS <password>   - Provide password
  STOR <filename>   - Upload file to server
  RETR <filename>   - Download file from server
  LIST              - List directory contents
  PORT <ip:port>    - Set data connection port (active mode)
  PASV              - Enter passive mode
  PWD               - Print working directory
  CWD <directory>   - Change working directory
  DEL <filename>    - Delete file on server
  LOGOUT            - Log out current user
  RAX               - Custom server command
  QUIT              - Disconnect and exit
  HELP              - Show this help message

Data Transfer Information:
  Default mode: Passive (PASV)
  Data commands (LIST, STOR, RETR) automatically establish data connections
  Use PORT command to switch to active mode
  Use PASV command to switch to passive mode

Current server: [SERVER_PLACEHOLDER]
Current state: [STATE_PLACEHOLDER]
Local directory: [LOCAL_DIR_PLACEHOLDER]
Data port range: [PORT_RANGE_PLACEHOLDER]",
    )
}
