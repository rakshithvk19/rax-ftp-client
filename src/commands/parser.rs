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
        "LIST" => FtpCommand::List,
        "PWD" => FtpCommand::Pwd,
        "PASV" => FtpCommand::Pasv,
        "LOGOUT" => FtpCommand::Logout,
        "RAX" => FtpCommand::Rax,
        "HELP" => FtpCommand::Help,
        _ => FtpCommand::Unknown(format!("Unknown command: {}", cmd)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_commands() {
        assert_eq!(parse_command("QUIT"), FtpCommand::Quit);
        assert_eq!(parse_command("quit"), FtpCommand::Quit);
        assert_eq!(parse_command("LIST"), FtpCommand::List);
        assert_eq!(parse_command("PWD"), FtpCommand::Pwd);
        assert_eq!(parse_command("HELP"), FtpCommand::Help);
        assert_eq!(parse_command("PASV"), FtpCommand::Pasv);
        assert_eq!(parse_command("RAX"), FtpCommand::Rax);
    }

    #[test]
    fn test_parse_commands_with_args() {
        assert_eq!(
            parse_command("USER john"),
            FtpCommand::User("john".to_string())
        );
        assert_eq!(
            parse_command("PASS secret"),
            FtpCommand::Pass("secret".to_string())
        );
        assert_eq!(
            parse_command("STOR file.txt"),
            FtpCommand::Stor("file.txt".to_string())
        );
        assert_eq!(
            parse_command("RETR file.txt"),
            FtpCommand::Retr("file.txt".to_string())
        );
        assert_eq!(
            parse_command("DEL file.txt"),
            FtpCommand::Del("file.txt".to_string())
        );
        assert_eq!(
            parse_command("CWD /home"),
            FtpCommand::Cwd("/home".to_string())
        );
        assert_eq!(
            parse_command("PORT 127.0.0.1:8080"),
            FtpCommand::Port("127.0.0.1:8080".to_string())
        );
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(parse_command("user"), FtpCommand::User("".to_string()));
        assert_eq!(parse_command("User"), FtpCommand::User("".to_string()));
        assert_eq!(parse_command("USER"), FtpCommand::User("".to_string()));
        assert_eq!(parse_command("list"), FtpCommand::List);
        assert_eq!(parse_command("List"), FtpCommand::List);
    }

    #[test]
    fn test_parse_invalid_commands() {
        match parse_command("USER") {
            FtpCommand::Unknown(msg) => assert!(msg.contains("USER requires username")),
            _ => panic!("Should return Unknown for USER without argument"),
        }

        match parse_command("STOR") {
            FtpCommand::Unknown(msg) => assert!(msg.contains("STOR requires filename")),
            _ => panic!("Should return Unknown for STOR without argument"),
        }

        match parse_command("INVALID") {
            FtpCommand::Unknown(msg) => assert!(msg.contains("Unknown command")),
            _ => panic!("Should return Unknown for invalid command"),
        }

        match parse_command("") {
            FtpCommand::Unknown(msg) => assert!(msg.contains("Empty command")),
            _ => panic!("Should return Unknown for empty command"),
        }
    }

    #[test]
    fn test_parse_with_extra_whitespace() {
        assert_eq!(
            parse_command("  USER   john  "),
            FtpCommand::User("john".to_string())
        );
        assert_eq!(parse_command("\tLIST\t"), FtpCommand::List);
        assert_eq!(
            parse_command("   "),
            FtpCommand::Unknown("Empty command".to_string())
        );
    }
}
