//! FTP status code definitions

// Success codes (2xx)
pub const USER_LOGGED_IN: u16 = 230;
pub const USER_NAME_OKAY_NEED_PASSWORD: u16 = 331;
pub const DIRECTORY_CHANGED: u16 = 250;
pub const FILE_DELETED: u16 = 250;
pub const CURRENT_DIRECTORY: u16 = 257;
pub const PORT_COMMAND_SUCCESSFUL: u16 = 200;
pub const PASSIVE_MODE: u16 = 227;
pub const LOGOUT_SUCCESSFUL: u16 = 221;
pub const GOODBYE: u16 = 221;
pub const RAX_RESPONSE: u16 = 200;

// Transfer codes (1xx)
pub const OPENING_DATA_CONNECTION: u16 = 150;
pub const TRANSFER_COMPLETE: u16 = 226;

// Error codes (4xx, 5xx)
pub const NOT_LOGGED_IN: u16 = 530;
pub const LOGIN_INCORRECT: u16 = 530;
pub const PERMISSION_DENIED: u16 = 532;
pub const FILE_NOT_FOUND: u16 = 550;
pub const FILE_ALREADY_EXISTS: u16 = 553;
pub const INSUFFICIENT_STORAGE: u16 = 552;
pub const SYNTAX_ERROR: u16 = 501;
pub const COMMAND_NOT_RECOGNIZED: u16 = 500;
pub const DATA_CONNECTION_FAILED: u16 = 425;
pub const TRANSFER_FAILED: u16 = 426;

/// Check if status code indicates success
pub fn is_success(code: u16) -> bool {
    code >= 200 && code < 300
}

/// Check if status code indicates intermediate response
pub fn is_intermediate(code: u16) -> bool {
    code >= 100 && code < 200
}

/// Check if status code indicates error
pub fn is_error(code: u16) -> bool {
    code >= 400
}

/// Check if status code indicates authentication success
pub fn is_authentication_success(code: u16) -> bool {
    code == USER_LOGGED_IN
}

/// Check if status code indicates need for password
pub fn is_need_password(code: u16) -> bool {
    code == USER_NAME_OKAY_NEED_PASSWORD
}
