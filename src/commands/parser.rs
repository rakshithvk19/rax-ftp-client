//! Command parsing functionality

use super::FtpCommand;

/// Parse user input into FtpCommand
pub fn parse_command(input: &str) -> FtpCommand {
    // TODO: Implement proper command parsing
    // For now, return Unknown
    FtpCommand::Unknown(input.to_string())
}
