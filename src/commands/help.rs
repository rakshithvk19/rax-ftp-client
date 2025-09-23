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
  MKD <directory>   - Create directory on server
  RMD <directory>   - Remove directory on server
  HELP              - Show this help message

Data Transfer Information:
  Set data connection mode using PASV (passive) or PORT (active) commands
  Data connection mode persists for entire session until changed
  Data commands (LIST, STOR, RETR) require connection mode to be set first
  Use PORT command to switch to active mode
  Use PASV command to switch to passive mode

Current server: [SERVER_PLACEHOLDER]
Current state: [STATE_PLACEHOLDER]
Local directory: [LOCAL_DIR_PLACEHOLDER]
Data port range: [PORT_RANGE_PLACEHOLDER]",
    )
}
