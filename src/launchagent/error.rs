//! Error types for LaunchAgent management.

use std::io;
use thiserror::Error;

/// LaunchAgent management error type.
#[derive(Debug, Error)]
pub enum LaunchAgentError {
    /// Failed to resolve pomodoro binary path.
    #[error("Failed to resolve pomodoro binary path: {0}")]
    BinaryPathResolution(String),

    /// Home directory not found.
    #[error("Failed to get home directory")]
    HomeDirectoryNotFound,

    /// Failed to create directory.
    #[error("Failed to create directory: {0}")]
    DirectoryCreation(#[source] io::Error),

    /// Failed to write plist file.
    #[error("Failed to write plist file: {0}")]
    PlistWrite(#[source] io::Error),

    /// Failed to remove plist file.
    #[error("Failed to remove plist file: {0}")]
    PlistRemove(#[source] io::Error),

    /// Failed to serialize plist.
    #[error("Failed to serialize plist: {0}")]
    PlistSerialize(#[source] plist::Error),

    /// Failed to convert plist to UTF-8 string.
    #[error("Failed to convert plist to UTF-8: {0}")]
    PlistUtf8(#[source] std::string::FromUtf8Error),

    /// Failed to set file permissions.
    #[error("Failed to set file permissions: {0}")]
    PermissionSet(#[source] io::Error),

    /// Failed to execute launchctl command.
    #[error("Failed to execute launchctl: {0}")]
    LaunchctlExecution(String),

    /// Failed to load LaunchAgent.
    #[error("Failed to load LaunchAgent: {0}")]
    ServiceLoad(String),

    /// Failed to unload LaunchAgent.
    #[error("Failed to unload LaunchAgent: {0}")]
    ServiceUnload(String),
}

/// Result type for LaunchAgent operations.
pub type Result<T> = std::result::Result<T, LaunchAgentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_binary_path_resolution() {
        let err = LaunchAgentError::BinaryPathResolution("not found".to_string());
        assert!(err.to_string().contains("binary path"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_error_display_home_directory_not_found() {
        let err = LaunchAgentError::HomeDirectoryNotFound;
        assert!(err.to_string().contains("home directory"));
    }

    #[test]
    fn test_error_display_launchctl_execution() {
        let err = LaunchAgentError::LaunchctlExecution("command failed".to_string());
        assert!(err.to_string().contains("launchctl"));
        assert!(err.to_string().contains("command failed"));
    }

    #[test]
    fn test_error_display_service_load() {
        let err = LaunchAgentError::ServiceLoad("already running".to_string());
        assert!(err.to_string().contains("load"));
        assert!(err.to_string().contains("already running"));
    }

    #[test]
    fn test_error_display_service_unload() {
        let err = LaunchAgentError::ServiceUnload("not running".to_string());
        assert!(err.to_string().contains("unload"));
        assert!(err.to_string().contains("not running"));
    }
}
