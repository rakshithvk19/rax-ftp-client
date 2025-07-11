//! Connection management for RAX FTP Client
//!
//! Handles both command and data connections for FTP operations.

pub mod command;
pub mod data;

// Re-export main types
pub use command::CommandConnection;
pub use data::{DataConnection, DataConnectionInfo, DataMode};
