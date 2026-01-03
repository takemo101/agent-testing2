//! Focus mode configuration types.
//!
//! This module defines the configuration structure for Shortcuts.app-based
//! focus mode integration. The configuration is designed for Phase 2 where
//! settings will be persisted to a configuration file.

use serde::{Deserialize, Serialize};

/// Default shortcut name for enabling focus mode.
fn default_enable_shortcut_name() -> String {
    "Enable Work Focus".to_string()
}

/// Default shortcut name for disabling focus mode.
fn default_disable_shortcut_name() -> String {
    "Disable Work Focus".to_string()
}

/// Default timeout for shortcut execution in seconds.
fn default_timeout_seconds() -> u64 {
    5
}

/// Focus mode integration configuration.
///
/// This structure holds the settings for Shortcuts.app-based focus mode
/// integration. The default configuration is disabled and uses standard
/// shortcut names.
///
/// # Example
///
/// ```
/// use pomodoro::focus::FocusModeConfig;
///
/// let config = FocusModeConfig::default();
/// assert!(!config.enabled);
/// assert_eq!(config.enable_shortcut_name, "Enable Work Focus");
/// assert_eq!(config.disable_shortcut_name, "Disable Work Focus");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FocusModeConfig {
    /// Whether focus mode integration is enabled.
    #[serde(default)]
    pub enabled: bool,

    /// The name of the shortcut that enables focus mode.
    #[serde(default = "default_enable_shortcut_name")]
    pub enable_shortcut_name: String,

    /// The name of the shortcut that disables focus mode.
    #[serde(default = "default_disable_shortcut_name")]
    pub disable_shortcut_name: String,

    /// Timeout for shortcut execution in seconds.
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

impl Default for FocusModeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            enable_shortcut_name: default_enable_shortcut_name(),
            disable_shortcut_name: default_disable_shortcut_name(),
            timeout_seconds: default_timeout_seconds(),
        }
    }
}

impl FocusModeConfig {
    /// Creates a new configuration with focus mode enabled.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Self::default()
        }
    }

    /// Creates a new configuration with custom shortcut names.
    #[must_use]
    pub fn with_shortcuts(enable_name: impl Into<String>, disable_name: impl Into<String>) -> Self {
        Self {
            enabled: true,
            enable_shortcut_name: enable_name.into(),
            disable_shortcut_name: disable_name.into(),
            ..Self::default()
        }
    }

    /// Returns whether focus mode is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Sets the timeout in seconds.
    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout_seconds = seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FocusModeConfig::default();

        assert!(!config.enabled);
        assert_eq!(config.enable_shortcut_name, "Enable Work Focus");
        assert_eq!(config.disable_shortcut_name, "Disable Work Focus");
        assert_eq!(config.timeout_seconds, 5);
    }

    #[test]
    fn test_enabled_config() {
        let config = FocusModeConfig::enabled();

        assert!(config.enabled);
        assert_eq!(config.enable_shortcut_name, "Enable Work Focus");
        assert_eq!(config.disable_shortcut_name, "Disable Work Focus");
    }

    #[test]
    fn test_with_shortcuts() {
        let config = FocusModeConfig::with_shortcuts("Custom Enable", "Custom Disable");

        assert!(config.enabled);
        assert_eq!(config.enable_shortcut_name, "Custom Enable");
        assert_eq!(config.disable_shortcut_name, "Custom Disable");
    }

    #[test]
    fn test_is_enabled() {
        let mut config = FocusModeConfig::default();
        assert!(!config.is_enabled());

        config.set_enabled(true);
        assert!(config.is_enabled());

        config.set_enabled(false);
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_set_timeout() {
        let mut config = FocusModeConfig::default();
        assert_eq!(config.timeout_seconds, 5);

        config.set_timeout(10);
        assert_eq!(config.timeout_seconds, 10);
    }

    #[test]
    fn test_serialize_json() {
        let config = FocusModeConfig::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize");

        assert!(json.contains("\"enabled\":false"));
        assert!(json.contains("\"enable_shortcut_name\":\"Enable Work Focus\""));
        assert!(json.contains("\"disable_shortcut_name\":\"Disable Work Focus\""));
        assert!(json.contains("\"timeout_seconds\":5"));
    }

    #[test]
    fn test_deserialize_json() {
        let json = r#"{
            "enabled": true,
            "enable_shortcut_name": "Custom Enable",
            "disable_shortcut_name": "Custom Disable",
            "timeout_seconds": 10
        }"#;

        let config: FocusModeConfig = serde_json::from_str(json).expect("Failed to deserialize");

        assert!(config.enabled);
        assert_eq!(config.enable_shortcut_name, "Custom Enable");
        assert_eq!(config.disable_shortcut_name, "Custom Disable");
        assert_eq!(config.timeout_seconds, 10);
    }

    #[test]
    fn test_deserialize_json_with_defaults() {
        // Test that missing fields use defaults
        let json = r#"{"enabled": true}"#;

        let config: FocusModeConfig = serde_json::from_str(json).expect("Failed to deserialize");

        assert!(config.enabled);
        assert_eq!(config.enable_shortcut_name, "Enable Work Focus");
        assert_eq!(config.disable_shortcut_name, "Disable Work Focus");
        assert_eq!(config.timeout_seconds, 5);
    }

    #[test]
    fn test_deserialize_empty_json() {
        let json = r#"{}"#;

        let config: FocusModeConfig = serde_json::from_str(json).expect("Failed to deserialize");

        assert!(!config.enabled);
        assert_eq!(config.enable_shortcut_name, "Enable Work Focus");
        assert_eq!(config.disable_shortcut_name, "Disable Work Focus");
        assert_eq!(config.timeout_seconds, 5);
    }

    #[test]
    fn test_clone() {
        let config = FocusModeConfig::enabled();
        let cloned = config.clone();

        assert_eq!(config, cloned);
    }

    #[test]
    fn test_eq() {
        let config1 = FocusModeConfig::default();
        let config2 = FocusModeConfig::default();
        let config3 = FocusModeConfig::enabled();

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_debug() {
        let config = FocusModeConfig::default();
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("FocusModeConfig"));
        assert!(debug_str.contains("enabled"));
    }
}
