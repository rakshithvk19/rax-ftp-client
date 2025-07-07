//! Transfer progress tracking

use std::time::{Duration, Instant};

/// Progress tracker for file transfers
pub struct TransferProgress {
    total_bytes: u64,
    transferred_bytes: u64,
    start_time: Instant,
    last_update: Instant,
}

impl TransferProgress {
    /// Create a new progress tracker
    pub fn new(total_bytes: u64) -> Self {
        let now = Instant::now();
        Self {
            total_bytes,
            transferred_bytes: 0,
            start_time: now,
            last_update: now,
        }
    }

    /// Update progress with new bytes transferred
    pub fn update(&mut self, bytes_transferred: u64) {
        self.transferred_bytes = bytes_transferred;
        self.last_update = Instant::now();
    }

    /// Add bytes to current progress
    pub fn add_bytes(&mut self, bytes: u64) {
        self.transferred_bytes += bytes;
        self.last_update = Instant::now();
    }

    /// Get current progress percentage
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.transferred_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Get transfer speed in bytes per second
    pub fn speed_bps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.transferred_bytes as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Format bytes as human readable
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

    /// Format speed as human readable
    pub fn format_speed(bps: f64) -> String {
        format!("{}/s", Self::format_bytes(bps as u64))
    }

    /// Display progress bar
    pub fn display_progress(&self, filename: &str) {
        let percentage = self.percentage();
        let speed = self.speed_bps();

        // Create progress bar (50 characters wide)
        let filled = (percentage / 2.0) as usize; // 50 chars = 100% / 2
        let bar = "#".repeat(filled) + &" ".repeat(50 - filled);

        print!(
            "\r{}: [{}] {:.1}% ({}) {}",
            filename,
            bar,
            percentage,
            Self::format_bytes(self.transferred_bytes),
            Self::format_speed(speed)
        );

        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }

    /// Check if transfer is complete
    pub fn is_complete(&self) -> bool {
        self.transferred_bytes >= self.total_bytes
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}
