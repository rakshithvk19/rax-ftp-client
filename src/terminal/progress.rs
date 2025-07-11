//! Progress display functionality for file transfers

use std::io::{self, Write};

/// Display transfer progress bar
pub fn display_progress(filename: &str, percentage: f64, transferred_bytes: u64, speed_bps: f64) {
    // Create progress bar (50 characters wide)
    let filled = (percentage / 2.0) as usize; // 50 chars = 100% / 2
    let bar = "#".repeat(filled) + &" ".repeat(50 - filled);
    
    print!(
        "\r{}: [{}] {:.1}% ({}) {}",
        filename,
        bar,
        percentage,
        format_bytes(transferred_bytes),
        format_speed(speed_bps)
    );
    
    if let Err(e) = io::stdout().flush() {
        eprintln!("\nError flushing stdout: {}", e);
    }
}

/// Clear the progress line and move to next line
pub fn finish_progress() {
    println!(); // Move to next line after progress bar
}

/// Format bytes as human readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format speed as human readable string
pub fn format_speed(bps: f64) -> String {
    format!("{}/s", format_bytes(bps as u64))
}

/// Display a simple spinner for operations without known progress
pub fn display_spinner(message: &str, step: usize) {
    const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner = SPINNER_CHARS[step % SPINNER_CHARS.len()];
    
    print!("\r{} {}", spinner, message);
    
    if let Err(e) = io::stdout().flush() {
        eprintln!("\nError flushing stdout: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }
    
    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(0.0), "0 B/s");
        assert_eq!(format_speed(1024.0), "1.0 KB/s");
        assert_eq!(format_speed(1048576.0), "1.0 MB/s");
    }
}
