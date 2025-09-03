//! FTP Command definitions

/// FTP commands supported by the RAX FTP Client
#[derive(Debug, Clone, PartialEq)]
pub enum FtpCommand {
    /// QUIT - Quit connection
    Quit,

    /// USER - Username for authentication
    User(String),

    /// PASS - Password for authentication  
    Pass(String),

    /// LOGOUT - Log out current user
    Logout,

    /// STOR - Store/upload file to server
    Stor(String),

    /// RETR - Retrieve/download file from server
    Retr(String),

    /// DEL - Delete file on server
    Del(String),

    /// LIST - List directory contents
    List,

    /// PWD - Print working directory
    Pwd,

    /// CWD - Change working directory
    Cwd(String),

    /// PORT - Active mode data port specification
    Port(String),

    /// PASV - Enter passive mode
    Pasv,

    /// RAX - Custom server command
    Rax,

    /// HELP - Show available commands (client-side only)
    Help,

    /// Unknown or unsupported command
    Unknown(String),
}

impl FtpCommand {
    /// Convert command to FTP protocol string
    pub fn to_ftp_string(&self) -> String {
        match self {
            FtpCommand::Quit => "QUIT".to_string(),
            FtpCommand::User(username) => format!("USER {username}"),
            FtpCommand::Pass(password) => format!("PASS {password}"),
            FtpCommand::Logout => "LOGOUT".to_string(),
            FtpCommand::Stor(filename) => format!("STOR {filename}"),
            FtpCommand::Retr(filename) => format!("RETR {filename}"),
            FtpCommand::Del(filename) => format!("DEL {filename}"),
            FtpCommand::List => "LIST".to_string(),
            FtpCommand::Pwd => "PWD".to_string(),
            FtpCommand::Cwd(path) => format!("CWD {path}"),
            FtpCommand::Port(addr) => format!("PORT {addr}"),
            FtpCommand::Pasv => "PASV".to_string(),
            FtpCommand::Rax => "RAX".to_string(),
            FtpCommand::Help => "HELP".to_string(),
            FtpCommand::Unknown(cmd) => cmd.clone(),
        }
    }

    /// Check if command is client-side only
    pub fn is_client_only(&self) -> bool {
        matches!(self, FtpCommand::Help)
    }
}

impl std::fmt::Display for FtpCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FtpCommand::Quit => write!(f, "QUIT"),
            FtpCommand::User(username) => write!(f, "USER {username}"),
            FtpCommand::Pass(_) => write!(f, "PASS [hidden]"),
            FtpCommand::Logout => write!(f, "LOGOUT"),
            FtpCommand::Stor(filename) => write!(f, "STOR {filename}"),
            FtpCommand::Retr(filename) => write!(f, "RETR {filename}"),
            FtpCommand::Del(filename) => write!(f, "DEL {filename}"),
            FtpCommand::List => write!(f, "LIST"),
            FtpCommand::Pwd => write!(f, "PWD"),
            FtpCommand::Cwd(path) => write!(f, "CWD {path}"),
            FtpCommand::Port(addr) => write!(f, "PORT {addr}"),
            FtpCommand::Pasv => write!(f, "PASV"),
            FtpCommand::Rax => write!(f, "RAX"),
            FtpCommand::Help => write!(f, "HELP"),
            FtpCommand::Unknown(cmd) => write!(f, "UNKNOWN({cmd})"),
        }
    }
}
