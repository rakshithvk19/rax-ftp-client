//! Command parsing functionality

use super::FtpCommand;

/// Parse user input into FtpCommand
pub fn parse_command(input: &str) -> FtpCommand {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return FtpCommand::Unknown("Empty command".to_string());
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let cmd = parts.next().unwrap_or("").to_uppercase();
    let arg = parts.next().unwrap_or("").trim();

    match cmd.as_str() {
        "QUIT" => FtpCommand::Quit,
        "USER" => {
            if arg.is_empty() {
                FtpCommand::Unknown("USER requires username".to_string())
            } else {
                FtpCommand::User(arg.to_string())
            }
        }
        "PASS" => {
            if arg.is_empty() {
                FtpCommand::Unknown("PASS requires password".to_string())
            } else {
                FtpCommand::Pass(arg.to_string())
            }
        }
        "STOR" => {
            if arg.is_empty() {
                FtpCommand::Unknown("STOR requires filename".to_string())
            } else {
                FtpCommand::Stor(arg.to_string())
            }
        }
        "RETR" => {
            if arg.is_empty() {
                FtpCommand::Unknown("RETR requires filename".to_string())
            } else {
                FtpCommand::Retr(arg.to_string())
            }
        }
        "DEL" => {
            if arg.is_empty() {
                FtpCommand::Unknown("DEL requires filename".to_string())
            } else {
                FtpCommand::Del(arg.to_string())
            }
        }
        "CWD" => {
            if arg.is_empty() {
                FtpCommand::Unknown("CWD requires directory".to_string())
            } else {
                FtpCommand::Cwd(arg.to_string())
            }
        }
        "PORT" => {
            if arg.is_empty() {
                FtpCommand::Unknown("PORT requires address".to_string())
            } else {
                FtpCommand::Port(arg.to_string())
            }
        }
        "MKD" => {
            if arg.is_empty() {
                FtpCommand::Unknown("MKD requires directory name".to_string())
            } else {
                FtpCommand::Mkd(arg.to_string())
            }
        }
        "RMD" => {
            if arg.is_empty() {
                FtpCommand::Unknown("RMD requires directory name".to_string())
            } else {
                FtpCommand::Rmd(arg.to_string())
            }
        }
        "LIST" => FtpCommand::List,
        "PWD" => FtpCommand::Pwd,
        "PASV" => FtpCommand::Pasv,
        "LOGOUT" => FtpCommand::Logout,
        "RAX" => FtpCommand::Rax,
        "HELP" => FtpCommand::Help,
        _ => FtpCommand::Unknown(format!("Unknown command: {cmd}")),
    }
}
