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

    /// Get total bytes
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Get transferred bytes
    pub fn transferred_bytes(&self) -> u64 {
        self.transferred_bytes
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
