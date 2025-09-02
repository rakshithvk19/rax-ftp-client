//! Directory listing display functionality

use std::fmt;

/// Represents a directory entry with metadata
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub entry_type: EntryType,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub permissions: Option<String>,
}

/// Type of directory entry
#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    File,
    Directory,
    Link,
    Unknown,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryType::File => write!(f, "File"),
            EntryType::Directory => write!(f, "Dir"),
            EntryType::Link => write!(f, "Link"),
            EntryType::Unknown => write!(f, "?"),
        }
    }
}

impl DirectoryEntry {
    /// Create a new directory entry from a raw string
    /// Parses format: "name|size|timestamp" or falls back to simple name
    pub fn from_raw(raw_entry: &str) -> Self {
        let trimmed = raw_entry.trim();
        
        // Check if this is the new format with metadata: "name|size|timestamp"
        if let Some(_) = trimmed.find('|') {
            let parts: Vec<&str> = trimmed.split('|').collect();
            if parts.len() == 3 {
                let name = parts[0];
                let size: Option<u64> = parts[1].parse().ok().filter(|&s| s > 0);
                let timestamp: Option<u64> = parts[2].parse().ok().filter(|&t| t > 0);
                
                // Convert timestamp to readable format
                let modified = timestamp.and_then(|ts| {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let system_time = UNIX_EPOCH + std::time::Duration::from_secs(ts);
                    let datetime: chrono::DateTime<chrono::Local> = system_time.into();
                    Some(datetime.format("%Y-%m-%d %H:%M").to_string())
                });
                
                let (entry_type, display_name) = if name == "." || name == ".." {
                    (EntryType::Directory, name.to_string())
                } else if name.ends_with('/') {
                    (EntryType::Directory, name.trim_end_matches('/').to_string())
                } else {
                    (EntryType::File, name.to_string())
                };
                
                return Self {
                    name: display_name,
                    entry_type,
                    size,
                    modified,
                    permissions: None,
                };
            }
        }
        
        // Handle parse failure gracefully (no panics)
        Self {
            name: trimmed.to_string(),
            entry_type: EntryType::Unknown,
            size: None,
            modified: None,
            permissions: None,
        }
    }

    /// Get color code for the entry type (for future color support)
    pub fn color_code(&self) -> &'static str {
        match self.entry_type {
            EntryType::Directory => "\x1b[34m", // Blue
            EntryType::Link => "\x1b[36m",      // Cyan
            EntryType::File => "\x1b[0m",       // Default
            EntryType::Unknown => "\x1b[90m",   // Gray
        }
    }

    /// Reset color code
    pub fn reset_color() -> &'static str {
        "\x1b[0m"
    }
}

/// Display a directory listing in formatted columns
pub fn format_directory_listing(raw_listing: &[String]) -> String {
    if raw_listing.is_empty() {
        return "Directory is empty.".to_string();
    }

    // Parse raw entries into structured format
    let entries: Vec<DirectoryEntry> = raw_listing
        .iter()
        .filter(|s| !s.trim().is_empty())
        .map(|s| DirectoryEntry::from_raw(s))
        .collect();

    // Check if terminal supports colors (simplified check)
    let supports_color = std::env::var("TERM").is_ok() && !cfg!(windows);

    let mut output = String::new();

    // Add header
    output.push_str(&format!(
        "{:<30} {:<8} {:<10} {:<20}\n",
        "Name", "Type", "Size", "Modified"
    ));
    output.push_str(&format!("{}\n", "-".repeat(68)));

    // Add each entry
    for entry in entries {
        let name_display = if supports_color {
            format!(
                "{}{}{}",
                entry.color_code(),
                truncate_name(&entry.name, 30),
                DirectoryEntry::reset_color()
            )
        } else {
            truncate_name(&entry.name, 30)
        };

        output.push_str(&format!(
            "{:<30} {:<8} {:<10} {:<20}\n",
            name_display,
            entry.entry_type,
            entry.size.map_or("-".to_string(), format_size),
            entry.modified.as_deref().unwrap_or("-")
        ));
    }

    output
}
/// Truncate long names to fit in column width
fn truncate_name(name: &str, max_width: usize) -> String {
    if name.len() <= max_width {
        name.to_string()
    } else if max_width > 3 {
        format!("{}...", &name[..max_width - 3])
    } else {
        name[..max_width].to_string()
    }
}

/// Format file size in human-readable format
fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_entry_from_raw() {
        let file_entry = DirectoryEntry::from_raw("test.txt");
        assert_eq!(file_entry.name, "test.txt");
        assert_eq!(file_entry.entry_type, EntryType::File);

        let dir_entry = DirectoryEntry::from_raw("folder/");
        assert_eq!(dir_entry.name, "folder");
        assert_eq!(dir_entry.entry_type, EntryType::Directory);

        let link_entry = DirectoryEntry::from_raw("link -> target");
        assert_eq!(link_entry.name, "link");
        assert_eq!(link_entry.entry_type, EntryType::Link);
    }

    #[test]
    fn test_truncate_name() {
        assert_eq!(truncate_name("short", 10), "short");
        assert_eq!(truncate_name("verylongfilename.txt", 10), "verylo...");
        assert_eq!(truncate_name("test", 2), "te");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
    }
}
