//! Configuration management
//!
//! Load and save user preferences to a TOML config file.

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::render_engines::{ascii::CharacterSet, unicode::UnicodeMode};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ascii: AsciiPreferences,
    pub unicode: UnicodePreferences,
    pub text: TextPreferences,
    pub ui: UiPreferences,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ascii: AsciiPreferences::default(),
            unicode: UnicodePreferences::default(),
            text: TextPreferences::default(),
            ui: UiPreferences::default(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "glyphgen", "glyphgen") {
            Ok(proj_dirs.config_dir().join("config.toml"))
        } else {
            // Fallback to current directory
            Ok(PathBuf::from("glyphgen.toml"))
        }
    }
}

/// ASCII rendering preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiPreferences {
    pub default_charset: CharacterSet,
    pub default_width: usize,
    pub edge_enhance: bool,
}

impl Default for AsciiPreferences {
    fn default() -> Self {
        Self {
            default_charset: CharacterSet::Extended,
            default_width: 80,
            edge_enhance: false,
        }
    }
}

/// Unicode rendering preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnicodePreferences {
    pub default_mode: UnicodeMode,
    pub default_width: usize,
}

impl Default for UnicodePreferences {
    fn default() -> Self {
        Self {
            default_mode: UnicodeMode::HalfBlocks,
            default_width: 80,
        }
    }
}

/// Text stylizer preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPreferences {
    pub default_style: String,
    pub default_gradient: String,
}

impl Default for TextPreferences {
    fn default() -> Self {
        Self {
            default_style: "Bold".to_string(),
            default_gradient: "None".to_string(),
        }
    }
}

/// UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub show_line_numbers: bool,
    pub word_wrap: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            show_line_numbers: false,
            word_wrap: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ascii.default_width, 80);
        assert!(!config.ascii.edge_enhance);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.ascii.default_width, config.ascii.default_width);
    }
}
