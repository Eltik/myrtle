use serde::{Deserialize, Serialize};
use std::path::Path;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Performance level (affects thread count)
    pub performance_level: PerformanceLevel,
    /// Log file path
    pub log_file: Option<String>,
    /// Log level (0-4)
    pub log_level: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PerformanceLevel {
    Low,
    Medium,
    High,
    Ultra,
}

impl PerformanceLevel {
    /// Get thread limit based on performance level
    pub fn thread_limit(&self) -> usize {
        match self {
            PerformanceLevel::Low => 2,
            PerformanceLevel::Medium => 4,
            PerformanceLevel::High => 8,
            PerformanceLevel::Ultra => num_cpus::get(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            performance_level: PerformanceLevel::High,
            log_file: None,
            log_level: 2,
        }
    }
}

impl Config {
    /// Load config from file or return default
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Config::default()
    }

    /// Save config to file
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
