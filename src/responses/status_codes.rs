//! FTP status code definitions

// Success codes (2xx)
pub const USER_LOGGED_IN: u16 = 230;

// Client-side error codes
pub const CLIENT_ERROR_NOT_AUTHENTICATED: u16 = 530;

/// Check if status code indicates authentication success
pub fn is_authentication_success(code: u16) -> bool {
    code == USER_LOGGED_IN
}
