//! Plist structure definition and generation logic for LaunchAgent.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{LaunchAgentError, Result};

/// LaunchAgent plist structure.
///
/// This struct represents the XML plist configuration for a macOS LaunchAgent.
/// It is used to configure the pomodoro daemon to run at login.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PomodoroLaunchAgent {
    /// Service label (reverse domain format)
    #[serde(rename = "Label")]
    pub label: String,

    /// Program to execute with its arguments
    #[serde(rename = "ProgramArguments")]
    pub program_arguments: Vec<String>,

    /// Whether to start at login
    #[serde(rename = "RunAtLoad")]
    pub run_at_load: bool,

    /// Whether to automatically restart if the process terminates
    #[serde(rename = "KeepAlive")]
    pub keep_alive: bool,

    /// Path to stdout log file
    #[serde(rename = "StandardOutPath")]
    pub standard_out_path: String,

    /// Path to stderr log file
    #[serde(rename = "StandardErrorPath")]
    pub standard_error_path: String,

    /// Working directory (optional)
    #[serde(rename = "WorkingDirectory", skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    /// Environment variables (optional)
    #[serde(
        rename = "EnvironmentVariables",
        skip_serializing_if = "Option::is_none"
    )]
    pub environment_variables: Option<HashMap<String, String>>,
}

impl PomodoroLaunchAgent {
    /// The default service label for the pomodoro LaunchAgent.
    pub const LABEL: &'static str = "com.example.pomodoro";

    /// Creates a new LaunchAgent configuration.
    ///
    /// # Arguments
    /// * `binary_path` - Absolute path to the pomodoro binary (e.g., "/usr/local/bin/pomodoro")
    /// * `log_dir` - Absolute path to the log directory (e.g., "/Users/username/.pomodoro/logs")
    ///
    /// # Returns
    /// A new LaunchAgent configuration with default settings.
    pub fn new(binary_path: impl Into<String>, log_dir: impl Into<String>) -> Self {
        let binary_path = binary_path.into();
        let log_dir = log_dir.into();

        Self {
            label: Self::LABEL.to_string(),
            program_arguments: vec![binary_path, "daemon".to_string()],
            run_at_load: true,
            keep_alive: true,
            standard_out_path: format!("{}/stdout.log", log_dir),
            standard_error_path: format!("{}/stderr.log", log_dir),
            working_directory: None,
            environment_variables: None,
        }
    }

    /// Generates the plist XML string.
    ///
    /// # Returns
    /// XML string representing the plist configuration.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_xml(&self) -> Result<String> {
        let mut buf = Vec::new();
        plist::to_writer_xml(&mut buf, self).map_err(LaunchAgentError::PlistSerialize)?;
        String::from_utf8(buf).map_err(LaunchAgentError::PlistUtf8)
    }

    /// Sets the working directory.
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Adds an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment_variables
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Disables automatic restart on exit.
    pub fn without_keep_alive(mut self) -> Self {
        self.keep_alive = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_valid_config() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        assert_eq!(plist.label, "com.example.pomodoro");
        assert_eq!(
            plist.program_arguments,
            vec!["/usr/local/bin/pomodoro", "daemon"]
        );
        assert!(plist.run_at_load);
        assert!(plist.keep_alive);
        assert_eq!(
            plist.standard_out_path,
            "/Users/test/.pomodoro/logs/stdout.log"
        );
        assert_eq!(
            plist.standard_error_path,
            "/Users/test/.pomodoro/logs/stderr.log"
        );
        assert!(plist.working_directory.is_none());
        assert!(plist.environment_variables.is_none());
    }

    #[test]
    fn test_to_xml_generates_valid_xml() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        let xml = plist.to_xml().expect("Failed to generate XML");

        // Verify XML contains expected elements
        assert!(xml.contains("<key>Label</key>"));
        assert!(xml.contains("<string>com.example.pomodoro</string>"));
        assert!(xml.contains("<key>ProgramArguments</key>"));
        assert!(xml.contains("<key>RunAtLoad</key>"));
        assert!(xml.contains("<true/>"));
        assert!(xml.contains("<key>KeepAlive</key>"));
        assert!(xml.contains("<key>StandardOutPath</key>"));
        assert!(xml.contains("<key>StandardErrorPath</key>"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        let xml = original.to_xml().expect("Failed to generate XML");
        let parsed: PomodoroLaunchAgent =
            plist::from_bytes(xml.as_bytes()).expect("Failed to parse XML");

        assert_eq!(original.label, parsed.label);
        assert_eq!(original.program_arguments, parsed.program_arguments);
        assert_eq!(original.run_at_load, parsed.run_at_load);
        assert_eq!(original.keep_alive, parsed.keep_alive);
        assert_eq!(original.standard_out_path, parsed.standard_out_path);
        assert_eq!(original.standard_error_path, parsed.standard_error_path);
    }

    #[test]
    fn test_with_working_directory() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs")
                .with_working_directory("/Users/test");

        assert_eq!(plist.working_directory, Some("/Users/test".to_string()));
    }

    #[test]
    fn test_with_env() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs")
                .with_env("RUST_LOG", "debug");

        let env = plist
            .environment_variables
            .expect("Environment variables should be set");
        assert_eq!(env.get("RUST_LOG"), Some(&"debug".to_string()));
    }

    #[test]
    fn test_with_multiple_env() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs")
                .with_env("RUST_LOG", "debug")
                .with_env("RUST_BACKTRACE", "1");

        let env = plist
            .environment_variables
            .expect("Environment variables should be set");
        assert_eq!(env.len(), 2);
        assert_eq!(env.get("RUST_LOG"), Some(&"debug".to_string()));
        assert_eq!(env.get("RUST_BACKTRACE"), Some(&"1".to_string()));
    }

    #[test]
    fn test_without_keep_alive() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs")
                .without_keep_alive();

        assert!(!plist.keep_alive);
    }

    #[test]
    fn test_label_constant() {
        assert_eq!(PomodoroLaunchAgent::LABEL, "com.example.pomodoro");
    }

    #[test]
    fn test_xml_does_not_contain_optional_fields_when_none() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs");

        let xml = plist.to_xml().expect("Failed to generate XML");

        // Optional fields should not be in XML when None
        assert!(!xml.contains("WorkingDirectory"));
        assert!(!xml.contains("EnvironmentVariables"));
    }

    #[test]
    fn test_xml_contains_optional_fields_when_set() {
        let plist =
            PomodoroLaunchAgent::new("/usr/local/bin/pomodoro", "/Users/test/.pomodoro/logs")
                .with_working_directory("/Users/test")
                .with_env("RUST_LOG", "debug");

        let xml = plist.to_xml().expect("Failed to generate XML");

        assert!(xml.contains("<key>WorkingDirectory</key>"));
        assert!(xml.contains("<key>EnvironmentVariables</key>"));
    }
}
